#!/usr/bin/env python3
"""Selemene Engine Bridge — CLI tool executor for Claude Code skill."""

import sys
import json
import os
from urllib.request import Request, urlopen
from urllib.error import URLError, HTTPError

RUST_URL = os.environ.get("SELEMENE_URL", "http://localhost:8080")
TS_URL = os.environ.get("SELEMENE_TS_URL", "http://localhost:3001")
API_KEY = os.environ.get("SELEMENE_API_KEY", "")

def _request(url, method="GET", body=None):
    headers = {"Content-Type": "application/json"}
    if API_KEY:
        headers["Authorization"] = f"Bearer {API_KEY}"
    data = json.dumps(body).encode() if body else None
    req = Request(url, data=data, headers=headers, method=method)
    try:
        with urlopen(req, timeout=30) as resp:
            return json.loads(resp.read())
    except HTTPError as e:
        return {"error": f"HTTP {e.code}", "detail": e.read().decode()}
    except URLError as e:
        return {"error": str(e.reason)}

TOOLS = {
    "selemene_health": lambda _: _request(f"{RUST_URL}/health"),
    "selemene_list_engines": lambda _: _request(f"{RUST_URL}/api/v1/engines"),
    "selemene_engine_info": lambda a: _request(f"{RUST_URL}/api/v1/engines/{a['engine_id']}/info"),
    "selemene_calculate": lambda a: _request(
        f"{RUST_URL}/api/v1/engines/{a['engine_id']}/calculate", "POST",
        {"parameters": a.get("parameters", {}), "consciousness_level": a.get("consciousness_level", 0)}
    ),
    "selemene_validate": lambda a: _request(
        f"{RUST_URL}/api/v1/engines/{a['engine_id']}/validate", "POST",
        {"parameters": a.get("parameters", {}), "consciousness_level": a.get("consciousness_level", 0)}
    ),
    "selemene_list_workflows": lambda _: _request(f"{RUST_URL}/api/v1/workflows"),
    "selemene_workflow_info": lambda a: _request(f"{RUST_URL}/api/v1/workflows/{a['workflow_id']}/info"),
    "selemene_workflow_execute": lambda a: _request(
        f"{RUST_URL}/api/v1/workflows/{a['workflow_id']}/execute", "POST",
        {"parameters": a.get("parameters", {}), "consciousness_level": a.get("consciousness_level", 0)}
    ),
    "selemene_ts_health": lambda _: _request(f"{TS_URL}/health"),
    "selemene_ts_list_engines": lambda _: _request(f"{TS_URL}/engines"),
    "selemene_ts_engine_info": lambda a: _request(f"{TS_URL}/engines/{a['engine_id']}/info"),
    "selemene_ts_calculate": lambda a: _request(
        f"{TS_URL}/engines/{a['engine_id']}/calculate", "POST",
        {"parameters": a.get("parameters", {}), "consciousness_level": a.get("consciousness_level", 0)}
    ),
}

def main():
    if len(sys.argv) < 2:
        print(json.dumps({"error": "Usage: bridge.py <tool_name> [json_args]"}))
        sys.exit(1)
    tool = sys.argv[1]
    args = json.loads(sys.argv[2]) if len(sys.argv) > 2 else {}
    if tool not in TOOLS:
        print(json.dumps({"error": f"Unknown tool: {tool}", "available": list(TOOLS.keys())}))
        sys.exit(1)
    print(json.dumps(TOOLS[tool](args), indent=2))

if __name__ == "__main__":
    main()
