/**
 * Generate LangChain/CrewAI tool definitions from OpenAPI spec.
 * Port of scripts/generate-langchain-tools.py — emits Python via template literals.
 */
import type {
  OpenAPISpec,
  OpenAPIOperation,
  OpenAPIParameter,
  JSONSchema,
  GeneratorResult,
  SelemeneConfig,
  HttpMethod,
} from "../core/types.js";

function pathToSnakeCase(path: string, _method: string): string {
  let clean = path.replace(/^\/api\/v\d+\//, "");
  clean = clean.replace(/^\/ts\//, "ts_");
  clean = clean.replace(/^\//, "");
  clean = clean.replace(/\{[^}]+\}/g, "");
  clean = clean.replace(/[/-]/g, "_");
  clean = clean.replace(/_+/g, "_").replace(/^_|_$/g, "");

  if (!clean) {
    const parts = path.split("/");
    clean = parts[parts.length - 1] || "root";
  }

  const prefix = path.includes("/ts/") ? "selemene_ts_" : "selemene_";
  return prefix + clean.toLowerCase();
}

function pythonTypeFromSchema(schema: JSONSchema): string {
  const typeMap: Record<string, string> = {
    string: "str",
    integer: "int",
    number: "float",
    boolean: "bool",
    object: "dict",
    array: "list",
  };
  return typeMap[schema.type ?? "string"] ?? "Any";
}

function escapeDescription(desc: string): string {
  return desc.replace(/\\/g, "\\\\").replace(/"/g, '\\"');
}

function buildInputSchemaClass(
  toolName: string,
  params: OpenAPIParameter[],
  bodySchema: JSONSchema | null
): { className: string; code: string; hasParams: boolean } {
  const className =
    toolName
      .split("_")
      .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
      .join("") + "Input";

  const fields: string[] = [];

  // Path parameters
  for (const param of params.filter((p) => p.in === "path")) {
    const pyType = pythonTypeFromSchema(param.schema ?? { type: "string" });
    const desc = escapeDescription(
      param.description ?? `${param.name} parameter`
    );
    fields.push(`    ${param.name}: ${pyType} = Field(description="${desc}")`);
  }

  // Query parameters
  for (const param of params.filter((p) => p.in === "query")) {
    const pyType = pythonTypeFromSchema(param.schema ?? { type: "string" });
    const desc = escapeDescription(
      param.description ?? `${param.name} parameter`
    );
    if (param.required) {
      fields.push(
        `    ${param.name}: ${pyType} = Field(description="${desc}")`
      );
    } else {
      fields.push(
        `    ${param.name}: Optional[${pyType}] = Field(None, description="${desc}")`
      );
    }
  }

  // Body parameters
  if (bodySchema?.properties) {
    const requiredList = bodySchema.required ?? [];
    for (const [propName, propSchema] of Object.entries(
      bodySchema.properties
    )) {
      const pyType = pythonTypeFromSchema(propSchema);
      const desc = escapeDescription(
        propSchema.description ?? `${propName} parameter`
      );
      if (requiredList.includes(propName)) {
        fields.push(`    ${propName}: ${pyType} = Field(description="${desc}")`);
      } else {
        fields.push(
          `    ${propName}: Optional[${pyType}] = Field(None, description="${desc}")`
        );
      }
    }
  }

  const hasParams = fields.length > 0;

  if (!hasParams) {
    return {
      className,
      code: `class ${className}(BaseModel):\n    pass\n`,
      hasParams: false,
    };
  }

  return {
    className,
    code: `class ${className}(BaseModel):\n${fields.join("\n")}\n`,
    hasParams: true,
  };
}

function buildFunction(
  toolName: string,
  path: string,
  method: string,
  params: OpenAPIParameter[],
  bodySchema: JSONSchema | null,
  description: string
): string {
  const funcName = toolName.replace(/^selemene_(ts_)?/, "");
  const baseUrl = path.includes("/ts/") ? "TS_URL" : "RUST_URL";

  // Build param definitions
  const paramDefs: string[] = [];
  const paramNames: string[] = [];

  for (const param of params) {
    const pyType = pythonTypeFromSchema(param.schema ?? { type: "string" });
    if (param.required) {
      paramDefs.push(`${param.name}: ${pyType}`);
    } else {
      paramDefs.push(`${param.name}: Optional[${pyType}] = None`);
    }
    paramNames.push(param.name);
  }

  if (bodySchema?.properties) {
    const requiredList = bodySchema.required ?? [];
    for (const [propName, propSchema] of Object.entries(
      bodySchema.properties
    )) {
      const pyType = pythonTypeFromSchema(propSchema);
      if (requiredList.includes(propName)) {
        paramDefs.push(`${propName}: ${pyType}`);
      } else {
        paramDefs.push(`${propName}: Optional[${pyType}] = None`);
      }
      paramNames.push(propName);
    }
  }

  const signature = `def ${funcName}(${paramDefs.join(", ")}) -> dict:`;
  const escapedDesc = escapeDescription(description);
  const urlLine = `    url = f"{${baseUrl}}${path}"`;

  if (method.toUpperCase() === "GET") {
    const queryParams = params.filter((p) => p.in === "query");
    if (queryParams.length > 0) {
      const items = queryParams
        .map((p) => `"${p.name}": ${p.name}`)
        .join(", ");
      return `${signature}\n    """${escapedDesc}"""\n${urlLine}\n    params = {${items}}\n    return _get(url, params=params)\n`;
    }
    return `${signature}\n    """${escapedDesc}"""\n${urlLine}\n    return _get(url)\n`;
  }

  // POST/PUT/PATCH/DELETE
  if (bodySchema?.properties) {
    const items = Object.keys(bodySchema.properties)
      .map((p) => `"${p}": ${p}`)
      .join(", ");
    return `${signature}\n    """${escapedDesc}"""\n${urlLine}\n    body = {${items}}\n    return _post(url, body)\n`;
  }
  return `${signature}\n    """${escapedDesc}"""\n${urlLine}\n    return _post(url, {})\n`;
}

function buildStructuredTool(
  toolName: string,
  description: string,
  inputClass: string,
  hasParams: boolean
): { code: string; varName: string } {
  const cleanFuncName = toolName.replace(/^selemene_(ts_)?/, "");
  const varName = `${toolName}_tool`;

  const argsLine = hasParams ? `\n    args_schema=${inputClass},` : "";
  const code = `${varName} = StructuredTool.from_function(
    func=${cleanFuncName},
    name="${toolName}",
    description="${escapeDescription(description)}",${argsLine}
)\n`;

  return { code, varName };
}

export function generateLangChainTools(
  spec: OpenAPISpec,
  config: SelemeneConfig
): GeneratorResult {
  const inputSchemas: string[] = [];
  const toolFunctions: string[] = [];
  const structuredTools: string[] = [];
  const toolVarNames: string[] = [];

  const methods: HttpMethod[] = ["get", "post", "put", "patch", "delete"];

  for (const [path, pathItem] of Object.entries(spec.paths)) {
    for (const method of methods) {
      const operation = pathItem[method] as OpenAPIOperation | undefined;
      if (!operation) continue;

      const params = operation.parameters ?? [];

      let bodySchema: JSONSchema | null = null;
      if (operation.requestBody?.content?.["application/json"]?.schema) {
        bodySchema = operation.requestBody.content["application/json"].schema;
      }

      const toolName = pathToSnakeCase(path, method);
      const description =
        operation.summary ||
        operation.description ||
        `${method.toUpperCase()} ${path}`;

      // Input schema class
      const schema = buildInputSchemaClass(toolName, params, bodySchema);
      if (schema.hasParams) {
        inputSchemas.push(schema.code);
      }

      // Function
      toolFunctions.push(
        buildFunction(toolName, path, method, params, bodySchema, description)
      );

      // StructuredTool
      const tool = buildStructuredTool(
        toolName,
        description,
        schema.className,
        schema.hasParams
      );
      structuredTools.push(tool.code);
      toolVarNames.push(tool.varName);
    }
  }

  const module = `"""Auto-generated Selemene Engine tools for LangChain/CrewAI.
Generated by @selemene/bridge from OpenAPI spec.
"""

import httpx
import os
from typing import Optional, Any
from pydantic import BaseModel, Field
from langchain_core.tools import StructuredTool


RUST_URL = os.environ.get("SELEMENE_RUST_URL", "${config.rustUrl}")
TS_URL = os.environ.get("SELEMENE_TS_URL", "${config.tsUrl}")
API_KEY = os.environ.get("SELEMENE_API_KEY", "")


def _headers() -> dict:
    """Build request headers with optional API key."""
    h = {"Content-Type": "application/json"}
    if API_KEY:
        h["Authorization"] = f"Bearer {API_KEY}"
    return h


def _get(url: str, params: dict = None) -> dict:
    """Execute GET request."""
    with httpx.Client(timeout=30.0) as client:
        r = client.get(url, headers=_headers(), params=params)
        r.raise_for_status()
        return r.json()


def _post(url: str, body: dict) -> dict:
    """Execute POST request."""
    with httpx.Client(timeout=30.0) as client:
        r = client.post(url, headers=_headers(), json=body)
        r.raise_for_status()
        return r.json()


# --- Input Schemas ---

${inputSchemas.length > 0 ? inputSchemas.join("\n") : "# No input schemas needed\n"}

# --- Tool Functions ---

${toolFunctions.join("\n")}

# --- StructuredTool instances ---

${structuredTools.join("\n")}

def get_all_tools() -> list:
    """Return all Selemene tools for LangChain/CrewAI."""
    return [${toolVarNames.join(", ")}]
`;

  const outputPath = `${config.outputDir}/langchain/selemene_tools.py`;

  return {
    files: [{ path: outputPath, content: module }],
    summary: `Generated ${toolVarNames.length} LangChain tools`,
  };
}
