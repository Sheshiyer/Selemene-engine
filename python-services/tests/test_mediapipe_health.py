"""Tests for MediaPipe Face Mesh service health endpoint."""

from fastapi.testclient import TestClient

from mediapipe_service.main import app

client = TestClient(app)


def test_health_returns_200() -> None:
    response = client.get("/health")
    assert response.status_code == 200


def test_health_returns_correct_service_name() -> None:
    response = client.get("/health")
    data = response.json()
    assert data["service"] == "mediapipe-face-mesh"


def test_health_returns_version() -> None:
    response = client.get("/health")
    data = response.json()
    assert data["version"] == "3.0.0"


def test_health_includes_mediapipe_availability() -> None:
    response = client.get("/health")
    data = response.json()
    assert "mediapipe_available" in data
    assert isinstance(data["mediapipe_available"], bool)


def test_health_status_is_healthy() -> None:
    response = client.get("/health")
    data = response.json()
    assert data["status"] == "healthy"
