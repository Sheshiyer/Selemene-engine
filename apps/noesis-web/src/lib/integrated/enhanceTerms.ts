// ─── enhanceTerms — post-process verse HTML to wrap technical terms ─────
// Runs after marked → HTML conversion. Replaces matched substrings with
// <span data-engine-term data-engine-id="X" data-term-definition="Y">…</span>
// markers. VerseFlow then hydrates these spans into <EngineTermLink/> via
// a useEffect-attached click handler on mount.
//
// Per design v2 § 5.10. Term dictionary covers common Vedic / HD /
// Gene-Keys / Tarot / I-Ching vocabulary likely to appear in the L3
// integrated reading prose.
//
// Replacement is whole-word, case-insensitive (but the original casing
// from the source is preserved). Replacement skips already-wrapped
// matches and HTML attribute payloads so we don't touch tag internals.

export interface TermEntry {
  /** The engine to open when the term is clicked. */
  engineId: string;
  /** 1-line definition shown in the hover tooltip. */
  definition: string;
  /** Optional alternate spellings / inflections that should resolve to the
   *  same engine + definition. */
  aliases?: string[];
}

// ─── Term dictionary ────────────────────────────────────────────────────
// Keys are the canonical display label (case as you'd want it in prose).
// Aliases are extra spellings that share the same destination.
export const TERM_DICT: Record<string, TermEntry> = {
  // ─── Vimshottari / dasha system ───────────────────────────────────────
  Mahadasha: {
    engineId: "vimshottari",
    definition: "Major planetary period in the Vimshottari dasha system.",
    aliases: ["Maha Dasha", "Maha-Dasha"],
  },
  Antardasha: {
    engineId: "vimshottari",
    definition: "Sub-period nested inside a Mahadasha.",
    aliases: ["Antar Dasha", "Antar-Dasha", "Bhukti"],
  },
  Pratyantar: {
    engineId: "vimshottari",
    definition: "Third-level sub-period inside an Antardasha.",
    aliases: ["Pratyantar Dasha", "Pratyantardasha"],
  },
  Atmakaraka: {
    engineId: "vimshottari",
    definition: "Soul-significator — the planet at the highest degree in the chart.",
  },
  "Sade Sati": {
    engineId: "transits",
    definition: "Seven-and-a-half-year Saturn transit through the signs flanking the natal Moon.",
    aliases: ["Sadesati", "Sade-Sati"],
  },

  // ─── Panchanga / nakshatra / classical Vedic terms ────────────────────
  Nakshatra: {
    engineId: "panchanga",
    definition: "One of 27 lunar mansions — the sky-region a planet occupies.",
    aliases: ["Nakshatras"],
  },
  Tithi: {
    engineId: "panchanga",
    definition: "Lunar day — one of 30 phase-units of the Moon-Sun cycle.",
  },
  Yoga: {
    engineId: "panchanga",
    definition: "One of 27 Sun-Moon longitudinal combinations of the Panchanga.",
  },
  Karana: {
    engineId: "panchanga",
    definition: "Half-tithi unit of the Panchanga (11 variants).",
  },
  Vara: {
    engineId: "panchanga",
    definition: "Weekday limb of the Panchanga, ruled by a planet.",
  },

  // ─── Lagna / vedic-clock ──────────────────────────────────────────────
  Lagna: {
    engineId: "vedic-clock",
    definition: "Ascendant — the rising sign at the moment of birth.",
    aliases: ["Ascendant"],
  },
  Hora: {
    engineId: "vedic-clock",
    definition: "Planetary hour — 24th-division ruler within a day.",
  },

  // ─── Transits ─────────────────────────────────────────────────────────
  "Jupiter return": {
    engineId: "transits",
    definition: "Twelve-year Jupiter cycle returning to its natal position.",
  },
  "Saturn return": {
    engineId: "transits",
    definition: "~29-year Saturn cycle returning to its natal position.",
  },
  Gochar: {
    engineId: "transits",
    definition: "Sanskrit term for planetary transits across the natal chart.",
  },

  // ─── Sacred geometry ──────────────────────────────────────────────────
  "Sri Yantra": {
    engineId: "sacred-geometry",
    definition: "Nine interlocking triangles — the central yantra of Sri Vidya.",
    aliases: ["Shri Yantra", "Sriyantra"],
  },
  Yantra: {
    engineId: "sacred-geometry",
    definition: "Geometric meditation diagram encoding a deity or principle.",
  },
  "Vesica Piscis": {
    engineId: "sacred-geometry",
    definition: "Almond-shape intersection of two equal circles — root of sacred geometry.",
  },

  // ─── Human Design ─────────────────────────────────────────────────────
  Bodygraph: {
    engineId: "human-design",
    definition: "Human Design's 9-center body diagram of channels and gates.",
  },
  "Incarnation Cross": {
    engineId: "human-design",
    definition: "The four-gate signature defining your life's purpose in Human Design.",
  },
  Authority: {
    engineId: "human-design",
    definition: "Inner decision-making mechanism in Human Design (Sacral, Splenic, Emotional, …).",
  },

  // ─── Gene Keys ────────────────────────────────────────────────────────
  "Shadow-Gift-Siddhi": {
    engineId: "gene-keys",
    definition: "The three-frequency spectrum of every Gene Key — shadow, gift, siddhi.",
    aliases: ["Shadow Gift Siddhi", "Shadow/Gift/Siddhi"],
  },
  Siddhi: {
    engineId: "gene-keys",
    definition: "Highest-frequency expression of a Gene Key.",
  },
  Hexagram: {
    engineId: "i-ching",
    definition: "One of 64 six-line I-Ching figures.",
  },

  // ─── Tarot ────────────────────────────────────────────────────────────
  "Major Arcana": {
    engineId: "tarot",
    definition: "The 22 trump cards of the Tarot — archetypal life-stage milestones.",
  },
  "Minor Arcana": {
    engineId: "tarot",
    definition: "The 56 suited cards of the Tarot — everyday currents.",
  },

  // ─── Numerology / Enneagram / Biorhythm ───────────────────────────────
  "Life Path": {
    engineId: "numerology",
    definition: "Numerology's core number derived from the birth date.",
    aliases: ["Life Path Number"],
  },
  Enneagram: {
    engineId: "enneagram",
    definition: "Nine-pointed map of personality types and their motivations.",
  },
  Biorhythm: {
    engineId: "biorhythm",
    definition: "Sine-cycle model of physical, emotional, and intellectual states.",
  },

  // ─── Raaga / Nadabrahman ──────────────────────────────────────────────
  Raaga: {
    engineId: "raaga",
    definition: "Melodic mode of Indian classical music tied to time, season, and mood.",
    aliases: ["Raga"],
  },
  Nadabrahman: {
    engineId: "nadabrahman",
    definition: "The sound-as-cosmos principle — sacred-vibration mapping in Vedic acoustics.",
    aliases: ["Nada Brahman", "Nada-Brahman"],
  },
};

// ─── HTML-safe term wrapper ─────────────────────────────────────────────

/** All canonical terms + aliases, sorted longest-first so multi-word
 *  matches win over their substring ("Maha Dasha" before "Dasha", etc.). */
function buildMatchPairs(): Array<{ pattern: string; canonical: string }> {
  const pairs: Array<{ pattern: string; canonical: string }> = [];
  for (const [canonical, entry] of Object.entries(TERM_DICT)) {
    pairs.push({ pattern: canonical, canonical });
    for (const alias of entry.aliases ?? []) {
      pairs.push({ pattern: alias, canonical });
    }
  }
  // Longest patterns first — important for "Maha Dasha" vs "Dasha".
  pairs.sort((a, b) => b.pattern.length - a.pattern.length);
  return pairs;
}

function escapeRegExp(s: string): string {
  return s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function escapeHtmlAttr(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/"/g, "&quot;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}

/**
 * Walk through HTML preserving tags verbatim and only rewriting *text*
 * nodes. Within each text segment, we replace each known term with a
 * span marker that carries the engine + definition payload.
 *
 * We are intentionally conservative: we never touch attribute payloads,
 * code blocks, or text already inside [data-engine-term].
 */
export function enhanceTermsInHtml(html: string): string {
  if (!html) return html;
  const pairs = buildMatchPairs();
  if (pairs.length === 0) return html;

  // Tokenise into tag / text segments. The regex captures tags as a whole
  // unit so we can leave them untouched.
  const TOKEN_RE = /(<[^>]+>)|([^<]+)/g;

  // Build a single combined regex once. Boundaries use lookarounds so
  // multi-word terms with spaces still match (\b doesn't fit there).
  const altern = pairs.map((p) => escapeRegExp(p.pattern)).join("|");
  // (^|non-word-or-tag-boundary) + term + (non-word-or-end). We rely on
  // the text-only segment scope to keep this safe.
  const TERM_RE = new RegExp(
    `(^|[^A-Za-z0-9_])(${altern})(?=[^A-Za-z0-9_]|$)`,
    "gi",
  );
  // Lookup canonical by lowercased pattern, longest-first preserved by
  // insertion order in the Map.
  const patternIndex = new Map<string, string>();
  for (const p of pairs) {
    const key = p.pattern.toLowerCase();
    if (!patternIndex.has(key)) patternIndex.set(key, p.canonical);
  }

  // Per-segment depth tracker: skip inside <code>, <pre>, <a> (already a
  // link), and our own [data-engine-term] markers — though the latter
  // only exist post-rewrite so are not a concern on first pass.
  const skipStack: string[] = [];

  let out = "";
  let m: RegExpExecArray | null;
  TOKEN_RE.lastIndex = 0;
  while ((m = TOKEN_RE.exec(html)) !== null) {
    const tag = m[1];
    const text = m[2];

    if (tag) {
      out += tag;
      // Track nesting we want to skip inside.
      const openMatch = tag.match(/^<\s*([a-zA-Z0-9]+)/);
      const closeMatch = tag.match(/^<\s*\/\s*([a-zA-Z0-9]+)/);
      if (closeMatch) {
        const name = closeMatch[1].toLowerCase();
        const idx = skipStack.lastIndexOf(name);
        if (idx >= 0) skipStack.splice(idx, 1);
      } else if (openMatch && !tag.endsWith("/>")) {
        const name = openMatch[1].toLowerCase();
        if (name === "code" || name === "pre" || name === "a") {
          skipStack.push(name);
        }
      }
      continue;
    }
    if (text == null) continue;
    if (skipStack.length > 0) {
      out += text;
      continue;
    }

    // Replace term occurrences. We don't replace if a previous match in
    // the same text segment already consumed the same span (handled by
    // RegExp.lastIndex bookkeeping).
    let last = 0;
    let rewrote = "";
    TERM_RE.lastIndex = 0;
    let tm: RegExpExecArray | null;
    while ((tm = TERM_RE.exec(text)) !== null) {
      const lead = tm[1];
      const match = tm[2];
      const matchStart = tm.index + lead.length;
      rewrote += text.slice(last, matchStart);
      const canonical = patternIndex.get(match.toLowerCase());
      if (!canonical) {
        rewrote += match;
      } else {
        const entry = TERM_DICT[canonical];
        rewrote +=
          `<span data-engine-term="1" ` +
          `data-engine-id="${escapeHtmlAttr(entry.engineId)}" ` +
          `data-term-definition="${escapeHtmlAttr(entry.definition)}" ` +
          `data-term-canonical="${escapeHtmlAttr(canonical)}">` +
          match +
          `</span>`;
      }
      last = matchStart + match.length;
    }
    rewrote += text.slice(last);
    out += rewrote;
  }
  return out;
}
