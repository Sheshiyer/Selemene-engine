# Railway Deployment Guide

## Architecture Overview

The Selemene Engine consists of three separate services that must be deployed together on Railway:

1.  **`Selemene-engine` (Rust API)**
    *   **Repo Root:** `.` (Uses `Dockerfile.prod` or `railway.toml`)
    *   **Port:** 8080
    *   **Role:** Main API gateway, orchestrator, and calculation engine for Vedic/astrology.

2.  **`ts-engines` (TypeScript Sidecar)**
    *   **Repo Root:** `ts-engines/` (Uses `railway.ts-engines.toml`)
    *   **Port:** 3001
    *   **Role:** Handles Tarot, I-Ching, Enneagram, Sacred Geometry, Sigil Forge.
    *   **Dependency:** The Rust API *depends* on this service being available at startup.

3.  **`noesis-redis` (Redis Cache)**
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