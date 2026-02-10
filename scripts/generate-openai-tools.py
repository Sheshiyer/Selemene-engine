#!/usr/bin/env python3
"""
Generate OpenAI function-calling definitions from OpenAPI spec.
Input: openapi-unified.json (or --input path)
Output: bridges/openai/functions.json
"""

import json
import re
import sys
from pathlib import Path
from typing import Any, Dict, List


def path_to_function_name(path: str, method: str) -> str:
    """Convert path like /api/v1/engines/{engine_id}/calculate to engines_calculate"""
    # Remove API version prefix
    cleaned = re.sub(r'^/api/v\d+/', '', path)
    cleaned = re.sub(r'^/ts/', '', cleaned)
    # Remove path parameters
    cleaned = re.sub(r'\{[^}]+\}', '', cleaned)
    # Convert to snake_case
    parts = [p for p in cleaned.split('/') if p]
    name = '_'.join(parts)
    # Add method prefix for non-GET
    if method.upper() != 'GET' and len(parts) > 1:
        return name
    return name or method.lower()


def extract_parameters(path: str, operation: Dict[str, Any]) -> Dict[str, Any]:
    """Build OpenAI parameters from OpenAPI operation"""
    properties = {}
    required = []

    # Path parameters
    path_params = re.findall(r'\{([^}]+)\}', path)
    for param in path_params:
        properties[param] = {
            "type": "string",
            "description": f"Path parameter: {param}"
        }
        required.append(param)

    # Query and other parameters
    for param in operation.get('parameters', []):
        name = param['name']
        schema = param.get('schema', {})
        properties[name] = {
            "type": schema.get('type', 'string'),
            "description": param.get('description', f"Parameter: {name}")
        }
        if param.get('required', False):
            required.append(name)

    # Request body
    if 'requestBody' in operation:
        content = operation['requestBody'].get('content', {})
        json_schema = content.get('application/json', {}).get('schema', {})

        if 'properties' in json_schema:
            for prop, prop_schema in json_schema['properties'].items():
                properties[prop] = {
                    "type": prop_schema.get('type', 'string'),
                    "description": prop_schema.get('description', f"Body parameter: {prop}")
                }
            if 'required' in json_schema:
                required.extend(json_schema['required'])

    return {
        "type": "object",
        "properties": properties,
        "required": required
    }


def convert_openapi_to_functions(spec: Dict[str, Any]) -> List[Dict[str, Any]]:
    """Convert OpenAPI spec to OpenAI function-calling format"""
    functions = []

    for path, path_item in spec.get('paths', {}).items():
        for method, operation in path_item.items():
            if method.upper() in ['GET', 'POST', 'PUT', 'DELETE', 'PATCH']:
                func_name = path_to_function_name(path, method)
                description = operation.get('summary') or operation.get('description') or f"{method.upper()} {path}"

                functions.append({
                    "type": "function",
                    "function": {
                        "name": func_name,
                        "description": description,
                        "parameters": extract_parameters(path, operation)
                    }
                })

    return functions


def main():
    # Parse args
    input_file = Path('openapi-unified.json')
    if '--input' in sys.argv:
        idx = sys.argv.index('--input')
        if idx + 1 < len(sys.argv):
            input_file = Path(sys.argv[idx + 1])

    output_file = Path('bridges/openai/functions.json')

    # Read OpenAPI spec
    if not input_file.exists():
        print(f"Error: {input_file} not found", file=sys.stderr)
        print("Run from project root or use --input <path>", file=sys.stderr)
        sys.exit(1)

    with open(input_file) as f:
        spec = json.load(f)

    # Convert to OpenAI functions
    functions = convert_openapi_to_functions(spec)

    # Write output
    output_file.parent.mkdir(parents=True, exist_ok=True)
    with open(output_file, 'w') as f:
        json.dump(functions, f, indent=2)

    # Summary
    print(f"✓ Generated {len(functions)} OpenAI function definitions")
    print(f"  Input:  {input_file}")
    print(f"  Output: {output_file}")
    print("\nFunctions:")
    for func in functions:
        name = func['function']['name']
        param_count = len(func['function']['parameters']['properties'])
        print(f"  - {name} ({param_count} params)")


if __name__ == '__main__':
    main()
