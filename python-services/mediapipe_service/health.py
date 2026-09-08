"""Health endpoint for MediaPipe Face Mesh service."""

from fastapi import APIRouter

from shared.models import HealthResponse
from shared.version import SERVICE_VERSION

router = APIRouter()


def _check_mediapipe() -> bool:
    try:
        import mediapipe  # noqa: F401
        return True
    except ImportError:
        return False


@router.get("/health", response_model=dict)
def health() -> dict:
    mediapipe_available = _check_mediapipe()
    resp = HealthResponse(
        status="healthy",
        service="mediapipe-face-mesh",
        version=SERVICE_VERSION,
        capability_status="available" if mediapipe_available else "unavailable",
    )
    return {
        **resp.model_dump(),
        "mediapipe_available": mediapipe_available,
    }
