/** Fixed, bounded adapter for the existing Swiss Ephemeris D1/D9 chart route. */
import { z } from "zod";
import type { ToolRegistration, ToolContext, ToolResult } from "../../selemene-mcp/src/tools/index.js";

const MAX_REQUEST_BYTES = 32 * 1024;
const MAX_RESPONSE_BYTES = 256 * 1024;
const TIMEOUT_MS = 45_000;
const ALLOWED_ORIGINS = new Set([
  "https://selemene-engine-production.up.railway.app",
  "https://selemene.tryambakam.space",
]);
const CHART_ROUTE = "/api/v1/charts/vedic";

function validDate(value: string): boolean {
  const [year, month, day] = value.split("-").map(Number);
  if (year < 1000 || year > 3000) return false;
  const parsed = new Date(Date.UTC(year, month - 1, day));
  return parsed.getUTCFullYear() === year && parsed.getUTCMonth() === month - 1 && parsed.getUTCDate() === day;
}
function validTimezone(value: string): boolean {
  // Named IANA zones match the backend's chrono_tz parser; numeric offsets are
  // intentionally excluded so a caller must resolve ambiguous dates explicitly.
  if (!/^[A-Za-z][A-Za-z0-9_+\-/]*$/.test(value) || value.length > 100) return false;
  try { new Intl.DateTimeFormat("en-US", { timeZone: value }); return true; }
  catch { return false; }
}
const DateInput = z.string().regex(/^\d{4}-\d{2}-\d{2}$/).refine(validDate, "invalid calendar date or year outside 1000..3000");
const TimeInput = z.string().regex(/^(?:[01]\d|2[0-3]):[0-5]\d(?::[0-5]\d)?$/, "time must be HH:MM or HH:MM:SS");
const BirthDataInput = z.object({
  name: z.string().trim().min(1).max(200).optional(),
  date: DateInput,
  time: TimeInput,
  latitude: z.number().finite().min(-90).max(90),
  longitude: z.number().finite().min(-180).max(180),
  timezone: z.string().refine(validTimezone, "unsupported IANA timezone"),
}).strict();
const VedicChartInput = z.object({
  birth_data: BirthDataInput,
  current_time: z.string().datetime({ offset: true }).optional(),
}).strict();

/** Count real UTC instants matching a local wall time, rather than accepting
 * the backend's earlier-of-two fallback for a DST fold. Sampling nearby offsets
 * also detects date-line gaps and half-hour changes without using the host TZ. */
function localDatetimeError(date: string, time: string, timezone: string): string | null {
  const [year, month, day] = date.split("-").map(Number);
  const [hour, minute, second = 0] = time.split(":").map(Number);
  const wall = Date.UTC(year, month - 1, day, hour, minute, second);
  const formatter = new Intl.DateTimeFormat("en-US", {
    timeZone: timezone, year: "numeric", month: "2-digit", day: "2-digit",
    hour: "2-digit", minute: "2-digit", second: "2-digit", hourCycle: "h23",
  });
  const wallAt = (instant: number): number => {
    const parts = formatter.formatToParts(new Date(instant));
    const field = (kind: string) => Number(parts.find(p => p.type === kind)?.value);
    return Date.UTC(field("year"), field("month") - 1, field("day"), field("hour"), field("minute"), field("second"));
  };
  const offsets = new Set<number>();
  for (let hours = -48; hours <= 48; hours += 6) {
    const instant = wall + hours * 3_600_000;
    offsets.add(wallAt(instant) - instant);
  }
  const matches = new Set<number>();
  for (const offset of offsets) {
    const instant = wall - offset;
    if (wallAt(instant) === wall) matches.add(instant);
  }
  if (matches.size === 1) return null;
  return matches.size === 0
    ? "birth local time does not exist in this timezone (clock gap); supply a verified UTC birth date/time with timezone UTC"
    : "birth local time is ambiguous in this timezone (clock fold); supply a verified UTC birth date/time with timezone UTC";
}

// These schemas mirror build_d1_chart/build_d9_chart in noesis-api. Zod strips
// unknown fields at every level; is_combust is deliberately absent because the
// upstream currently hardcodes that placeholder to false.
const Sign = z.enum(["aries", "taurus", "gemini", "cancer", "leo", "virgo", "libra", "scorpio", "sagittarius", "capricorn", "aquarius", "pisces"]);
const PlanetName = z.enum(["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn", "Rahu", "Ketu"]);
const NumberFinite = z.number().finite();
const Degree = NumberFinite.min(0).lt(30);
const Longitude = NumberFinite.min(0).lt(360);
const Text = z.string().min(1).max(100);
const Pada = z.number().int().min(1).max(4);
const Native = z.object({
  birth_date: DateInput, birth_time: TimeInput,
  latitude: NumberFinite.min(-90).max(90), longitude: NumberFinite.min(-180).max(180),
  timezone: NumberFinite.min(-24).max(24),
});
const Planet = z.object({
  name: PlanetName, longitude: Longitude, sign: Sign, degree: Degree,
  minutes: NumberFinite.min(0).lt(60), house: z.number().int().min(1).max(12),
  is_retrograde: z.boolean(), nakshatra: Text, pada: Pada,
  speed: NumberFinite, latitude: NumberFinite.min(-90).max(90),
});
const House = z.object({
  number: z.number().int().min(1).max(12), sign: Sign, cusp: Longitude, degree: Degree,
  house_type: z.enum(["dharma", "artha", "kama", "moksha"]),
  is_kendra: z.boolean(), is_panapara: z.boolean(), is_apoklima: z.boolean(),
});
const ChartBundle = z.object({
  d1: z.object({
    ascendant: z.object({ sign: Sign, degree: Degree, nakshatra: Text, pada: Pada }),
    ayanamsa: NumberFinite, house_system: z.literal("whole_sign"),
    houses: z.array(House).length(12),
    moon: z.object({ sign: Sign, degree: Degree, nakshatra: Text, pada: Pada, rashi_lord: Text }),
    native: Native,
    notes: z.array(z.literal("Native computation via Swiss Ephemeris (Lahiri ayanamsa, whole-sign houses)")).length(1),
    planets: z.array(Planet).length(9),
    special_points: z.object({ lagna: Longitude, midheaven: NumberFinite.nullable(), part_of_fortune: Longitude }),
  }),
  d9: z.object({
    d9_lagna: Sign,
    navamsa_positions: z.array(z.object({ planet: PlanetName, sign: Sign, degree: Degree, is_vargottama: z.boolean() })).length(9),
    source: Native, vargottama: z.array(PlanetName).max(9),
  }),
});
function fail(error: string, kind = "unavailable", status?: number): ToolResult {
  const payload = { error, kind, ...(status === undefined ? {} : { status }) };
  return { isError: true, content: [{ type: "text", text: JSON.stringify(payload) }], structuredContent: payload };
}
function statusError(status: number): ToolResult {
  if (status === 401 || status === 403) return fail("service authentication unavailable", "service_auth_unavailable", status);
  if (status === 404) return fail("Vedic chart route unavailable", "not_found", status);
  if (status === 429) return fail("service request limit reached", "rate_limited", status);
  return fail("Vedic chart service request failed", "unavailable", status);
}

async function boundedBody(response: Response, signal: AbortSignal): Promise<string> {
  const reader = response.body?.getReader();
  if (!reader) throw new Error("empty body");
  const chunks: Uint8Array[] = [];
  let total = 0;
  try {
    while (true) {
      let abort: () => void = () => {};
      const aborted = new Promise<never>((_, reject) => {
        abort = () => reject(new Error("body interrupted"));
        if (signal.aborted) abort();
        else signal.addEventListener("abort", abort, { once: true });
      });
      let next: ReadableStreamReadResult<Uint8Array>;
      try { next = await Promise.race([reader.read(), aborted]); }
      finally { signal.removeEventListener("abort", abort); }
      if (next.done) break;
      total += next.value.byteLength;
      if (total > MAX_RESPONSE_BYTES) throw new Error("body exceeds byte cap");
      chunks.push(next.value);
    }
  } finally {
    // Cancel on bounds, read failure, or timeout; do not await an uncooperative
    // upstream cancellation promise and extend the handler deadline.
    void reader.cancel().catch(() => {});
    reader.releaseLock();
  }
  const bytes = new Uint8Array(total);
  let position = 0;
  for (const chunk of chunks) { bytes.set(chunk, position); position += chunk.byteLength; }
  return new TextDecoder("utf-8", { fatal: true, ignoreBOM: false }).decode(bytes);
}

export const VEDIC_CHART_TOOL: ToolRegistration = {
  name: "selemene_vedic_chart",
  description: "Calculate natal Vedic D1 Rasi and D9 Navamsa facts using Swiss Ephemeris planetary positions, an approximate linear Lahiri ayanamsa formula, and whole-sign houses. Requires verified birth date, time, coordinates, and IANA timezone. Interpretation is composed by ChatGPT. Combustion, dignities, yogas, Atmakaraka, and Darakaraka are unavailable. The chart handler does not save charts; service authentication, rate limits, and operational logs still apply.",
  inputSchema: VedicChartInput,
  annotations: { title: "Vedic Chart (D1/D9)", readOnlyHint: true, idempotentHint: true, destructiveHint: false, openWorldHint: false },
  requiredScope: "calculate",
  securitySchemes: { noauth: true },
  async handler(ctx: ToolContext, args: Record<string, unknown>): Promise<ToolResult> {
    const origin = ctx.env.selemeneApiOrigin;
    if (!ALLOWED_ORIGINS.has(origin)) return fail("Vedic chart service configuration unavailable");
    if (!ctx.scope.read || !ctx.scope.calculate) return fail("calculation permission required", "permission_denied");
    const parsed = VedicChartInput.safeParse(args);
    if (!parsed.success) return fail("invalid Vedic chart input: provide a real date, birth time, bounded coordinates, and IANA timezone without extra fields", "invalid_input");
    const input = parsed.data;
    const localError = localDatetimeError(input.birth_data.date, input.birth_data.time, input.birth_data.timezone);
    if (localError) return fail(localError, "invalid_input");
    const apiKey = ctx.props.apiKey;
    if (typeof apiKey !== "string" || apiKey.length === 0 || apiKey.length > 512 || /[\r\n]/.test(apiKey)) return fail("service authentication unavailable", "service_auth_unavailable");
    const body = JSON.stringify({ birth_data: input.birth_data, current_time: input.current_time ?? new Date().toISOString(), precision: "Standard", options: {} });
    if (new TextEncoder().encode(body).byteLength > MAX_REQUEST_BYTES) return fail("chart request exceeds byte limit", "invalid_input");
    const controller = new AbortController();
    const timeout = setTimeout(() => controller.abort(), TIMEOUT_MS);
    try {
      const response = await fetch(`${origin}${CHART_ROUTE}`, {
        method: "POST", headers: { "Content-Type": "application/json", Accept: "application/json", "X-API-Key": apiKey },
        // Cloudflare Workers supports manual redirect handling, not "error".
        // A 3xx response is canceled and rejected by the status check below.
        body, redirect: "manual", signal: controller.signal,
      });
      if (!response.ok) {
        void response.body?.cancel().catch(() => {});
        return statusError(response.status);
      }
      const raw = await boundedBody(response, controller.signal);
      let data: unknown;
      try { data = JSON.parse(raw); } catch { return fail("Vedic chart service returned an invalid result"); }
      const validated = ChartBundle.safeParse(data);
      if (!validated.success) return fail("Vedic chart service returned an unsupported result contract");
      const payload = {
        chart_kind: "vedic-d1-d9", result: validated.data,
        calculation_method: "Swiss Ephemeris planetary positions, approximate linear Lahiri ayanamsa, whole-sign houses",
        limitations: {
          unavailable_fields: ["combustion", "dignities", "yogas", "atmakaraka", "darakaraka"],
          ayanamsa_precision: "The chart handler uses an approximate linear Lahiri formula (23.85 + Julian centuries since J2000 * 1.3968 degrees), rather than a separately verified sidereal ayanamsa implementation.",
          note: "This D1/D9 bundle is not the richer local vedic-kundali report supplement.",
        },
        persistence_status: "not_persisted_by_chart_handler",
      };
      return { content: [{ type: "text", text: JSON.stringify(payload) }], structuredContent: payload };
    } catch {
      return fail(controller.signal.aborted ? "Vedic chart request timed out" : "Vedic chart service transport unavailable", "transport_error");
    } finally { clearTimeout(timeout); }
  },
};
