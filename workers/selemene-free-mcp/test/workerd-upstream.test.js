// This is an actual workerd regression, rather than a Node fetch mock. It uses
// only an in-process synthetic upstream and a non-production credential.
import { beforeAll, afterAll, beforeEach, describe, expect, it } from "vitest";
import { build } from "esbuild";
import { Miniflare, convertV4MiniflareOptions } from "miniflare";
import { fileURLToPath } from "node:url";

const syntheticKey = "nk_synthetic_workerd_test_only";
let runtime;
const received = [];

beforeAll(async () => {
  const upstreamFile = fileURLToPath(new URL("../../selemene-mcp/src/upstream.ts", import.meta.url));
  const bundled = await build({
    stdin: {
      contents: `
        import { callSelemene } from ${JSON.stringify(upstreamFile)};
        export default { async fetch(request) {
          const path = new URL(request.url).pathname;
          if (path === '/unsupported-runtime-mode') {
            try {
              await fetch('https://synthetic.invalid/api/v1/engines', {redirect:'error'});
              return Response.json({ rejected: false });
            } catch (error) {
              return Response.json({ rejected: true, name: error.name, message: error.message });
            }
          }
          const outcome = await callSelemene(
            {selemeneApiOrigin:'https://synthetic.invalid'},
            {method:'GET', path: path === '/redirect' ? '/api/v1/users/me' : '/api/v1/engines', apiKey:${JSON.stringify(syntheticKey)}}
          );
          return Response.json(outcome);
        }};
      `,
      resolveDir: fileURLToPath(new URL("../", import.meta.url)),
      sourcefile: "synthetic-workerd-regression.js",
    },
    bundle: true, write: false, format: "esm", platform: "browser", target: "es2022",
  });
  runtime = new Miniflare(convertV4MiniflareOptions({
    modules: true, compatibilityDate: "2025-08-01", script: bundled.outputFiles[0].text,
    outboundService: async (request) => {
      received.push({ url: request.url, key: request.headers.get("X-API-Key") });
      if (new URL(request.url).pathname === "/api/v1/users/me") {
        return new Response(`upstream credential echo ${syntheticKey}`, {
          status: 307, headers: { Location: `https://untrusted.invalid/?api_key=${syntheticKey}` },
        });
      }
      return Response.json({ safe: true });
    },
  }));
}, 20_000);

beforeEach(() => { received.length = 0; });
afterAll(async () => { await runtime?.dispose(); });

describe("Cloudflare upstream fetch contract", () => {
  it("documents that error redirect mode fails before any outbound request", async () => {
    const result = await (await runtime.dispatchFetch("https://connector.invalid/unsupported-runtime-mode")).json();
    expect(result.rejected).toBe(true);
    expect(result.name).toBe("TypeError");
    expect(result.message).toMatch(/redirect/i);
    expect(received).toHaveLength(0);
  });

  it("executes the shared upstream client in workerd with manual redirects", async () => {
    const result = await (await runtime.dispatchFetch("https://connector.invalid/success")).json();
    expect(result).toEqual({ kind: "ok", status: 200, data: { safe: true } });
    expect(received).toEqual([{ url: "https://synthetic.invalid/api/v1/engines", key: syntheticKey }]);
  });

  it("refuses redirect responses without following or exposing credentials", async () => {
    const response = await runtime.dispatchFetch("https://connector.invalid/redirect");
    const result = await response.text();
    expect(JSON.parse(result)).toEqual({ kind: "unavailable", status: 307, sanitized: "upstream redirect refused" });
    expect(result).not.toContain(syntheticKey);
    expect(result).not.toContain("untrusted.invalid");
    expect(received).toEqual([{ url: "https://synthetic.invalid/api/v1/users/me", key: syntheticKey }]);
  });
});
