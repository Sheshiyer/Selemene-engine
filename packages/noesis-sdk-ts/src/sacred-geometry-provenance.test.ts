/**
 * GH #1401: the sacred-geometry API envelope must survive the TS SDK.
 *
 * The fixture is the real noesis-api /calculate envelope, built by
 * `sacred_geometry_api_envelope_matches_committed_fixture` in
 * crates/noesis-api from the ts-engines sidecar fixture. Synthetic input only.
 */
import { readFileSync } from "node:fs";
import { afterEach, describe, expect, it, vi } from "vitest";
import { NoesisClient, type EngineOutput } from "./index.js";

const envelope = JSON.parse(
  readFileSync(
    new URL(
      "../../../ts-engines/tests/fixtures/sacred-geometry/api-envelope-seed-1401.json",
      import.meta.url,
    ),
    "utf8",
  ),
) as EngineOutput;

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("NoesisClient sacred-geometry provenance (GH #1401)", () => {
  it("returns the contract provenance block and per-part provenance_detail unchanged", async () => {
    const fetchMock = vi.fn(async () => new Response(JSON.stringify(envelope), { status: 200 }));
    vi.stubGlobal("fetch", fetchMock);
    const client = new NoesisClient("https://example.com", { authToken: "token" });

    const output = await client.calculate("sacred-geometry", {
      consciousness_level: 0,
      parameters: { svg_template: "<svg><script>x()</script></svg>" },
      seed: 1401,
    });

    expect(fetchMock).toHaveBeenCalledWith(
      "https://example.com/api/v1/engines/sacred-geometry/calculate",
      expect.anything(),
    );
    expect(output).toEqual(envelope);
    expect(output.provenance).toEqual({
      runtime_kind: "typescript",
      implementation_version: "1.0.0",
      cached: false,
      fallback_used: false,
      confidence: 1,
    });

    const detail = output.result.provenance_detail as Record<string, Record<string, unknown>>;
    expect(detail.form.kind).toBe("symbolic_lookup");
    expect(detail.form_selection.kind).toBe("seeded_random");
    expect(detail.calculated_geometry.status).toBe("not_implemented");
    expect(detail.rendered_artifact.status).toBe("not_implemented");
    expect(detail.svg_preview.status).toBe("rejected");
    expect(output.result).not.toHaveProperty("__contract_provenance");
  });
});
