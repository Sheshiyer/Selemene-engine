#!/usr/bin/env python3
"""Universal Tool Execution Bridge for Selemene Engine."""

from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
import httpx
import json
import os

app = FastAPI(title="Selemene Universal Tool Bridge", version="1.0.0")

RUST_URL = os.environ.get("SELEMENE_RUST_URL", "http://localhost:8080")
TS_URL = os.environ.get("SELEMENE_TS_URL", "http://localhost:3001")
API_KEY = os.environ.get("SELEMENE_API_KEY", "")
SPEC_PATH = os.environ.get("SELEMENE_SPEC_PATH", "openapi-unified.json")

# Tool name → (base_url, method, path_template)
TOOL_ROUTES = {
    "health": (RUST_URL, "GET", "/health"),
    "health_ready": (RUST_URL, "GET", "/health/ready"),
    "status": (RUST_URL, "GET", "/api/v1/status"),
    "list_engines": (RUST_URL, "GET", "/api/v1/engines"),
    "engine_info": (RUST_URL, "GET", "/api/v1/engines/{engine_id}/info"),
    "engine_calculate": (RUST_URL, "POST", "/api/v1/engines/{engine_id}/calculate"),
    "engine_validate": (RUST_URL, "POST", "/api/v1/engines/{engine_id}/validate"),
    "list_workflows": (RUST_URL, "GET", "/api/v1/workflows"),
    "workflow_info": (RUST_URL, "GET", "/api/v1/workflows/{workflow_id}/info"),
    "workflow_execute": (RUST_URL, "POST", "/api/v1/workflows/{workflow_id}/execute"),
    "ts_health": (TS_URL, "GET", "/health"),
    "ts_list_engines": (TS_URL, "GET", "/engines"),
    "ts_engine_info": (TS_URL, "GET", "/engines/{id}/info"),
    "ts_engine_calculate": (TS_URL, "POST", "/engines/{id}/calculate"),
}

class ToolRequest(BaseModel):
    name: str
    arguments: dict = {}

class ToolResponse(BaseModel):
    tool: str
    success: bool
    data: dict | None = None
    error: str | None = None

@app.post("/tools/execute", response_model=ToolResponse)
async def execute_tool(request: ToolRequest):
    """Execute a Selemene tool by name."""
    if request.name not in TOOL_ROUTES:
        raise HTTPException(404, f"Unknown tool: {request.name}")

    base_url, method, path_template = TOOL_ROUTES[request.name]

    # Extract path params from arguments
    path = path_template
    body = dict(request.arguments)
    for key in list(body.keys()):
        placeholder = "{" + key + "}"
        if placeholder in path:
            path = path.replace(placeholder, str(body.pop(key)))

    url = f"{base_url}{path}"
    headers = {"Content-Type": "application/json"}
    if API_KEY:
        headers["Authorization"] = f"Bearer {API_KEY}"

    async with httpx.AsyncClient(timeout=30.0) as client:
        try:
            if method == "GET":
                resp = await client.get(url, headers=headers, params=body if body else None)
            else:
                resp = await client.post(url, headers=headers, json=body)
            resp.raise_for_status()
            return ToolResponse(tool=request.name, success=True, data=resp.json())
        except httpx.HTTPStatusError as e:
            return ToolResponse(tool=request.name, success=False, error=f"HTTP {e.response.status_code}: {e.response.text}")
        except httpx.RequestError as e:
            return ToolResponse(tool=request.name, success=False, error=str(e))

@app.get("/tools")
async def list_tools():
    """List all available tools with their routing info."""
    tools = []
    for name, (base_url, method, path) in TOOL_ROUTES.items():
        tools.append({
            "name": name,
            "method": method,
            "path": path,
            "server": "rust" if base_url == RUST_URL else "ts",
        })
    return {"tools": tools, "count": len(tools)}

@app.get("/health")
async def bridge_health():
    """Check health of bridge and upstream services."""
    results = {"bridge": "healthy", "rust_engine": "unknown", "ts_engines": "unknown"}
    async with httpx.AsyncClient(timeout=5.0) as client:
        try:
            r = await client.get(f"{RUST_URL}/health")
            results["rust_engine"] = "healthy" if r.status_code == 200 else "unhealthy"
        except:
            results["rust_engine"] = "unreachable"
        try:
            r = await client.get(f"{TS_URL}/health")
            results["ts_engines"] = "healthy" if r.status_code == 200 else "unhealthy"
        except:
            results["ts_engines"] = "unreachable"
    return results

if __name__ == "__main__":
    import uvicorn
    port = int(os.environ.get("BRIDGE_PORT", "8000"))
    uvicorn.run(app, host="0.0.0.0", port=port)
