import { describe, expect, it, vi } from "vitest";
import {
  CONTRACT_VERSION,
  ENGINE_IDS,
  NoesisClient,
  SelemeneError,
  WORKFLOW_IDS,
  type EngineInput,
  type ContractEngineCapability,
  type ContractEngineResult,
} from "./index.js";

const input: EngineInput = {
  birth_data: {
    date: "1991-08-13",
    time: "13:31",
    latitude: 12.9716,
    longitude: 77.5946,
    timezone: "Asia/Kolkata",
  },
};

const canonicalCapability: ContractEngineCapability = {
  contract_version: "v1",
  engine_id: "numerology",
  display_name: "Numerology",
  availability: "available",
  runtime_kind: "native",
  dependencies: [],
};

const canonicalResult: ContractEngineResult = {
  contract_version: "v1",
  engine_id: "numerology",
  result: { life_path_number: 7 },
  consciousness_level: 2,
  witness_prompts: [{ prompt: "What is witnessed?" }],
  calculated_at: "2026-08-26T06:30:00Z",
  processing_time_ms: 12.5,
  provenance: {
    runtime_kind: "native",
    implementation_version: "3.3.1",
    cached: false,
    fallback_used: false,
  },
};

const singularLegacyResult: ContractEngineResult = {
  contract_version: "v1",
  engine_id: "numerology",
  result: {},
  consciousness_level: 2,
  witness_prompt: "What is witnessed?",
  calculated_at: "2026-08-26T06:30:00Z",
  processing_time_ms: 1,
};

describe("contract authority v1", () => {
  it("retains public mirror IDs and canonical envelope fields", () => {
    expect(CONTRACT_VERSION).toBe("v1");
    expect(ENGINE_IDS).toHaveLength(17);
    expect(ENGINE_IDS).toContain(canonicalCapability.engine_id);
    expect(canonicalResult.contract_version).toBe(CONTRACT_VERSION);
    expect(canonicalResult.witness_prompts?.[0]?.prompt).toBe("What is witnessed?");
    expect(singularLegacyResult.provenance).toBeUndefined();
  });
});

describe("NoesisClient", () => {
  it("supports all 16 engine calculate calls", async () => {
    const fetchMock = vi.fn(async () =>
      new Response(JSON.stringify({ engine_id: "ok", result: {} }), { status: 200 }),
    );
    vi.stubGlobal("fetch", fetchMock);

    const client = new NoesisClient("https://example.com", { authToken: "token" });

    for (const engine of ENGINE_IDS) {
      const res = await client.calculate(engine, input);
      expect(res.engine_id).toBe("ok");
    }

    expect(fetchMock).toHaveBeenCalledTimes(ENGINE_IDS.length);
  });

  it("supports all 6 workflow execute calls", async () => {
    const fetchMock = vi.fn(async () =>
      new Response(JSON.stringify({ workflow_id: "ok", engine_outputs: [] }), { status: 200 }),
    );
    vi.stubGlobal("fetch", fetchMock);

    const client = new NoesisClient("https://example.com", { authToken: "token" });

    for (const workflow of WORKFLOW_IDS) {
      const res = await client.workflow(workflow, input);
      expect(res.workflow_id).toBe("ok");
    }

    expect(fetchMock).toHaveBeenCalledTimes(6);
  });

  it("retries on 5xx and then succeeds", async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ error: "temporary" }), { status: 503 }),
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ engine_id: "numerology", result: {} }), {
          status: 200,
          headers: {
            "x-ratelimit-limit": "200",
            "x-ratelimit-remaining": "199",
            "x-ratelimit-reset": "1700000000",
          },
        }),
      );

    vi.stubGlobal("fetch", fetchMock);

    const client = new NoesisClient("https://example.com", {
      maxRetries: 2,
      backoffMs: 1,
    });

    const res = await client.calculate("numerology", input);
    expect(res.engine_id).toBe("numerology");
    expect(fetchMock).toHaveBeenCalledTimes(2);
    expect(client.rateLimitInfo.remaining).toBe(199);
  });

  it("throws SelemeneError after retries exhausted", async () => {
    const fetchMock = vi.fn(async () =>
      new Response(JSON.stringify({ error: "down" }), { status: 500 }),
    );
    vi.stubGlobal("fetch", fetchMock);

    const client = new NoesisClient("https://example.com", {
      maxRetries: 1,
      backoffMs: 1,
    });

    await expect(client.health()).rejects.toBeInstanceOf(SelemeneError);
    expect(fetchMock).toHaveBeenCalledTimes(2);
  });

  it("supports AbortController cancellation", async () => {
    const fetchMock = vi.fn(async (_url: string, init?: RequestInit) => {
      await new Promise((resolve) => setTimeout(resolve, 30));
      if (init?.signal?.aborted) {
        throw new DOMException("Aborted", "AbortError");
      }
      return new Response(JSON.stringify({ status: "ok" }), { status: 200 });
    });

    vi.stubGlobal("fetch", fetchMock);

    const client = new NoesisClient("https://example.com");
    const controller = new AbortController();
    controller.abort();

    await expect(
      client.health({ signal: controller.signal }),
    ).rejects.toBeInstanceOf(DOMException);
  });
});
