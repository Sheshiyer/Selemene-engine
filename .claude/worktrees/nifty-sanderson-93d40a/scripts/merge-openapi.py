#!/usr/bin/env python3
"""Merge Rust and TypeScript OpenAPI specs into a unified spec."""

import json
import sys
import os
from urllib.request import urlopen
from urllib.error import URLError

RUST_URL = "http://localhost:8080/api/openapi.json"
TS_URL = "http://localhost:3001/docs/json"
RUST_STATIC = "crates/noesis-api/openapi.json"
TS_STATIC = "ts-engines/openapi.json"
OUTPUT = "openapi-unified.json"

def fetch_spec(url, static_path, offline=False):
    """Fetch OpenAPI spec from URL or static file."""
    if offline:
        print(f"[OFFLINE] Loading from {static_path}")
        with open(static_path, 'r') as f:
            return json.load(f)

    try:
        print(f"Fetching from {url}")
        with urlopen(url, timeout=5) as response:
            return json.loads(response.read())
    except (URLError, TimeoutError) as e:
        print(f"Failed to fetch from {url}: {e}")
        print(f"Falling back to {static_path}")
        with open(static_path, 'r') as f:
            return json.load(f)

def merge_specs(rust_spec, ts_spec):
    """Merge two OpenAPI specs."""
    unified = {
        "openapi": "3.0.3",
        "info": {
            "title": "Noesis Unified API",
            "version": "1.0.0",
            "description": "Unified API for all 14 Selemene consciousness engines (9 Rust + 5 TypeScript) and 6 workflows"
        },
        "paths": {},
        "components": {"schemas": {}, "securitySchemes": {}},
        "tags": []
    }

    # Add Rust paths directly
    unified["paths"].update(rust_spec.get("paths", {}))

    # Add TS paths with /ts prefix
    for path, operations in ts_spec.get("paths", {}).items():
        prefixed_path = f"/ts{path}"
        unified["paths"][prefixed_path] = operations

    # Merge Rust schemas
    rust_schemas = rust_spec.get("components", {}).get("schemas", {})
    unified["components"]["schemas"].update(rust_schemas)

    # Merge TS schemas with "Ts" prefix to avoid collisions
    ts_schemas = ts_spec.get("components", {}).get("schemas", {})
    for schema_name, schema_def in ts_schemas.items():
        unified["components"]["schemas"][f"Ts{schema_name}"] = schema_def

    # Merge security schemes
    rust_security = rust_spec.get("components", {}).get("securitySchemes", {})
    ts_security = ts_spec.get("components", {}).get("securitySchemes", {})
    unified["components"]["securitySchemes"].update(rust_security)
    unified["components"]["securitySchemes"].update(ts_security)

    # Merge tags
    rust_tags = rust_spec.get("tags", [])
    ts_tags = ts_spec.get("tags", [])
    unified["tags"] = rust_tags + ts_tags

    return unified

def main():
    offline = "--offline" in sys.argv

    try:
        rust_spec = fetch_spec(RUST_URL, RUST_STATIC, offline)
        ts_spec = fetch_spec(TS_URL, TS_STATIC, offline)

        unified = merge_specs(rust_spec, ts_spec)

        with open(OUTPUT, 'w') as f:
            json.dump(unified, f, indent=2)

        print(f"\n✓ Unified OpenAPI spec written to {OUTPUT}")
        print(f"  - Rust paths: {len(rust_spec.get('paths', {}))}")
        print(f"  - TS paths: {len(ts_spec.get('paths', {}))} (prefixed with /ts)")
        print(f"  - Total paths: {len(unified['paths'])}")
        print(f"  - Total schemas: {len(unified['components']['schemas'])}")

    except FileNotFoundError as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"Unexpected error: {e}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()
