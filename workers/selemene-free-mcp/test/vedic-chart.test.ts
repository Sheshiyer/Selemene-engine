import { afterEach, describe, expect, it, vi } from "vitest";
import { VEDIC_CHART_TOOL } from "../src/vedic-chart.js";
import type { ToolContext } from "../../selemene-mcp/src/tools/index.js";

const origin = "https://selemene-engine-production.up.railway.app";
const ctx = {
  env: { selemeneApiOrigin: origin },
  props: { apiKey: "fixture-key", consciousnessLevel: 5 },
  scope: { read: true, calculate: true },
} as unknown as ToolContext;
const input = {
  birth_data: { date: "1990-01-15", time: "12:00", latitude: 0, longitude: 0, timezone: "UTC" },
  current_time: "2026-10-07T00:00:00Z",
};

// Shape matches noesis-api's build_d1_chart/build_d9_chart exactly. Synthetic
// values here test the adapter contract, not the ephemeris mathematics.
function fixture() {
  const names = ["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn", "Rahu", "Ketu"];
  const native = { birth_date: "1990-01-15", birth_time: "12:00:00", latitude: 0, longitude: 0, timezone: 0, user_id: "private-owner" };
  const planets = names.map(name => ({ name, longitude: 1, sign: "aries", degree: 1, minutes: 0, house: 1, is_retrograde: false, is_combust: false, nakshatra: "Ashwini", pada: 1, speed: 1, latitude: 0, api_key: "private-owner" }));
  return {
    user_id: "private-owner",
    d1: {
      ascendant: { sign: "aries", degree: 1, nakshatra: "Ashwini", pada: 1, private: "private-owner" },
      ayanamsa: 23.7, house_system: "whole_sign",
      houses: Array.from({ length: 12 }, (_, i) => ({ number: i + 1, sign: "aries", cusp: i * 30, degree: 0, house_type: "dharma", is_kendra: false, is_panapara: false, is_apoklima: false })),
      moon: { sign: "aries", degree: 1, nakshatra: "Ashwini", pada: 1, rashi_lord: "Mars" },
      native, notes: ["Native computation via Swiss Ephemeris (Lahiri ayanamsa, whole-sign houses)"], planets,
      special_points: { lagna: 1, midheaven: null, part_of_fortune: 1 },
      owner_history: ["private-owner"],
    },
    d9: { d9_lagna: "aries", navamsa_positions: names.map(planet => ({ planet, sign: "aries", degree: 9, is_vargottama: true, private: "private-owner" })), source: native, vargottama: names, owner: "private-owner" },
  };
}
function mockSuccess() {
  const fn = vi.fn().mockResolvedValue(new Response(JSON.stringify(fixture()), { status: 200 }));
  vi.stubGlobal("fetch", fn);
  return fn;
}
afterEach(() => { vi.unstubAllGlobals(); vi.useRealTimers(); });

describe("selemene_vedic_chart", () => {
  it("exports strict tool metadata and translates only the fixed route/body", async () => {
    const fetch = mockSuccess();
    expect(VEDIC_CHART_TOOL.name).toBe("selemene_vedic_chart");
    expect(VEDIC_CHART_TOOL.requiredScope).toBe("calculate");
    expect(VEDIC_CHART_TOOL.securitySchemes).toEqual({ noauth: true });
    const result = await VEDIC_CHART_TOOL.handler(ctx, input);
    expect(result.isError).toBeUndefined();
    expect(fetch).toHaveBeenCalledOnce();
    const [url, init] = fetch.mock.calls[0];
    expect(url).toBe(origin + "/api/v1/charts/vedic");
    expect(init.redirect).toBe("manual");
    expect(init.signal).toBeInstanceOf(AbortSignal);
    expect(init.headers).toEqual({ "Content-Type": "application/json", Accept: "application/json", "X-API-Key": "fixture-key" });
    expect(JSON.parse(init.body)).toEqual({ ...input, precision: "Standard", options: {} });
    const payload = result.structuredContent as any;
    expect(payload.chart_kind).toBe("vedic-d1-d9");
    expect(payload.persistence_status).toBe("not_persisted_by_chart_handler");
    expect(payload.result.d1.planets).toHaveLength(9);
    expect(payload.result.d1.houses).toHaveLength(12);
    expect(payload.result.d9.navamsa_positions).toHaveLength(9);
    expect(JSON.stringify(payload)).not.toContain("private-owner");
    expect(JSON.stringify(payload)).not.toContain("is_combust");
    expect(JSON.stringify(payload)).not.toContain("fixture-key");
    expect(payload.limitations.unavailable_fields).toContain("combustion");
    expect(payload.calculation_method).toContain("approximate linear Lahiri");
    expect(payload.limitations.ayanamsa_precision).toContain("approximate linear Lahiri formula");
    expect(payload.result.d1.notes[0]).toBe("Native computation via Swiss Ephemeris (Lahiri ayanamsa, whole-sign houses)");
  });

  it.each([
    { birth_data: { ...input.birth_data, date: "1990-02-30" } },
    { birth_data: { ...input.birth_data, date: "0900-01-01" } },
    { birth_data: { ...input.birth_data, date: "2026-13-01" } },
    { birth_data: { ...input.birth_data, time: "24:00" } },
    { birth_data: { ...input.birth_data, time: "12:61" } },
    { birth_data: { ...input.birth_data, time: "" } },
    { birth_data: { ...input.birth_data, latitude: 91 } },
    { birth_data: { ...input.birth_data, longitude: Infinity } },
    { birth_data: { ...input.birth_data, timezone: "made/up" } },
    { birth_data: { ...input.birth_data, timezone: "+01:00" } },
    { birth_data: { ...input.birth_data, options: {} } },
    { ...input, media_url: "https://example.com" },
    { ...input, current_time: "2026-10-07T00:00:00" },
    { birth_data: { date: "1990-01-15", latitude: 0, longitude: 0, timezone: "UTC" } },
  ])("rejects invalid/missing/unknown input before fetch %#", async args => {
    const fetch = mockSuccess();
    const result = await VEDIC_CHART_TOOL.handler(ctx, args);
    expect(result.isError).toBe(true);
    expect(result.structuredContent?.kind).toBe("invalid_input");
    expect(fetch).not.toHaveBeenCalled();
  });

  it.each([
    ["2024-03-10", "02:30", "America/New_York", "gap"],
    ["2024-11-03", "01:30", "America/New_York", "fold"],
    ["2024-10-06", "02:15", "Australia/Lord_Howe", "gap"],
    ["2024-04-07", "01:45", "Australia/Lord_Howe", "fold"],
    ["2011-12-30", "12:00", "Pacific/Apia", "gap"],
  ])("rejects ambiguous/nonexistent birth wall time %s %s %s", async (date, time, timezone, reason) => {
    const fetch = mockSuccess();
    const result = await VEDIC_CHART_TOOL.handler(ctx, { birth_data: { ...input.birth_data, date, time, timezone } });
    expect(result.isError).toBe(true);
    expect(JSON.stringify(result)).toContain(reason);
    expect(fetch).not.toHaveBeenCalled();
  });

  it.each(["UTC", "Etc/UTC", "Asia/Kolkata", "America/New_York"])("accepts a real local time in %s with seconds precision", async timezone => {
    const fetch = mockSuccess();
    const result = await VEDIC_CHART_TOOL.handler(ctx, { birth_data: { ...input.birth_data, time: "12:00:31", timezone } });
    expect(result.isError).toBeUndefined();
    expect(JSON.parse(fetch.mock.calls[0][1].body).birth_data.time).toBe("12:00:31");
  });

  it("rejects unsupported origins, missing credentials, and missing calculate scope", async () => {
    const fetch = mockSuccess();
    for (const context of [
      { ...ctx, env: { ...ctx.env, selemeneApiOrigin: "https://attacker.invalid" } },
      { ...ctx, env: { ...ctx.env, selemeneApiOrigin: origin + "/extra" } },
      { ...ctx, props: { ...ctx.props, apiKey: "" } },
      { ...ctx, scope: { read: true, calculate: false } },
    ]) expect((await VEDIC_CHART_TOOL.handler(context as ToolContext, input)).isError).toBe(true);
    expect(fetch).not.toHaveBeenCalled();
  });

  it.each([401, 403, 404, 429, 500, 422])("sanitizes upstream status %i without body leakage", async status => {
    vi.stubGlobal("fetch", vi.fn().mockResolvedValue(new Response("private-owner fixture-key", { status })));
    const result = await VEDIC_CHART_TOOL.handler(ctx, input);
    expect(result.isError).toBe(true);
    expect(result.structuredContent?.status).toBe(status);
    expect(JSON.stringify(result)).not.toContain("private-owner");
    expect(JSON.stringify(result)).not.toContain("fixture-key");
  });

  it.each([301, 302, 303, 307, 308])("refuses redirect %i without following or retrying", async status => {
    const cancel = vi.fn();
    const body = new ReadableStream<Uint8Array>({ start() {}, cancel });
    const fetch = vi.fn().mockResolvedValue(new Response(body, { status, headers: { Location: "https://attacker.invalid/private-owner" } }));
    vi.stubGlobal("fetch", fetch);
    const result = await VEDIC_CHART_TOOL.handler(ctx, input);
    expect(result.isError).toBe(true);
    expect(result.structuredContent?.status).toBe(status);
    expect(fetch).toHaveBeenCalledOnce();
    expect(fetch.mock.calls[0][1].redirect).toBe("manual");
    expect(cancel).toHaveBeenCalledOnce();
    expect(JSON.stringify(result)).not.toContain("attacker.invalid");
    expect(JSON.stringify(result)).not.toContain("private-owner");
  });

  it.each(["not json", JSON.stringify({ d1: {}, d9: {} }), JSON.stringify({ ...fixture(), d9: null })])("fails closed on malformed success payload %#", async body => {
    vi.stubGlobal("fetch", vi.fn().mockResolvedValue(new Response(body)));
    expect((await VEDIC_CHART_TOOL.handler(ctx, input)).isError).toBe(true);
  });

  it("enforces a byte cap for multibyte upstream data", async () => {
    const cancel = vi.fn();
    const body = new ReadableStream<Uint8Array>({ start(controller) { controller.enqueue(new TextEncoder().encode("é".repeat(140_000))); }, cancel });
    vi.stubGlobal("fetch", vi.fn().mockResolvedValue(new Response(body)));
    const result = await VEDIC_CHART_TOOL.handler(ctx, input);
    expect(result.isError).toBe(true);
    expect(cancel).toHaveBeenCalled();
  });

  it("keeps the timeout active while a successful response body stalls", async () => {
    vi.useFakeTimers();
    const cancel = vi.fn();
    const body = new ReadableStream<Uint8Array>({ start() {}, cancel });
    vi.stubGlobal("fetch", vi.fn().mockResolvedValue(new Response(body)));
    const pending = VEDIC_CHART_TOOL.handler(ctx, input);
    await vi.advanceTimersByTimeAsync(45_000);
    const result = await pending;
    expect(result.isError).toBe(true);
    expect(result.structuredContent?.error).toContain("timed out");
    expect(cancel).toHaveBeenCalled();
  });

  it("sanitizes fetch exceptions and aborts the fetch deadline", async () => {
    vi.useFakeTimers();
    vi.stubGlobal("fetch", vi.fn((_url, init) => new Promise((_resolve, reject) => {
      init.signal.addEventListener("abort", () => reject(new Error("private-owner fixture-key")), { once: true });
    })));
    const pending = VEDIC_CHART_TOOL.handler(ctx, input);
    await vi.advanceTimersByTimeAsync(45_000);
    const result = await pending;
    expect(result.isError).toBe(true);
    expect(JSON.stringify(result)).not.toContain("fixture-key");
    expect(result.structuredContent?.error).toContain("timed out");
  });
});
