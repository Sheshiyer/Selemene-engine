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


def _check_facemesh() -> tuple[bool, str | None]:
    """True only when the Face Mesh graph actually initialises.

    Importing mediapipe is not enough: mediapipe 1.x imports fine but has no
    legacy solutions API, so the analyze route would silently fall back.
    """
    from mediapipe_service.analyze import _get_mediapipe_facemesh, facemesh_init_error

    mesh = _get_mediapipe_facemesh()
    return mesh is not None, facemesh_init_error()


@router.get("/health", response_model=dict)
def health() -> dict:
    mediapipe_available = _check_mediapipe()
    facemesh_ready, facemesh_error = _check_facemesh() if mediapipe_available else (False, None)
    if facemesh_ready:
        capability_status = "available"
    elif mediapipe_available:
        capability_status = "degraded"
    else:
        capability_status = "unavailable"
    resp = HealthResponse(
        status="healthy",
        service="mediapipe-face-mesh",
        version=SERVICE_VERSION,
        capability_status=capability_status,
    )
    return {
        **resp.model_dump(),
        "mediapipe_available": mediapipe_available,
        "facemesh_ready": facemesh_ready,
        "facemesh_error": facemesh_error,
    }
