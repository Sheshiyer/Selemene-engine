/**
 * Fetch and merge Rust + TypeScript OpenAPI specs into a unified spec.
 * Port of scripts/merge-openapi.py
 */
import { fetchJSON } from "../core/http.js";
import type { OpenAPISpec } from "../core/types.js";

interface MergeOptions {
  rustUrl: string;
  tsUrl: string;
  apiKey?: string;
}

interface MergeResult {
  spec: OpenAPISpec;
  rustPathCount: number;
  tsPathCount: number;
  totalPaths: number;
  totalSchemas: number;
  errors: string[];
}

async function fetchSpec(
  url: string,
  apiKey?: string
): Promise<OpenAPISpec | null> {
  try {
    return await fetchJSON<OpenAPISpec>(url, { apiKey });
  } catch {
    return null;
  }
}

export async function mergeSpecs(opts: MergeOptions): Promise<MergeResult> {
  const errors: string[] = [];

  const rustSpecUrl = `${opts.rustUrl}/api/openapi.json`;
  const tsSpecUrl = `${opts.tsUrl}/docs/json`;

  const [rustSpec, tsSpec] = await Promise.all([
    fetchSpec(rustSpecUrl, opts.apiKey),
    fetchSpec(tsSpecUrl, opts.apiKey),
  ]);

  if (!rustSpec) errors.push(`Failed to fetch Rust spec from ${rustSpecUrl}`);
  if (!tsSpec) errors.push(`Failed to fetch TS spec from ${tsSpecUrl}`);

  const unified: OpenAPISpec = {
    openapi: "3.0.3",
    info: {
      title: "Noesis Unified API",
      version: "1.0.0",
      description:
        "Unified API for all 14 Selemene consciousness engines (9 Rust + 5 TypeScript) and 6 workflows",
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
    Object.assign(unified.paths, rustPaths);
    rustPathCount = Object.keys(rustPaths).length;

    // Merge Rust schemas
    const rustSchemas = rustSpec.components?.schemas ?? {};
    Object.assign(unified.components!.schemas!, rustSchemas);

    // Merge Rust security schemes
    const rustSecurity = rustSpec.components?.securitySchemes ?? {};
    Object.assign(unified.components!.securitySchemes!, rustSecurity);

    // Merge Rust tags
    unified.tags = [...(rustSpec.tags ?? [])];
  }

  // Add TS paths with /ts prefix
  if (tsSpec) {
    const tsPaths = tsSpec.paths ?? {};
    for (const [path, operations] of Object.entries(tsPaths)) {
      unified.paths[`/ts${path}`] = operations;
    }
    tsPathCount = Object.keys(tsPaths).length;

    // Merge TS schemas with "Ts" prefix to avoid collisions
    const tsSchemas = tsSpec.components?.schemas ?? {};
    for (const [name, schema] of Object.entries(tsSchemas)) {
      unified.components!.schemas![`Ts${name}`] = schema;
    }

    // Merge TS security schemes
    const tsSecurity = tsSpec.components?.securitySchemes ?? {};
    Object.assign(unified.components!.securitySchemes!, tsSecurity);

    // Merge TS tags
    unified.tags = [...(unified.tags ?? []), ...(tsSpec.tags ?? [])];
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
