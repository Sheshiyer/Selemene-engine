// Runtime environment shape for the Selemene MCP worker.
// KV binding is provided by @cloudflare/workers-oauth-provider; two string vars
// are validated on first request and returned by `readEnv`.

export interface Env {
  OAUTH_KV: KVNamespace;
  PUBLIC_ORIGIN: string;
  SELEMENE_API_ORIGIN: string;
}

export interface ValidatedEnv {
  publicOrigin: string;
  mcpResource: string;
  authorizeEndpoint: string;
  tokenEndpoint: string;
  clientRegistrationEndpoint: string;
  selemeneApiOrigin: string;
}

// Fixed HTTPS allowlist for the Selemene Rust API. Only verified upstream
// origins are accepted. Speculative or unverified hosts are rejected.
const ALLOWED_SELEMENE_ORIGINS = new Set<string>([
  "https://selemene-engine-production.up.railway.app",
  "https://selemene.tryambakam.space",
]);

export function readEnv(env: Env): ValidatedEnv {
  const publicOrigin = requireHttpsOrigin(env.PUBLIC_ORIGIN, "PUBLIC_ORIGIN");
  const apiOrigin = requireHttpsOrigin(env.SELEMENE_API_ORIGIN, "SELEMENE_API_ORIGIN");
  if (!ALLOWED_SELEMENE_ORIGINS.has(apiOrigin)) {
    throw new Error(
      `SELEMENE_API_ORIGIN ${apiOrigin} is not in the fixed HTTPS allowlist`,
    );
  }
  return {
    publicOrigin,
    mcpResource: `${publicOrigin}/mcp`,
    authorizeEndpoint: `${publicOrigin}/authorize`,
    tokenEndpoint: `${publicOrigin}/oauth/token`,
    clientRegistrationEndpoint: `${publicOrigin}/oauth/register`,
    selemeneApiOrigin: apiOrigin,
  };
}

function requireHttpsOrigin(raw: string | undefined, name: string): string {
  if (!raw || raw.startsWith("REPLACE_WITH_")) {
    throw new Error(`${name} env var is not configured`);
  }
  let url: URL;
  try {
    url = new URL(raw);
  } catch {
    throw new Error(`${name} is not a valid URL`);
  }
  if (url.protocol !== "https:") {
    throw new Error(`${name} must use https://`);
  }
  // Reject userinfo (user:password@host) — credentials in URLs are unsafe.
  if (url.username || url.password) {
    throw new Error(`${name} must not include userinfo (user:password@host)`);
  }
  // Reject non-default ports — only standard HTTPS (443) is allowed.
  if (url.port && url.port !== "443") {
    throw new Error(`${name} must not include a non-standard port`);
  }
  if (url.pathname !== "/" && url.pathname !== "") {
    throw new Error(`${name} must not include a path`);
  }
  if (url.search || url.hash) {
    throw new Error(`${name} must not include query or fragment`);
  }
  return `${url.protocol}//${url.host}`;
}
