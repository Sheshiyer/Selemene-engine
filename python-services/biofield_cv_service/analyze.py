"""Real /analyze endpoint for Biofield CV service.

Implements all 11 spatial algorithms using OpenCV + NumPy + SciPy.
Replaced stub-metrics/v1 with real-cv/v1.
"""

import json
import time
from typing import Any

import cv2
import numpy as np
from fastapi import APIRouter, File, Form, HTTPException, UploadFile, status
from scipy.spatial.distance import pdist
from skimage.feature import local_binary_pattern

from shared.models import (
    BiofieldCVResponse,
    EnergyBands,
    QualityAssessment,
    SpatialMetrics,
)

router = APIRouter()

CONTRACT_VERSION = "biofield-cv/v1"
ANALYSIS_VERSION = "real-cv/v1"

ALL_ALGORITHMS = [
    "light_quanta_density",
    "normalized_area",
    "average_intensity",
    "inner_noise",
    "energy_analysis",
    "entropy_form_coefficient",
    "fractal_dimension",
    "correlation_dimension",
    "body_symmetry",
    "contour_complexity",
    "pattern_regularity",
]


# ─── T-674-1: Segmentation mask ─────────────────────────────────────────────

def _extract_mask(img_bgr: np.ndarray) -> np.ndarray:
    """Return a binary mask (H×W uint8, 0 or 255) for the person/subject.

    PIP captures already have background removed by MediaPipe on the client,
    so the background is either black or transparent (alpha=0 in PNG). We use
    Otsu thresholding on the luminance channel plus a morphological close to
    fill small gaps. Falls back to a full-frame mask if Otsu produces empty output.
    """
    gray = cv2.cvtColor(img_bgr, cv2.COLOR_BGR2GRAY)
    _, mask = cv2.threshold(gray, 0, 255, cv2.THRESH_BINARY + cv2.THRESH_OTSU)
    kernel = np.ones((5, 5), np.uint8)
    mask = cv2.morphologyEx(mask, cv2.MORPH_CLOSE, kernel)

    if mask.sum() == 0:
        mask = np.ones_like(gray, dtype=np.uint8) * 255

    return mask


def _decode_image(image_bytes: bytes) -> tuple[np.ndarray, np.ndarray]:
    """Decode image bytes → (img_bgr, img_gray). Raises 422 on decode failure."""
    arr = np.frombuffer(image_bytes, np.uint8)
    img_bgr = cv2.imdecode(arr, cv2.IMREAD_COLOR)
    if img_bgr is None:
        raise HTTPException(
            status_code=status.HTTP_422_UNPROCESSABLE_CONTENT,
            detail="image could not be decoded — must be a valid PNG/JPEG/WebP",
        )
    img_gray = cv2.cvtColor(img_bgr, cv2.COLOR_BGR2GRAY)
    return img_bgr, img_gray


# ─── T-674-2: Photometric metrics ───────────────────────────────────────────

def _photometric(img_gray: np.ndarray, mask: np.ndarray) -> dict[str, float]:
    mask_bool = mask > 0
    total_px = float(img_gray.size)
    mask_area = float(mask_bool.sum())

    if mask_area == 0:
        return {
            "normalized_area": 0.0,
            "average_intensity": 0.0,
            "inner_noise": 0.0,
            "light_quanta_density": 0.0,
        }

    pixels = img_gray[mask_bool].astype(float)
    normalized_area = mask_area / total_px
    average_intensity = pixels.mean() / 255.0
    mean_px = pixels.mean()
    inner_noise = float(pixels.std() / (mean_px + 1e-8))
    light_quanta_density = float(average_intensity * normalized_area * 100.0)

    return {
        "normalized_area": round(float(normalized_area), 6),
        "average_intensity": round(float(average_intensity), 6),
        "inner_noise": round(inner_noise, 6),
        "light_quanta_density": round(light_quanta_density, 4),
    }


# ─── T-674-3: Energy analysis (FFT frequency bands) ─────────────────────────

def _energy_analysis(img_gray: np.ndarray, mask: np.ndarray) -> EnergyBands:
    masked = img_gray.astype(float) * (mask > 0).astype(float)
    f = np.fft.fft2(masked)
    mag = np.abs(np.fft.fftshift(f))
    total = float(mag.sum())

    if total < 1e-8:
        return EnergyBands(low=0.0, medium=0.0, high=0.0, total=0.0)

    h, w = mag.shape
    cy, cx = h // 2, w // 2
    r_low = max(1, min(h, w) // 10)
    r_mid = max(2, min(h, w) // 3)

    low_band = float(mag[cy - r_low:cy + r_low, cx - r_low:cx + r_low].sum())
    mid_band = float(mag[cy - r_mid:cy + r_mid, cx - r_mid:cx + r_mid].sum())
    high_band = float(total - mid_band)
    medium_band = float(mid_band - low_band)

    return EnergyBands(
        low=round(low_band / total, 6),
        medium=round(medium_band / total, 6),
        high=round(high_band / total, 6),
        total=1.0,
    )


# ─── T-674-4: Information-theoretic metrics ──────────────────────────────────

def _entropy_form_coefficient(img_gray: np.ndarray, mask: np.ndarray) -> float:
    hist = cv2.calcHist([img_gray], [0], mask, [256], [0, 256]).flatten()
    hist = hist[hist > 0]
    hist = hist / hist.sum()
    entropy = float(-np.sum(hist * np.log2(hist)))
    return round(entropy, 6)


def _fractal_dimension(mask: np.ndarray) -> float:
    edges = cv2.Canny(mask, 50, 150)
    sizes = [2, 4, 8, 16, 32]
    counts = []
    for s in sizes:
        # Downsample by factor s using max pooling
        h, w = edges.shape
        h_t = (h // s) * s
        w_t = (w // s) * s
        if h_t == 0 or w_t == 0:
            counts.append(1)
            continue
        blocks = edges[:h_t, :w_t].reshape(h_t // s, s, w_t // s, s)
        counts.append(int((blocks.max(axis=(1, 3)) > 0).sum()))

    counts_arr = np.array(counts, dtype=float)
    counts_arr = np.maximum(counts_arr, 1.0)
    sizes_arr = np.array(sizes, dtype=float)

    coeffs = np.polyfit(np.log(sizes_arr), np.log(counts_arr), 1)
    fd = float(-coeffs[0])
    return round(float(np.clip(fd, 1.0, 2.0)), 6)


def _correlation_dimension(mask: np.ndarray) -> float:
    ys, xs = np.where(mask > 0)
    if len(ys) < 10:
        return 1.0

    # Sample up to 300 boundary / interior points
    n_pts = min(300, len(ys))
    idx = np.random.default_rng(42).choice(len(ys), n_pts, replace=False)
    pts = np.stack([xs[idx], ys[idx]], axis=1).astype(float)

    dists = pdist(pts, metric="euclidean")
    if len(dists) == 0:
        return 1.0

    d_max = dists.max()
    if d_max < 1e-8:
        return 1.0

    radii = np.logspace(np.log10(d_max * 0.05), np.log10(d_max * 0.5), 8)
    c_r = np.array([float((dists < r).sum()) for r in radii])
    c_r = np.maximum(c_r, 1.0)

    coeffs = np.polyfit(np.log(radii), np.log(c_r), 1)
    cd = float(np.clip(coeffs[0], 0.5, 4.0))
    return round(cd, 6)


# ─── T-674-5: Shape and symmetry metrics ────────────────────────────────────

def _body_symmetry(mask: np.ndarray) -> float:
    mid = mask.shape[1] // 2
    if mid < 2:
        return 0.0
    left = mask[:, :mid].astype(float).ravel()
    right = np.fliplr(mask[:, mid : mid * 2]).astype(float).ravel()
    if left.std() < 1e-8 or right.std() < 1e-8:
        return float(np.isclose(left, right).mean() * 2.0 - 1.0)
    corr = float(np.corrcoef(left, right)[0, 1])
    return round(float(np.clip(corr, -1.0, 1.0)), 6)


def _contour_complexity(mask: np.ndarray) -> float:
    contours, _ = cv2.findContours(mask, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)
    if not contours:
        return 0.0
    c = max(contours, key=cv2.contourArea)
    area = cv2.contourArea(c)
    perimeter = cv2.arcLength(c, True)
    if area < 1e-8:
        return 0.0
    return round(float((perimeter ** 2) / area), 4)


def _pattern_regularity(img_gray: np.ndarray, mask: np.ndarray) -> float:
    lbp = local_binary_pattern(img_gray, P=8, R=1, method="uniform")
    lbp_masked = lbp[mask > 0]
    if len(lbp_masked) == 0:
        return 0.0
    hist, _ = np.histogram(lbp_masked, bins=10, density=True)
    regularity = float(1.0 - hist.std())
    return round(float(np.clip(regularity, 0.0, 1.0)), 6)


# ─── T-674-6: Real QualityAssessment ─────────────────────────────────────────

def _quality_assessment(img_gray: np.ndarray) -> QualityAssessment:
    lap_var = float(cv2.Laplacian(img_gray, cv2.CV_64F).var())
    sharpness = round(float(np.clip(lap_var / 1000.0, 0.0, 1.0)), 6)

    contrast_raw = float(img_gray.std())
    contrast = round(float(np.clip(contrast_raw / 128.0, 0.0, 1.0)), 6)

    lap_std = float(cv2.Laplacian(img_gray, cv2.CV_64F).std())
    noise_level = round(float(np.clip(lap_std / 500.0, 0.0, 1.0)), 6)

    mean_lum = float(img_gray.mean()) / 255.0
    exposure = round(float(1.0 - abs(mean_lum - 0.5) * 2.0), 6)

    sufficient_quality = (
        lap_var > 50.0
        and contrast_raw > 20.0
        and exposure > 0.3
    )

    return QualityAssessment(
        sharpness=sharpness,
        contrast=contrast,
        noise_level=noise_level,
        exposure=exposure,
        sufficient_quality=sufficient_quality,
    )


# ─── Utilities ───────────────────────────────────────────────────────────────

def _unprocessable(detail: str) -> HTTPException:
    return HTTPException(status_code=status.HTTP_422_UNPROCESSABLE_CONTENT, detail=detail)


def _parse_algorithms(raw: str | None) -> list[str]:
    if raw is None:
        return list(ALL_ALGORITHMS)

    try:
        requested = json.loads(raw)
    except json.JSONDecodeError as exc:
        raise _unprocessable(f"algorithms must be valid JSON: {exc.msg}") from exc

    if not isinstance(requested, list):
        raise _unprocessable("algorithms must be a JSON array of known algorithm names")

    if any(not isinstance(item, str) for item in requested):
        raise _unprocessable("algorithms must be a JSON array of strings")

    unknown = [item for item in requested if item not in ALL_ALGORITHMS]
    if unknown:
        raise _unprocessable(
            "algorithms contains unsupported values: " + ", ".join(sorted(set(unknown)))
        )

    deduped: list[str] = []
    for algorithm in requested:
        if algorithm not in deduped:
            deduped.append(algorithm)

    return deduped


def _parse_json_object(raw: str | None, field_name: str) -> dict[str, Any] | None:
    if raw is None:
        return None

    try:
        parsed = json.loads(raw)
    except json.JSONDecodeError as exc:
        raise _unprocessable(f"{field_name} must be valid JSON: {exc.msg}") from exc

    if not isinstance(parsed, dict):
        raise _unprocessable(f"{field_name} must be a JSON object")

    return parsed


# ─── /analyze endpoint ────────────────────────────────────────────────────────

@router.post("/analyze", response_model=BiofieldCVResponse)
async def analyze(
    image: UploadFile = File(..., description="PIP or webcam capture image"),
    algorithms: str | None = Form(None, description="JSON array of algorithm names to run"),
    options: str | None = Form(None, description="JSON string with processing options"),
    capture_metadata: str | None = Form(
        None,
        description="Optional JSON object with capture metadata",
    ),
) -> BiofieldCVResponse:
    start = time.time()

    image_bytes = await image.read()

    algorithms_run = _parse_algorithms(algorithms)
    _parse_json_object(options, "options")
    _parse_json_object(capture_metadata, "capture_metadata")

    img_bgr, img_gray = _decode_image(image_bytes)
    mask = _extract_mask(img_bgr)

    phot = _photometric(img_gray, mask)
    energy = _energy_analysis(img_gray, mask)
    quality = _quality_assessment(img_gray)

    metrics = SpatialMetrics(
        light_quanta_density=phot["light_quanta_density"],
        normalized_area=phot["normalized_area"],
        average_intensity=phot["average_intensity"],
        inner_noise=phot["inner_noise"],
        energy_analysis=energy,
        entropy_form_coefficient=_entropy_form_coefficient(img_gray, mask),
        fractal_dimension=_fractal_dimension(mask),
        correlation_dimension=_correlation_dimension(mask),
        body_symmetry=_body_symmetry(mask),
        contour_complexity=_contour_complexity(mask),
        pattern_regularity=_pattern_regularity(img_gray, mask),
    )

    elapsed_ms = (time.time() - start) * 1000

    return BiofieldCVResponse(
        contract_version=CONTRACT_VERSION,
        analysis_version=ANALYSIS_VERSION,
        metrics=metrics,
        quality_assessment=quality,
        algorithms_run=algorithms_run,
        processing_time_ms=round(elapsed_ms, 2),
    )

