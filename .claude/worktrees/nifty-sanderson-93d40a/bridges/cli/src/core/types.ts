import { z } from "zod";

// --- Config Schema ---

export const SelemeneConfigSchema = z.object({
  version: z.string().default("1.0"),
  rustUrl: z.string().url(),
  tsUrl: z.string().url(),
  apiKey: z.string().optional(),
  frameworks: z.array(z.enum(["claude", "openai", "langchain"])),
  outputDir: z.string().default("./selemene-tools"),
  lastGenerated: z.string().optional(),
});

export type SelemeneConfig = z.infer<typeof SelemeneConfigSchema>;

// --- OpenAPI Types ---

export interface OpenAPIParameter {
  name: string;
  in: "path" | "query" | "header" | "cookie";
  required?: boolean;
  description?: string;
  schema?: JSONSchema;
}

export interface JSONSchema {
  type?: string;
  properties?: Record<string, JSONSchema>;
  required?: string[];
  description?: string;
  items?: JSONSchema;
  enum?: string[];
  format?: string;
  default?: unknown;
}

export interface OpenAPIOperation {
  summary?: string;
  description?: string;
  operationId?: string;
  parameters?: OpenAPIParameter[];
  requestBody?: {
    content?: {
      "application/json"?: {
        schema?: JSONSchema;
      };
    };
  };
  responses?: Record<string, unknown>;
  tags?: string[];
}

export interface OpenAPIPathItem {
  get?: OpenAPIOperation;
  post?: OpenAPIOperation;
  put?: OpenAPIOperation;
  patch?: OpenAPIOperation;
  delete?: OpenAPIOperation;
  parameters?: OpenAPIParameter[];
}

export interface OpenAPISpec {
  openapi: string;
  info: {
    title: string;
    version: string;
    description?: string;
  };
  paths: Record<string, OpenAPIPathItem>;
  components?: {
    schemas?: Record<string, JSONSchema>;
    securitySchemes?: Record<string, unknown>;
  };
  tags?: Array<{ name: string; description?: string }>;
}

// --- Generator Types ---

export interface GeneratedFile {
  path: string;
  content: string;
}

export interface GeneratorResult {
  files: GeneratedFile[];
  summary: string;
}

export type HttpMethod = "get" | "post" | "put" | "patch" | "delete";

export const HTTP_METHODS: HttpMethod[] = [
  "get",
  "post",
  "put",
  "patch",
  "delete",
];
