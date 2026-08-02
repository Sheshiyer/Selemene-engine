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


@router.get("/health", response_model=dict)
def health() -> dict:
    resp = HealthResponse(
        status="healthy",
        service="biofield-cv",
        version=SERVICE_VERSION,
    )
    return {
        **resp.model_dump(),
        "opencv_available": _check_opencv(),
        "numpy_available": _check_numpy(),
        "mediapipe_available": _check_mediapipe(),
    }
