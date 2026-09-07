import test from "node:test";
import assert from "node:assert/strict";
import { decodeEngineCapabilityList } from "@selemene/engine-sdk";
const PUBLIC_MIRROR_IDS = [
  "biofield", "biorhythm", "enneagram", "face-reading", "gene-keys", "human-design",
  "i-ching", "nadabrahman", "numerology", "panchanga", "raaga", "sacred-geometry",
  "sigil-forge", "tarot", "transits", "vedic-clock", "vimshottari"
];
const fixture = JSON.parse(await import("node:fs/promises").then((fs) => fs.readFile(
  new URL("../../../../contracts/v1/fixtures/engine-capability-list.json", import.meta.url),
  "utf8"
)));

test("canonical capability fixture preserves 19 rows and 17 mirrors", () => {
  const decoded = decodeEngineCapabilityList(fixture);
  assert.equal(decoded.capabilities.length, 19);
  assert.equal(decoded.count, 19);
  assert.equal(decoded.public_mirror_count, 17);
  assert.deepEqual([...PUBLIC_MIRROR_IDS].sort(), decoded.capabilities
    .map((row) => row.engine_id)
    .filter((id) => id !== "financial-biosensor" && id !== "biofield-capture")
    .sort());
});

test("shared decoder rejects unknown fields, invalid state and malformed counts", () => {
  for (const mutation of [
    { ...fixture, extra: true },
    { ...fixture, contract_version: "v2" },
    { ...fixture, count: 18 },
    { ...fixture, capabilities: fixture.capabilities.map((row, index) => index === 0 ? { ...row, availability: "online" } : row) }
  ]) assert.throws(() => decodeEngineCapabilityList(mutation));
});
