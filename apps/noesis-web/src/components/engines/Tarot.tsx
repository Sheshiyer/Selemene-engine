// ─── Helpers ─────────────────────────────────────────────────────────────────

function str(v: unknown): string {
  if (v === null || v === undefined) return "";
  return String(v);
}

function obj(v: unknown): Record<string, unknown> {
  if (v && typeof v === "object" && !Array.isArray(v)) return v as Record<string, unknown>;
  return {};
}

function arr(v: unknown): unknown[] {
  return Array.isArray(v) ? v : [];
}

// ─── Suit / Element Logic ────────────────────────────────────────────────────

function suitSymbol(suit: string | undefined): string {
  if (!suit) return "◈";
  const s = suit.toLowerCase();
  if (s.includes("wand") || s.includes("fire")) return "⟁";
  if (s.includes("cup") || s.includes("water")) return "◯";
  if (s.includes("sword") || s.includes("air")) return "✦";
  if (s.includes("pentacle") || s.includes("earth") || s.includes("coin")) return "⬡";
  return "◈";
}

function elementColor(element: string | undefined): string {
  if (!element) return "transparent";
  const e = element.toLowerCase();
  if (e.includes("fire")) return "var(--error, #C65D3B)";
  if (e.includes("water")) return "var(--c-indigo, #0B50FB)";
  if (e.includes("air")) return "rgba(255,255,255,0.7)";
  if (e.includes("earth")) return "var(--c-emerald, #10B5A7)";
  return "transparent";
}

function elementFromSuit(suit: string | undefined): string | undefined {
  if (!suit) return undefined;
  const s = suit.toLowerCase();
  if (s.includes("wand")) return "Fire";
  if (s.includes("cup")) return "Water";
  if (s.includes("sword")) return "Air";
  if (s.includes("pentacle") || s.includes("coin")) return "Earth";
  return undefined;
}

// ─── Data Extraction ─────────────────────────────────────────────────────────

interface CardData {
  cardName: string;
  suit: string | undefined;
  number: string | undefined;
  isReversed: boolean;
  element: string | undefined;
  keywords: string[];
  interpretation: string | undefined;
  description: string | undefined;
  uprightMeaning: string | undefined;
  reversedMeaning: string | undefined;
  positionDesc: string | undefined;
}

function extractCardData(raw: Record<string, unknown>): CardData {
  const card = obj(raw.card ?? raw);
  const reading = obj(raw.reading);

  const cardName = str(card.name ?? raw.card_name ?? raw.name) || "Unknown";
  const suit = str(card.suit ?? raw.suit) || undefined;
  const numRaw = card.number ?? card.arcana_number ?? raw.number;
  const number = numRaw != null ? str(numRaw) : undefined;
  const isReversed = !!(card.reversed ?? raw.reversed ?? raw.is_reversed);
  const rawElement = str(card.element ?? raw.element) || undefined;
  const element = rawElement ?? elementFromSuit(suit);

  const kwRaw = card.keywords ?? raw.keywords;
  const keywords: string[] = Array.isArray(kwRaw)
    ? kwRaw.map((k: unknown) => str(k)).filter(Boolean)
    : [];

  const interpretation = str(
    reading.present ?? reading.interpretation ?? raw.interpretation
  ) || undefined;

  const description = str(card.description ?? raw.description) || undefined;
  const uprightMeaning = str(card.meaning ?? card.key_meaning ?? card.upright_meaning ?? raw.meaning ?? raw.key_meaning) || undefined;
  const reversedMeaning = str(card.reversed_meaning ?? raw.reversed_meaning) || undefined;
  const positionDesc = str(card.position_description ?? raw.position_description) || undefined;

  return {
    cardName, suit, number, isReversed, element,
    keywords, interpretation, description, uprightMeaning,
    reversedMeaning, positionDesc,
  };
}

// ─── Card Face Component ─────────────────────────────────────────────────────

const CARD_W = 140;
const CARD_H = Math.round(CARD_W * (5 / 3)); // 233px

const cardFaceStyle: React.CSSProperties = {
  position: "relative",
  width: CARD_W,
  height: CARD_H,
  borderRadius: 8,
  border: "1px solid var(--c-gold, #C5A017)",
  background:
    "radial-gradient(ellipse at 50% 40%, var(--c-violet, #6B21A8) 0%, var(--c-void, #0A0A12) 100%)",
  boxShadow:
    "inset 0 0 24px rgba(197,160,23,0.15), 0 0 32px rgba(16,181,167,0.12)",
  display: "flex",
  flexDirection: "column",
  alignItems: "center",
  justifyContent: "center",
  overflow: "hidden",
  margin: "0 auto",
  flexShrink: 0,
};

const suitGlyphStyle: React.CSSProperties = {
  fontSize: "2.8rem",
  color: "var(--c-gold, #C5A017)",
  opacity: 0.85,
  lineHeight: 1,
  textShadow: "0 0 18px rgba(197,160,23,0.35), 0 0 40px rgba(16,181,167,0.18)",
  userSelect: "none",
};

const cardNumberStyle: React.CSSProperties = {
  position: "absolute",
  top: 8,
  left: 10,
  fontSize: "0.7rem",
  fontFamily: "var(--font-display, serif)",
  fontWeight: 700,
  color: "var(--c-gold, #C5A017)",
  opacity: 0.8,
  lineHeight: 1,
};

const cardNameLabelStyle: React.CSSProperties = {
  position: "absolute",
  bottom: 10,
  left: 0,
  right: 0,
  textAlign: "center",
  fontSize: "0.55rem",
  fontFamily: "var(--font-body, sans-serif)",
  fontWeight: 700,
  color: "var(--c-gold, #C5A017)",
  textTransform: "uppercase",
  letterSpacing: "0.1em",
  lineHeight: 1.2,
  padding: "0 6px",
  opacity: 0.9,
};

const reversedBadgeStyle: React.CSSProperties = {
  position: "absolute",
  top: 6,
  right: 6,
  fontSize: "0.45rem",
  fontWeight: 800,
  color: "var(--c-terracotta, #C65D3B)",
  textTransform: "uppercase",
  letterSpacing: "0.08em",
  lineHeight: 1,
  transform: "rotate(180deg)",
};

const elementDotStyle = (color: string): React.CSSProperties => ({
  position: "absolute",
  bottom: 10,
  right: 10,
  width: 6,
  height: 6,
  borderRadius: "50%",
  background: color,
  boxShadow: `0 0 6px ${color}`,
});

interface CardFaceProps {
  data: CardData;
}

function CardFace({ data }: CardFaceProps) {
  const symbol = suitSymbol(data.suit);
  const elColor = elementColor(data.element);

  return (
    <div style={cardFaceStyle}>
      {/* Card number — top left */}
      {data.number && (
        <span style={cardNumberStyle}>{data.number}</span>
      )}

      {/* Reversed badge — top right, upside-down */}
      {data.isReversed && (
        <span style={reversedBadgeStyle}>Reversed</span>
      )}

      {/* Central suit sigil */}
      <span style={suitGlyphStyle} aria-label={data.suit ?? "Major Arcana"}>
        {symbol}
      </span>

      {/* Card name — bottom center */}
      <span style={cardNameLabelStyle}>{data.cardName}</span>

      {/* Element dot — bottom right */}
      {data.element && (
        <span
          style={elementDotStyle(elColor)}
          aria-label={data.element}
        />
      )}
    </div>
  );
}

// ─── Styles ──────────────────────────────────────────────────────────────────

const styles = {
  container: {
    display: "flex",
    flexDirection: "column" as const,
    gap: "0.75rem",
  },
  cardSlot: {
    display: "flex",
    flexDirection: "column" as const,
    alignItems: "center" as const,
    gap: "0.6rem",
    padding: "1rem 0.75rem 0.75rem",
    background: "var(--field)",
    borderRadius: "var(--radius)",
  },
  positionLabel: {
    fontSize: "0.65rem",
    color: "var(--text-muted)",
    textTransform: "uppercase" as const,
    letterSpacing: "0.08em",
    fontWeight: 600,
    alignSelf: "flex-start" as const,
  },
  keywordsRow: {
    display: "flex",
    gap: "0.35rem",
    flexWrap: "wrap" as const,
    justifyContent: "center" as const,
  },
  kwPill: {
    fontSize: "0.6rem",
    padding: "0.15rem 0.45rem",
    borderRadius: 10,
    fontWeight: 600,
    background: "rgba(197,160,23,0.08)",
    color: "var(--c-gold, #C5A017)",
    border: "1px solid rgba(197,160,23,0.18)",
    lineHeight: 1.4,
  },
  interpretation: {
    fontSize: "0.85rem",
    color: "var(--text)",
    lineHeight: 1.55,
    borderLeft: "3px solid var(--c-violet, #6B21A8)",
    paddingLeft: "0.65rem",
    fontStyle: "italic" as const,
    width: "100%",
  },
  section: {
    display: "flex",
    flexDirection: "column" as const,
    gap: "0.2rem",
    paddingLeft: "0.5rem",
    borderLeft: "2px solid var(--line-gold, rgba(197,160,23,0.2))",
    width: "100%",
  },
  sectionTitle: {
    fontSize: "0.7rem",
    color: "var(--text-muted)",
    textTransform: "uppercase" as const,
    letterSpacing: "0.06em",
    fontWeight: 600,
  },
  sectionText: {
    fontSize: "0.85rem",
    color: "var(--text)",
    lineHeight: 1.5,
  },
};

// ─── Single Card Render ──────────────────────────────────────────────────────

interface CardSlotProps {
  data: CardData;
  position?: string;
}

function CardSlot({ data, position }: CardSlotProps) {
  return (
    <div style={styles.cardSlot}>
      {/* Position label */}
      {position && (
        <span style={styles.positionLabel}>{position}</span>
      )}

      {/* Bioluminescent card face */}
      <CardFace data={data} />

      {/* Keywords as pill tags */}
      {data.keywords.length > 0 && (
        <div style={styles.keywordsRow}>
          {data.keywords.map((kw) => (
            <span key={kw} style={styles.kwPill}>{kw}</span>
          ))}
        </div>
      )}

      {/* Interpretation blockquote */}
      {data.interpretation && (
        <p style={styles.interpretation}>{data.interpretation}</p>
      )}

      {/* Description */}
      {data.description && (
        <div style={styles.section}>
          <span style={styles.sectionTitle}>Description</span>
          <p style={styles.sectionText}>{data.description}</p>
        </div>
      )}

      {/* Position description */}
      {data.positionDesc && (
        <div style={styles.section}>
          <span style={styles.sectionTitle}>Position</span>
          <p style={styles.sectionText}>{data.positionDesc}</p>
        </div>
      )}

      {/* Upright meaning */}
      {data.uprightMeaning && !data.isReversed && (
        <div style={styles.section}>
          <span style={styles.sectionTitle}>Meaning</span>
          <p style={styles.sectionText}>{data.uprightMeaning}</p>
        </div>
      )}

      {/* Reversed meaning */}
      {data.reversedMeaning && data.isReversed && (
        <div style={styles.section}>
          <span style={styles.sectionTitle}>Reversed Meaning</span>
          <p style={styles.sectionText}>{data.reversedMeaning}</p>
        </div>
      )}
    </div>
  );
}

// ─── Main Component ──────────────────────────────────────────────────────────

const POSITIONS = ["Past", "Present", "Future"];

interface TarotProps {
  result: Record<string, unknown>;
}

export default function Tarot({ result }: TarotProps) {
  const cards = arr(result.cards ?? result.spread);

  // Single card reading
  if (cards.length === 0) {
    const data = extractCardData(result);
    return (
      <div style={styles.container}>
        <CardSlot data={data} position="Card" />
      </div>
    );
  }

  // Multi-card spread
  return (
    <div style={styles.container}>
      {cards.map((c, i) => {
        const raw = obj(c);
        const data = extractCardData(raw);
        const position = str(raw.position) || POSITIONS[i] || `Card ${i + 1}`;
        return <CardSlot key={i} data={data} position={position} />;
      })}
    </div>
  );
}
