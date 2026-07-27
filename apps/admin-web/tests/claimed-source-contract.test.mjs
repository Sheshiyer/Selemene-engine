import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import test from "node:test";

const read = (path) => readFileSync(resolve(process.cwd(), path), "utf8");

test("renders claimed sources as escaped React text with a legacy fallback", () => {
  const page = read("app/(protected)/readings/page.tsx");

  assert.ok(
    page.includes('item.claimed_source_client ?? "Legacy / unknown"'),
    "table must label readings that predate client attribution"
  );
  assert.ok(
    page.includes('selected.claimed_source_client ?? "Legacy / unknown"'),
    "detail view must label readings that predate client attribution"
  );
  assert.doesNotMatch(
    page,
    /dangerouslySetInnerHTML/,
    "claimed source values must remain React text nodes"
  );
});

test("keeps the claimed-source filter on the admin readings request", () => {
  const page = read("app/(protected)/readings/page.tsx");
  const api = read("src/lib/api.ts");

  assert.match(page, /claimed_source_client:\s*currentClaimedSourceClient/);
  assert.match(api, /claimed_source_client\?:\s*string/);
});
