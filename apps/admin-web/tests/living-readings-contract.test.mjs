import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import test from "node:test";

const read = (path) => readFileSync(resolve(process.cwd(), path), "utf8");

test("links the protected navigation item to the archive page", () => {
  const layout = read("app/(protected)/layout.tsx");
  const permissions = read("src/lib/permissions.ts");

  assert.match(layout, /href:\s*"\/living-readings"/);
  assert.match(layout, /permission:\s*"admin:analytics:read"/);
  assert.match(permissions, /startsWith\("\/living-readings"\)/);
});

test("calls the protected list and detail endpoints", () => {
  const api = read("src/lib/api.ts");

  assert.match(api, /\/api\/v1\/admin\/living-readings/);
  assert.match(api, /\/api\/v1\/admin\/living-readings\/\$\{readingId\}/);
});

test("renders rows and separates provenance layers in detail", () => {
  const page = read("app/(protected)/living-readings/page.tsx");

  for (const contract of [
    "Living Readings Archive",
    "item.owner_email",
    "subjectLabel(item)",
    "detail.source",
    "detail.import_run",
    "detail.artifacts",
    "detail.editorial_history",
  ]) {
    assert.ok(page.includes(contract), `missing ${contract}`);
  }
});
