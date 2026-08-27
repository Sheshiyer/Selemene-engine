"""Tests for MediaPipe Face Mesh service health endpoint."""

from fastapi import FastAPI
from fastapi.testclient import TestClient

import mediapipe_service.health as health_module
from shared.version import SERVICE_VERSION

app = FastAPI()
app.include_router(health_module.router)
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
    assert data["version"] == SERVICE_VERSION


def test_health_includes_mediapipe_availability() -> None:
    response = client.get("/health")
    data = response.json()
    assert "mediapipe_available" in data
    assert isinstance(data["mediapipe_available"], bool)


def test_health_status_is_healthy_when_mediapipe_is_available(monkeypatch) -> None:
    monkeypatch.setattr(health_module, "_check_mediapipe", lambda: True)

    response = client.get("/health")
    data = response.json()
    assert data["status"] == "healthy"


def test_health_reports_available_capability_when_mediapipe_is_available(
    monkeypatch,
) -> None:
    monkeypatch.setattr(health_module, "_check_mediapipe", lambda: True)

    response = client.get("/health")
    data = response.json()

    assert data["capabilities"] == [
        {
            "contract_version": "v1",
            "engine_id": "mediapipe-face-mesh",
            "display_name": "MediaPipe Face Mesh",
            "availability": "available",
            "runtime_kind": "python",
            "dependencies": ["mediapipe"],
        }
    ]


def test_health_reports_unavailable_capability_when_mediapipe_is_missing(
    monkeypatch,
) -> None:
    monkeypatch.setattr(health_module, "_check_mediapipe", lambda: False)

    response = client.get("/health")
    data = response.json()

    assert data["status"] == "degraded"
    assert data["capabilities"][0]["availability"] == "unavailable"
