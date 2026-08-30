"""Shared Pydantic models for Selemene Python services."""

from __future__ import annotations

from typing import Literal

from pydantic import BaseModel, Field
from shared.version import SERVICE_VERSION


# ---------- Common ----------

CapabilityStatus = Literal["available", "degraded", "unavailable"]


class HealthResponse(BaseModel):
    status: str = "healthy"
    service: str
    version: str = SERVICE_VERSION
    capability_status: CapabilityStatus = Field(
        description=(
            "Explicit sidecar capability state, computed only from local "
            "self-check booleans (no provider/network/database calls)."
        ),
    )


# ---------- MediaPipe service models ----------

class Landmark(BaseModel):
    index: int = Field(ge=0, le=467)
    x: float = Field(description="Normalized x (0-1)")
    y: float = Field(description="Normalized y (0-1)")
    z: float = Field(description="Depth estimate")


class FacialProportions(BaseModel):
    golden_ratio_score: float = Field(ge=0, le=1)
    symmetry_score: float = Field(ge=0, le=1)
    face_width_height_ratio: float
    eye_distance_ratio: float
    nose_mouth_ratio: float
    forehead_ratio: float
    jaw_width_ratio: float


class ImageQuality(BaseModel):
    sharpness: float
    brightness: float
    face_size_ratio: float = Field(description="Face area vs image area")
    sufficient_quality: bool


# face-cv-hook-p3 (T-027 landmark hook -> real CV): contract fields added per
# FROZEN (P1W1-CONTRACTS-FROZEN.md face example) + T-065 biofield pattern
# (contract_version / analysis_version / consent echo).
# phase:integration-p1 wave:integration-w2 engine-face-reading
class FaceMeshResponse(BaseModel):
    contract_version: str = Field(default="face-cv/v1")
    analysis_version: str = Field(
        default="deterministic-fallback/v1",
        description="mediapipe-facemesh/v1 when real CV ran, deterministic-fallback/v1 otherwise",
    )
    landmark_source: str = Field(
        default="deterministic-fallback",
        description="mediapipe-facemesh | deterministic-fallback",
    )
    face_detected: bool
    num_faces: int | None = None
    landmarks: list[Landmark]
    face_oval: list[int] | None = None
    proportions: FacialProportions | None = None
    processing_time_ms: float
    image_quality: ImageQuality | None = None
    consent: dict | None = Field(
        default=None,
        description="Consent object echoed from caller (local-first invariant, goal-understanding.md)",
    )
    consent_granted: bool = False


# ---------- Biofield CV service models ----------

class EnergyBands(BaseModel):
    low: float
    medium: float
    high: float
    total: float


class SpatialMetrics(BaseModel):
    light_quanta_density: float = Field(description="Quanta per unit area")
    normalized_area: float = Field(ge=0, le=1)
    average_intensity: float = Field(ge=0, le=1)
    inner_noise: float = Field(description="Variance coefficient")
    energy_analysis: EnergyBands
    entropy_form_coefficient: float = Field(description="Shannon entropy in bits")
    fractal_dimension: float = Field(ge=1, le=2)
    correlation_dimension: float
    body_symmetry: float = Field(ge=-1, le=1)
    contour_complexity: float
    pattern_regularity: float = Field(ge=0, le=1)


class QualityAssessment(BaseModel):
    sharpness: float
    contrast: float
    noise_level: float
    exposure: float
    sufficient_quality: bool


class BiofieldCVResponse(BaseModel):
    contract_version: str = Field(default="biofield-cv/v1")
    analysis_version: str
    metrics: SpatialMetrics
    quality_assessment: QualityAssessment
    algorithms_run: list[str]
    processing_time_ms: float
