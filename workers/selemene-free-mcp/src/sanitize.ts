export const SHARED_STORAGE_DISCLOSURE =
  "Free readings use a shared Selemene administrator account. Birth inputs and calculated results may be stored there; usage, profile, XP and phase may change. There is no separate private user history.";
const PRIVATE_KEYS = new Set([
  "auth", "userid", "accountid", "apikey", "token", "accesstoken", "refreshtoken",
  "authorization", "cookie", "setcookie", "password", "secret", "credentials",
  "consciousnesslevel", "linkedat", "email",
]);
const normalized = (key: string) => key.replace(/[^a-z0-9]/gi, "").toLowerCase();

export function sanitizeText(text: string, secret: string): string {
  const variants = [secret, encodeURIComponent(secret), JSON.stringify(secret).slice(1, -1)]
    .filter(Boolean).sort((a, b) => b.length - a.length);
  let clean = text;
  for (const variant of variants) clean = clean.split(variant).join("[REDACTED]");
  let decoded = clean;
  for (let pass = 0; pass < 3; pass++) {
    try {
      const next = decodeURIComponent(decoded);
      if (secret && next.includes(secret)) return "[REDACTED]";
      if (next === decoded) break;
      decoded = next;
    } catch { break; }
  }
  clean = clean.replace(/\bnk_[a-z0-9_-]+/gi, "[REDACTED]");
  clean = clean.replace(/https?:\/\/[^\s<>"']+/gi, (raw) => {
    try {
      const url = new URL(raw);
      if (url.username || url.password || [...url.searchParams.keys()].some((k) =>
        PRIVATE_KEYS.has(normalized(k)))) return "[REDACTED URL]";
    } catch { return "[REDACTED URL]"; }
    return raw;
  });
  return clean;
}
export function sanitizeValue(value: unknown, secret: string, depth = 0): unknown {
  if (depth > 48) return "[OMITTED]";
  if (typeof value === "string") {
    // Tool content contains serialized JSON; remove its private fields too.
    if (/^\s*[\[{]/.test(value)) {
      try { return JSON.stringify(sanitizeValue(JSON.parse(value), secret, depth + 1)); }
      catch { /* Non-JSON text still receives credential redaction. */ }
    }
    return sanitizeText(value, secret);
  }
  if (Array.isArray(value)) return value.map((item) => sanitizeValue(item, secret, depth + 1));
  if (value && typeof value === "object") {
    const clean: Record<string, unknown> = {};
    for (const [key, item] of Object.entries(value)) {
      if (key === "_meta" || PRIVATE_KEYS.has(normalized(key))) continue;
      clean[sanitizeText(key, secret)] = sanitizeValue(item, secret, depth + 1);
    }
    return clean;
  }
  return value;
}
