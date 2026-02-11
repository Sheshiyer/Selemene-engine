"""Tests for MediaPipe Face Mesh /analyze endpoint."""

import io


def test_analyze_returns_200(mediapipe_client, fake_jpeg):
    buf, name = fake_jpeg
    response = mediapipe_client.post("/analyze", files={"image": (name, buf, "image/jpeg")})
    assert response.status_code == 200


def test_analyze_returns_468_landmarks(mediapipe_client, fake_jpeg):
    buf, name = fake_jpeg
    data = mediapipe_client.post(
        "/analyze", files={"image": (name, buf, "image/jpeg")}
    ).json()
    assert len(data["landmarks"]) == 468


def test_analyze_landmarks_have_xyz(mediapipe_client, fake_jpeg):
    buf, name = fake_jpeg
    data = mediapipe_client.post(
        "/analyze", files={"image": (name, buf, "image/jpeg")}
    ).json()
    lm = data["landmarks"][0]
    assert "x" in lm and "y" in lm and "z" in lm
    assert 0.0 <= lm["x"] <= 1.0
    assert 0.0 <= lm["y"] <= 1.0


def test_analyze_landmarks_are_deterministic(mediapipe_client):
    """Same filename produces identical landmarks."""
    buf1 = io.BytesIO(b"\xff\xd8" + b"\x00" * 100)
    buf2 = io.BytesIO(b"\xff\xd8" + b"\xAB" * 200)  # different bytes, same name
    r1 = mediapipe_client.post(
        "/analyze", files={"image": ("same.jpg", buf1, "image/jpeg")}
    ).json()
    r2 = mediapipe_client.post(
        "/analyze", files={"image": ("same.jpg", buf2, "image/jpeg")}
    ).json()
    assert r1["landmarks"] == r2["landmarks"]


def test_analyze_face_detected_true(mediapipe_client, fake_jpeg):
    buf, name = fake_jpeg
    data = mediapipe_client.post(
        "/analyze", files={"image": (name, buf, "image/jpeg")}
    ).json()
    assert data["face_detected"] is True
    assert data["num_faces"] == 1


def test_analyze_face_oval_indices(mediapipe_client, fake_jpeg):
    buf, name = fake_jpeg
    data = mediapipe_client.post(
        "/analyze", files={"image": (name, buf, "image/jpeg")}
    ).json()
    assert isinstance(data["face_oval"], list)
    assert len(data["face_oval"]) == 36
    assert all(0 <= idx <= 467 for idx in data["face_oval"])


def test_analyze_proportions_present(mediapipe_client, fake_jpeg):
    buf, name = fake_jpeg
    data = mediapipe_client.post(
        "/analyze", files={"image": (name, buf, "image/jpeg")}
    ).json()
    props = data["proportions"]
    assert 0.0 <= props["golden_ratio_score"] <= 1.0
    assert 0.0 <= props["symmetry_score"] <= 1.0


def test_analyze_image_quality_present(mediapipe_client, fake_jpeg):
    buf, name = fake_jpeg
    data = mediapipe_client.post(
        "/analyze", files={"image": (name, buf, "image/jpeg")}
    ).json()
    quality = data["image_quality"]
    assert "sharpness" in quality
    assert "brightness" in quality
    assert "sufficient_quality" in quality


def test_analyze_processing_time_positive(mediapipe_client, fake_jpeg):
    buf, name = fake_jpeg
    data = mediapipe_client.post(
        "/analyze", files={"image": (name, buf, "image/jpeg")}
    ).json()
    assert data["processing_time_ms"] >= 0
