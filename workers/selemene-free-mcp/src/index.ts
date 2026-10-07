import { readEnv, type Env } from "./env.js";
import { handleMcp } from "./mcp.js";
import { clientHash, reserve } from "./rate-limiter.js";
export { FreeRateLimiter } from "./rate-limiter.js";

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);
    if (url.pathname !== "/mcp" && url.pathname !== "/health") return new Response(null, { status: 404 });
    if (url.pathname === "/health" && request.method !== "GET") return new Response(null, { status: 405, headers: { Allow: "GET" } });
    if (url.pathname === "/mcp" && request.method !== "POST") return new Response(null, { status: 405, headers: { Allow: "POST" } });
    try {
      const service = readEnv(env);
      if (url.pathname === "/health") return Response.json({
        status: "ok", service: "selemene-free-mcp", authentication: "none",
        storage: "shared-upstream-administrator-account", rate_limiter: "configured",
        interpretation: "host-chatgpt", version: "0.2.0",
      }, { headers: { "cache-control": "no-store" } });
      const hash = await clientHash(request, service.salt);
      if (!hash) return Response.json({ error: "Service request identity unavailable." }, { status: 503, headers: { "cache-control": "no-store" } });
      const admission = await reserve(service, hash, "request");
      if (!admission.allowed) return Response.json({ error: "Free service quota reached. Try again later." },
        { status: admission.reason === "unavailable" ? 503 : 429, headers: { "cache-control": "no-store", "retry-after": String(admission.retryAfter) } });
      return await handleMcp(request, { service, reserveCalculation: () => reserve(service, hash, "calculation") });
    } catch { return Response.json({ error: "Sponsored service is unavailable." }, { status: 503, headers: { "cache-control": "no-store" } }); }
  },
};
