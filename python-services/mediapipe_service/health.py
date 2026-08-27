"""Health endpoint for MediaPipe Face Mesh service."""

from fastapi import APIRouter

from shared.models import HealthResponse, RuntimeCapability
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
    capability = RuntimeCapability(
        engine_id="mediapipe-face-mesh",
        display_name="MediaPipe Face Mesh",
        availability="available" if mediapipe_available else "unavailable",
        dependencies=["mediapipe"],
    )
    resp = HealthResponse(
        status="healthy" if mediapipe_available else "degraded",
        service="mediapipe-face-mesh",
        version=SERVICE_VERSION,
    )
    return {
        **resp.model_dump(),
        "mediapipe_available": mediapipe_available,
        "capabilities": [capability.model_dump()],
    }
