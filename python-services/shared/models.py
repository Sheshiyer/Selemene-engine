"""Shared Pydantic models for Selemene Python services."""

from __future__ import annotations

from pydantic import BaseModel, Field


# ---------- Common ----------

class HealthResponse(BaseModel):
    status: str = "healthy"
    service: str
    version: str = "3.0.0"


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


class FaceMeshResponse(BaseModel):
    face_detected: bool
    num_faces: int | None = None
    landmarks: list[Landmark]
    face_oval: list[int] | None = None
    proportions: FacialProportions | None = None
    processing_time_ms: float
    image_quality: ImageQuality | None = None


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
