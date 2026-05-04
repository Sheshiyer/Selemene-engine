"""Tests for Biofield CV /analyze endpoint — real-cv/v1."""

import json

CONTRACT_VERSION = "biofield-cv/v1"
ANALYSIS_VERSION = "real-cv/v1"


def analyze_json(biofield_client, file_tuple, **data):
    buf, name = file_tuple
    response = biofield_client.post(
        "/analyze",
        files={"image": (name, buf, "image/png")},
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
    assert abs(total - 1.0) < 0.02, f"Energy bands should sum to ~1.0, got {total}"


def test_analyze_fractal_dimension_in_range(biofield_client, fake_jpeg):
    _, data = analyze_json(biofield_client, fake_jpeg)
    fd = data["metrics"]["fractal_dimension"]
    assert 1.0 <= fd <= 2.0, f"Fractal dimension out of range [1,2]: {fd}"


def test_analyze_normalized_area_in_range(biofield_client, fake_jpeg):
    _, data = analyze_json(biofield_client, fake_jpeg)
    na = data["metrics"]["normalized_area"]
    assert 0.0 < na < 1.0, f"normalized_area out of (0,1): {na}"


def test_analyze_body_symmetry_in_range(biofield_client, fake_jpeg):
    _, data = analyze_json(biofield_client, fake_jpeg)
    bs = data["metrics"]["body_symmetry"]
    assert -1.0 <= bs <= 1.0, f"body_symmetry out of [-1,1]: {bs}"


def test_analyze_pattern_regularity_in_range(biofield_client, fake_jpeg):
    _, data = analyze_json(biofield_client, fake_jpeg)
    pr = data["metrics"]["pattern_regularity"]
    assert 0.0 <= pr <= 1.0, f"pattern_regularity out of [0,1]: {pr}"


def test_analyze_symmetric_image_has_high_body_symmetry(biofield_client, symmetric_image):
    _, data = analyze_json(biofield_client, symmetric_image)
    bs = data["metrics"]["body_symmetry"]
    assert bs > 0.5, f"Symmetric image should have body_symmetry > 0.5, got {bs}"


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
    buf, name = fake_jpeg
    response = biofield_client.post(
        "/analyze",
        files={"image": (name, buf, "image/png")},
        data={"algorithms": "[not valid json"},
    )
    assert response.status_code == 422
    assert "algorithms" in response.json()["detail"]


def test_analyze_rejects_non_array_algorithms(biofield_client, fake_jpeg):
    buf, name = fake_jpeg
    response = biofield_client.post(
        "/analyze",
        files={"image": (name, buf, "image/png")},
        data={"algorithms": json.dumps({"name": "fractal_dimension"})},
    )
    assert response.status_code == 422
    assert "JSON array" in response.json()["detail"]


def test_analyze_rejects_unknown_algorithms(biofield_client, fake_jpeg):
    buf, name = fake_jpeg
    response = biofield_client.post(
        "/analyze",
        files={"image": (name, buf, "image/png")},
        data={"algorithms": json.dumps(["fractal_dimension", "unknown_metric"])},
    )
    assert response.status_code == 422
    assert "unsupported values" in response.json()["detail"]


def test_analyze_rejects_malformed_options_json(biofield_client, fake_jpeg):
    buf, name = fake_jpeg
    response = biofield_client.post(
        "/analyze",
        files={"image": (name, buf, "image/png")},
        data={"options": "{"},
    )
    assert response.status_code == 422
    assert "options" in response.json()["detail"]


def test_analyze_rejects_non_object_options(biofield_client, fake_jpeg):
    buf, name = fake_jpeg
    response = biofield_client.post(
        "/analyze",
        files={"image": (name, buf, "image/png")},
        data={"options": json.dumps(["not", "an", "object"])},
    )
    assert response.status_code == 422
    assert "JSON object" in response.json()["detail"]


def test_analyze_rejects_malformed_capture_metadata_json(biofield_client, fake_jpeg):
    buf, name = fake_jpeg
    response = biofield_client.post(
        "/analyze",
        files={"image": (name, buf, "image/png")},
        data={"capture_metadata": "{"},
    )
    assert response.status_code == 422
    assert "capture_metadata" in response.json()["detail"]


def test_analyze_requires_image_upload(biofield_client):
    response = biofield_client.post("/analyze")
    assert response.status_code == 422


def test_analyze_quality_solid_colour_is_insufficient(biofield_client, solid_grey):
    """Uniform grey image has no sharpness/contrast — should fail quality."""
    _, data = analyze_json(biofield_client, solid_grey)
    assert data["quality_assessment"]["sufficient_quality"] is False


def test_analyze_quality_real_image_passes(biofield_client, large_jpeg):
    """Noisy 256×256 PNG with texture should pass quality check."""
    _, data = analyze_json(biofield_client, large_jpeg)
    assert data["quality_assessment"]["sufficient_quality"] is True


def test_analyze_processing_time_positive(biofield_client, fake_jpeg):
    _, data = analyze_json(biofield_client, fake_jpeg)
    assert data["processing_time_ms"] >= 0


def test_analyze_rejects_invalid_image_bytes(biofield_client):
    """Random garbage bytes that can't be decoded should return 422."""
    import io
    response = biofield_client.post(
        "/analyze",
        files={"image": ("bad.png", io.BytesIO(b"not an image at all"), "image/png")},
    )
    assert response.status_code == 422

