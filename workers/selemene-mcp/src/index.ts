// Selemene MCP worker — top-level export.
//
// Wires @cloudflare/workers-oauth-provider with the SDK-based MCP handler.
// Provider cache is keyed by PUBLIC_ORIGIN to prevent stale configuration
// if env vars were to differ (they shouldn't in production, but defensive).

import { OAuthProvider, getOAuthApi, type OAuthProviderOptions } from "@cloudflare/workers-oauth-provider";

import type { Env } from "./env.js";
import { readEnv } from "./env.js";
import { handleAuthorize } from "./consent.js";
import { handleMcp } from "./mcp.js";
import { renderHealth, renderLanding } from "./landing.js";
import { ALL_SCOPES } from "./props.js";
import type { SelemeneAuthProps } from "./props.js";

const AUTHORIZE_PATH = "/authorize";
const TOKEN_PATH = "/oauth/token";
const REGISTER_PATH = "/oauth/register";
const MCP_PATH = "/mcp";

interface OAuthCtx {
  props: SelemeneAuthProps;
  auth: { scope: string[] };
}

const mcpApiHandler = {
  async fetch(request: Request, env: Env, ctx: { waitUntil: (p: Promise<unknown>) => void }) {
    const validated = readEnv(env);
    const url = new URL(request.url);
    if (url.pathname !== MCP_PATH) {
      return new Response("not found", { status: 404 });
    }
    const oauthCtx = ctx as unknown as OAuthCtx;
    return handleMcp(request, {
      env: validated,
      props: oauthCtx.props,
      scope: oauthCtx.auth.scope,
    });
  },
} satisfies ExportedHandler<Env>;

const defaultHandler: ExportedHandler<Env> = {
  async fetch(request, env) {
    let validated;
    try {
      validated = readEnv(env);
    } catch {
      return new Response("worker not configured", { status: 503 });
    }
    const url = new URL(request.url);
    if (url.pathname === "/health") {
      return renderHealth();
    }
    if (url.pathname === "/" || url.pathname === "") {
      return renderLanding(validated.publicOrigin, validated.mcpResource);
    }
    if (url.pathname === AUTHORIZE_PATH) {
      const helpers = getOAuthApi(providerOptions(env), env);
      return handleAuthorize(request, env, helpers);
    }
    if (url.pathname === "/oauth/password") {
      return new Response(
        JSON.stringify({
          error: "gone",
          error_description: "password login retired",
        }),
        { status: 410, headers: { "content-type": "application/json" } },
      );
    }
    return new Response("not found", { status: 404 });
  },
} satisfies ExportedHandler<Env>;

function providerOptions(env: Env): OAuthProviderOptions<Env> {
  const validated = readEnv(env);
  return {
    apiRoute: MCP_PATH,
    apiHandler: mcpApiHandler,
    defaultHandler,
    authorizeEndpoint: `${validated.publicOrigin}${AUTHORIZE_PATH}`,
    tokenEndpoint: `${validated.publicOrigin}${TOKEN_PATH}`,
    clientRegistrationEndpoint: `${validated.publicOrigin}${REGISTER_PATH}`,
    scopesSupported: [...ALL_SCOPES],
    resourceMetadata: {
      resource: validated.mcpResource,
      authorization_servers: [validated.publicOrigin],
      bearer_methods_supported: ["header"],
      resource_name: "Selemene Engine",
    },
    requiredScopes: ["mcp:read"],
    accessTokenTTL: 3600, // 1 hour
    refreshTokenTTL: 30 * 24 * 3600, // 30 days
    disallowPublicClientRegistration: false,
  };
}

// Provider cache keyed by PUBLIC_ORIGIN. If env vars change (shouldn't in
// production, but defensive), a new provider is created.
let cachedOrigin: string | null = null;
let cachedProvider: OAuthProvider<Env> | null = null;

export default {
  async fetch(
    request: Request,
    env: Env,
    ctx: ExecutionContext,
  ): Promise<Response> {
    let validated;
    try {
      validated = readEnv(env);
    } catch {
      return new Response(
        JSON.stringify({
          error: "server_configuration_error",
          error_description: "Worker configuration is unavailable.",
        }),
        { status: 503, headers: { "content-type": "application/json" } },
      );
    }

    // Bind token exchanges to this resource, including refreshes. Bound bytes
    // before parsing rather than trusting an optional Content-Length header.
    if (new URL(request.url).pathname === TOKEN_PATH && request.method === "POST") {
      const contentType = request.headers.get("content-type") ?? "";
      if (!contentType.startsWith("application/x-www-form-urlencoded")) return Response.json({error:"invalid_request"},{status:415});
      const reader = request.body?.getReader(); const chunks: Uint8Array[] = []; let total=0;
      try {
        if(reader) while(true) { const {value,done}=await reader.read();if(done)break;total+=value.byteLength;
          if(total>16384){void reader.cancel().catch(()=>{});return Response.json({error:"invalid_request"},{status:413});}chunks.push(value);
        }
      } catch {return Response.json({error:"invalid_request"},{status:400});}
      const bytes=new Uint8Array(total);let offset=0;for(const chunk of chunks){bytes.set(chunk,offset);offset+=chunk.byteLength;}
      const form=new URLSearchParams(new TextDecoder().decode(bytes));
      const grant=form.get("grant_type");
      if (grant === "authorization_code" || grant === "refresh_token") {
        const resources=form.getAll("resource");
        if(resources.length!==1 || resources[0]!==validated.mcpResource) return Response.json({error:"invalid_target",error_description:"Use the canonical MCP resource."},{status:400});
      }
      request=new Request(request.url,{method:"POST",headers:request.headers,body:bytes});
    }
    // Invalidate cache if origin changed (defensive).
    if (cachedOrigin !== validated.publicOrigin) {
      cachedProvider = null;
      cachedOrigin = null;
    }
    if (!cachedProvider) {
      cachedProvider = new OAuthProvider<Env>(providerOptions(env));
      cachedOrigin = validated.publicOrigin;
    }
    return cachedProvider.fetch(request, env, ctx);
  },
} satisfies ExportedHandler<Env>;

export { mcpApiHandler, defaultHandler };
