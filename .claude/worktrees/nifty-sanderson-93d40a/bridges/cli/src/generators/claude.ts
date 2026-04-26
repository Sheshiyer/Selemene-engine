/**
 * Generate Claude tool definitions from OpenAPI spec.
 * Port of scripts/generate-claude-tools.py
 */
import type {
  OpenAPISpec,
  OpenAPIOperation,
  OpenAPIParameter,
  JSONSchema,
  GeneratorResult,
  SelemeneConfig,
  HttpMethod,
  HTTP_METHODS,
} from "../core/types.js";

function pathToSnakeCase(path: string, _method: string): string {
  // Remove leading slash and /api/v1 prefix
  let clean = path.replace(/^\/api\/v\d+\//, "");
  clean = clean.replace(/^\/ts\//, "ts_");
  clean = clean.replace(/^\//, "");

  // Replace path params {id} with nothing
  clean = clean.replace(/\{[^}]+\}/g, "");

  // Replace slashes and hyphens with underscores
  clean = clean.replace(/[/-]/g, "_");

  // Remove duplicate underscores and trim
  clean = clean.replace(/_+/g, "_").replace(/^_|_$/g, "");

  // If empty, use last part of original path
  if (!clean) {
    const parts = path.split("/");
    clean = parts[parts.length - 1] || "root";
  }

  return clean.toLowerCase();
}

function mergeSchemas(
  pathParams: OpenAPIParameter[],
  queryParams: OpenAPIParameter[],
  bodySchema: JSONSchema | null
): JSONSchema {
  const properties: Record<string, JSONSchema> = {};
  const required: string[] = [];

  // Add path parameters
  for (const param of pathParams) {
    const prop: JSONSchema = param.schema ?? { type: "string" };
    if (param.description) prop.description = param.description;
    properties[param.name] = prop;
    if (param.required) required.push(param.name);
  }

  // Add query parameters
  for (const param of queryParams) {
    const prop: JSONSchema = param.schema ?? { type: "string" };
    if (param.description) prop.description = param.description;
    properties[param.name] = prop;
    if (param.required) required.push(param.name);
  }

  // Add request body properties
  if (bodySchema?.properties) {
    Object.assign(properties, bodySchema.properties);
    if (bodySchema.required) {
      required.push(...bodySchema.required);
    }
  }

  return {
    type: "object",
    properties,
    required: [...new Set(required)],
  };
}

export function generateClaudeTools(
  spec: OpenAPISpec,
  config: SelemeneConfig
): GeneratorResult {
  const tools: Array<{
    name: string;
    description: string;
    input_schema: JSONSchema;
  }> = [];

  const methods: HttpMethod[] = ["get", "post", "put", "patch", "delete"];

  for (const [path, pathItem] of Object.entries(spec.paths)) {
    for (const method of methods) {
      const operation = pathItem[method] as OpenAPIOperation | undefined;
      if (!operation) continue;

      const params = operation.parameters ?? [];
      const pathParams = params.filter((p) => p.in === "path");
      const queryParams = params.filter((p) => p.in === "query");

      let bodySchema: JSONSchema | null = null;
      if (operation.requestBody?.content?.["application/json"]?.schema) {
        bodySchema = operation.requestBody.content["application/json"].schema;
      }

      tools.push({
        name: pathToSnakeCase(path, method),
        description:
          operation.summary ||
          operation.description ||
          `${method.toUpperCase()} ${path}`,
        input_schema: mergeSchemas(pathParams, queryParams, bodySchema),
      });
    }
  }

  const content = JSON.stringify(tools, null, 2);
  const outputPath = `${config.outputDir}/claude/tools.json`;

  return {
    files: [{ path: outputPath, content }],
    summary: `Generated ${tools.length} Claude tools`,
  };
}
