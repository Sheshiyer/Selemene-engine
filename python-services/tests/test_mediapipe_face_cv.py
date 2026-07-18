"""Tests for face-cv-hook-p3: real CV path + consent + graceful fallback.

Covers the FROZEN face contract additions (contract_version, analysis_version,
landmark_source, consent echo) and the pure-geometry helpers that run on real
MediaPipe landmarks. mediapipe is optional in this venv, so tests assert the
fallback contract and exercise the real-path helpers directly with synthetic
landmarks.

Cites: p1-w1-worker-bootstrap-packet.md + gaps-and-improvements.md (face: no
real CV -> now real path) + goal-understanding.md (consent) +
P1W1-CONTRACTS-FROZEN.md (face example) + detailed-task-list.md (T-027).
Tags: phase:integration-p1 wave:integration-w2 engine-face-reading
"""

import json

from mediapipe_service.analyze import (
    NUM_LANDMARKS,
    _generate_landmarks,
    _proportions_from_landmarks,
)


def _post_with_consent(client, fake_jpeg, consent):
    buf, name = fake_jpeg
    return client.post(
        "/analyze",
        files={"image": (name, buf, "image/png")},
        data={"options": json.dumps({"consent": consent, "source": "pytest"})},
    )


def test_contract_fields_present(mediapipe_client, fake_jpeg):
    buf, name = fake_jpeg
    data = mediapipe_client.post(
        "/analyze", files={"image": (name, buf, "image/png")}
    ).json()
    assert data["contract_version"] == "face-cv/v1"
    assert data["analysis_version"] in {
        "mediapipe-facemesh/v1",
        "deterministic-fallback/v1",
    }
    assert data["landmark_source"] in {"mediapipe-facemesh", "deterministic-fallback"}


def test_consent_echoed_when_granted(mediapipe_client, fake_jpeg):
    consent = {
        "granted": True,
        "scopes": ["face-image"],
        "timestamp": "2026-07-17T12:00:00Z",
        "token": "consent-face-001",
    }
    data = _post_with_consent(mediapipe_client, fake_jpeg, consent).json()
    assert data["consent_granted"] is True
    assert data["consent"]["token"] == "consent-face-001"
    assert data["consent"]["scopes"] == ["face-image"]


def test_consent_not_granted_when_absent(mediapipe_client, fake_jpeg):
    buf, name = fake_jpeg
    data = mediapipe_client.post(
        "/analyze", files={"image": (name, buf, "image/png")}
    ).json()
    assert data["consent_granted"] is False
    assert data["consent"] is None


def test_consent_not_granted_when_false(mediapipe_client, fake_jpeg):
    consent = {"granted": False, "scopes": []}
    data = _post_with_consent(mediapipe_client, fake_jpeg, consent).json()
    assert data["consent_granted"] is False
    assert data["consent"]["granted"] is False


def test_invalid_bytes_still_graceful_fallback(mediapipe_client):
    """Undecodable bytes must not 5xx — deterministic fallback keeps contract."""
    response = mediapipe_client.post(
        "/analyze", files={"image": ("broken.jpg", b"\xff\xd8\x00\x01", "image/jpeg")}
    )
    assert response.status_code == 200
    data = response.json()
    assert len(data["landmarks"]) == NUM_LANDMARKS
    assert data["landmark_source"] == "deterministic-fallback"


def test_fallback_used_when_mediapipe_missing(mediapipe_client, fake_jpeg, monkeypatch):
    """Force mediapipe unavailable -> deterministic fallback on a real PNG."""
    monkeypatch.setattr("mediapipe_service.analyze._mp_facemesh", False)
    buf, name = fake_jpeg
    data = mediapipe_client.post(
        "/analyze", files={"image": (name, buf, "image/png")}
    ).json()
    assert data["landmark_source"] == "deterministic-fallback"
    assert data["analysis_version"] == "deterministic-fallback/v1"
    assert len(data["landmarks"]) == NUM_LANDMARKS


def test_proportions_from_synthetic_landmarks_in_range():
    """Real-path geometry helper produces FROZEN-shaped proportions."""
    landmarks = _generate_landmarks(42)
    props = _proportions_from_landmarks(landmarks)
    assert 0.0 <= props.golden_ratio_score <= 1.0
    assert 0.0 <= props.symmetry_score <= 1.0
    assert props.face_width_height_ratio > 0.0
    assert props.eye_distance_ratio > 0.0
    assert props.jaw_width_ratio > 0.0


def test_proportions_symmetry_higher_for_mirrored_face():
    """A perfectly mirrored landmark set must outscore a skewed one."""
    from shared.models import Landmark

    right_of = {r for _, r in __import__("mediapipe_service.analyze", fromlist=["_BILATERAL_PAIRS"])._BILATERAL_PAIRS}
    pairs = __import__("mediapipe_service.analyze", fromlist=["_BILATERAL_PAIRS"])._BILATERAL_PAIRS
    left_x = {l: 0.4 for l, _ in pairs}

    symmetric = []
    for i in range(NUM_LANDMARKS):
        if i in left_x:
            symmetric.append(Landmark(index=i, x=left_x[i], y=0.5, z=0.0))
        elif i in right_of:
            symmetric.append(Landmark(index=i, x=0.6, y=0.5, z=0.0))  # mirror of 0.4
        else:
            symmetric.append(Landmark(index=i, x=0.5, y=0.5, z=0.0))  # on axis (incl. 168)

    skewed = [lm.model_copy() for lm in symmetric]
    for lm in skewed:
        if lm.index in right_of:
            lm.x = min(1.0, lm.x + 0.1)  # pull right side outward, break mirror

    sym_score = _proportions_from_landmarks(symmetric).symmetry_score
    skew_score = _proportions_from_landmarks(skewed).symmetry_score
    assert sym_score > skew_score


def test_real_path_helper_returns_none_without_mediapipe(monkeypatch):
    """_analyze_with_mediapipe must degrade to None when mediapipe is absent."""
    import numpy as np

    import mediapipe_service.analyze as mod

    monkeypatch.setattr(mod, "_mp_facemesh", False)
    img = np.zeros((8, 8, 3), dtype=np.uint8)
    assert mod._analyze_with_mediapipe(img) is None
