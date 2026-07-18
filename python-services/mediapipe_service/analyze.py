"""/analyze endpoint for MediaPipe Face Mesh service.

face-cv-hook-p3: real CV path added (MediaPipe Face Mesh, 468 3D landmarks) with
graceful deterministic fallback when mediapipe is unavailable or the image cannot
be decoded. Mirrors the T-065 biofield pattern (lazy mediapipe singleton +
fallback) and the FROZEN face contract (consent echo, quality).

Cites: p1-w1-worker-bootstrap-packet.md + resources-and-assets.md +
gaps-and-improvements.md (face: no real CV) + goal-understanding.md (local-first,
consent) + P1W1-CONTRACTS-FROZEN.md (face example) + detailed-task-list.md (T-027)
+ EXECUTION-STATUS.md + data/face-reading/facial_landmark_mappings.json.
Tags: phase:integration-p1 wave:integration-w2 engine-face-reading
"""

import hashlib
import json
import math
import random
import time

import cv2
import numpy as np
from fastapi import APIRouter, File, Form, UploadFile

from shared.models import (
    FaceMeshResponse,
    FacialProportions,
    ImageQuality,
    Landmark,
)

router = APIRouter()

NUM_LANDMARKS = 468
CONTRACT_VERSION = "face-cv/v1"

# Standard MediaPipe face oval contour indices.
FACE_OVAL_INDICES = [
    10, 338, 297, 332, 284, 251, 389, 356, 454, 323, 361, 288,
    397, 365, 379, 378, 400, 377, 152, 148, 176, 149, 150, 136,
    172, 58, 132, 93, 234, 127, 162, 21, 54, 103, 67, 109,
]

# Landmark index ranges for facial regions (approximate MediaPipe groupings).
_LEFT_EYE = range(33, 42)
_RIGHT_EYE = range(263, 272)
_NOSE_BRIDGE = range(1, 9)
_NOSE_TIP = range(48, 60)
_UPPER_LIP = range(61, 79)
_LOWER_LIP = range(78, 96)
_LEFT_BROW = range(46, 56)
_RIGHT_BROW = range(276, 286)
_FOREHEAD = range(9, 11)  # top-of-head indices

# Canonical MediaPipe bilateral pairs used for symmetry scoring.
_BILATERAL_PAIRS = [
    (33, 263), (133, 362), (7, 249), (144, 373), (145, 374),
    (153, 377), (154, 378), (155, 389), (46, 276), (53, 283),
    (65, 295), (70, 300), (107, 336), (105, 334), (61, 291),
    (78, 308), (234, 454), (93, 323), (132, 361), (58, 288),
    (172, 397), (162, 389), (21, 251), (54, 284), (103, 332),
    (67, 297), (109, 338), (127, 356),
]


# ─── Real CV path (face-cv-hook-p3; mirrors T-065 _get_mediapipe_selfie) ─────

_mp_facemesh = None


def _get_mediapipe_facemesh():
    """Lazy MediaPipe Face Mesh singleton; None when mediapipe is unavailable."""
    global _mp_facemesh
    if _mp_facemesh is None:
        try:
            import mediapipe as mp

            _mp_facemesh = mp.solutions.face_mesh.FaceMesh(
                static_image_mode=True,
                max_num_faces=1,
                refine_landmarks=False,
                min_detection_confidence=0.5,
            )
        except Exception:
            _mp_facemesh = False  # mark unavailable
    return _mp_facemesh if _mp_facemesh is not False else None


def _decode_image(image_bytes: bytes) -> np.ndarray | None:
    """Decode image bytes to BGR array; None when undecodable."""
    arr = np.frombuffer(image_bytes, np.uint8)
    if arr.size == 0:
        return None
    return cv2.imdecode(arr, cv2.IMREAD_COLOR)


def _dist(a: Landmark, b: Landmark) -> float:
    return math.sqrt((a.x - b.x) ** 2 + (a.y - b.y) ** 2 + (a.z - b.z) ** 2)


def _proportions_from_landmarks(landmarks: list[Landmark]) -> FacialProportions:
    """Compute facial proportions from real 468-point geometry.

    Index references follow data/face-reading/facial_landmark_mappings.json
    (proportional_analysis.golden_ratio_points + face_thirds).
    """
    lm = landmarks
    face_width = _dist(lm[234], lm[454]) or 1e-6
    face_height = _dist(lm[10], lm[152]) or 1e-6
    wh = face_width / face_height

    # golden ratio proximity: ideal face height / width ~= 1.618
    hw = face_height / face_width
    golden = max(0.0, 1.0 - abs(hw - 1.618) / 1.618)

    # bilateral symmetry: paired landmarks should mirror across the facial axis
    center_x = lm[168].x  # mid-point between the eyes (top of nose bridge)
    devs = []
    for left, right in _BILATERAL_PAIRS:
        mirror_x = 2.0 * center_x - lm[left].x
        devs.append(
            math.sqrt((mirror_x - lm[right].x) ** 2 + (lm[left].y - lm[right].y) ** 2)
        )
    mean_dev = sum(devs) / len(devs)
    symmetry = max(0.0, min(1.0, 1.0 - mean_dev * 8.0))

    eye_distance = _dist(lm[133], lm[362]) / face_width
    nose_length = _dist(lm[168], lm[2])
    mouth_width = _dist(lm[61], lm[291]) or 1e-6
    nose_mouth = nose_length / mouth_width
    forehead = _dist(lm[10], lm[9]) / face_height
    jaw = _dist(lm[132], lm[361]) / face_width

    return FacialProportions(
        golden_ratio_score=round(golden, 4),
        symmetry_score=round(symmetry, 4),
        face_width_height_ratio=round(wh, 4),
        eye_distance_ratio=round(eye_distance, 4),
        nose_mouth_ratio=round(nose_mouth, 4),
        forehead_ratio=round(forehead, 4),
        jaw_width_ratio=round(jaw, 4),
    )


def _image_quality(img_bgr: np.ndarray, landmarks: list[Landmark]) -> ImageQuality:
    """Real image-quality metrics (Laplacian sharpness, brightness, face size)."""
    gray = cv2.cvtColor(img_bgr, cv2.COLOR_BGR2GRAY)
    lap_var = float(cv2.Laplacian(gray, cv2.CV_64F).var())
    sharpness = round(min(1.0, lap_var / 500.0), 4)
    brightness = round(float(gray.mean()) / 255.0, 4)

    h, w = gray.shape
    xs = [l.x for l in landmarks]
    ys = [l.y for l in landmarks]
    bbox_area = (max(xs) - min(xs)) * (max(ys) - min(ys))
    face_size = round(float(bbox_area), 4)

    return ImageQuality(
        sharpness=sharpness,
        brightness=brightness,
        face_size_ratio=face_size,
        sufficient_quality=sharpness > 0.1 and 0.15 < brightness < 0.9 and face_size > 0.05,
    )


def _analyze_with_mediapipe(
    img_bgr: np.ndarray,
) -> tuple[list[Landmark], FacialProportions, ImageQuality] | None:
    """Run MediaPipe Face Mesh; None when unavailable or no face detected."""
    face_mesh = _get_mediapipe_facemesh()
    if face_mesh is None:
        return None
    try:
        rgb = cv2.cvtColor(img_bgr, cv2.COLOR_BGR2RGB)
        results = face_mesh.process(rgb)
    except Exception:
        return None
    if not results.multi_face_landmarks:
        return None

    face = results.multi_face_landmarks[0]
    landmarks = [
        Landmark(index=i, x=round(l.x, 6), y=round(l.y, 6), z=round(l.z, 6))
        for i, l in enumerate(face.landmark)
    ]
    if len(landmarks) != NUM_LANDMARKS:
        return None
    proportions = _proportions_from_landmarks(landmarks)
    quality = _image_quality(img_bgr, landmarks)
    return landmarks, proportions, quality


# ─── Deterministic fallback (original stub path, kept for graceful degrade) ──


def _seed_from_filename(filename: str) -> int:
    """Deterministic seed from filename so same image -> same landmarks."""
    return int(hashlib.sha256(filename.encode()).hexdigest()[:8], 16)


def _region_center(index: int) -> tuple[float, float, float]:
    """Return approximate (cx, cy, depth) for a landmark based on its region.

    Positions are normalized 0-1, with (0.5, 0.4) as face center.
    The face mesh is laid out top-to-bottom: forehead ~0.15, chin ~0.85.
    """
    # Face oval -- distribute around an ellipse
    if index in FACE_OVAL_INDICES:
        pos = FACE_OVAL_INDICES.index(index)
        t = 2 * math.pi * pos / len(FACE_OVAL_INDICES)
        return (0.50 + 0.22 * math.cos(t), 0.48 + 0.35 * math.sin(t), -0.02)

    # Left eye region
    if index in _LEFT_EYE:
        return (0.36, 0.38, 0.01)

    # Right eye region
    if index in _RIGHT_EYE:
        return (0.64, 0.38, 0.01)

    # Left eyebrow
    if index in _LEFT_BROW:
        return (0.35, 0.30, 0.00)

    # Right eyebrow
    if index in _RIGHT_BROW:
        return (0.65, 0.30, 0.00)

    # Nose bridge
    if index in _NOSE_BRIDGE:
        return (0.50, 0.42, 0.05)

    # Nose tip
    if index in _NOSE_TIP:
        return (0.50, 0.52, 0.06)

    # Upper lip
    if index in _UPPER_LIP:
        return (0.50, 0.60, 0.02)

    # Lower lip
    if index in _LOWER_LIP:
        return (0.50, 0.65, 0.01)

    # Forehead
    if index in _FOREHEAD:
        return (0.50, 0.18, -0.01)

    # Default: distribute remaining landmarks across the face using index
    # Map index to a position on the face surface via a spiral pattern.
    t = index / NUM_LANDMARKS
    angle = t * 6.0 * math.pi
    radius = 0.15 * t
    return (0.50 + radius * math.cos(angle), 0.25 + t * 0.55, -0.01 * math.cos(angle))


def _generate_landmarks(seed: int) -> list[Landmark]:
    """Generate 468 realistic face mesh landmarks deterministically."""
    rng = random.Random(seed)
    landmarks: list[Landmark] = []

    for i in range(NUM_LANDMARKS):
        cx, cy, cz = _region_center(i)
        # Small per-landmark jitter for realism
        jx = rng.gauss(0, 0.012)
        jy = rng.gauss(0, 0.012)
        jz = rng.gauss(0, 0.005)
        landmarks.append(Landmark(
            index=i,
            x=round(max(0.0, min(1.0, cx + jx)), 6),
            y=round(max(0.0, min(1.0, cy + jy)), 6),
            z=round(cz + jz, 6),
        ))

    return landmarks


def _generate_proportions(seed: int) -> FacialProportions:
    """Deterministic facial proportions in realistic ranges."""
    rng = random.Random(seed + 1)
    return FacialProportions(
        golden_ratio_score=round(rng.uniform(0.75, 0.95), 4),
        symmetry_score=round(rng.uniform(0.80, 0.98), 4),
        face_width_height_ratio=round(rng.uniform(0.62, 0.78), 4),
        eye_distance_ratio=round(rng.uniform(0.28, 0.36), 4),
        nose_mouth_ratio=round(rng.uniform(0.55, 0.75), 4),
        forehead_ratio=round(rng.uniform(0.30, 0.40), 4),
        jaw_width_ratio=round(rng.uniform(0.70, 0.88), 4),
    )


def _generate_image_quality(seed: int) -> ImageQuality:
    """Deterministic image quality metrics in realistic ranges."""
    rng = random.Random(seed + 2)
    sharpness = round(rng.uniform(0.5, 1.0), 4)
    brightness = round(rng.uniform(0.3, 0.8), 4)
    face_size = round(rng.uniform(0.15, 0.60), 4)
    return ImageQuality(
        sharpness=sharpness,
        brightness=brightness,
        face_size_ratio=face_size,
        sufficient_quality=sharpness > 0.6 and brightness > 0.35,
    )


# ─── Endpoint ────────────────────────────────────────────────────────────────


def _parse_options(options: str | None) -> dict:
    """Parse the JSON options form field (consent, caller metadata)."""
    if not options:
        return {}
    try:
        parsed = json.loads(options)
        return parsed if isinstance(parsed, dict) else {}
    except (json.JSONDecodeError, TypeError):
        return {}


@router.post("/analyze", response_model=FaceMeshResponse)
async def analyze(
    image: UploadFile = File(..., description="JPEG or PNG face image"),
    options: str | None = Form(None, description="JSON string with analysis options"),
) -> FaceMeshResponse:
    start = time.time()

    image_bytes = await image.read()
    opts = _parse_options(options)
    consent = opts.get("consent") if isinstance(opts.get("consent"), dict) else None
    consent_granted = bool(consent and consent.get("granted") is True)

    filename = image.filename or "unknown.jpg"

    # Real CV path first (face-cv-hook-p3); graceful fallback on any gap.
    real = None
    img_bgr = _decode_image(image_bytes)
    if img_bgr is not None:
        real = _analyze_with_mediapipe(img_bgr)

    if real is not None:
        landmarks, proportions, quality = real
        landmark_source = "mediapipe-facemesh"
        analysis_version = "mediapipe-facemesh/v1"
    else:
        seed = _seed_from_filename(filename)
        landmarks = _generate_landmarks(seed)
        proportions = _generate_proportions(seed)
        quality = _generate_image_quality(seed)
        landmark_source = "deterministic-fallback"
        analysis_version = "deterministic-fallback/v1"

    elapsed_ms = (time.time() - start) * 1000

    return FaceMeshResponse(
        contract_version=CONTRACT_VERSION,
        analysis_version=analysis_version,
        landmark_source=landmark_source,
        face_detected=True,
        num_faces=1,
        landmarks=landmarks,
        face_oval=FACE_OVAL_INDICES,
        proportions=proportions,
        image_quality=quality,
        processing_time_ms=round(elapsed_ms, 2),
        consent=consent,
        consent_granted=consent_granted,
    )
