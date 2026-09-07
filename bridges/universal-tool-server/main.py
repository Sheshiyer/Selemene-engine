#!/usr/bin/env python3
"""Protected Rust universal tool bridge for Selemene.

The bridge exposes a small, typed descriptor catalogue. User supplied values are
validated as identifiers before they are encoded into an established Rust route;
there is no free-form path interpolation and no direct sidecar surface.
"""
from __future__ import annotations

from dataclasses import dataclass
import json
import os
from pathlib import Path
import re
from typing import Any, Callable

import httpx
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel, Field

app = FastAPI(title="Selemene Universal Tool Bridge", version="2.0.0")

RUST_URL = os.environ.get("SELEMENE_RUST_URL", "http://localhost:8080").rstrip("/")
API_KEY = os.environ.get("SELEMENE_API_KEY", "")
BEARER_TOKEN = os.environ.get("SELEMENE_BEARER_TOKEN", "")
_IDENTIFIER = re.compile(r"^[a-z0-9]+(?:-[a-z0-9]+)*$")


def _load_engine_registry() -> dict[str, dict[str, Any]]:
    registry_path = Path(__file__).resolve().parents[2] / "contracts" / "v1" / "registries" / "engines.json"
    try:
        payload = json.loads(registry_path.read_text())
        rows = payload["engines"]
        return {row["id"]: row for row in rows}
    except (OSError, ValueError, KeyError, TypeError):
        return {}


_ENGINE_REGISTRY = _load_engine_registry()
_PUBLIC_ENGINE_IDS = tuple(
    engine_id
    for engine_id, row in _ENGINE_REGISTRY.items()
    if row.get("public_mirror_group") and row.get("operations", {}).get("calculate") == "supported"
)
_SUPPORTED_WORKFLOW_IDS = (
    "birth-blueprint",
    "creative-expression",
    "daily-practice",
    "decision-support",
    "self-inquiry",
)


@dataclass(frozen=True)
class ToolDescriptor:
    name: str
    method: str
    route: str
    kind: str
    status: str = "supported"
    description: str = ""



def _catalogue() -> tuple[ToolDescriptor, ...]:
    descriptors: list[ToolDescriptor] = [
        ToolDescriptor("health", "GET", "/health", "health", description="Protected Rust health"),
        ToolDescriptor("health_ready", "GET", "/health/ready", "health", description="Protected Rust readiness"),
        ToolDescriptor("status", "GET", "/api/v1/status", "status", description="Protected Rust status"),
        ToolDescriptor("list_capabilities", "GET", "/api/v1/engines/capabilities", "capabilities", description="Canonical 19-row capability list"),
        ToolDescriptor("list_engines", "GET", "/api/v1/engines", "engines", description="Legacy engine list adapter"),
        ToolDescriptor("list_workflows", "GET", "/api/v1/workflows", "workflows", description="Canonical workflow list"),
        ToolDescriptor(
            "workflow_execute",
            "POST",
            "/api/v1/workflows/{workflow_id}/execute",
            "workflow",
            description="Execute one of five supported workflows",
        ),
        ToolDescriptor(
            "full_spectrum",
            "POST",
            "",
            "workflow",
            status="unsupported",
            description="Full Spectrum is unsupported pending a lossless adapter",
        ),
    ]
    descriptors.extend(
        ToolDescriptor(
            name=f"engine_calculate:{engine_id}",
            method="POST",
            route=f"/api/v1/engines/{engine_id}/calculate",
            kind="engine",
            description=f"Calculate {row.get('display_name', engine_id)}",
        )
        for engine_id, row in _ENGINE_REGISTRY.items()
        if row.get("operations", {}).get("calculate") == "supported"
    )
    return tuple(descriptors)


CATALOGUE = _catalogue()


class ToolRequest(BaseModel):
    name: str
    arguments: dict[str, Any] = Field(default_factory=dict)


class ToolResponse(BaseModel):
    tool: str
    success: bool
    data: dict[str, Any] | list[Any] | None = None
    error: str | None = None
    status: str | None = None


def _safe_identifier(value: Any, field: str) -> str:
    if not isinstance(value, str) or not _IDENTIFIER.fullmatch(value):
        raise HTTPException(400, f"Invalid {field}")
    return value


def _auth_headers() -> dict[str, str]:
    if API_KEY and BEARER_TOKEN:
        raise HTTPException(400, "Choose either API key or bearer token")
    headers = {"Content-Type": "application/json"}
    if API_KEY:
        headers["X-API-Key"] = API_KEY
    elif BEARER_TOKEN:
        headers["Authorization"] = f"Bearer {BEARER_TOKEN}"
    return headers


def _descriptor(name: str) -> ToolDescriptor:
    for descriptor in CATALOGUE:
        if descriptor.name == name:
            return descriptor
    raise HTTPException(404, "Unknown tool")


def _engine_descriptor(engine_id: str) -> ToolDescriptor:
    _safe_identifier(engine_id, "engine_id")
    if engine_id not in _PUBLIC_ENGINE_IDS:
        raise HTTPException(400, "Engine is not a public calculate capability")
    return ToolDescriptor(
        name="engine_calculate",
        method="POST",
        route=f"/api/v1/engines/{engine_id}/calculate",
        kind="engine",
    )


def _workflow_route(workflow_id: str) -> ToolDescriptor:
    _safe_identifier(workflow_id, "workflow_id")
    if workflow_id == "full-spectrum":
        raise HTTPException(400, "Workflow is unsupported")
    if workflow_id not in _SUPPORTED_WORKFLOW_IDS:
        raise HTTPException(400, "Workflow is not supported")
    return ToolDescriptor(
        name="workflow_execute",
        method="POST",
        route=f"/api/v1/workflows/{workflow_id}/execute",
        kind="workflow",
    )


def _client_factory(timeout: float) -> httpx.AsyncClient:
    factory: Callable[[float], httpx.AsyncClient] = getattr(app.state, "client_factory", httpx.AsyncClient)
    return factory(timeout=timeout)


async def _request(descriptor: ToolDescriptor, arguments: dict[str, Any]) -> ToolResponse:
    body = dict(arguments)
    route = descriptor.route
    if descriptor.kind == "engine":
        engine_id = _safe_identifier(body.pop("engine_id", ""), "engine_id")
        descriptor = _engine_descriptor(engine_id)
        route = descriptor.route
    elif descriptor.kind == "workflow":
        workflow_id = body.pop("workflow_id", "")
        descriptor = _workflow_route(workflow_id)
        route = descriptor.route

    if descriptor.status == "unsupported":
        raise HTTPException(400, "Workflow is unsupported")

    headers = _auth_headers()
    timeout = httpx.Timeout(30.0)
    try:
        async with _client_factory(timeout=timeout) as client:
            url = f"{RUST_URL}{route}"
            if descriptor.method == "GET":
                response = await client.get(url, headers=headers, params=body or None)
            else:
                response = await client.post(url, headers=headers, json=body)
            response.raise_for_status()
            try:
                payload = response.json()
            except ValueError:
                return ToolResponse(tool=descriptor.name, success=False, error="UPSTREAM_INVALID_RESPONSE")
            if not isinstance(payload, (dict, list)):
                return ToolResponse(tool=descriptor.name, success=False, error="UPSTREAM_INVALID_RESPONSE")
            return ToolResponse(tool=descriptor.name, success=True, data=payload)
    except HTTPException:
        raise
    except httpx.TimeoutException:
        return ToolResponse(tool=descriptor.name, success=False, error="UPSTREAM_TIMEOUT")
    except httpx.HTTPStatusError:
        return ToolResponse(tool=descriptor.name, success=False, error="UPSTREAM_ERROR")
    except httpx.RequestError:
        return ToolResponse(tool=descriptor.name, success=False, error="UPSTREAM_UNAVAILABLE")


@app.post("/tools/execute", response_model=ToolResponse)
async def execute_tool(request: ToolRequest) -> ToolResponse:
    """Execute a canonical protected Rust tool."""
    descriptor = (
        _engine_descriptor(request.arguments.get("engine_id"))
        if request.name == "engine_calculate"
        else _descriptor(request.name)
    )
    # Generic compatibility names are resolved only through typed descriptors.
    if request.name == "engine_calculate":
        engine_id = request.arguments.get("engine_id")
        descriptor = _engine_descriptor(engine_id)
    elif request.name == "workflow_execute":
        descriptor = _workflow_route(request.arguments.get("workflow_id"))
    return await _request(descriptor, request.arguments)


@app.get("/tools")
async def list_tools() -> dict[str, Any]:
    """List public descriptors and explicit unsupported workflow metadata."""
    tools = [
        {
            "name": descriptor.name,
            "method": descriptor.method,
            "path": descriptor.route,
            "server": "rust",
            "status": descriptor.status,
            "description": descriptor.description,
        }
        for descriptor in CATALOGUE
    ]
    return {"tools": tools, "count": len(tools), "public_engine_count": len(_PUBLIC_ENGINE_IDS), "runtime_engine_count": len(_ENGINE_REGISTRY)}


@app.get("/health")
async def bridge_health() -> dict[str, str]:
    """Check only the protected Rust upstream."""
    results = {"bridge": "healthy", "rust_engine": "unknown"}
    try:
        async with _client_factory(timeout=httpx.Timeout(5.0)) as client:
            response = await client.get(f"{RUST_URL}/health", headers=_auth_headers())
            results["rust_engine"] = "healthy" if response.status_code == 200 else "unhealthy"
    except httpx.TimeoutException:
        results["rust_engine"] = "timeout"
    except httpx.RequestError:
        results["rust_engine"] = "unreachable"
    return results


if __name__ == "__main__":
    import uvicorn

    port = int(os.environ.get("BRIDGE_PORT", "8000"))
    uvicorn.run(app, host="0.0.0.0", port=port)
