import { describe, expect, it } from "vitest";
import { sanitizeText, sanitizeValue } from "../src/sanitize.js";
describe("secret and identity redaction", () => {
  it("removes private nested fields and serialized tool JSON while preserving profile/authority", () => {
    const key = "nk_test_only_quote\"and/slash";
    const output = sanitizeValue({ _auth: { user_id: "private-id" }, profile: "2/4", authority: "Emotional",
      account_id: "private-account", email: "private-email", cookie: "private-cookie",
      content: [{ type: "text", text: JSON.stringify({ api_key: key, token: "private-token", profile: "2/4", note: key }) }],
      _meta: { "mcp/www_authenticate": ["private-challenge"] } }, key);
    const text = JSON.stringify(output);
    for (const forbidden of [key, "private-id", "private-account", "private-email", "private-cookie", "private-token", "private-challenge"]) expect(text).not.toContain(forbidden);
    expect(text).toContain("Emotional"); expect(text).toContain("2/4");
  });
  it("redacts exact/escaped/partially encoded keys and credential-bearing URLs", () => {
    const key = "nk_test_only_secret";
    for (const input of [key, JSON.stringify(key), "%6e%6b_test_only_secret",
      "https://user:password@example.test/path", "https://example.test/?api_key=hidden",
      "https://example.test/?token=hidden", "https://example.test/?authorization=hidden"]) {
      const safe = sanitizeText(input, key);
      expect(safe).not.toContain(key); expect(safe).not.toContain("%6e%6b"); expect(safe).not.toContain("password"); expect(safe).not.toContain("hidden");
    }
    expect(sanitizeText("https://example.test/documentation", key)).toBe("https://example.test/documentation");
  });
});
