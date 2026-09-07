---
slug: /
title: API Overview
---

Welcome to the **Selemene Engine Developer Portal**.

## What you can build

- Call the 19 runtime identities (17 public mirrors) via `/api/v1/engines/{engine_id}/calculate`
- Run 6 synthesis workflows via `/api/v1/workflows/{workflow_id}/execute`
- Integrate with Rust and TypeScript SDKs
- Authenticate with JWT Bearer or `X-API-Key`

## Base URL

`https://selemene-engine-production.up.railway.app`

## Core response contracts

- `EngineOutput` for single-engine calculations
- `WorkflowResult` for orchestrated multi-engine runs

Continue with [Authentication](./authentication) and [SDK Quickstarts](./sdk-quickstarts).
## Contract v1 capability boundary

Noesis exposes 19 runtime identities and 17 public mirrors through authenticated Rust routes. Use exactly one `X-API-Key` or `Authorization: Bearer` header. Full Spectrum is `unsupported` pending a lossless adapter test.
