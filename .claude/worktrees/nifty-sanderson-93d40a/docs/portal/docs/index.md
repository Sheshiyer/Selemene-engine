---
slug: /
title: API Overview
---

Welcome to the **Selemene Engine Developer Portal**.

## What you can build

- Call 16 consciousness engines via `/api/v1/engines/{engine_id}/calculate`
- Run 6 synthesis workflows via `/api/v1/workflows/{workflow_id}/execute`
- Integrate with Rust and TypeScript SDKs
- Authenticate with JWT Bearer or `X-API-Key`

## Base URL

`https://selemene-engine-production.up.railway.app`

## Core response contracts

- `EngineOutput` for single-engine calculations
- `WorkflowResult` for orchestrated multi-engine runs

Continue with [Authentication](./authentication) and [SDK Quickstarts](./sdk-quickstarts).
