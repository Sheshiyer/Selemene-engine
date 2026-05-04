"""Shared test fixtures for Selemene Python services."""

import io

import cv2
import numpy as np
import pytest
from fastapi.testclient import TestClient

from biofield_cv_service.main import app as biofield_app


@pytest.fixture
def biofield_client() -> TestClient:
    return TestClient(biofield_app)


def _encode_png(arr: np.ndarray) -> io.BytesIO:
    """Encode a numpy BGR image array to PNG bytes in a BytesIO buffer."""
    ok, buf = cv2.imencode(".png", arr)
    assert ok, "cv2.imencode failed"
    return io.BytesIO(buf.tobytes())


@pytest.fixture
def fake_jpeg() -> tuple[io.BytesIO, str]:
    """64×64 synthetic PNG with a person-like bright centre on dark background."""
    img = np.zeros((64, 64, 3), dtype=np.uint8)
    cv2.circle(img, (32, 32), 20, (180, 160, 140), -1)  # bright oval "person"
    cv2.rectangle(img, (20, 32), (44, 60), (160, 140, 120), -1)  # body
    return _encode_png(img), "test_face.png"


@pytest.fixture
def symmetric_image() -> tuple[io.BytesIO, str]:
    """64×64 perfectly bilaterally symmetric PNG — body_symmetry should be high."""
    img = np.zeros((64, 64, 3), dtype=np.uint8)
    half = np.random.default_rng(0).integers(50, 200, (64, 32, 3), dtype=np.uint8)
    img[:, :32] = half
    img[:, 32:] = half[:, ::-1]
    return _encode_png(img), "symmetric.png"


@pytest.fixture
def solid_grey() -> tuple[io.BytesIO, str]:
    """64×64 uniform grey PNG — should fail quality check (no sharpness/contrast)."""
    img = np.full((64, 64, 3), 128, dtype=np.uint8)
    return _encode_png(img), "solid_grey.png"


@pytest.fixture
def large_jpeg() -> tuple[io.BytesIO, str]:
    """256×256 detailed PNG with high-frequency texture — should pass quality."""
    rng = np.random.default_rng(42)
    img = rng.integers(0, 255, (256, 256, 3), dtype=np.uint8)
    # Add a bright foreground to ensure mask extraction works
    cv2.circle(img, (128, 128), 80, (200, 180, 160), -1)
    return _encode_png(img), "large_capture.png"

