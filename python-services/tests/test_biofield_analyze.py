"""Tests for Biofield CV /analyze endpoint."""

import io
import json


def test_analyze_returns_200(biofield_client, fake_jpeg):
    buf, name = fake_jpeg
    response = biofield_client.post("/analyze", files={"image": (name, buf, "image/jpeg")})
    assert response.status_code == 200


def test_analyze_returns_all_11_metrics(biofield_client, fake_jpeg):
    buf, name = fake_jpeg
    data = biofield_client.post(
        "/analyze", files={"image": (name, buf, "image/jpeg")}
    ).json()
    metrics = data["metrics"]
    expected_keys = [
        "light_quanta_density", "normalized_area", "average_intensity",
        "inner_noise", "energy_analysis", "entropy_form_coefficient",
        "fractal_dimension", "correlation_dimension", "body_symmetry",
        "contour_complexity", "pattern_regularity",
    ]
    for key in expected_keys:
        assert key in metrics, f"Missing metric: {key}"


def test_analyze_energy_bands_sum_to_one(biofield_client, fake_jpeg):
    buf, name = fake_jpeg
    data = biofield_client.post(
        "/analyze", files={"image": (name, buf, "image/jpeg")}
    ).json()
    bands = data["metrics"]["energy_analysis"]
    total = bands["low"] + bands["medium"] + bands["high"]
    assert abs(total - 1.0) < 0.01, f"Energy bands should sum to ~1.0, got {total}"


def test_analyze_fractal_dimension_in_range(biofield_client, fake_jpeg):
    buf, name = fake_jpeg
    data = biofield_client.post(
        "/analyze", files={"image": (name, buf, "image/jpeg")}
    ).json()
    fd = data["metrics"]["fractal_dimension"]
    assert 1.0 <= fd <= 2.0, f"Fractal dimension should be 1-2, got {fd}"


def test_analyze_deterministic(biofield_client):
    """Same filename produces identical metrics."""
    buf1 = io.BytesIO(b"\x00" * 100)
    buf2 = io.BytesIO(b"\xFF" * 100)
    r1 = biofield_client.post(
        "/analyze", files={"image": ("same.png", buf1, "image/png")}
    ).json()
    r2 = biofield_client.post(
        "/analyze", files={"image": ("same.png", buf2, "image/png")}
    ).json()
    assert r1["metrics"] == r2["metrics"]


def test_analyze_runs_all_algorithms_by_default(biofield_client, fake_jpeg):
    buf, name = fake_jpeg
    data = biofield_client.post(
        "/analyze", files={"image": (name, buf, "image/jpeg")}
    ).json()
    assert len(data["algorithms_run"]) == 11


def test_analyze_algorithm_filter(biofield_client, fake_jpeg):
    buf, name = fake_jpeg
    subset = json.dumps(["fractal_dimension", "body_symmetry"])
    data = biofield_client.post(
        "/analyze",
        files={"image": (name, buf, "image/jpeg")},
        data={"algorithms": subset},
    ).json()
    assert data["algorithms_run"] == ["fractal_dimension", "body_symmetry"]


def test_analyze_quality_small_image(biofield_client):
    """Small image (<100KB) should get low-quality tier."""
    buf = io.BytesIO(b"\x00" * 50)
    data = biofield_client.post(
        "/analyze", files={"image": ("tiny.jpg", buf, "image/jpeg")}
    ).json()
    quality = data["quality_assessment"]
    assert quality["sufficient_quality"] is False


def test_analyze_quality_large_image(biofield_client, large_jpeg):
    """Large image (>500KB) should get high-quality tier."""
    buf, name = large_jpeg
    data = biofield_client.post(
        "/analyze", files={"image": (name, buf, "image/jpeg")}
    ).json()
    quality = data["quality_assessment"]
    assert quality["sufficient_quality"] is True
    assert quality["sharpness"] >= 0.8


def test_analyze_processing_time_positive(biofield_client, fake_jpeg):
    buf, name = fake_jpeg
    data = biofield_client.post(
        "/analyze", files={"image": (name, buf, "image/jpeg")}
    ).json()
    assert data["processing_time_ms"] >= 0
