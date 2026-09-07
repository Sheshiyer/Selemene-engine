import { describe, expect, it, vi } from "vitest";
import { readFileSync } from "node:fs";
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
import { decodeEngineCapabilityList, decodeWorkflowOutcome } from "@selemene/engine-sdk";

const capabilityFixture = JSON.parse(
  readFileSync(new URL("../../../contracts/v1/fixtures/engine-capability-list.json", import.meta.url), "utf8"),
);

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
  it("decodes the canonical capability envelope and sends bearer auth", async () => {
    const fetchMock = vi.fn(async (_url: string, init?: RequestInit) => {
      expect(new Headers(init?.headers).get("Authorization")).toBe("Bearer token");
      expect(new Headers(init?.headers).get("X-API-Key")).toBeNull();
      return new Response(JSON.stringify(capabilityFixture), {
        status: 200,
        headers: { "content-type": "application/json" },
      });
    });
    vi.stubGlobal("fetch", fetchMock);
    const client = new NoesisClient("https://example.com", { authToken: "token" });
    const result = await client.listCapabilities();
    expect(result.count).toBe(19);
    expect(result.public_mirror_count).toBe(17);
    expect(decodeEngineCapabilityList(result).capabilities).toHaveLength(19);
  });

  it("rejects ambiguous auth before making a request", () => {
    const fetchMock = vi.fn();
    vi.stubGlobal("fetch", fetchMock);
    expect(() => new NoesisClient("https://example.com", { authToken: "token", apiKey: "key" })).toThrow(
      "either apiKey or authToken",
    );
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it("uses PATCH for profile updates", async () => {
    const fetchMock = vi.fn(async (_url: string, init?: RequestInit) => {
      expect(init?.method).toBe("PATCH");
      return new Response(JSON.stringify({ id: "u1" }), { status: 200 });
    });
    vi.stubGlobal("fetch", fetchMock);
    await new NoesisClient("https://example.com", { authToken: "token" }).updateMe({ full_name: "A" });
  });

  it("bounds malformed and sensitive error bodies", async () => {
    const fetchMock = vi.fn(async () => new Response("token=do-not-leak", { status: 502 }));
    vi.stubGlobal("fetch", fetchMock);
    const client = new NoesisClient("https://example.com");
    await expect(client.health()).rejects.toMatchObject({ status: 502, details: { error_code: "UPSTREAM_INVALID_RESPONSE" } });
    await expect(client.health()).rejects.not.toHaveProperty("details.raw");
  });

  it("strictly decodes canonical workflow outcomes", () => {
    const outcome = {
      contract_version: "v1",
      workflow_id: "full-spectrum",
      requested_engine_ids: ["numerology"],
      engine_outputs: {},
      engine_failures: [{ engine_id: "numerology", error_code: "OPERATION_UNSUPPORTED", message: "unsupported" }],
      execution_status: "failed",
      synthesis_status: "unsupported",
    };
    expect(decodeWorkflowOutcome(outcome).synthesis_status).toBe("unsupported");
    expect(() => decodeWorkflowOutcome({ ...outcome, extra: true })).toThrow();
  });

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
