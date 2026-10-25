# Railway Deployment Guide

This deployment topology exists to keep mirror-based engine and workflow surfaces resilient in production.

## Architecture Overview

The Selemene Engine consists of three separate services that must be deployed together on Railway:

1.  **`Selemene-engine` (Rust API)**
    *   **Repo Root:** `.` (Uses `Dockerfile.prod` or `railway.toml`)
    *   **Port:** 8080
    *   **Role:** Main API gateway, orchestrator, and calculation engine for Vedic/astrology.

2.  **`ts-engines` (TypeScript Sidecar)**
    *   **Repo Root:** `ts-engines/` (Uses `railway.ts-engines.toml`)
    *   **Port:** 3001
    *   **Role:** Handles Tarot, I-Ching, Enneagram, Sacred Geometry, Sigil Forge, Raaga.
    *   **Dependency:** The Rust API *depends* on this service being available at startup.
    *   **Raaga clips:** rendered in-process and uploaded to Cloudflare R2 when
        `RAAGA_CLIP_MODE=r2` (`R2_ACCOUNT_ID`, `R2_ACCESS_KEY_ID`, `R2_SECRET_ACCESS_KEY`,
        `R2_PUBLIC_BASE_URL`, bucket `selemene-raga-clips`). No separate audio service
        is part of this topology; the former `suno-bridge` service and its `SUNO_COOKIE`
        variable are retired and must not be re-added.

3.  **`biofield-cv-service` and `mediapipe-service` (Python sidecars)**
    *   **Repo Root:** `python-services/` for both. They share `python-services/railway.toml`
        and build the same `Dockerfile.sidecar`; Railway config-as-code at the root
        directory overrides per-service build settings, so two services rooted at the
        same directory must share one config.
    *   **Selection:** the `SIDECAR_SERVICE` variable picks the app at start.
        Unset means `biofield_cv_service` (port 8002). `mediapipe-service` sets
        `SIDECAR_SERVICE=mediapipe_service` and `PORT=8001`.
    *   **Wiring:** the Rust API reaches them through private networking:
        `PYTHON_BIOFIELD_URL=http://${{biofield-cv-service.RAILWAY_PRIVATE_DOMAIN}}:8002`
        and `SELEMENE_FACE_CV_URL=http://${{mediapipe-service.RAILWAY_PRIVATE_DOMAIN}}:8001`.
    *   **Truth check:** `GET /health/dependencies` on the API reports each sidecar's
        own `capability_status`; a face sidecar whose MediaPipe graph cannot initialise
        reports `degraded`, not `ready`. The sidecar's own `/health` exposes
        `facemesh_ready` and a sanitized `facemesh_error`.
    *   **Pin:** `mediapipe==0.10.18`. Newer 0.10.x and 1.x wheels drop the legacy
        `mediapipe.solutions` API and the sidecar would silently fall back to
        deterministic landmarks.

4.  **`noesis-redis` (Redis Cache)**
    *   **Role:** Caching layer for high-performance calculations.

## ⚠️ Critical Startup Dependency

The Rust API (`Selemene-engine`) attempts to connect to `ts-engines` immediately upon startup to register the Tarot/I-Ching engines.

**If `ts-engines` is offline or starting up slowly:**
*   The Rust API will now **retry for 30 seconds** (6 attempts x 5s).
*   If still unavailable, the API will start in **degraded mode** (Tarot/I-Ching disabled).

### Troubleshooting Missing Engines

If `curl /api/v1/engines` does not list `tarot`:

1.  **Check `ts-engines` status:** Is the service active and healthy in Railway?
2.  **Restart `Selemene-engine`:** If `ts-engines` is online, restart the Rust service to trigger the connection handshake again.
3.  **Check Logs:** Look for "Waiting for TS engines..." or "TS engines unavailable" in the Rust service logs.

## Configuration Files

*   **Rust API:** `railway.toml` (Matches changes in `crates/**`, `src/**`)
*   **TS Engines:** `railway.ts-engines.toml` (Matches changes in `ts-engines/**`)

Ensure your Railway services are configured to use the correct config file!