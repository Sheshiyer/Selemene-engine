// Browser consent surface at GET/POST /authorize.
//
// Uses the library's beginConsent/approveConsent for browser-bound consent
// transactions. No ad-hoc nonce/KV — the library manages state internally.
//
// Security:
//   - beginConsent returns browser-binding cookies + framing protection headers
//   - Resource validation: wrong/missing resource on auth request is rejected
//   - S256 PKCE required: code_challenge_method must be S256
//   - Origin exact match on POST
//   - Secure HttpOnly SameSite __Host- cookie for API key binding
//   - Unknown scopes rejected
//   - Generic errors; never raw exceptions
//   - Consent form: same-origin referrer policy preserves navigation POST Origin.
//     Cross-origin referrers remain suppressed; errors/redirects use no-referrer.
//   - Customer API key only in encrypted grant props
//   - Backend users/me id validated as non-empty string
//   - Clear cookies on deny

import type { AuthRequest, OAuthHelpers } from "@cloudflare/workers-oauth-provider";

import type { Env } from "./env.js";
import { readEnv } from "./env.js";
import { SCOPES, ALL_SCOPES, type SelemeneAuthProps } from "./props.js";
import { callSelemene } from "./upstream.js";

const MAX_FORM_BYTES = 8 * 1024; // 8 KiB form body bound

export async function handleAuthorize(
  request: Request,
  env: Env,
  helpers: OAuthHelpers,
): Promise<Response> {
  const validated = readEnv(env);
  if (request.method === "GET") {
    return renderConsent(request, env, helpers, validated);
  }
  if (request.method === "POST") {
    return handleConsentPost(request, env, helpers, validated);
  }
  return new Response("method not allowed", {
    status: 405,
    headers: { allow: "GET, POST" },
  });
}

// ---- GET /authorize --------------------------------------------------------

async function renderConsent(
  request: Request,
  env: Env,
  helpers: OAuthHelpers,
  validated: ReturnType<typeof readEnv>,
): Promise<Response> {
  let parsed: AuthRequest;
  try {
    parsed = await helpers.parseAuthRequest(request);
  } catch {
    return renderLocalError(400, "Invalid authorization request.");
  }

  // Validate resource: must match our canonical MCP resource.
  const resources = new URL(request.url).searchParams.getAll("resource");
  if (resources.length !== 1 || resources[0] !== validated.mcpResource) {
    return renderLocalError(
      400,
      "Resource mismatch. This authorization server serves a specific resource.",
    );
  }

  // Require S256 PKCE. Plain PKCE is rejected.
  if (!parsed.codeChallenge) {
    return renderLocalError(
      400,
      "PKCE code_challenge is required. Use S256 method.",
    );
  }
  if (parsed.codeChallengeMethod !== "S256") {
    return renderLocalError(
      400,
      "Only S256 code_challenge_method is supported.",
    );
  }

  // Reject unknown scopes.
  const knownScopes = new Set(ALL_SCOPES);
  for (const s of parsed.scope) {
    if (!knownScopes.has(s)) {
      return renderLocalError(400, "Unsupported scope.");
    }
  }

  // Use library's beginConsent for browser-bound transaction.
  let handle: string;
  let libHeaders: Headers;
  try {
    const tx = await helpers.beginConsent(parsed);
    handle = tx.handle;
    libHeaders = tx.headers;
  } catch {
    return renderLocalError(500, "Could not initiate consent transaction.");
  }

  // Ask the library for client display info.
  let clientName: string;
  let redirectHost: string;
  try {
    const consent = await helpers.describeConsent(parsed);
    clientName = consent.clientDomain ? `${consent.clientName} (${consent.clientDomain})` : `${consent.clientName} (unverified client name)`;
    redirectHost = consent.redirectHost;
  } catch {
    clientName = parsed.clientId;
    redirectHost = "(unknown)";
  }

  const body = renderHtml({
    clientName,
    redirectHost,
    scopes: parsed.scope,
    handle,
    formAction: `${validated.publicOrigin}/authorize`,
  });

  // Merge library headers (browser-binding cookies, framing protection) with
  // our response headers.
  const headers = new Headers(libHeaders);
  headers.set("content-type", "text/html; charset=utf-8");
  headers.set("cache-control", "no-store");
  // Fetch treats no-referrer navigation POSTs as Origin:null, even same-origin.
  // Preserve our exact Origin CSRF check while suppressing cross-origin referrers.
  headers.set("referrer-policy", "same-origin");
  headers.set(
    "content-security-policy",
    "default-src 'none'; style-src 'unsafe-inline'; form-action 'self'; frame-ancestors 'none'",
  );

  return new Response(body, { status: 200, headers });
}

// ---- POST /authorize -------------------------------------------------------

async function handleConsentPost(
  request: Request,
  _env: Env,
  helpers: OAuthHelpers,
  validated: ReturnType<typeof readEnv>,
): Promise<Response> {
  // Bound form bytes before parse.
  const contentLength = parseInt(request.headers.get("content-length") ?? "0", 10);
  if (contentLength > MAX_FORM_BYTES) {
    return renderLocalError(413, "Form submission too large.");
  }

  const contentType = (request.headers.get("content-type") ?? "").toLowerCase();
  if (!contentType.includes("application/x-www-form-urlencoded")) {
    return renderLocalError(415, "Content-Type must be application/x-www-form-urlencoded.");
  }

  // Origin exact match.
  const origin = request.headers.get("origin");
  if (origin !== validated.publicOrigin) {
    return renderLocalError(403, "Origin mismatch.");
  }

  let formData: FormData;
  try {
    const reader = request.body?.getReader();
    let bytes = 0; const chunks: Uint8Array[] = [];
    if (reader) {
      while (true) {
        const { value, done } = await reader.read(); if (done) break;
        bytes += value.byteLength;
        if (bytes > MAX_FORM_BYTES) { void reader.cancel().catch(() => {}); return renderLocalError(413, "Form submission too large."); }
        chunks.push(value);
      }
    }
    const raw = new Uint8Array(bytes); let offset = 0;
    for (const chunk of chunks) { raw.set(chunk, offset); offset += chunk.byteLength; }
    const params = new URLSearchParams(new TextDecoder().decode(raw));
    formData = new FormData(); for (const [key, value] of params) formData.append(key, value);
    for (const key of ["handle", "action", "selemene_api_key"]) if (params.getAll(key).length > 1) return renderLocalError(400, "Duplicate form field.");
  } catch {
    return renderLocalError(400, "Could not parse form data.");
  }

  const handle = String(formData.get("handle") ?? "");
  const action = String(formData.get("action") ?? "");
  const apiKey = String(formData.get("selemene_api_key") ?? "").trim();

  if (!handle || handle.length < 16 || handle.length > 512) {
    return renderLocalError(400, "Missing or malformed consent handle.");
  }

  // Only approve and deny are valid actions.
  if (action !== "approve" && action !== "deny") {
    return renderLocalError(400, "Unknown action.");
  }

  if (action === "deny") {
    try {
      const denied = await helpers.denyConsent(request, handle);
      const headers = new Headers(denied.headers);
      headers.set("location", denied.redirectTo);
      headers.set("referrer-policy", "no-referrer");
      return new Response(null, { status: 302, headers });
    } catch {
      return renderLocalError(400, "Consent transaction expired or already used.");
    }
  }

  // action === "approve"
  if (!apiKey.startsWith("nk_") || apiKey.length < 16 || apiKey.length > 256) {
    return renderLocalError(400, "Selemene API key must start with nk_ and be at least 16 characters.");
  }

  // Verify the key against the Selemene backend. Validate the returned user_id.
  let meData: Record<string, unknown> | null;
  try {
    const meOutcome = await callSelemene(
      { selemeneApiOrigin: validated.selemeneApiOrigin } as ReturnType<typeof readEnv>,
      {
        method: "GET",
        path: "/api/v1/users/me",
        apiKey,
      },
    );
    if (meOutcome.kind !== "ok") {
      return renderLocalError(401, "Could not verify Selemene API key.");
    }
    meData = meOutcome.data as Record<string, unknown> | null;
  } catch {
    return renderLocalError(502, "Could not reach Selemene backend.");
  }

  const userId = typeof meData?.id === "string" ? meData.id : null;
  if (!userId || !/^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(userId)) {
    return renderLocalError(401, "Backend did not return a valid user ID.");
  }
  // Validate phase if present.
  if (meData?.consciousness_level !== undefined) {
    const phase = meData.consciousness_level;
    if (typeof phase !== "number" || phase < 0 || phase > 5 || !Number.isInteger(phase)) {
      return renderLocalError(401, "Backend returned invalid phase.");
    }
  }

  const props: SelemeneAuthProps = {
    userId,
    tier: typeof meData?.tier === "string" ? meData.tier : "free",
    consciousnessLevel:
      typeof meData?.consciousness_level === "number"
        ? meData.consciousness_level
        : 0,
    apiKey, // stored only in encrypted grant props
    linkedAt: Math.floor(Date.now() / 1000),
  };

  try {
    const approved = await helpers.approveConsent(request, handle);
    const grantedScopes = approved.request.scope;
    if (!grantedScopes.includes(SCOPES.read) || grantedScopes.some((scope) => !ALL_SCOPES.includes(scope))) return renderLocalError(400, "Unsupported consent scopes.");

    const headers = new Headers(approved.headers);
    // Merge with completeAuthorization to get the redirect.
    const { redirectTo } = await helpers.completeAuthorization({
      request: approved.request,
      userId,
      metadata: { tier: props.tier, linkedAt: props.linkedAt },
      scope: grantedScopes,
      props,
    });
    headers.set("location", redirectTo);
    headers.set("referrer-policy", "no-referrer");

    return new Response(null, { status: 302, headers });
  } catch {
    return renderLocalError(
      400,
      "Consent transaction expired, already used, or browser binding mismatch.",
    );
  }
}

// ---- HTML rendering --------------------------------------------------------

function renderHtml(opts: {
  clientName: string;
  redirectHost: string;
  scopes: string[];
  handle: string;
  formAction: string;
}): string {
  const clientName = escapeHtml(opts.clientName);
  const redirectHost = escapeHtml(opts.redirectHost);
  const scopesLi = opts.scopes.map((s) => `<li>${escapeHtml(s)}</li>`).join("");
  return `<!doctype html>
<html lang="en"><meta charset="utf-8"/>
<title>Selemene MCP consent</title>
<style>
body{font-family:system-ui,sans-serif;max-width:36rem;margin:2rem auto;padding:0 1rem;color:#111;background:#fafafa}
main{background:white;border:1px solid #ddd;border-radius:8px;padding:1.5rem}
input[type=password]{width:100%;padding:0.5rem;font-family:monospace}
button{padding:0.5rem 1rem;border:0;border-radius:4px;cursor:pointer}
button.primary{background:#333;color:white}
button.secondary{background:#eee}
.warn{background:#fff8e1;border-left:4px solid #f6c000;padding:0.75rem;margin:1rem 0;font-size:0.95rem}
</style>
<main>
<h1>Connect Selemene to <em>${clientName}</em></h1>
<p>The client at <code>${redirectHost}</code> is requesting these scopes:</p>
<ul>${scopesLi}</ul>
<div class="warn">
<strong>Persistence disclosure:</strong> Selemene calculations attempt to save reading inputs and results to your account,
log usage, award experience points, and may auto-populate your profile from birth data on first use.
</div>
<form method="POST" action="${escapeHtml(opts.formAction)}" autocomplete="off">
  <input type="hidden" name="handle" value="${escapeHtml(opts.handle)}"/>
  <label for="k"><strong>Paste your Selemene API key (nk_...)</strong></label>
  <p style="font-size:0.9rem;color:#555">Never paste your key into chat. This form sends it once directly to this connector, which stores it only inside your encrypted OAuth grant.</p>
  <input id="k" type="password" name="selemene_api_key" required pattern="nk_.{8,}" minlength="12" maxlength="256"/>
  <p style="margin-top:1rem">
    <button class="primary" type="submit" name="action" value="approve">Approve</button>
    <button class="secondary" type="submit" name="action" value="deny" formnovalidate>Deny</button>
  </p>
</form>
</main>
</html>`;
}

// ---- Error pages -----------------------------------------------------------

function renderLocalError(status: number, message: string): Response {
  const body = `<!doctype html><meta charset="utf-8"><title>Consent error</title>
<main style="font-family:system-ui;padding:2rem">
<h1>Consent error</h1>
<p>${escapeHtml(message)}</p>
</main>`;
  return new Response(body, {
    status,
    headers: {
      "content-type": "text/html; charset=utf-8",
      "cache-control": "no-store",
      "referrer-policy": "no-referrer",
      "content-security-policy": "frame-ancestors 'none'",
    },
  });
}

// ---- Utilities -------------------------------------------------------------

function escapeHtml(input: string): string {
  return input
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}
