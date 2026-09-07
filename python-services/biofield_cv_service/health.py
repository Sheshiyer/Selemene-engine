"""Health endpoint for Biofield CV service."""

from fastapi import APIRouter

from shared.models import CapabilityObservation, DependencyObservation, HealthResponse
from shared.version import SERVICE_VERSION

router = APIRouter()


def _check_opencv() -> bool:
    try:
        import cv2  # noqa: F401
        return True
    except Exception:
        return False


def _check_numpy() -> bool:
    try:
        import numpy  # noqa: F401
        return True
    except Exception:
        return False


def _check_mediapipe() -> bool:
    try:
        import mediapipe  # noqa: F401
        return True
    except Exception:
        return False


def _safe_check(checker) -> bool:
    try:
        return bool(checker())
    except Exception:
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


def _dependency_observations(
    opencv_available: bool, numpy_available: bool, mediapipe_available: bool
) -> CapabilityObservation:
    required_available = opencv_available and numpy_available
    if not required_available:
        availability = "unavailable"
        reason_code = "REQUIRED_DEPENDENCY_UNAVAILABLE"
    elif not mediapipe_available:
        availability = "degraded"
        reason_code = "OPTIONAL_DEPENDENCY_UNAVAILABLE"
    else:
        availability = "available"
        reason_code = "CAPABILITY_AVAILABLE"
    return CapabilityObservation(
        engine_id="biofield",
        availability=availability,
        reason_code=reason_code,
        dependency_observations=[
            DependencyObservation(
                dependency_id="python:biofield-cv",
                dependency_kind="python",
                requirement="required",
                availability="available" if required_available else "unavailable",
                reason_code="CAPABILITY_AVAILABLE"
                if required_available
                else "MODULE_UNAVAILABLE",
            ),
            DependencyObservation(
                dependency_id="python:mediapipe-face-mesh",
                dependency_kind="python",
                requirement="optional",
                availability="available" if mediapipe_available else "unavailable",
                reason_code="CAPABILITY_AVAILABLE"
                if mediapipe_available
                else "MODULE_UNAVAILABLE",
            ),
        ],
    )


@router.get("/health", response_model=dict)
def health() -> dict:
    opencv_available = _safe_check(_check_opencv)
    numpy_available = _safe_check(_check_numpy)
    mediapipe_available = _safe_check(_check_mediapipe)
    resp = HealthResponse(
        status="healthy",
        service="biofield-cv",
        version=SERVICE_VERSION,
        capability_status=_capability_status(opencv_available, numpy_available, mediapipe_available),
        capability_observations=[
            _dependency_observations(opencv_available, numpy_available, mediapipe_available)
        ],
    )
    return {
        **resp.model_dump(),
        "opencv_available": opencv_available,
        "numpy_available": numpy_available,
        "mediapipe_available": mediapipe_available,
    }
