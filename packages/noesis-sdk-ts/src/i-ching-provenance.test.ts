/**
 * GH #1341: i-ching provenance and cast/derived origins must survive the TS SDK.
 *
 * Fixtures are the real noesis-api /calculate and workflow responses, pinned by
 * `i_ching_api_envelope_and_persisted_reading_keep_derivation` and
 * `i_ching_workflow_response_and_persisted_reading_keep_derivation` in
 * crates/noesis-api from the ts-engines seed-42 sidecar fixture. Synthetic only.
 */
import { readFileSync } from "node:fs";
import { afterEach, describe, expect, it, vi } from "vitest";
import { NoesisClient, type EngineOutput, type WorkflowResult } from "./index.js";

function fixture<T>(name: string): T {
  return JSON.parse(
    readFileSync(
      new URL(`../../../ts-engines/tests/fixtures/i-ching/${name}`, import.meta.url),
      "utf8",
    ),
  ) as T;
}

const envelope = fixture<EngineOutput>("api-envelope-seed-42.json");
const workflowResponse = fixture<WorkflowResult>("api-workflow-seed-42.json");

const provenance = {
  runtime_kind: "typescript",
  implementation_version: "1.1.0",
  cached: false,
  fallback_used: false,
  confidence: 1,
};

function serve(body: unknown) {
  const fetchMock = vi.fn(async () => new Response(JSON.stringify(body), { status: 200 }));
  vi.stubGlobal("fetch", fetchMock);
  return { fetchMock, client: new NoesisClient("https://example.com", { authToken: "token" }) };
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("NoesisClient i-ching provenance (GH #1341)", () => {
  it("calculate returns the contract block and result.derivation unchanged", async () => {
    const { client } = serve(envelope);

    const output = await client.calculate("i-ching", { consciousness_level: 0, seed: 42 });

    expect(output).toEqual(envelope);
    expect(output.provenance).toEqual(provenance);
    const derivation = output.result.derivation as Record<string, Record<string, unknown>>;
    expect(derivation.line_values.origin).toBe("cast");
    expect(derivation.changing_lines.origin).toBe("derived");
    expect(derivation.relating_hexagram.origin).toBe("derived");
    expect(derivation.changing_line_texts.origin).toBe("unavailable");
    expect(output.result).not.toHaveProperty("__contract_provenance");
  });

  it("workflow keeps per-engine provenance on engine_results only", async () => {
    const { client, fetchMock } = serve(workflowResponse);

    const result = await client.workflow("decision-support", {});

    expect(fetchMock).toHaveBeenCalledWith(
      "https://example.com/api/v1/workflows/decision-support/execute",
      expect.anything(),
    );
    expect(result).toEqual(workflowResponse);
    expect(result.engine_results?.["i-ching"].provenance).toEqual(provenance);
    // engine_outputs is the persisted shape: same result, no contract block.
    expect(result.engine_outputs?.["i-ching"].provenance).toBeUndefined();
    expect(result.engine_outputs?.["i-ching"].result).toEqual(
      result.engine_results?.["i-ching"].result,
    );
    expect(
      (result.engine_outputs?.["i-ching"].result.derivation as Record<string, unknown>).line_values,
    ).toMatchObject({ origin: "cast" });
  });
});
