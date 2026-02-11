"""Stub /analyze endpoint for MediaPipe Face Mesh service.

Returns realistic mock landmark data generated algorithmically.
Sprint 5 will replace with real MediaPipe inference.
"""

import hashlib
import math
import random
import time

from fastapi import APIRouter, File, Form, UploadFile

from shared.models import (
    FaceMeshResponse,
    FacialProportions,
    ImageQuality,
    Landmark,
)

router = APIRouter()

NUM_LANDMARKS = 468

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


@router.post("/analyze", response_model=FaceMeshResponse)
async def analyze(
    image: UploadFile = File(..., description="JPEG or PNG face image"),
    options: str | None = Form(None, description="JSON string with analysis options"),
) -> FaceMeshResponse:
    start = time.time()

    # Consume the upload so the connection closes cleanly
    await image.read()

    filename = image.filename or "unknown.jpg"
    seed = _seed_from_filename(filename)

    landmarks = _generate_landmarks(seed)
    proportions = _generate_proportions(seed)
    quality = _generate_image_quality(seed)
    elapsed_ms = (time.time() - start) * 1000

    return FaceMeshResponse(
        face_detected=True,
        num_faces=1,
        landmarks=landmarks,
        face_oval=FACE_OVAL_INDICES,
        proportions=proportions,
        image_quality=quality,
        processing_time_ms=round(elapsed_ms, 2),
    )
