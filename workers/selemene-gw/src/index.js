// selemene-gw — Cloudflare Worker gateway for the Selemene API.
// Holds the Selemene nk_ API key in Workers KV (binding SELEMENE_SECRETS,
// key name `nk_current`) so no client ever sees it. Clients authenticate
// with an owner bearer token (KV key `owner_token`).
//
// Routes:
//   GET  /health      -> worker version + KV key presence booleans (never values)
//   ALL  /api/v1/*    -> authenticated proxy to the Selemene flagship
//   *    everything   -> 404
//
// Rate limiting: 60 req/min per client IP, in-memory sliding window per
// isolate. This is best-effort (per-isolate, not globally consistent) —
// chosen to stay dependency-light; upgrade to CF Rate Limiting rules or
// Durable Objects if a hard global limit is needed.
// Audit: JSON console.log of {method, path, engine, status} — never headers/keys.

const VERSION = "0.1.0";
const UPSTREAM = "https://selemene.tryambakam.space";
const RATE_LIMIT_PER_MIN = 60;
const hits = new Map(); // ip -> [timestamps]

function allowedOrigin(request) {
  const origin = request.headers.get("Origin");
  if (!origin) return null;
  if (origin === "https://selemene.tryambakam.space") return origin;
  if (origin === "tauri://localhost") return origin;
  if (/^https?:\/\/(localhost|127\.0\.0\.1)(:\d+)?$/.test(origin)) return origin;
  return null;
}

function corsHeaders(origin) {
  return {
    "Access-Control-Allow-Origin": origin,
    "Access-Control-Allow-Methods": "GET, POST, PUT, PATCH, DELETE, OPTIONS",
    "Access-Control-Allow-Headers": "Authorization, Content-Type",
    "Access-Control-Max-Age": "86400",
    Vary: "Origin",
  };
}

function isRateLimited(ip) {
  const now = Date.now();
  const cutoff = now - 60_000;
  let list = (hits.get(ip) || []).filter((t) => t > cutoff);
  if (list.length >= RATE_LIMIT_PER_MIN) {
    hits.set(ip, list);
    return true;
  }
  list.push(now);
  hits.set(ip, list);
  if (hits.size > 10_000) hits.clear(); // bound memory
  return false;
}

function json(body, status, origin) {
  return Response.json(body, {
    status,
    headers: origin ? corsHeaders(origin) : {},
  });
}

export default {
  async fetch(request, env) {
    const url = new URL(request.url);
    const origin = allowedOrigin(request);

    // CORS preflight
    if (request.method === "OPTIONS") {
      if (!origin) return new Response(null, { status: 403 });
      return new Response(null, { status: 204, headers: corsHeaders(origin) });
    }

    // Health: version + key-presence booleans only, never values
    if (url.pathname === "/health" && request.method === "GET") {
      const [nk, ot] = await Promise.all([
        env.SELEMENE_SECRETS.get("nk_current"),
        env.SELEMENE_SECRETS.get("owner_token"),
      ]);
      return json(
        {
          status: "ok",
          worker: "selemene-gw",
          version: VERSION,
          kv: { nk_current: nk !== null, owner_token: ot !== null },
        },
        200,
        origin
      );
    }

    // Authenticated proxy
    if (url.pathname.startsWith("/api/v1/")) {
      const ip = request.headers.get("CF-Connecting-IP") || "unknown";
      let status = 500;
      try {
        if (isRateLimited(ip)) {
          status = 429;
          return json({ error: "rate_limited", limit: "60/min" }, status, origin);
        }

        const auth = request.headers.get("Authorization") || "";
        const token = auth.startsWith("Bearer ") ? auth.slice(7) : "";
        const [ownerToken, nkKey] = await Promise.all([
          env.SELEMENE_SECRETS.get("owner_token"),
          env.SELEMENE_SECRETS.get("nk_current"),
        ]);

        if (!ownerToken || !token || token.length !== ownerToken.length || token !== ownerToken) {
          status = 401;
          return json({ error: "unauthorized" }, status, origin);
        }
        if (!nkKey) {
          status = 502;
          return json({ error: "upstream_key_missing" }, status, origin);
        }

        const headers = new Headers(request.headers);
        headers.set("X-API-Key", nkKey);
        headers.delete("Authorization");
        headers.delete("Host");
        headers.delete("Cookie");

        const resp = await fetch(UPSTREAM + url.pathname + url.search, {
          method: request.method,
          headers,
          body: ["GET", "HEAD"].includes(request.method) ? undefined : request.body,
          redirect: "manual",
        });
        status = resp.status;

        const outHeaders = new Headers(resp.headers);
        if (origin) {
          for (const [k, v] of Object.entries(corsHeaders(origin))) outHeaders.set(k, v);
        }
        return new Response(resp.body, { status: resp.status, headers: outHeaders });
      } finally {
        const m = url.pathname.match(/^\/api\/v1\/engines\/([^/]+)/);
        console.log(
          JSON.stringify({
            audit: true,
            ts: new Date().toISOString(),
            method: request.method,
            path: url.pathname,
            engine: m ? m[1] : null,
            status,
          })
        );
      }
    }

    return json({ error: "not_found" }, 404, origin);
  },
};
