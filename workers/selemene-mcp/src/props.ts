// Encrypted OAuth grant props exposed to the MCP handler as `ctx.props`.
//
// The Selemene user's API key is persisted only inside these encrypted props
// (the library encrypts them with a per-grant key wrapped by the token; see
// @cloudflare/workers-oauth-provider storage-schema.md). The plaintext key is
// never written to KV state or logs, and never sent to ChatGPT.

export interface Scope {
  read: boolean;
  calculate: boolean;
}

export interface SelemeneAuthProps {
  userId: string;         // Selemene UUID from GET /api/v1/users/me
  email?: string;         // for the plugin session display only, never logged
  tier: string;
  consciousnessLevel: number;
  apiKey: string;         // nk_... — encrypted at rest by workers-oauth-provider
  linkedAt: number;       // epoch seconds
}

export const SCOPES = {
  read: "mcp:read",
  calculate: "mcp:calculate",
} as const;

export const ALL_SCOPES: readonly string[] = [SCOPES.read, SCOPES.calculate];

export function parseScopes(scope: readonly string[] | undefined): Scope {
  const set = new Set(scope ?? []);
  return {
    read: set.has(SCOPES.read),
    calculate: set.has(SCOPES.calculate),
  };
}
