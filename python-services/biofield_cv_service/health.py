"""Health endpoint for Biofield CV service."""

from fastapi import APIRouter

from shared.models import HealthResponse, RuntimeCapability
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


def _capability_availability(
    *,
    opencv_available: bool,
    numpy_available: bool,
    mediapipe_available: bool,
) -> str:
    if not opencv_available or not numpy_available:
        return "unavailable"
    if not mediapipe_available:
        return "degraded"
    return "available"


@router.get("/health", response_model=dict)
def health() -> dict:
    opencv_available = _check_opencv()
    numpy_available = _check_numpy()
    mediapipe_available = _check_mediapipe()
    availability = _capability_availability(
        opencv_available=opencv_available,
        numpy_available=numpy_available,
        mediapipe_available=mediapipe_available,
    )
    capability = RuntimeCapability(
        engine_id="biofield-cv",
        display_name="Biofield CV",
        availability=availability,
        dependencies=["opencv", "numpy", "mediapipe"],
    )
    resp = HealthResponse(
        status="healthy" if availability == "available" else "degraded",
        service="biofield-cv",
        version=SERVICE_VERSION,
    )
    return {
        **resp.model_dump(),
        "opencv_available": opencv_available,
        "numpy_available": numpy_available,
        "mediapipe_available": mediapipe_available,
        "capabilities": [capability.model_dump()],
    }
