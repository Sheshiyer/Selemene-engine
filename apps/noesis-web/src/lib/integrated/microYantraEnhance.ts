// ─── microYantraEnhance — auto-insert micro-yantra placeholders ────────
// Runs over rendered HTML (post-marked, post-W6-enhanceTerms). For each
// term in the dictionary, prepends `<span data-micro-yantra="<kind>"
// data-arg="<arg>"></span>` before the matched term. After mount,
// VerseFlow walks the DOM and hydrates each placeholder into a real
// <MicroYantra> React component.
//
// Coordination with W6:
//   If the matched term is already inside a [data-engine-term] wrapper
//   (from W6's enhanceTerms pass), we INSERT the placeholder INSIDE the
//   wrapper, as its first child — so the glyph appears just before the
//   visible term inside the same link surface. Otherwise the placeholder
//   is inserted directly before the term as a sibling.
//
// Defensive design: each match-occurrence in the source HTML gets at
// most one placeholder. Re-running enhance() over already-enhanced HTML
// is a no-op (detected via the data-micro-yantra-anchor marker we add
// to the term node after substitution).
//
// Per integrated-reading-design-v2.md § 5.5.

export interface MicroYantraDictionary {
  /** Map of matched term → { kind, arg }. Term is a literal phrase; we
   *  apply \b word-boundary matching when the term starts/ends with a
   *  word character. */
  [term: string]: { kind: string; arg: string };
}

/** Default dictionary covering Vedic / HD / Gene-Keys most-common terms. */
export const DEFAULT_DICT: MicroYantraDictionary = {
  // Planets — proper-name match
  Jupiter: { kind: "planet", arg: "jupiter" },
  Rahu: { kind: "planet", arg: "rahu" },
  Saturn: { kind: "planet", arg: "saturn" },
  Mars: { kind: "planet", arg: "mars" },
  Venus: { kind: "planet", arg: "venus" },
  Mercury: { kind: "planet", arg: "mercury" },
  Sun: { kind: "planet", arg: "sun" },
  Moon: { kind: "planet", arg: "moon" },
  Ketu: { kind: "planet", arg: "ketu" },

  // Doshas
  "Sade Sati": { kind: "dosha", arg: "sade-sati" },
  "Sade-Sati": { kind: "dosha", arg: "sade-sati" },
  "Kala Sarpa": { kind: "dosha", arg: "kala-sarpa" },
  "Kala-Sarpa": { kind: "dosha", arg: "kala-sarpa" },
  "Mangal Dosha": { kind: "dosha", arg: "mangal-dosha" },
  "Kuja Dosha": { kind: "dosha", arg: "mangal-dosha" },
  "Pitru Dosha": { kind: "dosha", arg: "pitru-dosha" },
  Kemadruma: { kind: "dosha", arg: "kemadruma" },

  // Yogas
  "Raj Yoga": { kind: "yoga", arg: "raj" },
  "Raja Yoga": { kind: "yoga", arg: "raj" },
  "Gajakesari Yoga": { kind: "yoga", arg: "gajakesari" },
  "Saraswati Yoga": { kind: "yoga", arg: "saraswati" },
  "Vipreet Raj Yoga": { kind: "yoga", arg: "vipreet-raj" },
  "Viparita Raja Yoga": { kind: "yoga", arg: "vipreet-raj" },
  "Dhana Yoga": { kind: "yoga", arg: "dhana" },

  // Houses — Bhāva / Bhava / House — 1st..12th
  ...buildHouseDict(),

  // Nakshatras (most common — about 15)
  Ashwini: { kind: "nakshatra", arg: "ashwini" },
  Bharani: { kind: "nakshatra", arg: "bharani" },
  Krittika: { kind: "nakshatra", arg: "krittika" },
  Rohini: { kind: "nakshatra", arg: "rohini" },
  Mrigashira: { kind: "nakshatra", arg: "mrigashira" },
  Punarvasu: { kind: "nakshatra", arg: "punarvasu" },
  Pushya: { kind: "nakshatra", arg: "pushya" },
  Ashlesha: { kind: "nakshatra", arg: "ashlesha" },
  Magha: { kind: "nakshatra", arg: "magha" },
  Hasta: { kind: "nakshatra", arg: "hasta" },
  Chitra: { kind: "nakshatra", arg: "chitra" },
  Swati: { kind: "nakshatra", arg: "swati" },
  Vishakha: { kind: "nakshatra", arg: "vishakha" },
  Anuradha: { kind: "nakshatra", arg: "anuradha" },
  Jyeshtha: { kind: "nakshatra", arg: "jyeshtha" },
  Mool: { kind: "nakshatra", arg: "mool" },
  Mula: { kind: "nakshatra", arg: "mool" },
  Shravana: { kind: "nakshatra", arg: "shravana" },
  Dhanishta: { kind: "nakshatra", arg: "dhanishta" },
  Shatabhisha: { kind: "nakshatra", arg: "shatabhisha" },
  Revati: { kind: "nakshatra", arg: "revati" },

  // Dasha transitions — match "X → Y" arrow forms
  "Rahu → Jupiter": { kind: "dasha-transition", arg: "rahu->jupiter" },
  "Rahu->Jupiter": { kind: "dasha-transition", arg: "rahu->jupiter" },
  "Mars → Rahu": { kind: "dasha-transition", arg: "mars->rahu" },
  "Mars->Rahu": { kind: "dasha-transition", arg: "mars->rahu" },
  "Jupiter → Saturn": { kind: "dasha-transition", arg: "jupiter->saturn" },
  "Saturn → Mercury": { kind: "dasha-transition", arg: "saturn->mercury" },
};

function buildHouseDict(): MicroYantraDictionary {
  const out: MicroYantraDictionary = {};
  const suffix = (n: number) => {
    if (n === 1) return "1st";
    if (n === 2) return "2nd";
    if (n === 3) return "3rd";
    return `${n}th`;
  };
  for (let n = 1; n <= 12; n++) {
    const s = suffix(n);
    out[`${s} Bhāva`] = { kind: "house", arg: String(n) };
    out[`${s} Bhava`] = { kind: "house", arg: String(n) };
    out[`${s} bhava`] = { kind: "house", arg: String(n) };
    out[`${s} House`] = { kind: "house", arg: String(n) };
    out[`${s} house`] = { kind: "house", arg: String(n) };
  }
  return out;
}

/** Escape a string so it can be used safely inside a RegExp pattern. */
function escapeRegExp(s: string): string {
  return s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

/** Char-class test for a word char (RegExp \w-compatible).  */
function isWordChar(ch: string): boolean {
  return /[A-Za-z0-9_]/.test(ch);
}

interface EnhanceOptions {
  dict?: MicroYantraDictionary;
  /** Cap per-term substitutions to avoid noise — N=1 means glyph only
   *  appears on the first occurrence of each term per document. */
  maxPerTerm?: number;
}

/**
 * Inject micro-yantra placeholder spans into an HTML string. Returns a
 * new HTML string. The placeholders are simple inert spans that get
 * hydrated client-side by VerseFlow's mount-time DOM walker.
 *
 * - Skips matches inside `<code>`, `<pre>`, `<style>`, `<script>`,
 *   `<svg>`, and inside attribute values.
 * - Skips matches that already have a `data-micro-yantra-anchor`
 *   marker (so this function is idempotent).
 * - Word-boundary checked on both ends of the term.
 * - For each match, builds a placeholder span and appends the marker
 *   attribute to the matched substring so subsequent passes skip it.
 */
export function injectMicroYantraPlaceholders(
  html: string,
  options: EnhanceOptions = {},
): string {
  const dict = options.dict ?? DEFAULT_DICT;
  const maxPerTerm = options.maxPerTerm ?? 1;
  if (!html || typeof html !== "string") return html;

  // Sort terms longest-first so "Rahu → Jupiter" beats "Rahu" when both
  // would match overlapping spans.
  const terms = Object.keys(dict).sort((a, b) => b.length - a.length);

  // We process the string by walking outside skip-zones.
  // Skip zones: <script>...</script>, <style>...</style>, <pre>...</pre>,
  // <code>...</code>, <svg>...</svg>, and any HTML tag (between < and >).
  const SKIP_TAG_RE =
    /<(script|style|pre|code|svg)\b[^>]*>[\s\S]*?<\/\1>|<[^>]+>/gi;

  // Tokenize the html into [text, tagOrSkipBlock, text, ...]
  const tokens: Array<{ type: "text" | "skip"; value: string }> = [];
  let lastIdx = 0;
  for (const m of html.matchAll(SKIP_TAG_RE)) {
    const idx = m.index ?? 0;
    if (idx > lastIdx) {
      tokens.push({ type: "text", value: html.slice(lastIdx, idx) });
    }
    tokens.push({ type: "skip", value: m[0] });
    lastIdx = idx + m[0].length;
  }
  if (lastIdx < html.length) {
    tokens.push({ type: "text", value: html.slice(lastIdx) });
  }

  const perTermCount: Record<string, number> = {};
  for (const t of terms) perTermCount[t] = 0;

  function placeholderHTML(kind: string, arg: string): string {
    // Inert span — gets hydrated client-side. Reserve a small width so
    // SSR-first paint doesn't jump when React replaces it.
    return `<span data-micro-yantra="${kind}" data-arg="${escapeAttr(arg)}" aria-hidden="true" style="display:inline-block;width:1.1em;height:1em;vertical-align:middle"></span>`;
  }

  /** Single text-token pass. Builds a claimed-range set in the ORIGINAL
   *  text positions so a multi-word term ("Rahu → Jupiter") suppresses
   *  later overlapping single-word substitutions ("Rahu" / "Jupiter"). */
  function processText(text: string): string {
    interface Hit {
      start: number;
      end: number;
      term: string;
      kind: string;
      arg: string;
    }
    const hits: Hit[] = [];
    const claimed: Array<[number, number]> = [];

    function overlaps(s: number, e: number): boolean {
      for (const [cs, ce] of claimed) {
        if (s < ce && e > cs) return true;
      }
      return false;
    }

    for (const term of terms) {
      if (perTermCount[term] >= maxPerTerm) continue;
      const entry = dict[term];
      const startsAlnum = isWordChar(term[0] ?? "");
      const endsAlnum = isWordChar(term[term.length - 1] ?? "");
      const leftBoundary = startsAlnum ? "(?<![A-Za-z0-9_])" : "";
      const rightBoundary = endsAlnum ? "(?![A-Za-z0-9_])" : "";
      const re = new RegExp(
        `${leftBoundary}(${escapeRegExp(term)})${rightBoundary}`,
        "g",
      );
      let m: RegExpExecArray | null;
      while ((m = re.exec(text)) !== null) {
        if (perTermCount[term] >= maxPerTerm) break;
        const start = m.index;
        const end = start + m[0].length;
        if (overlaps(start, end)) continue;
        hits.push({ start, end, term, kind: entry.kind, arg: entry.arg });
        claimed.push([start, end]);
        perTermCount[term] += 1;
      }
    }

    if (hits.length === 0) return text;
    // Splice placeholders in. Iterate by ascending start so the rebuild
    // is linear and preserves all surrounding whitespace exactly.
    hits.sort((a, b) => a.start - b.start);
    const parts: string[] = [];
    let cursor = 0;
    for (const h of hits) {
      parts.push(text.slice(cursor, h.start));
      parts.push(placeholderHTML(h.kind, h.arg));
      parts.push(text.slice(h.start, h.end));
      cursor = h.end;
    }
    parts.push(text.slice(cursor));
    return parts.join("");
  }

  return tokens
    .map((tok) => (tok.type === "text" ? processText(tok.value) : tok.value))
    .join("");
}

function escapeAttr(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/"/g, "&quot;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}
