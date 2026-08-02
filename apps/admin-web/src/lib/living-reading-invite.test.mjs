import assert from "node:assert/strict";
import test from "node:test";
import {
  buildUraniaConversationUrl,
  wrapVerifiedHtmlForSandbox
} from "./living-reading-invite.ts";

test("conversation URL carries only the canonical reading id", () => {
  const url = buildUraniaConversationUrl(
    "reading/id",
    "https://urania.example/base?token=invite-token#old"
  );

  assert.equal(url, "https://urania.example/base/#/chat?reading=reading%2Fid");
  assert.doesNotMatch(url, /invite-token|[?&]token=/);
});

test("unsafe Urania configuration falls back to the public host", () => {
  assert.equal(
    buildUraniaConversationUrl("reading-1", "javascript:alert(1)"),
    "https://urania.tryambakam.space/#/chat?reading=reading-1"
  );
});

test("sandbox wrapper injects a network-denying CSP before content", () => {
  const body = "<html><head><title>Reading</title></head><body>Body</body></html>";
  const wrapped = wrapVerifiedHtmlForSandbox(body);
  const cspIndex = wrapped.indexOf('http-equiv="Content-Security-Policy"');

  assert.ok(cspIndex > wrapped.indexOf("<head>"));
  assert.ok(cspIndex < wrapped.indexOf("<title>"));
  assert.match(wrapped, /default-src 'none'/);
  assert.match(wrapped, /connect-src 'none'/);
  assert.match(wrapped, /img-src data:/);
  assert.match(wrapped, /font-src data:/);
  assert.doesNotMatch(wrapped, /navigate-to/);
  assert.ok(wrapped.includes(body.slice(body.indexOf("<title>"))));
});
