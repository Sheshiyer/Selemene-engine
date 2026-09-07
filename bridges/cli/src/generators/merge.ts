/**
 * Fetch the protected Rust OpenAPI contract for public tool generation.
 *
 * The TypeScript URL remains in configuration for legacy diagnostics, but
 * direct sidecar paths are intentionally never merged into generated tools.
 */
import { fetchJSON } from "../core/http.js";
import type { OpenAPISpec } from "../core/types.js";

interface MergeOptions {
  rustUrl: string;
  tsUrl: string;
  apiKey?: string;
  bearerToken?: string;
}

interface MergeResult {
  spec: OpenAPISpec;
  rustPathCount: number;
  tsPathCount: number;
  totalPaths: number;
  totalSchemas: number;
  errors: string[];
}

function isPublicRustPath(path: string): boolean {
  return (
    path === "/api/v1/status" ||
    path.startsWith("/api/v1/engines") ||
    path.startsWith("/api/v1/workflows")
  );
}

async function fetchSpec(
  url: string,
  credentials: Pick<MergeOptions, "apiKey" | "bearerToken">
): Promise<OpenAPISpec | null> {
  try {
    return await fetchJSON<OpenAPISpec>(url, credentials);
  } catch {
    return null;
  }
}

export async function mergeSpecs(opts: MergeOptions): Promise<MergeResult> {
  const errors: string[] = [];

  const rustSpecUrl = `${opts.rustUrl}/api/openapi.json`;
  const rustSpec = await fetchSpec(rustSpecUrl, opts);

  if (!rustSpec) errors.push(`Failed to fetch Rust spec from ${rustSpecUrl}`);

  const unified: OpenAPISpec = {
    openapi: "3.0.3",
    info: {
      title: "Noesis Unified API",
      version: "1.0.0",
      description:
        "Protected Rust API for canonical Selemene engine and workflow operations",
    },
    paths: {},
    components: { schemas: {}, securitySchemes: {} },
    tags: [],
  };

  let rustPathCount = 0;
  let tsPathCount = 0;

  // Add Rust paths directly
  if (rustSpec) {
    const rustPaths = rustSpec.paths ?? {};
    for (const [path, operations] of Object.entries(rustPaths)) {
      if (isPublicRustPath(path)) unified.paths[path] = operations;
    }
    rustPathCount = Object.keys(unified.paths).length;

    // Merge Rust schemas
    const rustSchemas = rustSpec.components?.schemas ?? {};
    Object.assign(unified.components!.schemas!, rustSchemas);

    // Merge Rust security schemes
    const rustSecurity = rustSpec.components?.securitySchemes ?? {};
    Object.assign(unified.components!.securitySchemes!, rustSecurity);

    // Merge Rust tags
    unified.tags = [...(rustSpec.tags ?? [])];
  }

  return {
    spec: unified,
    rustPathCount,
    tsPathCount,
    totalPaths: Object.keys(unified.paths).length,
    totalSchemas: Object.keys(unified.components?.schemas ?? {}).length,
    errors,
  };
}
