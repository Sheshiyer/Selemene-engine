"""Health endpoint for Biofield CV service."""

from fastapi import APIRouter

from shared.models import HealthResponse
from shared.version import SERVICE_VERSION

router = APIRouter()


def _check_opencv() -> bool:
    try:
        import cv2  # noqa: F401
        return True
    except ImportError:
        return False


def _check_numpy() -> bool:
    try:
        import numpy  # noqa: F401
        return True
    except ImportError:
        return False


def _check_mediapipe() -> bool:
    try:
        import mediapipe  # noqa: F401
        return True
    except ImportError:
        return False


def _capability_status(opencv_available: bool, numpy_available: bool, mediapipe_available: bool) -> str:
    """biofield-cv is "available" only if both opencv and numpy are available
    (its two hard dependencies); "degraded" if opencv+numpy are up but
    mediapipe is missing; "unavailable" if opencv or numpy is missing."""
    if not (opencv_available and numpy_available):
        return "unavailable"
    if not mediapipe_available:
        return "degraded"
    return "available"


@router.get("/health", response_model=dict)
def health() -> dict:
    opencv_available = _check_opencv()
    numpy_available = _check_numpy()
    mediapipe_available = _check_mediapipe()
    resp = HealthResponse(
        status="healthy",
        service="biofield-cv",
        version=SERVICE_VERSION,
        capability_status=_capability_status(opencv_available, numpy_available, mediapipe_available),
    )
    return {
        **resp.model_dump(),
        "opencv_available": opencv_available,
        "numpy_available": numpy_available,
        "mediapipe_available": mediapipe_available,
    }
