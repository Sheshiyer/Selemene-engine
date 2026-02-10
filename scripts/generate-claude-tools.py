#!/usr/bin/env python3
"""
Generate Anthropic Claude tool definitions from OpenAPI spec.

Reads openapi-unified.json and outputs Claude-compatible tool definitions
to bridges/claude/tools.json.
"""

import json
import os
import re
import sys
from pathlib import Path


def path_to_snake_case(path: str, method: str) -> str:
    """Convert OpenAPI path to snake_case tool name."""
    # Remove leading slash and /api/v1 prefix
    clean = re.sub(r'^/api/v\d+/', '', path)
    clean = re.sub(r'^/ts/', 'ts_', clean)
    clean = clean.lstrip('/')

    # Replace path params {id} with generic name
    clean = re.sub(r'\{[^}]+\}', '', clean)

    # Replace slashes and hyphens with underscores
    clean = re.sub(r'[/-]', '_', clean)

    # Remove duplicate underscores and trailing underscores
    clean = re.sub(r'_+', '_', clean)
    clean = clean.strip('_')

    # If empty (e.g., /health), use the last part of original path
    if not clean:
        clean = path.split('/')[-1] or 'root'

    return clean.lower()


def merge_schemas(path_params: dict, query_params: dict, body_schema: dict) -> dict:
    """Merge path params, query params, and request body into single schema."""
    properties = {}
    required = []

    # Add path parameters
    for param in path_params:
        name = param['name']
        properties[name] = param.get('schema', {'type': 'string'})
        if param.get('required', False):
            required.append(name)
        if 'description' in param:
            properties[name]['description'] = param['description']

    # Add query parameters
    for param in query_params:
        name = param['name']
        properties[name] = param.get('schema', {'type': 'string'})
        if param.get('required', False):
            required.append(name)
        if 'description' in param:
            properties[name]['description'] = param['description']

    # Add request body properties
    if body_schema and 'properties' in body_schema:
        properties.update(body_schema['properties'])
        if 'required' in body_schema:
            required.extend(body_schema['required'])

    return {
        'type': 'object',
        'properties': properties,
        'required': list(set(required))  # Remove duplicates
    }


def generate_tools(openapi_path: str) -> list:
    """Generate Claude tools from OpenAPI spec."""
    with open(openapi_path, 'r') as f:
        spec = json.load(f)

    tools = []

    for path, path_item in spec.get('paths', {}).items():
        for method, operation in path_item.items():
            if method not in ['get', 'post', 'put', 'patch', 'delete']:
                continue

            # Extract parameters
            params = operation.get('parameters', [])
            path_params = [p for p in params if p.get('in') == 'path']
            query_params = [p for p in params if p.get('in') == 'query']

            # Extract request body schema
            body_schema = {}
            if 'requestBody' in operation:
                content = operation['requestBody'].get('content', {})
                json_content = content.get('application/json', {})
                body_schema = json_content.get('schema', {})

            # Build tool
            tool = {
                'name': path_to_snake_case(path, method),
                'description': operation.get('summary') or operation.get('description') or f"{method.upper()} {path}",
                'input_schema': merge_schemas(path_params, query_params, body_schema)
            }

            tools.append(tool)

    return tools


def main():
    # Determine input and output paths
    project_root = Path(__file__).parent.parent
    input_path = project_root / 'openapi-unified.json'
    output_path = project_root / 'bridges' / 'claude' / 'tools.json'

    # Check for --input flag
    if len(sys.argv) > 2 and sys.argv[1] == '--input':
        input_path = Path(sys.argv[2])

    if not input_path.exists():
        print(f"ERROR: OpenAPI spec not found at {input_path}", file=sys.stderr)
        print("Run from project root or use --input flag to specify path", file=sys.stderr)
        sys.exit(1)

    # Generate tools
    tools = generate_tools(str(input_path))

    # Ensure output directory exists
    output_path.parent.mkdir(parents=True, exist_ok=True)

    # Write output
    with open(output_path, 'w') as f:
        json.dump(tools, f, indent=2)

    print(f"✓ Generated {len(tools)} Claude tools from {input_path}")
    print(f"✓ Written to {output_path}")


if __name__ == '__main__':
    main()
