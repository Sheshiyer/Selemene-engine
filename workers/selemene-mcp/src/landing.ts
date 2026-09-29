// Minimal purpose landing at GET /. No fabricated publisher, legal, or
// marketing claims.

export function renderLanding(publicOrigin: string, resource: string): Response {
  const escapedResource = escapeHtml(resource);
  const escapedOrigin = escapeHtml(publicOrigin);
  const body = `<!doctype html><meta charset="utf-8"/>
<title>Selemene MCP connector</title>
<style>body{font-family:system-ui;max-width:36rem;margin:2rem auto;padding:0 1rem}</style>
<main>
<h1>Selemene MCP connector</h1>
<p>This Cloudflare Worker exposes a scoped, stateless Streamable HTTP MCP endpoint for
the Selemene Engine. It is intended for programmatic use by an MCP-capable client.</p>
<ul>
<li>Resource identifier: <code>${escapedResource}</code></li>
<li>OAuth metadata: <code>${escapedOrigin}/.well-known/oauth-protected-resource</code></li>
<li>Authorization server metadata: <code>${escapedOrigin}/.well-known/oauth-authorization-server</code></li>
<li>Health check: <code>${escapedOrigin}/health</code></li>
</ul>
<p>Non-media allowlist and per-user OAuth apply. See project documentation.</p>
</main>`;
  return new Response(body, {
    status: 200,
    headers: {
      "content-type": "text/html; charset=utf-8",
      "cache-control": "no-store",
    },
  });
}

export function renderHealth(): Response {
  return new Response(
    JSON.stringify({
      status: "ok",
      service: "selemene-mcp",
      version: "0.1.0",
    }),
    {
      status: 200,
      headers: {
        "content-type": "application/json",
        "cache-control": "no-store",
      },
    },
  );
}

function escapeHtml(input: string): string {
  return input
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}
