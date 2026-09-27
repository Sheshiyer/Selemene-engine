"""Tests for the explicit sidecar capability-status field on /health.

Rule under test:
- biofield-cv:  "available" only if opencv AND numpy are both available (its
  two hard dependencies); "degraded" if opencv+numpy are up but mediapipe is
  missing; "unavailable" if opencv or numpy is missing.
- mediapipe-face-mesh: "available" if mediapipe is available, "unavailable"
  otherwise.

All computed from the existing local self-check booleans only -- no
provider/network/database calls.
"""

from fastapi.testclient import TestClient

import biofield_cv_service.health as biofield_health
import mediapipe_service.health as mediapipe_health

ALLOWED_STATUSES = {"available", "degraded", "unavailable"}


# ---------- biofield-cv ----------

def test_biofield_health_includes_capability_status(biofield_client: TestClient) -> None:
    response = biofield_client.get("/health")
    data = response.json()
    assert "capability_status" in data
    assert data["capability_status"] in ALLOWED_STATUSES


def test_biofield_capability_status_available_when_all_deps_present(
    biofield_client: TestClient, monkeypatch
) -> None:
    monkeypatch.setattr(biofield_health, "_check_opencv", lambda: True)
    monkeypatch.setattr(biofield_health, "_check_numpy", lambda: True)
    monkeypatch.setattr(biofield_health, "_check_mediapipe", lambda: True)

    data = biofield_client.get("/health").json()

    assert data["capability_status"] == "available"


def test_biofield_capability_status_degraded_without_mediapipe(
    biofield_client: TestClient, monkeypatch
) -> None:
    monkeypatch.setattr(biofield_health, "_check_opencv", lambda: True)
    monkeypatch.setattr(biofield_health, "_check_numpy", lambda: True)
    monkeypatch.setattr(biofield_health, "_check_mediapipe", lambda: False)

    data = biofield_client.get("/health").json()

    assert data["capability_status"] == "degraded"


def test_biofield_capability_status_unavailable_without_opencv(
    biofield_client: TestClient, monkeypatch
) -> None:
    monkeypatch.setattr(biofield_health, "_check_opencv", lambda: False)
    monkeypatch.setattr(biofield_health, "_check_numpy", lambda: True)
    monkeypatch.setattr(biofield_health, "_check_mediapipe", lambda: True)

    data = biofield_client.get("/health").json()

    assert data["capability_status"] == "unavailable"


def test_biofield_capability_status_unavailable_without_numpy(
    biofield_client: TestClient, monkeypatch
) -> None:
    monkeypatch.setattr(biofield_health, "_check_opencv", lambda: True)
    monkeypatch.setattr(biofield_health, "_check_numpy", lambda: False)
    monkeypatch.setattr(biofield_health, "_check_mediapipe", lambda: True)

    data = biofield_client.get("/health").json()

    assert data["capability_status"] == "unavailable"


# ---------- mediapipe-face-mesh ----------

def test_mediapipe_health_includes_capability_status(mediapipe_client: TestClient) -> None:
    response = mediapipe_client.get("/health")
    data = response.json()
    assert "capability_status" in data
    assert data["capability_status"] in ALLOWED_STATUSES


def test_mediapipe_capability_status_available_when_mediapipe_present(
    mediapipe_client: TestClient, monkeypatch
) -> None:
    monkeypatch.setattr(mediapipe_health, "_check_mediapipe", lambda: True)

    data = mediapipe_client.get("/health").json()

    assert data["capability_status"] == "available"


def test_mediapipe_capability_status_unavailable_without_mediapipe(
    mediapipe_client: TestClient, monkeypatch
) -> None:
    monkeypatch.setattr(mediapipe_health, "_check_mediapipe", lambda: False)

    data = mediapipe_client.get("/health").json()

    assert data["capability_status"] == "unavailable"
