"""Health endpoint for MediaPipe Face Mesh service."""

from fastapi import APIRouter

from shared.models import HealthResponse

router = APIRouter()


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
        service="mediapipe-face-mesh",
        version="3.0.0",
    )
    return {
        **resp.model_dump(),
        "mediapipe_available": _check_mediapipe(),
    }
