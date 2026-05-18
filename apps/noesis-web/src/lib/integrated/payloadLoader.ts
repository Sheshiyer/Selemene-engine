// ─── payloadLoader — load a reading payload by id or URL hash ──────────
// Client-side loader. Two paths to obtain a payload:
//
//   1. URL hash (zero-roundtrip): the landing app encodes the full
//      payload as base64-JSON in the URL hash on redirect. We decode
//      and return it directly — no fetch.
//      Example: https://depth.tryambakam.space/r/abc123#payload=eyJ...
//
//   2. Backend fetch (fallback): if no hash payload, GET the reading
//      by id from the backend. Useful for shareable URLs and refresh.
//      Example: https://depth.tryambakam.space/r/abc123
//                 → GET https://48.tryambakam.space/api/v1/readings/abc123
//
// The payload shape is loose — different workflows return different
// fields. `buildSectionsFromPayload` adapts.

export interface ReadingPart {
  id?: string;
  numeral?: string;
  title?: string;
  /** The body markdown for this part. May contain tables/lists/headings. */
  markdown: string;
}

export interface ReadingPayload {
  reading_id?: string;
  /** Which workflow produced this — daily-practice, integrated-reading, etc. */
  workflow_id?: string;
  created_at?: string;
  subject?: {
    name?: string;
    birth_date?: string;
    birth_time?: string;
    timezone?: string;
    location_label?: string;
  };
  /** For integrated readings — 11+ parts as markdown. Present if and
   *  only if this is an integrated/multi-part reading. */
  passes?: ReadingPart[];
  /** Daily-practice witness layer — single short reading body. */
  witness_layer?: {
    inference?: {
      content?: string;
      markdown?: string;
      text?: string;
      body?: string;
      [key: string]: unknown;
    };
    [key: string]: unknown;
  };
  /** Per-engine outputs — used by the "compendium" section to surface
   *  a chart-at-a-glance summary. */
  engine_results?: Record<string, unknown>;
  /** Arbitrary additional fields — preserved for downstream rendering. */
  [key: string]: unknown;
}

const BACKEND_URL =
  process.env.NEXT_PUBLIC_BACKEND_URL ?? "https://48.tryambakam.space";

/** Decode a URL hash like `#payload=base64...&other=x` into an object. */
function readHashParams(): Record<string, string> {
  if (typeof window === "undefined") return {};
  const hash = window.location.hash.replace(/^#/, "");
  if (!hash) return {};
  const out: Record<string, string> = {};
  for (const pair of hash.split("&")) {
    if (!pair) continue;
    const [k, v = ""] = pair.split("=");
    out[decodeURIComponent(k)] = decodeURIComponent(v);
  }
  return out;
}

/** Base64-decode a URL-safe string back to JSON. */
function base64Decode(b64: string): string {
  // URL-safe → standard base64
  const normalized = b64.replace(/-/g, "+").replace(/_/g, "/");
  // Add padding back if missing
  const padded = normalized + "===".slice((normalized.length + 3) % 4);
  if (typeof atob === "function") return atob(padded);
  // Node fallback (build-time SSR shouldn't hit this, but guard anyway)
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  return (globalThis as any).Buffer.from(padded, "base64").toString("utf-8");
}

/** Try to extract an inline payload from the URL hash. Returns null if
 *  not present or malformed. */
export function tryDecodeHashPayload(): ReadingPayload | null {
  if (typeof window === "undefined") return null;
  const params = readHashParams();
  const encoded = params["payload"];
  if (!encoded) return null;
  try {
    const json = base64Decode(encoded);
    const payload = JSON.parse(json) as ReadingPayload;
    return payload;
  } catch (e) {
    console.warn("[payloadLoader] hash decode failed:", e);
    return null;
  }
}

/** Fetch a reading payload by id from the backend. */
export async function fetchPayloadById(id: string): Promise<ReadingPayload | null> {
  try {
    const res = await fetch(`${BACKEND_URL}/api/v1/readings/${encodeURIComponent(id)}`, {
      headers: { Accept: "application/json" },
      // Don't send credentials cross-origin unless needed
      credentials: "omit",
    });
    if (!res.ok) {
      console.warn(`[payloadLoader] fetch ${id} returned HTTP ${res.status}`);
      return null;
    }
    return (await res.json()) as ReadingPayload;
  } catch (e) {
    console.error("[payloadLoader] fetch failed:", e);
    return null;
  }
}

/** Convenience — try hash first, fall back to fetch. */
export async function loadPayload(id: string): Promise<ReadingPayload | null> {
  const inline = tryDecodeHashPayload();
  if (inline) return inline;
  return fetchPayloadById(id);
}
