"""Health endpoint for MediaPipe Face Mesh service."""

from fastapi import APIRouter

from shared.models import CapabilityObservation, DependencyObservation, HealthResponse
from shared.version import SERVICE_VERSION

router = APIRouter()


def _check_mediapipe() -> bool:
    try:
        import mediapipe  # noqa: F401
        return True
    except Exception:
        return False


def _dependency_observations(mediapipe_available: bool) -> CapabilityObservation:
    return CapabilityObservation(
        engine_id="face-reading",
        availability="available" if mediapipe_available else "unavailable",
        reason_code="CAPABILITY_AVAILABLE"
        if mediapipe_available
        else "REQUIRED_DEPENDENCY_UNAVAILABLE",
        dependency_observations=[
            DependencyObservation(
                dependency_id="python:mediapipe-face-mesh",
                dependency_kind="python",
                requirement="required",
                availability="available" if mediapipe_available else "unavailable",
                reason_code="CAPABILITY_AVAILABLE"
                if mediapipe_available
                else "MODULE_UNAVAILABLE",
            )
        ],
    )


@router.get("/health", response_model=dict)
def health() -> dict:
    mediapipe_available = _check_mediapipe()
    resp = HealthResponse(
        status="healthy",
        service="mediapipe-face-mesh",
        version=SERVICE_VERSION,
        capability_status="available" if mediapipe_available else "unavailable",
        capability_observations=[_dependency_observations(mediapipe_available)],
    )
    return {
        **resp.model_dump(),
        "mediapipe_available": mediapipe_available,
    }
