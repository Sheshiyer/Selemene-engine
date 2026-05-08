const styles = {
  container: { display: "flex", flexDirection: "column" as const, gap: "0.75rem" },
  cardRow: {
    background: "var(--field)",
    borderRadius: "var(--radius)",
    padding: "0.75rem 1rem",
    display: "flex",
    flexDirection: "column" as const,
    gap: "0.25rem",
  },
  position: {
    fontSize: "0.7rem",
    color: "var(--text-muted)",
    textTransform: "uppercase" as const,
    letterSpacing: "0.06em",
    fontWeight: 600,
  },
  cardName: {
    fontSize: "1.1rem",
    fontWeight: 700,
    color: "var(--gold)",
    fontFamily: "'Exo 2', sans-serif",
  },
  meta: {
    display: "flex",
    gap: "0.5rem",
    flexWrap: "wrap" as const,
  },
  tag: {
    fontSize: "0.7rem",
    padding: "0.15rem 0.4rem",
    borderRadius: 4,
    fontWeight: 600,
    background: "var(--field-hover)",
    color: "var(--text-muted)",
  },
  reversed: {
    background: "rgba(239,107,115,0.12)",
    color: "var(--danger)",
  },
  meaning: {
    fontSize: "0.85rem",
    color: "var(--text)",
    lineHeight: 1.5,
  },
};

function str(v: unknown): string {
  if (v === null || v === undefined) return "—";
  return String(v);
}

function obj(v: unknown): Record<string, unknown> {
  if (v && typeof v === "object" && !Array.isArray(v)) return v as Record<string, unknown>;
  return {};
}

function arr(v: unknown): unknown[] {
  return Array.isArray(v) ? v : [];
}

const POSITIONS = ["Past", "Present", "Future"];

interface TarotProps {
  result: Record<string, unknown>;
}

export default function Tarot({ result }: TarotProps) {
  const cards = arr(result.cards ?? result.spread);

  if (cards.length === 0) {
    const card = obj(result.card ?? result);
    return (
      <div style={styles.container}>
        <div style={styles.cardRow}>
          <span style={styles.position}>Card</span>
          <span style={styles.cardName}>{str(card.name)}</span>
          <div style={styles.meta}>
            {card.suit != null && <span style={styles.tag}>{str(card.suit)}</span>}
            {card.arcana != null && <span style={styles.tag}>{str(card.arcana)}</span>}
            {Boolean(card.reversed) && <span style={{ ...styles.tag, ...styles.reversed }}>Reversed</span>}
          </div>
          {card.meaning != null && <p style={styles.meaning}>{str(card.meaning)}</p>}
          {card.key_meaning != null && <p style={styles.meaning}>{str(card.key_meaning)}</p>}
        </div>
      </div>
    );
  }

  return (
    <div style={styles.container}>
      {cards.map((c, i) => {
        const card = obj(c);
        return (
          <div key={i} style={styles.cardRow}>
            <span style={styles.position}>
              {str(card.position ?? POSITIONS[i] ?? `Card ${i + 1}`)}
            </span>
            <span style={styles.cardName}>{str(card.name)}</span>
            <div style={styles.meta}>
              {card.suit != null && <span style={styles.tag}>{str(card.suit)}</span>}
              {card.arcana != null && <span style={styles.tag}>{str(card.arcana)}</span>}
              {Boolean(card.reversed) && <span style={{ ...styles.tag, ...styles.reversed }}>Reversed</span>}
            </div>
            {(card.meaning != null || card.key_meaning != null) && (
              <p style={styles.meaning}>{str(card.meaning ?? card.key_meaning)}</p>
            )}
          </div>
        );
      })}
    </div>
  );
}
