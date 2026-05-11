/* ── constants ───────────────────────────────────────────── */

const CX = 150;
const CY = 150;
const DEG = Math.PI / 180;

const ZODIAC_SIGNS: readonly { name: string; symbol: string; element: "fire" | "earth" | "air" | "water" }[] = [
  { name: "Aries",       symbol: "♈", element: "fire"  },
  { name: "Taurus",      symbol: "♉", element: "earth" },
  { name: "Gemini",      symbol: "♊", element: "air"   },
  { name: "Cancer",      symbol: "♋", element: "water" },
  { name: "Leo",         symbol: "♌", element: "fire"  },
  { name: "Virgo",       symbol: "♍", element: "earth" },
  { name: "Libra",       symbol: "♎", element: "air"   },
  { name: "Scorpio",     symbol: "♏", element: "water" },
  { name: "Sagittarius", symbol: "♐", element: "fire"  },
  { name: "Capricorn",   symbol: "♑", element: "earth" },
  { name: "Aquarius",    symbol: "♒", element: "air"   },
  { name: "Pisces",      symbol: "♓", element: "water" },
] as const;

const ELEMENT_COLORS: Record<string, string> = {
  fire:  "rgba(239,107,115,0.25)",
  earth: "rgba(197,160,23,0.15)",
  air:   "rgba(255,255,255,0.08)",
  water: "rgba(11,80,251,0.2)",
};

const SIGN_INDEX: Record<string, number> = {
  aries: 0, taurus: 1, gemini: 2, cancer: 3, leo: 4, virgo: 5,
  libra: 6, scorpio: 7, sagittarius: 8, capricorn: 9, aquarius: 10, pisces: 11,
};

const PLANET_META: Record<string, { symbol: string; color: string }> = {
  sun:     { symbol: "☉", color: "#C5A017" },
  moon:    { symbol: "☽", color: "rgba(255,255,255,0.85)" },
  mercury: { symbol: "☿", color: "rgba(197,160,23,0.7)" },
  venus:   { symbol: "♀", color: "rgba(16,181,167,0.9)" },
  mars:    { symbol: "♂", color: "rgba(239,107,115,0.9)" },
  jupiter: { symbol: "♃", color: "rgba(197,160,23,0.9)" },
  saturn:  { symbol: "♄", color: "rgba(150,120,80,0.9)" },
  uranus:  { symbol: "♅", color: "rgba(100,180,220,0.8)" },
  neptune: { symbol: "♆", color: "rgba(11,80,251,0.8)" },
  pluto:   { symbol: "♇", color: "rgba(140,80,180,0.8)" },
  rahu:    { symbol: "☊", color: "rgba(120,120,120,0.7)" },
  ketu:    { symbol: "☋", color: "rgba(120,120,120,0.7)" },
};

/* ── helper types ────────────────────────────────────────── */

interface PlacedPlanet {
  key: string;
  symbol: string;
  color: string;
  angle: number;          // zodiac degrees 0-360
  retrograde: boolean;
}

/* ── helpers ─────────────────────────────────────────────── */

function str(v: unknown): string {
  if (v === null || v === undefined) return "—";
  return String(v);
}

function obj(v: unknown): Record<string, unknown> {
  if (v && typeof v === "object" && !Array.isArray(v))
    return v as Record<string, unknown>;
  return {};
}

function arr(v: unknown): unknown[] {
  return Array.isArray(v) ? v : [];
}

/** Resolve sign name → index (case-insensitive). Returns -1 if unknown. */
function signIdx(sign: unknown): number {
  if (typeof sign !== "string") return -1;
  return SIGN_INDEX[sign.toLowerCase().trim()] ?? -1;
}

/** Parse a degree from a value that might be a number or a string like "15°". */
function parseDeg(v: unknown): number {
  if (typeof v === "number") return v;
  if (typeof v === "string") {
    const n = parseFloat(v.replace("°", ""));
    return Number.isFinite(n) ? n : 0;
  }
  return 0;
}

/** Parse a position string like "Aries 15°" → { signIndex, deg } or null. */
function parsePositionString(s: unknown): { signIndex: number; deg: number } | null {
  if (typeof s !== "string") return null;
  for (const [name, idx] of Object.entries(SIGN_INDEX)) {
    const re = new RegExp(`${name}\\s*([\\d.]+)`, "i");
    const m = s.match(re);
    if (m) return { signIndex: idx, deg: parseFloat(m[1]) || 0 };
  }
  return null;
}

/** Build placed-planet list from whatever shape the result has. */
function extractPlanets(result: Record<string, unknown>): PlacedPlanet[] {
  const out: PlacedPlanet[] = [];
  const seen = new Set<string>();

  const add = (
    name: string,
    sIdx: number,
    deg: number,
    retro: boolean,
  ) => {
    const key = name.toLowerCase().trim();
    if (seen.has(key) || sIdx < 0 || sIdx > 11) return;
    seen.add(key);
    const meta = PLANET_META[key];
    if (!meta) return;
    out.push({
      key,
      symbol: meta.symbol,
      color: meta.color,
      angle: sIdx * 30 + Math.min(Math.max(deg, 0), 29.99),
      retrograde: retro,
    });
  };

  // 1. planetary_positions / positions array
  for (const p of arr(result.planetary_positions ?? result.positions)) {
    const po = obj(p);
    const name = str(po.planet ?? po.name);
    const sIdx = signIdx(po.sign ?? po.rashi);
    if (sIdx >= 0) {
      add(name, sIdx, parseDeg(po.degree ?? po.deg), Boolean(po.retrograde ?? po.retro));
    } else if (po.position) {
      const parsed = parsePositionString(po.position);
      if (parsed) add(name, parsed.signIndex, parsed.deg, Boolean(po.retrograde ?? po.retro));
    }
  }

  // 2. current_positions / planets as Record<name, info>
  const posMap = obj(result.current_positions) ?? obj(result.planets);
  for (const [name, val] of Object.entries(posMap)) {
    const po = obj(val);
    const sIdx = signIdx(po.sign ?? po.rashi);
    if (sIdx >= 0) {
      add(name, sIdx, parseDeg(po.degree ?? po.deg), Boolean(po.retrograde ?? po.retro));
    } else if (po.position) {
      const parsed = parsePositionString(po.position);
      if (parsed) add(name, parsed.signIndex, parsed.deg, Boolean(po.retrograde ?? po.retro));
    }
  }

  // 3. transits array
  for (const t of arr(result.transits)) {
    const to = obj(t);
    const name = str(to.planet ?? to.name);
    const sIdx = signIdx(to.sign ?? to.rashi);
    if (sIdx >= 0) {
      add(name, sIdx, parseDeg(to.degree ?? to.deg), Boolean(to.retrograde ?? to.retro));
    }
  }

  return out;
}

/** Convert zodiac degree → SVG x,y at given radius. 0° Aries starts at 3-o'clock. */
function zodiacToXY(zodiacDeg: number, r: number): { x: number; y: number } {
  const rad = (zodiacDeg - 90) * DEG;          // -90 so 0° is top? No — spec says 3 o'clock.
  // 0° Aries at 3 o'clock means angle 0 = right, which is the default for cos/sin.
  // But SVG y-axis is inverted, so we just use standard math:
  const rad2 = zodiacDeg * DEG;
  return { x: CX + r * Math.cos(rad2), y: CY + r * Math.sin(rad2) };
}

/** Build an SVG arc path for a ring segment (annular wedge). */
function arcSegment(
  startDeg: number,
  endDeg: number,
  rInner: number,
  rOuter: number,
): string {
  const s1 = startDeg * DEG;
  const e1 = endDeg * DEG;

  const outerStart = { x: CX + rOuter * Math.cos(s1), y: CY + rOuter * Math.sin(s1) };
  const outerEnd   = { x: CX + rOuter * Math.cos(e1), y: CY + rOuter * Math.sin(e1) };
  const innerEnd   = { x: CX + rInner * Math.cos(e1), y: CY + rInner * Math.sin(e1) };
  const innerStart = { x: CX + rInner * Math.cos(s1), y: CY + rInner * Math.sin(s1) };

  return [
    `M ${outerStart.x} ${outerStart.y}`,
    `A ${rOuter} ${rOuter} 0 0 1 ${outerEnd.x} ${outerEnd.y}`,
    `L ${innerEnd.x} ${innerEnd.y}`,
    `A ${rInner} ${rInner} 0 0 0 ${innerStart.x} ${innerStart.y}`,
    "Z",
  ].join(" ");
}

/* ── sub-components ──────────────────────────────────────── */

function ZodiacWheel({ planets, sadeSatiActive, saturnAngle }: {
  planets: PlacedPlanet[];
  sadeSatiActive: boolean;
  saturnAngle: number | null;
}) {
  return (
    <div style={{ display: "flex", justifyContent: "center", marginBottom: "0.75rem" }}>
      <svg
        viewBox="0 0 300 300"
        xmlns="http://www.w3.org/2000/svg"
        style={{ width: "100%", maxWidth: 300, height: "auto" }}
      >
        {/* ── inner dark disc ─────────────────────────── */}
        <circle cx={CX} cy={CY} r={78} fill="rgba(15,15,20,0.85)" />
        <text
          x={CX}
          y={CY + 1}
          textAnchor="middle"
          dominantBaseline="central"
          fill="rgba(255,255,255,0.18)"
          fontSize={11}
          fontWeight={700}
          letterSpacing="0.18em"
        >
          TRANSITS
        </text>

        {/* ── outer zodiac ring: 12 sign segments ─────── */}
        {ZODIAC_SIGNS.map((sign, i) => {
          const startDeg = i * 30;
          const endDeg = startDeg + 30;
          const midDeg = startDeg + 15;
          const labelPos = zodiacToXY(midDeg, 137.5);
          return (
            <g key={sign.name}>
              <path
                d={arcSegment(startDeg, endDeg, 130, 145)}
                fill={ELEMENT_COLORS[sign.element]}
                stroke="rgba(255,255,255,0.06)"
                strokeWidth={0.5}
              />
              <text
                x={labelPos.x}
                y={labelPos.y}
                textAnchor="middle"
                dominantBaseline="central"
                fontSize={9}
                fill="rgba(255,255,255,0.55)"
              >
                {sign.symbol}
              </text>
            </g>
          );
        })}

        {/* ── spoke lines every 30° ──────────────────── */}
        {Array.from({ length: 12 }).map((_, i) => {
          const a = i * 30 * DEG;
          return (
            <line
              key={`spoke-${i}`}
              x1={CX + 80 * Math.cos(a)}
              y1={CY + 80 * Math.sin(a)}
              x2={CX + 128 * Math.cos(a)}
              y2={CY + 128 * Math.sin(a)}
              stroke="rgba(255,255,255,0.06)"
              strokeWidth={0.5}
            />
          );
        })}

        {/* ── middle ring border circles ──────────────── */}
        <circle cx={CX} cy={CY} r={128} fill="none" stroke="rgba(255,255,255,0.06)" strokeWidth={0.5} />
        <circle cx={CX} cy={CY} r={80}  fill="none" stroke="rgba(255,255,255,0.08)" strokeWidth={0.5} />
        <circle cx={CX} cy={CY} r={78}  fill="none" stroke="rgba(255,255,255,0.04)" strokeWidth={0.5} />

        {/* ── sade-sati arc indicator ─────────────────── */}
        {sadeSatiActive && saturnAngle !== null && (() => {
          const arcStart = (saturnAngle - 20) * DEG;
          const arcEnd   = (saturnAngle + 20) * DEG;
          const r = 125;
          return (
            <path
              d={[
                `M ${CX + r * Math.cos(arcStart)} ${CY + r * Math.sin(arcStart)}`,
                `A ${r} ${r} 0 0 1 ${CX + r * Math.cos(arcEnd)} ${CY + r * Math.sin(arcEnd)}`,
              ].join(" ")}
              fill="none"
              stroke="rgba(197,160,23,0.35)"
              strokeWidth={2.5}
              strokeLinecap="round"
            />
          );
        })()}

        {/* ── planet dots ────────────────────────────── */}
        {planets.map((p) => {
          const pos = zodiacToXY(p.angle, 104);
          return (
            <g key={p.key}>
              <circle cx={pos.x} cy={pos.y} r={6} fill={p.color} opacity={0.85} />
              <text
                x={pos.x}
                y={pos.y + 0.5}
                textAnchor="middle"
                dominantBaseline="central"
                fontSize={9}
                fill="#fff"
                fontWeight={600}
              >
                {p.symbol}
              </text>
              {p.retrograde && (
                <text
                  x={pos.x + 7}
                  y={pos.y + 5}
                  fontSize={6}
                  fill="rgba(239,107,115,0.9)"
                  fontWeight={700}
                >
                  R
                </text>
              )}
            </g>
          );
        })}
      </svg>
    </div>
  );
}

/* ── styles ──────────────────────────────────────────────── */

const styles = {
  container: { display: "flex", flexDirection: "column" as const, gap: "0.75rem" },
  grid: {
    display: "grid",
    gridTemplateColumns: "repeat(auto-fit, minmax(200px, 1fr))",
    gap: "0.75rem",
  },
  cell: {
    background: "var(--field)",
    borderRadius: "var(--radius)",
    padding: "0.75rem",
    display: "flex",
    flexDirection: "column" as const,
    gap: "0.375rem",
  },
  label: {
    fontSize: "0.7rem",
    color: "var(--text-muted)",
    textTransform: "uppercase" as const,
    letterSpacing: "0.06em",
    fontWeight: 600,
  },
  value: {
    fontSize: "0.9rem",
    fontWeight: 600,
    color: "var(--text)",
  },
  sectionTitle: {
    fontSize: "0.75rem",
    color: "var(--gold)",
    fontWeight: 600,
    textTransform: "uppercase" as const,
    letterSpacing: "0.06em",
    marginTop: "0.5rem",
  },
  row: {
    display: "flex",
    justifyContent: "space-between",
    padding: "0.375rem 0.5rem",
    borderBottom: "1px solid var(--line)",
    fontSize: "0.85rem",
  },
  badge: {
    fontSize: "0.7rem",
    padding: "0.15rem 0.4rem",
    borderRadius: 4,
    fontWeight: 600,
  },
};

/* ── main component ──────────────────────────────────────── */

interface TransitsProps {
  result: Record<string, unknown>;
}

export default function Transits({ result }: TransitsProps) {
  const positions = arr(result.planetary_positions ?? result.positions ?? []);
  const aspects = arr(result.significant_aspects ?? result.aspects ?? []);
  const sadeSati = obj(result.sade_sati);

  // Extract planet placements for the wheel
  const placedPlanets = extractPlanets(result);

  // Determine sade-sati highlight
  const sadeSatiActive = Boolean(sadeSati.active ?? result.sade_sati_active);
  const saturn = placedPlanets.find((p) => p.key === "saturn");

  return (
    <div style={styles.container}>
      {/* ── SVG Zodiac Wheel ───────────────────────── */}
      <ZodiacWheel
        planets={placedPlanets}
        sadeSatiActive={sadeSatiActive}
        saturnAngle={saturn?.angle ?? null}
      />

      {/* ── Existing text cells below ─────────────── */}
      {positions.length > 0 && (
        <div>
          <span style={styles.sectionTitle}>Planetary Positions</span>
          <div style={styles.grid}>
            {positions.map((p, i) => {
              const planet = obj(p);
              return (
                <div key={i} style={styles.cell}>
                  <span style={styles.label}>{str(planet.planet ?? planet.name)}</span>
                  <span style={styles.value}>
                    {str(planet.sign ?? planet.rashi)} {planet.degree != null ? `${str(planet.degree)}°` : ""}
                  </span>
                  {planet.nakshatra != null && (
                    <span style={{ fontSize: "0.8rem", color: "var(--text-muted)" }}>
                      {str(planet.nakshatra)}
                    </span>
                  )}
                </div>
              );
            })}
          </div>
        </div>
      )}

      {aspects.length > 0 && (
        <div>
          <span style={styles.sectionTitle}>Significant Aspects</span>
          {aspects.map((a, i) => {
            const aspect = obj(a);
            return (
              <div key={i} style={styles.row}>
                <span style={{ color: "var(--text)" }}>
                  {str(aspect.planet1 ?? aspect.from)} — {str(aspect.planet2 ?? aspect.to)}
                </span>
                <span
                  style={{
                    ...styles.badge,
                    background: "var(--gold-soft)",
                    color: "var(--gold)",
                  }}
                >
                  {str(aspect.type ?? aspect.aspect)}
                </span>
              </div>
            );
          })}
        </div>
      )}

      {sadeSati.active != null && (
        <div>
          <span style={styles.sectionTitle}>Sade Sati</span>
          <div style={styles.row}>
            <span>Status</span>
            <span
              style={{
                ...styles.badge,
                background: sadeSati.active ? "rgba(239,107,115,0.12)" : "var(--emerald-soft)",
                color: sadeSati.active ? "var(--danger)" : "var(--emerald)",
              }}
            >
              {sadeSati.active ? "Active" : "Not Active"}
            </span>
          </div>
          {sadeSati.phase != null && (
            <div style={styles.row}>
              <span>Phase</span>
              <span style={{ color: "var(--text)" }}>{str(sadeSati.phase)}</span>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
