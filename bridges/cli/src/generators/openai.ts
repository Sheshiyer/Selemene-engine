/**
 * Generate OpenAI function-calling definitions from OpenAPI spec.
 * Port of scripts/generate-openai-tools.py
 */
import type {
  OpenAPISpec,
  OpenAPIOperation,
  JSONSchema,
  GeneratorResult,
  SelemeneConfig,
  HttpMethod,
} from "../core/types.js";

function pathToFunctionName(path: string, method: string): string {
  // Remove API version prefix
  let cleaned = path.replace(/^\/api\/v\d+\//, "");
  cleaned = cleaned.replace(/^\/ts\//, "");
  // Remove path parameters
  cleaned = cleaned.replace(/\{[^}]+\}/g, "");
  // Convert to snake_case
  const parts = cleaned.split("/").filter(Boolean);
  const name = parts.join("_");
  // Add method prefix for non-GET if there are parts
  if (method.toUpperCase() !== "GET" && parts.length > 1) {
    return name || method.toLowerCase();
  }
  return name || method.toLowerCase();
}

function extractParameters(
  path: string,
  operation: OpenAPIOperation
): JSONSchema {
  const properties: Record<string, JSONSchema> = {};
  const required: string[] = [];

  // Path parameters from URL template
  const pathParamMatches = path.matchAll(/\{([^}]+)\}/g);
  for (const match of pathParamMatches) {
    const param = match[1];
    properties[param] = {
      type: "string",
      description: `Path parameter: ${param}`,
    };
    required.push(param);
  }

  // Query and other parameters
  for (const param of operation.parameters ?? []) {
    properties[param.name] = {
      type: param.schema?.type ?? "string",
      description: param.description ?? `Parameter: ${param.name}`,
    };
    if (param.required) required.push(param.name);
  }

  // Request body
  const bodySchema =
    operation.requestBody?.content?.["application/json"]?.schema;
  if (bodySchema?.properties) {
    for (const [prop, propSchema] of Object.entries(bodySchema.properties)) {
      properties[prop] = {
        type: propSchema.type ?? "string",
        description: propSchema.description ?? `Body parameter: ${prop}`,
      };
    }
    if (bodySchema.required) {
      required.push(...bodySchema.required);
    }
  }

  return { type: "object", properties, required };
}

export function generateOpenAIFunctions(
  spec: OpenAPISpec,
  config: SelemeneConfig
): GeneratorResult {
  const functions: Array<{
    type: "function";
    function: { name: string; description: string; parameters: JSONSchema };
  }> = [];

  const methods: HttpMethod[] = ["get", "post", "put", "patch", "delete"];

  for (const [path, pathItem] of Object.entries(spec.paths)) {
    for (const method of methods) {
      const operation = pathItem[method] as OpenAPIOperation | undefined;
      if (!operation) continue;

      functions.push({
        type: "function",
        function: {
          name: pathToFunctionName(path, method),
          description:
            operation.summary ||
            operation.description ||
            `${method.toUpperCase()} ${path}`,
          parameters: extractParameters(path, operation),
        },
      });
    }
  }

  const content = JSON.stringify(functions, null, 2);
  const outputPath = `${config.outputDir}/openai/functions.json`;

  return {
    files: [{ path: outputPath, content }],
    summary: `Generated ${functions.length} OpenAI function definitions`,
  };
}
