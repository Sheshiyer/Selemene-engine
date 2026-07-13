/**
 * Same-origin proxy for the Selemene admin dashboard.
 *
 * The admin-web frontend is served from 144.tryambakam.space (Vercel).
 * The API backend runs on Railway under selemene-engine-production.up.railway.app.
 *
 * Cloudflare Access protects both, but the browser still treats them as different
 * origins for CORS. Preflight OPTIONS requests are blocked by Access before the
 * backend can respond, so direct cross-origin API calls fail.
 *
 * This Worker runs on 144.tryambakam.space/api/* and forwards requests to the
 * Railway backend. Because the frontend calls its own origin, no CORS preflight
 * is triggered, and the Cloudflare Access cookie/identity reaches the backend.
 */

const API_BASE_URL = "https://selemene-engine-production.up.railway.app";

export interface Env {
  // No bindings required; API base is hardcoded above.
}

export default {
  async fetch(
    request: Request,
    _env: Env,
    ctx: ExecutionContext
  ): Promise<Response> {
    const url = new URL(request.url);

    // Keep the full path (/api/v1/admin/session) because the Railway backend
    // exposes the API under the same /api prefix.
    const targetUrl = new URL(url.pathname + url.search, API_BASE_URL);

    // Clone headers. Remove CF-specific headers that might confuse the backend.
    const headers = new Headers(request.headers);
    headers.delete("host");

    // Cloudflare Access stores the JWT in a CF_Authorization cookie, but the
    // Railway middleware expects it as a request header. Extract it from the
    // cookie and add it as a header if not already present.
    const existingCfAuth =
      headers.get("cf-authorization") ??
      headers.get("CF_Authorization") ??
      headers.get("CF-Access-Jwt-Assertion");
    const cookieHeader = headers.get("cookie") ?? "";
    const cookieNames = cookieHeader
      .split(";")
      .map((c) => c.trim().split("=")[0])
      .filter(Boolean);
    console.log("[admin-api-proxy]", request.method, url.pathname, "existingCfAuth=", existingCfAuth ? "present" : "missing", "cookies=", cookieNames.join(","));
    if (!existingCfAuth) {
      const cfAuthCookie = cookieHeader
        .split(";")
        .map((c) => c.trim())
        .find((c) => c.startsWith("CF_Authorization="));
      if (cfAuthCookie) {
        const token = cfAuthCookie.slice("CF_Authorization=".length);
        headers.set("CF_Authorization", token);
        console.log("[admin-api-proxy] set CF_Authorization header from cookie");
      } else {
        console.log("[admin-api-proxy] no CF_Authorization cookie found");
      }
    }

    const init: RequestInit<RequestInitCfProperties> = {
      method: request.method,
      headers,
      body: request.body,
      redirect: "manual",
      cf: {
        // Forward the same cache semantics; admin API should be no-store.
        cacheTtl: 0,
      },
    };

    const response = await fetch(targetUrl.toString(), init);

    // CORS: allow the admin dashboard origin. With the proxy, this is technically
    // same-origin, but set permissive headers defensively.
    const corsHeaders = new Headers(response.headers);
    corsHeaders.set("Access-Control-Allow-Origin", "https://144.tryambakam.space");
    corsHeaders.set("Access-Control-Allow-Credentials", "true");

    return new Response(response.body, {
      status: response.status,
      statusText: response.statusText,
      headers: corsHeaders,
    });
  },
};
