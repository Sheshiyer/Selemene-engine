import { describe, expect, it, vi } from "vitest";
import { BiofieldClient, BiofieldClientError } from "./biofield-client.js";

describe("BiofieldClient", () => {
  it("creates sessions against the frozen route namespace", async () => {
    const fetchMock = vi.fn(async () =>
      new Response(
        JSON.stringify({
          id: "session-1",
          status: "active",
          started_at: "2026-04-05T12:00:00Z",
          closed_at: null,
        }),
        { status: 200 },
      ),
    );

    const client = new BiofieldClient("https://example.com", { fetchImpl: fetchMock as typeof fetch });
    const session = await client.createSession({ viewer_version: "0.1.0" });

    expect(session.id).toBe("session-1");
    expect(fetchMock).toHaveBeenCalledWith(
      "https://example.com/api/v1/biofield/sessions",
      expect.objectContaining({
        method: "POST",
      }),
    );
  });

  it("lists readings with query params", async () => {
    const fetchMock = vi.fn(async () =>
      new Response(JSON.stringify({ items: [], limit: 10, offset: 20 }), { status: 200 }),
    );

    const client = new BiofieldClient("https://example.com", { fetchImpl: fetchMock as typeof fetch });
    await client.listReadings({ limit: 10, offset: 20 });

    expect(fetchMock).toHaveBeenCalledWith(
      "https://example.com/api/v1/biofield/readings?limit=10&offset=20",
      expect.objectContaining({
        method: "GET",
      }),
    );
  });

  it("posts reprocess and baseline routes against the frozen namespace", async () => {
    const fetchMock = vi.fn(async () =>
      new Response(JSON.stringify({ reading_id: "reading-2" }), { status: 200 }),
    );

    const client = new BiofieldClient("https://example.com", { fetchImpl: fetchMock as typeof fetch });
    await client.reprocessReading("reading-1");
    await client.listBaselines();
    await client.createBaseline({ name: "Morning", reading_ids: ["reading-1"] });

    expect(fetchMock).toHaveBeenNthCalledWith(
      1,
      "https://example.com/api/v1/biofield/readings/reading-1/reprocess",
      expect.objectContaining({ method: "POST" }),
    );
    expect(fetchMock).toHaveBeenNthCalledWith(
      2,
      "https://example.com/api/v1/biofield/baselines",
      expect.objectContaining({ method: "GET" }),
    );
    expect(fetchMock).toHaveBeenNthCalledWith(
      3,
      "https://example.com/api/v1/biofield/baselines",
      expect.objectContaining({ method: "POST" }),
    );
  });

  it("raises a typed error on failed requests", async () => {
    const fetchMock = vi.fn(async () =>
      new Response(JSON.stringify({ error: "nope" }), { status: 503 }),
    );

    const client = new BiofieldClient("https://example.com", { fetchImpl: fetchMock as typeof fetch });

    await expect(client.getReading("reading-1")).rejects.toBeInstanceOf(BiofieldClientError);
  });

  it("surfaces backend message in typed errors", async () => {
    const fetchMock = vi.fn(async () =>
      new Response(
        JSON.stringify({
          message: "Biofield analysis service is unavailable",
          error_code: "BIOFIELD_ANALYSIS_UNAVAILABLE",
        }),
        { status: 503 },
      ),
    );

    const client = new BiofieldClient("https://example.com", { fetchImpl: fetchMock as typeof fetch });

    await expect(client.getReading("reading-2")).rejects.toMatchObject({
      name: "BiofieldClientError",
      message: "Biofield analysis service is unavailable",
      status: 503,
    });
  });

  it("captures raw text payload when response is not json", async () => {
    const fetchMock = vi.fn(async () =>
      new Response("upstream unavailable", {
        status: 502,
        headers: { "Content-Type": "text/plain" },
      }),
    );

    const client = new BiofieldClient("https://example.com", { fetchImpl: fetchMock as typeof fetch });

    await expect(client.getReading("reading-3")).rejects.toMatchObject({
      name: "BiofieldClientError",
      message: "Request failed: 502",
      status: 502,
      details: { raw: "upstream unavailable" },
    });
  });
});
