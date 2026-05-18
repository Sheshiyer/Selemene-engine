// ─── witnessAccess — TS port of the landing's witness API client ──────
// Drop-in equivalent of witness-agents-intro-web/js/lib/witnessAccess.js.
// Exposes:
//   - Workflow URLs (daily, integrated)
//   - City location list with timezone + coordinates
//   - encodePayloadForUrl / buildDepthReadingUrl for redirect handoff
// All client-safe (no Node APIs).

export const WITNESS_ORIGIN =
  process.env.NEXT_PUBLIC_WITNESS_ORIGIN ?? "https://48.tryambakam.space";

export const DAILY_WITNESS_WORKFLOW_URL = `${WITNESS_ORIGIN}/api/v1/workflows/daily-practice/execute`;
export const INTEGRATED_READING_WORKFLOW_URL = `${WITNESS_ORIGIN}/api/v1/workflows/integrated-reading/execute`;

export interface WitnessLocation {
  key: string;
  label: string;
  group: string;
  timezone: string;
  latitude: number;
  longitude: number;
}

// Same city list as the Vite landing — when expanding, mirror both.
export const WITNESS_LOCATIONS: WitnessLocation[] = [
  { key: "bengaluru",   label: "Bengaluru, India",        group: "India",        timezone: "Asia/Kolkata",      latitude: 12.9716, longitude:  77.5946 },
  { key: "mumbai",      label: "Mumbai, India",           group: "India",        timezone: "Asia/Kolkata",      latitude: 19.0760, longitude:  72.8777 },
  { key: "new-delhi",   label: "New Delhi, India",        group: "India",        timezone: "Asia/Kolkata",      latitude: 28.6139, longitude:  77.2090 },
  { key: "chennai",     label: "Chennai, India",          group: "India",        timezone: "Asia/Kolkata",      latitude: 13.0827, longitude:  80.2707 },
  { key: "kolkata",     label: "Kolkata, India",          group: "India",        timezone: "Asia/Kolkata",      latitude: 22.5726, longitude:  88.3639 },
  { key: "hyderabad",   label: "Hyderabad, India",        group: "India",        timezone: "Asia/Kolkata",      latitude: 17.3850, longitude:  78.4867 },
  { key: "london",      label: "London, UK",              group: "Europe",       timezone: "Europe/London",     latitude: 51.5074, longitude:  -0.1278 },
  { key: "paris",       label: "Paris, France",           group: "Europe",       timezone: "Europe/Paris",      latitude: 48.8566, longitude:   2.3522 },
  { key: "berlin",      label: "Berlin, Germany",         group: "Europe",       timezone: "Europe/Berlin",     latitude: 52.5200, longitude:  13.4050 },
  { key: "new-york",    label: "New York, USA",           group: "Americas",     timezone: "America/New_York",  latitude: 40.7128, longitude: -74.0060 },
  { key: "los-angeles", label: "Los Angeles, USA",        group: "Americas",     timezone: "America/Los_Angeles",latitude: 34.0522, longitude:-118.2437 },
  { key: "san-francisco", label: "San Francisco, USA",    group: "Americas",     timezone: "America/Los_Angeles",latitude: 37.7749, longitude:-122.4194 },
  { key: "toronto",     label: "Toronto, Canada",         group: "Americas",     timezone: "America/Toronto",   latitude: 43.6532, longitude: -79.3832 },
  { key: "tokyo",       label: "Tokyo, Japan",            group: "Asia Pacific", timezone: "Asia/Tokyo",        latitude: 35.6762, longitude: 139.6503 },
  { key: "singapore",   label: "Singapore",               group: "Asia Pacific", timezone: "Asia/Singapore",    latitude:  1.3521, longitude: 103.8198 },
  { key: "sydney",      label: "Sydney, Australia",       group: "Asia Pacific", timezone: "Australia/Sydney",  latitude: -33.8688, longitude: 151.2093 },
  { key: "dubai",       label: "Dubai, UAE",              group: "Middle East",  timezone: "Asia/Dubai",        latitude: 25.2048, longitude:  55.2708 },
];

export const getWitnessLocation = (key: string): WitnessLocation | null =>
  WITNESS_LOCATIONS.find((l) => l.key === key) ?? null;

export const getWitnessLocationGroups = (): Array<{
  label: string;
  locations: WitnessLocation[];
}> => {
  const groups: Record<string, WitnessLocation[]> = {};
  for (const l of WITNESS_LOCATIONS) {
    if (!groups[l.group]) groups[l.group] = [];
    groups[l.group].push(l);
  }
  return Object.entries(groups).map(([label, locations]) => ({ label, locations }));
};

// ─── Depth-reading URL builder (mirror of the Vite version) ─────────────

/** Where the depth-reading viewer lives. In Next.js single-app mode the
 *  redirect is in-app — but we still build a full URL for share/copy
 *  scenarios. Override via env or localStorage. */
export const getDepthReadingHost = (): string => {
  if (typeof window !== "undefined") {
    try {
      const override = window.localStorage.getItem("depth-reading.host");
      if (override) return override.replace(/\/$/, "");
    } catch {
      /* ignore */
    }
  }
  return typeof window !== "undefined" ? window.location.origin : "";
};

/** UTF-8-safe URL-friendly base64 encode of a payload. */
export const encodePayloadForUrl = (payload: unknown): string => {
  const json = JSON.stringify(payload);
  // encodeURIComponent → unescape → btoa is the standard UTF-8 → b64 path
  // eslint-disable-next-line @typescript-eslint/no-deprecated
  const bytes = unescape(encodeURIComponent(json));
  return btoa(bytes).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
};

/** Build the /r/[id] route + hash payload for in-app navigation. */
export const buildDepthReadingPath = (payload: { reading_id?: string } & Record<string, unknown>): string => {
  const id = (typeof payload?.reading_id === "string" && payload.reading_id.trim()) || "latest";
  const encoded = encodePayloadForUrl(payload);
  return `/r/${encodeURIComponent(id)}#payload=${encoded}`;
};

/** Full external URL — used for share/copy buttons, not for in-app nav. */
export const buildDepthReadingUrl = (
  payload: { reading_id?: string } & Record<string, unknown>,
): string => {
  return `${getDepthReadingHost()}${buildDepthReadingPath(payload)}`;
};

// ─── Form storage (per-user remembered form data) ───────────────────────
export const WITNESS_FORM_STORAGE_KEY = "witness-agents.daily-witness-form";

export interface StoredWitnessForm {
  name?: string;
  birth_date?: string;
  birth_time?: string;
  location_key?: string;
}

export const readWitnessForm = (): StoredWitnessForm => {
  if (typeof window === "undefined") return {};
  try {
    const raw = window.localStorage.getItem(WITNESS_FORM_STORAGE_KEY);
    if (!raw) return {};
    return JSON.parse(raw) as StoredWitnessForm;
  } catch {
    return {};
  }
};

export const writeWitnessForm = (form: StoredWitnessForm): void => {
  if (typeof window === "undefined") return;
  try {
    window.localStorage.setItem(WITNESS_FORM_STORAGE_KEY, JSON.stringify(form));
  } catch {
    /* storage may be unavailable in private-browse */
  }
};
