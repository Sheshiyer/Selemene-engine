"""Tests for Biofield CV /analyze endpoint."""

import io
import json

CONTRACT_VERSION = "biofield-cv/v1"
ANALYSIS_VERSION = "stub-metrics/v1"


def analyze_json(biofield_client, file_tuple, **data):
    buf, name = file_tuple
    response = biofield_client.post(
        "/analyze",
        files={"image": (name, buf, "image/jpeg")},
        data=data or None,
    )
    return response, response.json()


def test_analyze_returns_200(biofield_client, fake_jpeg):
    response, _ = analyze_json(biofield_client, fake_jpeg)
    assert response.status_code == 200


def test_analyze_returns_required_contract_fields(biofield_client, fake_jpeg):
    response, data = analyze_json(biofield_client, fake_jpeg)
    assert response.status_code == 200
    assert data["contract_version"] == CONTRACT_VERSION
    assert data["analysis_version"] == ANALYSIS_VERSION


def test_analyze_matches_required_top_level_response_shape(biofield_client, fake_jpeg):
    _, data = analyze_json(biofield_client, fake_jpeg)
    assert set(data.keys()) == {
        "contract_version",
        "analysis_version",
        "metrics",
        "quality_assessment",
        "algorithms_run",
        "processing_time_ms",
    }

    assert set(data["quality_assessment"].keys()) == {
        "sharpness",
        "contrast",
        "noise_level",
        "exposure",
        "sufficient_quality",
    }


def test_analyze_returns_all_11_metrics(biofield_client, fake_jpeg):
    _, data = analyze_json(biofield_client, fake_jpeg)
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
    _, data = analyze_json(biofield_client, fake_jpeg)
    bands = data["metrics"]["energy_analysis"]
    total = bands["low"] + bands["medium"] + bands["high"]
    assert abs(total - 1.0) < 0.01, f"Energy bands should sum to ~1.0, got {total}"


def test_analyze_fractal_dimension_in_range(biofield_client, fake_jpeg):
    _, data = analyze_json(biofield_client, fake_jpeg)
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
    assert r1["analysis_version"] == r2["analysis_version"]


def test_analyze_runs_all_algorithms_by_default(biofield_client, fake_jpeg):
    _, data = analyze_json(biofield_client, fake_jpeg)
    assert len(data["algorithms_run"]) == 11


def test_analyze_algorithm_filter(biofield_client, fake_jpeg):
    response, data = analyze_json(
        biofield_client,
        fake_jpeg,
        algorithms=json.dumps(["fractal_dimension", "body_symmetry"]),
    )
    assert response.status_code == 200
    assert data["algorithms_run"] == ["fractal_dimension", "body_symmetry"]


def test_analyze_rejects_malformed_algorithms_json(biofield_client, fake_jpeg):
    response = biofield_client.post(
        "/analyze",
        files={"image": (fake_jpeg[1], fake_jpeg[0], "image/jpeg")},
        data={"algorithms": "[not valid json"},
    )
    assert response.status_code == 422
    assert "algorithms" in response.json()["detail"]


def test_analyze_rejects_non_array_algorithms(biofield_client, fake_jpeg):
    response = biofield_client.post(
        "/analyze",
        files={"image": (fake_jpeg[1], fake_jpeg[0], "image/jpeg")},
        data={"algorithms": json.dumps({"name": "fractal_dimension"})},
    )
    assert response.status_code == 422
    assert "JSON array" in response.json()["detail"]


def test_analyze_rejects_unknown_algorithms(biofield_client, fake_jpeg):
    response = biofield_client.post(
        "/analyze",
        files={"image": (fake_jpeg[1], fake_jpeg[0], "image/jpeg")},
        data={"algorithms": json.dumps(["fractal_dimension", "unknown_metric"])},
    )
    assert response.status_code == 422
    assert "unsupported values" in response.json()["detail"]


def test_analyze_rejects_malformed_options_json(biofield_client, fake_jpeg):
    response = biofield_client.post(
        "/analyze",
        files={"image": (fake_jpeg[1], fake_jpeg[0], "image/jpeg")},
        data={"options": "{"},
    )
    assert response.status_code == 422
    assert "options" in response.json()["detail"]


def test_analyze_rejects_non_object_options(biofield_client, fake_jpeg):
    response = biofield_client.post(
        "/analyze",
        files={"image": (fake_jpeg[1], fake_jpeg[0], "image/jpeg")},
        data={"options": json.dumps(["not", "an", "object"])},
    )
    assert response.status_code == 422
    assert "JSON object" in response.json()["detail"]


def test_analyze_rejects_malformed_capture_metadata_json(biofield_client, fake_jpeg):
    response = biofield_client.post(
        "/analyze",
        files={"image": (fake_jpeg[1], fake_jpeg[0], "image/jpeg")},
        data={"capture_metadata": "{"},
    )
    assert response.status_code == 422
    assert "capture_metadata" in response.json()["detail"]


def test_analyze_rejects_non_object_capture_metadata(biofield_client, fake_jpeg):
    response = biofield_client.post(
        "/analyze",
        files={"image": (fake_jpeg[1], fake_jpeg[0], "image/jpeg")},
        data={"capture_metadata": json.dumps(["mobile", "ios"])},
    )
    assert response.status_code == 422
    assert "JSON object" in response.json()["detail"]


def test_analyze_requires_image_upload(biofield_client):
    response = biofield_client.post("/analyze")
    assert response.status_code == 422


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
    _, data = analyze_json(biofield_client, large_jpeg)
    quality = data["quality_assessment"]
    assert quality["sufficient_quality"] is True
    assert quality["sharpness"] >= 0.8


def test_analyze_processing_time_positive(biofield_client, fake_jpeg):
    _, data = analyze_json(biofield_client, fake_jpeg)
    assert data["processing_time_ms"] >= 0
