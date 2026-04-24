"""Stub /analyze endpoint for Biofield CV service.

Returns deterministic mock spatial metrics seeded by image filename.
Sprint 4 will replace with real OpenCV algorithms.
"""

import hashlib
import json
import random
import time
from typing import Any

from fastapi import APIRouter, File, Form, HTTPException, UploadFile, status

from shared.models import (
    BiofieldCVResponse,
    EnergyBands,
    QualityAssessment,
    SpatialMetrics,
)

router = APIRouter()

CONTRACT_VERSION = "biofield-cv/v1"
ANALYSIS_VERSION = "stub-metrics/v1"

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


def _seed_from_filename(filename: str) -> int:
    """Derive a deterministic PRNG seed from the image filename."""
    return int(hashlib.sha256(filename.encode()).hexdigest()[:8], 16)


def _mock_metrics(seed: int) -> SpatialMetrics:
    """Generate varied but deterministic metrics from a seed."""
    rng = random.Random(seed)

    # Energy bands that sum to 1.0
    low = rng.uniform(0.15, 0.45)
    high = rng.uniform(0.10, 0.35)
    medium = 1.0 - low - high

    return SpatialMetrics(
        light_quanta_density=round(rng.uniform(15.0, 85.0), 2),
        normalized_area=round(rng.uniform(0.20, 0.95), 4),
        average_intensity=round(rng.uniform(0.15, 0.90), 4),
        inner_noise=round(rng.uniform(0.02, 0.30), 4),
        energy_analysis=EnergyBands(
            low=round(low, 4),
            medium=round(medium, 4),
            high=round(high, 4),
            total=1.0,
        ),
        entropy_form_coefficient=round(rng.uniform(1.5, 6.5), 3),
        fractal_dimension=round(rng.uniform(1.05, 1.95), 4),
        correlation_dimension=round(rng.uniform(0.8, 3.5), 4),
        body_symmetry=round(rng.uniform(-0.8, 0.9), 4),
        contour_complexity=round(rng.uniform(0.5, 5.0), 3),
        pattern_regularity=round(rng.uniform(0.10, 0.95), 4),
    )


def _mock_quality(image_size_bytes: int) -> QualityAssessment:
    """Quality assessment that scales with image size."""
    if image_size_bytes < 100_000:
        # Small image: lower quality
        tier = (0.35, 0.40, 0.18, 0.30, False)
    elif image_size_bytes < 500_000:
        # Medium image: moderate quality
        tier = (0.65, 0.60, 0.10, 0.50, True)
    else:
        # Large image: higher quality
        tier = (0.90, 0.80, 0.04, 0.60, True)

    return QualityAssessment(
        sharpness=tier[0],
        contrast=tier[1],
        noise_level=tier[2],
        exposure=tier[3],
        sufficient_quality=tier[4],
    )


def _unprocessable(detail: str) -> HTTPException:
    return HTTPException(status_code=status.HTTP_422_UNPROCESSABLE_CONTENT, detail=detail)


def _parse_algorithms(raw: str | None) -> list[str]:
    """Parse and validate the algorithms filter. Returns ALL_ALGORITHMS if None."""
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
    """Parse optional JSON object form fields and reject malformed or non-object values."""
    if raw is None:
        return None

    try:
        parsed = json.loads(raw)
    except json.JSONDecodeError as exc:
        raise _unprocessable(f"{field_name} must be valid JSON: {exc.msg}") from exc

    if not isinstance(parsed, dict):
        raise _unprocessable(f"{field_name} must be a JSON object")

    return parsed


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
    filename = image.filename or "unknown"

    algorithms_run = _parse_algorithms(algorithms)
    _parse_json_object(options, "options")
    _parse_json_object(capture_metadata, "capture_metadata")

    seed = _seed_from_filename(filename)
    metrics = _mock_metrics(seed)
    quality = _mock_quality(len(image_bytes))
    elapsed_ms = (time.time() - start) * 1000

    return BiofieldCVResponse(
        contract_version=CONTRACT_VERSION,
        analysis_version=ANALYSIS_VERSION,
        metrics=metrics,
        quality_assessment=quality,
        algorithms_run=algorithms_run,
        processing_time_ms=round(elapsed_ms, 2),
    )
