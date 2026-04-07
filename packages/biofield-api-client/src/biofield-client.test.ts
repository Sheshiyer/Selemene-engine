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
      new Response(JSON.stringify([]), { status: 200 }),
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

  it("raises a typed error on failed requests", async () => {
    const fetchMock = vi.fn(async () =>
      new Response(JSON.stringify({ error: "nope" }), { status: 503 }),
    );

    const client = new BiofieldClient("https://example.com", { fetchImpl: fetchMock as typeof fetch });

    await expect(client.getReading("reading-1")).rejects.toBeInstanceOf(BiofieldClientError);
  });
});
