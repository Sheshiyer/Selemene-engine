"""Shared test fixtures for Selemene Python services."""

import io

import pytest
from fastapi.testclient import TestClient

from mediapipe_service.main import app as mediapipe_app
from biofield_cv_service.main import app as biofield_app


@pytest.fixture
def mediapipe_client() -> TestClient:
    return TestClient(mediapipe_app)


@pytest.fixture
def biofield_client() -> TestClient:
    return TestClient(biofield_app)


@pytest.fixture
def fake_jpeg() -> tuple[io.BytesIO, str]:
    """Minimal JPEG-like bytes with a stable filename for deterministic tests."""
    # JPEG magic bytes + padding
    data = b"\xff\xd8\xff\xe0" + b"\x00" * 1024
    return io.BytesIO(data), "test_face.jpg"


@pytest.fixture
def large_jpeg() -> tuple[io.BytesIO, str]:
    """Large image (>500KB) to trigger high-quality tier in biofield."""
    data = b"\xff\xd8\xff\xe0" + b"\x00" * 600_000
    return io.BytesIO(data), "large_capture.jpg"
