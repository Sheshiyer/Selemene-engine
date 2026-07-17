//! Face CV landmark client + landmark → FaceAnalysis mapping (face-cv-hook-p3).
//!
//! Wires the T-027 `landmark_hook` placeholder to the python mediapipe face
//! service (`python-services/mediapipe_service`, port 8001) following the T-065
//! biofield pattern (config-driven URL, graceful fallback, consent echo).
//!
//! Cites: p1-w1-worker-bootstrap-packet.md + resources-and-assets.md +
//! gaps-and-improvements.md (face: no landmark detection / no real CV) +
//! goal-understanding.md (local-first, explicit consent before backend) +
//! P1W1-CONTRACTS-FROZEN.md (face image_data + consent example) +
//! detailed-task-list.md (T-027) + EXECUTION-STATUS.md +
//! data/face-reading/facial_landmark_mappings.json (five_elements_landmarks,
//! proportional_analysis) + python-services/mediapipe_service/analyze.py.
//! Tags: phase:integration-p1 wave:integration-w2 engine-face-reading

use std::collections::HashMap;
use std::time::Duration;

use base64::Engine as _;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::models::{
    BodyType, ConstitutionAnalysis, Dosha, Element, ElementalBalance, FaceAnalysis, FaceZone,
    HealthIndicator, PersonalityTrait,
};

/// Default base URL for the local mediapipe face-mesh sidecar (PYTHON_SIDECAR_GUIDE.md).
pub const DEFAULT_FACE_CV_URL: &str = "http://127.0.0.1:8001";
/// Default per-call timeout; kept short so fallback stays snappy in tests/CI.
pub const DEFAULT_FACE_CV_TIMEOUT_MS: u64 = 2_000;

/// Config for the face CV sidecar. Resolved per-call (options override env).
#[derive(Debug, Clone)]
pub struct FaceCvConfig {
    pub base_url: String,
    pub timeout: Duration,
}

impl FaceCvConfig {
    /// Resolve config: `options["face_cv_url"]` → env `SELEMENE_FACE_CV_URL` → default.
    /// Returns None when explicitly disabled via `SELEMENE_FACE_CV_DISABLED` (truthy)
    /// or `options["face_cv_disabled"] == true`.
    pub fn resolve(options: &HashMap<String, Value>) -> Option<Self> {
        let disabled_opt = options
            .get("face_cv_disabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let disabled_env = std::env::var("SELEMENE_FACE_CV_DISABLED")
            .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "yes"))
            .unwrap_or(false);
        if disabled_opt || disabled_env {
            return None;
        }

        let base_url = options
            .get("face_cv_url")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| std::env::var("SELEMENE_FACE_CV_URL").ok())
            .unwrap_or_else(|| DEFAULT_FACE_CV_URL.to_string());

        let timeout_ms = options
            .get("face_cv_timeout_ms")
            .and_then(|v| v.as_u64())
            .or_else(|| {
                std::env::var("SELEMENE_FACE_CV_TIMEOUT_MS")
                    .ok()
                    .and_then(|v| v.parse::<u64>().ok())
            })
            .unwrap_or(DEFAULT_FACE_CV_TIMEOUT_MS);

        Some(Self {
            base_url,
            timeout: Duration::from_millis(timeout_ms),
        })
    }
}

/// One 3D landmark from the face service.
#[derive(Debug, Clone, Deserialize)]
pub struct CvLandmark {
    pub index: usize,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Facial proportions as returned by the face service.
#[derive(Debug, Clone, Deserialize)]
pub struct CvProportions {
    pub golden_ratio_score: f64,
    pub symmetry_score: f64,
    pub face_width_height_ratio: f64,
    pub eye_distance_ratio: f64,
    pub nose_mouth_ratio: f64,
    pub forehead_ratio: f64,
    pub jaw_width_ratio: f64,
}

/// Image quality as returned by the face service.
#[derive(Debug, Clone, Deserialize)]
pub struct CvImageQuality {
    pub sharpness: f64,
    pub brightness: f64,
    pub face_size_ratio: f64,
    pub sufficient_quality: bool,
}

/// Response envelope for POST /analyze (shared/models.py FaceMeshResponse).
#[derive(Debug, Clone, Deserialize)]
pub struct FaceCvResponse {
    pub face_detected: bool,
    pub landmarks: Vec<CvLandmark>,
    pub landmark_source: String,
    pub analysis_version: String,
    pub proportions: Option<CvProportions>,
    pub image_quality: Option<CvImageQuality>,
    pub processing_time_ms: f64,
}

/// POST image bytes to the face CV service; returns parsed landmarks response.
/// Multipart shape mirrors PythonServiceClient::analyze_image (noesis-bridge)
/// and BiofieldClient::analyze_capture (noesis-api) from the T-065 pattern.
pub async fn fetch_face_landmarks(
    config: &FaceCvConfig,
    image_bytes: Vec<u8>,
    mime_type: &str,
    consent: Option<&Value>,
) -> Result<FaceCvResponse, String> {
    let client = reqwest::Client::builder()
        .timeout(config.timeout)
        .build()
        .map_err(|e| format!("face-cv client build failed: {e}"))?;

    let ext = if mime_type.contains("png") {
        "png"
    } else {
        "jpg"
    };
    let file_part = reqwest::multipart::Part::bytes(image_bytes)
        .file_name(format!("face.{ext}"))
        .mime_str(mime_type)
        .map_err(|e| format!("face-cv multipart mime failed: {e}"))?;

    let options_json = json!({
        "consent": consent.cloned().unwrap_or(Value::Null),
        "source": "engine-face-reading",
        "hook": "face-cv-hook-p3",
    })
    .to_string();

    let form = reqwest::multipart::Form::new()
        .part("image", file_part)
        .text("options", options_json);

    let url = format!("{}/analyze", config.base_url.trim_end_matches('/'));
    let response = client
        .post(&url)
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("face-cv request failed: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("face-cv status {}", response.status()));
    }

    response
        .json::<FaceCvResponse>()
        .await
        .map_err(|e| format!("face-cv decode failed: {e}"))
}

/// Whether a service response carries real (non-fallback) landmarks we trust.
pub fn is_real_landmark_response(resp: &FaceCvResponse) -> bool {
    resp.face_detected
        && resp.landmark_source == "mediapipe-facemesh"
        && resp.landmarks.len() >= 468
}

/// Best-effort base64 decode of the FROZEN `image_data.b64` payload; falls back
/// to the raw bytes when it is not valid base64 (e.g. truncated FROZEN sample).
pub fn decode_image_bytes(raw: &[u8]) -> Vec<u8> {
    base64::engine::general_purpose::STANDARD
        .decode(raw)
        .unwrap_or_else(|_| raw.to_vec())
}

// ─── Landmark → zone/element mapping ─────────────────────────────────────────
// Index sets from data/face-reading/facial_landmark_mappings.json
// (mediapipe_landmarks.five_elements_landmarks), FROZEN reference data.

const WOOD_LANDMARKS: [usize; 14] = [
    9, 10, 151, 337, 299, 333, 298, 301, 175, 199, 200, 16, 17, 18,
];
const FIRE_LANDMARKS: [usize; 16] = [
    33, 7, 163, 144, 145, 153, 154, 155, 133, 173, 175, 199, 200, 16, 17, 18,
];
const EARTH_LANDMARKS: [usize; 16] = [
    116, 117, 118, 119, 120, 121, 128, 126, 142, 36, 205, 206, 207, 213, 192, 147,
];
const METAL_LANDMARKS: [usize; 15] = [1, 2, 5, 4, 6, 19, 20, 94, 125, 51, 48, 115, 131, 134, 102];
const WATER_LANDMARKS: [usize; 14] = [
    175, 199, 200, 16, 17, 18, 116, 117, 118, 119, 120, 121, 128, 126,
];

fn lm(landmarks: &[CvLandmark], index: usize) -> Option<&CvLandmark> {
    landmarks.iter().find(|l| l.index == index)
}

fn dist(a: &CvLandmark, b: &CvLandmark) -> f64 {
    ((a.x - b.x).powi(2) + (a.y - b.y).powi(2) + (a.z - b.z).powi(2)).sqrt()
}

/// Normalized spread of a landmark region relative to the full face bounding box,
/// blended with the region's left/right balance around the facial axis.
fn region_score(landmarks: &[CvLandmark], indices: &[usize], symmetry: f64) -> f64 {
    let points: Vec<&CvLandmark> = indices.iter().filter_map(|i| lm(landmarks, *i)).collect();
    if points.is_empty() {
        return 0.1;
    }

    let (top, bottom, left, right) = (
        lm(landmarks, 10),
        lm(landmarks, 152),
        lm(landmarks, 234),
        lm(landmarks, 454),
    );
    let (face_h, face_w) = match (top, bottom, left, right) {
        (Some(t), Some(b), Some(l), Some(r)) => (dist(t, b).max(1e-6), dist(l, r).max(1e-6)),
        _ => (1.0, 1.0),
    };

    let min_x = points.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
    let max_x = points.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max);
    let min_y = points.iter().map(|p| p.y).fold(f64::INFINITY, f64::min);
    let max_y = points.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max);
    let spread = (((max_x - min_x) / face_w) * ((max_y - min_y) / face_h)).clamp(0.0, 1.0);

    let center_x = lm(landmarks, 168).map(|c| c.x).unwrap_or(0.5);
    let mean_x = points.iter().map(|p| p.x).sum::<f64>() / points.len() as f64;
    let balance = (1.0 - (mean_x - center_x).abs() * 2.0).clamp(0.0, 1.0);

    (0.45 * spread + 0.35 * balance + 0.20 * symmetry).clamp(0.05, 1.0)
}

fn element_to_dosha(element: Element) -> Dosha {
    match element {
        Element::Fire => Dosha::Pitta,
        Element::Earth | Element::Water => Dosha::Kapha,
        Element::Wood | Element::Metal => Dosha::Vata,
    }
}

/// Map a real 468-landmark service response to FaceAnalysis zones/indicators
/// per FROZEN mappings (five elements, face thirds, proportional analysis).
pub fn analysis_from_landmarks(resp: &FaceCvResponse) -> FaceAnalysis {
    let symmetry = resp
        .proportions
        .as_ref()
        .map(|p| p.symmetry_score)
        .unwrap_or(0.5);

    let mut balance = ElementalBalance {
        wood: region_score(&resp.landmarks, &WOOD_LANDMARKS, symmetry),
        fire: region_score(&resp.landmarks, &FIRE_LANDMARKS, symmetry),
        earth: region_score(&resp.landmarks, &EARTH_LANDMARKS, symmetry),
        metal: region_score(&resp.landmarks, &METAL_LANDMARKS, symmetry),
        water: region_score(&resp.landmarks, &WATER_LANDMARKS, symmetry),
    };
    balance.normalize();
    let dominant = balance.dominant();

    let (primary_dosha, body_type) = match &resp.proportions {
        Some(p) => {
            let dosha = if p.face_width_height_ratio < 0.62 {
                Dosha::Vata // long, narrow frame
            } else if p.face_width_height_ratio > 0.78 {
                Dosha::Kapha // broad, settled frame
            } else {
                Dosha::Pitta // balanced, focused frame
            };
            let body = if p.jaw_width_ratio > 0.82 {
                BodyType::Endomorph
            } else if p.jaw_width_ratio < 0.72 {
                BodyType::Ectomorph
            } else {
                BodyType::Mesomorph
            };
            (dosha, body)
        }
        None => (element_to_dosha(dominant), BodyType::Mesomorph),
    };

    let constitution = ConstitutionAnalysis {
        primary_dosha,
        secondary_dosha: Some(element_to_dosha(dominant)),
        tcm_element: dominant,
        body_type,
    };

    let mut personality_indicators = Vec::new();
    if let Some(p) = &resp.proportions {
        personality_indicators.push(PersonalityTrait {
            trait_name: if p.symmetry_score > 0.85 {
                "Structural Balance".to_string()
            } else {
                "Adaptive Asymmetry".to_string()
            },
            facial_indicator: format!(
                "bilateral landmark symmetry {:.2} across 28 canonical pairs",
                p.symmetry_score
            ),
            description: "Measured left/right landmark mirroring reflects how evenly the \
                          nervous system distributes tone across the face."
                .to_string(),
        });
        personality_indicators.push(PersonalityTrait {
            trait_name: if (0.30..=0.36).contains(&p.eye_distance_ratio) {
                "Measured Focus".to_string()
            } else {
                "Peripheral Awareness".to_string()
            },
            facial_indicator: format!(
                "interocular distance ratio {:.2} relative to face width",
                p.eye_distance_ratio
            ),
            description: "Eye-set spacing relative to face width is a classic Mian Xiang \
                          indicator for attention style."
                .to_string(),
        });
        personality_indicators.push(PersonalityTrait {
            trait_name: "Harmonic Proportion".to_string(),
            facial_indicator: format!(
                "golden-ratio proximity {:.2} (height/width)",
                p.golden_ratio_score
            ),
            description: "Proximity of face length-to-width to the golden section, used in \
                          Western physiognomy as a coherence marker."
                .to_string(),
        });
    }

    let mut health_indicators = Vec::new();
    if let Some(p) = &resp.proportions {
        health_indicators.push(HealthIndicator {
            zone: FaceZone::Forehead,
            associated_organ: "Nervous System".to_string(),
            observation: format!(
                "Forehead third ratio {:.2} with wood-region spread {:.2}; observed via landmarks 9/10/151/337.",
                p.forehead_ratio, balance.wood
            ),
        });
        health_indicators.push(HealthIndicator {
            zone: FaceZone::Eyes,
            associated_organ: "Liver/Stress Axis".to_string(),
            observation: format!(
                "Periorbital symmetry {:.2} and eye spacing {:.2} indicate current pacing load on the fire region.",
                p.symmetry_score, p.eye_distance_ratio
            ),
        });
        health_indicators.push(HealthIndicator {
            zone: FaceZone::Nose,
            associated_organ: "Heart/Small Intestine".to_string(),
            observation: format!(
                "Nose-to-mouth proportion {:.2} with metal-region score {:.2}; bridge landmarks 1/2/5 read for tension.",
                p.nose_mouth_ratio, balance.metal
            ),
        });
        health_indicators.push(HealthIndicator {
            zone: FaceZone::Jawline,
            associated_organ: "Kidneys/Bladder".to_string(),
            observation: format!(
                "Jaw width ratio {:.2} with water-region score {:.2}; lower-face landmarks 175/199/200 read for grounding.",
                p.jaw_width_ratio, balance.water
            ),
        });
    }

    let quality = resp.image_quality.as_ref().map(|q| {
        json!({
            "sharpness": q.sharpness,
            "brightness": q.brightness,
            "face_size_ratio": q.face_size_ratio,
            "sufficient_quality": q.sufficient_quality,
            "source": "mediapipe-face-cv",
        })
    });

    FaceAnalysis {
        constitution,
        personality_indicators,
        elemental_balance: balance,
        health_indicators,
        is_mock_data: false,
        consent: None,
        quality,
    }
}

/// Compact JSON summary of the CV pass for the engine result envelope.
pub fn landmark_summary_json(resp: &FaceCvResponse) -> Value {
    json!({
        "landmark_source": resp.landmark_source,
        "analysis_version": resp.analysis_version,
        "num_landmarks": resp.landmarks.len(),
        "proportions": resp.proportions.as_ref().map(|p| json!({
            "golden_ratio_score": p.golden_ratio_score,
            "symmetry_score": p.symmetry_score,
            "face_width_height_ratio": p.face_width_height_ratio,
            "eye_distance_ratio": p.eye_distance_ratio,
            "nose_mouth_ratio": p.nose_mouth_ratio,
            "forehead_ratio": p.forehead_ratio,
            "jaw_width_ratio": p.jaw_width_ratio,
        })),
        "image_quality": resp.image_quality.as_ref().map(|q| json!({
            "sharpness": q.sharpness,
            "brightness": q.brightness,
            "face_size_ratio": q.face_size_ratio,
            "sufficient_quality": q.sufficient_quality,
        })),
        "processing_time_ms": resp.processing_time_ms,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synthetic_response() -> FaceCvResponse {
        // 468 synthetic landmarks on a rough face grid (deterministic).
        let landmarks = (0..468)
            .map(|i| CvLandmark {
                index: i,
                x: 0.5 + 0.2 * ((i as f64 * 0.37).sin()),
                y: 0.15 + 0.7 * (i as f64 / 468.0),
                z: 0.01 * (i as f64 * 0.11).cos(),
            })
            .collect();
        FaceCvResponse {
            face_detected: true,
            landmarks,
            landmark_source: "mediapipe-facemesh".to_string(),
            analysis_version: "mediapipe-facemesh/v1".to_string(),
            proportions: Some(CvProportions {
                golden_ratio_score: 0.88,
                symmetry_score: 0.9,
                face_width_height_ratio: 0.7,
                eye_distance_ratio: 0.33,
                nose_mouth_ratio: 0.65,
                forehead_ratio: 0.35,
                jaw_width_ratio: 0.78,
            }),
            image_quality: Some(CvImageQuality {
                sharpness: 0.8,
                brightness: 0.55,
                face_size_ratio: 0.4,
                sufficient_quality: true,
            }),
            processing_time_ms: 12.0,
        }
    }

    #[test]
    fn analysis_from_landmarks_produces_balanced_elements() {
        let resp = synthetic_response();
        let analysis = analysis_from_landmarks(&resp);
        assert!(!analysis.is_mock_data);
        let b = &analysis.elemental_balance;
        let sum = b.wood + b.fire + b.earth + b.metal + b.water;
        assert!(
            (sum - 1.0).abs() < 1e-6,
            "elements must normalize, got {sum}"
        );
        assert!(analysis.personality_indicators.len() >= 3);
        assert!(analysis.health_indicators.len() >= 4);
        assert!(analysis.quality.is_some());
    }

    #[test]
    fn analysis_from_landmarks_pitta_for_mid_ratio() {
        let resp = synthetic_response();
        let analysis = analysis_from_landmarks(&resp);
        assert_eq!(analysis.constitution.primary_dosha, Dosha::Pitta);
        assert_eq!(analysis.constitution.body_type, BodyType::Mesomorph);
    }

    #[test]
    fn is_real_landmark_response_gates_fallback() {
        let mut resp = synthetic_response();
        assert!(is_real_landmark_response(&resp));
        resp.landmark_source = "deterministic-fallback".to_string();
        assert!(!is_real_landmark_response(&resp));
    }

    #[test]
    fn decode_image_bytes_handles_b64_and_raw() {
        let encoded = base64::engine::general_purpose::STANDARD.encode(b"hello-image");
        assert_eq!(decode_image_bytes(encoded.as_bytes()), b"hello-image");
        assert_eq!(decode_image_bytes(b"not b64!!"), b"not b64!!");
    }

    #[test]
    fn config_resolve_honors_option_override_and_disabled() {
        let mut options = HashMap::new();
        options.insert("face_cv_url".to_string(), json!("http://example:9000"));
        let cfg = FaceCvConfig::resolve(&options).unwrap();
        assert_eq!(cfg.base_url, "http://example:9000");

        options.insert("face_cv_disabled".to_string(), json!(true));
        assert!(FaceCvConfig::resolve(&options).is_none());
    }
}
