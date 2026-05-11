/* ─── styles ─── */

const styles = {
  grid: {
    display: "grid",
    gridTemplateColumns: "repeat(auto-fit, minmax(180px, 1fr))",
    gap: "0.75rem",
  },
  cell: {
    background: "var(--field)",
    borderRadius: "var(--radius)",
    padding: "0.75rem",
    display: "flex",
    flexDirection: "column" as const,
    gap: "0.25rem",
  },
  label: {
    fontSize: "0.7rem",
    color: "var(--text-muted)",
    textTransform: "uppercase" as const,
    letterSpacing: "0.06em",
    fontWeight: 600,
  },
  value: {
    fontSize: "1rem",
    fontWeight: 600,
    color: "var(--text)",
  },
  sub: {
    fontSize: "0.8rem",
    color: "var(--text-muted)",
    fontFamily: "var(--font-mono)",
  },
  section: {
    marginTop: "0.75rem",
    display: "flex",
    flexDirection: "column" as const,
    gap: "0.375rem",
  },
  sectionTitle: {
    fontSize: "0.75rem",
    color: "var(--gold)",
    fontWeight: 600,
    textTransform: "uppercase" as const,
    letterSpacing: "0.06em",
  },
  centerRow: {
    display: "flex",
    justifyContent: "space-between",
    padding: "0.25rem 0.5rem",
    borderBottom: "1px solid var(--line)",
    fontSize: "0.85rem",
  },
  bodygraphWrap: {
    display: "flex",
    justifyContent: "center" as const,
    marginBottom: "1rem",
  },
};

/* ─── helpers ─── */

function str(v: unknown): string {
  if (v === null || v === undefined) return "—";
  return String(v);
}

function obj(v: unknown): Record<string, unknown> {
  if (v && typeof v === "object" && !Array.isArray(v))
    return v as Record<string, unknown>;
  return {};
}

/* ─── center definitions ─── */

const CENTER_ALIASES: Record<string, string[]> = {
  head: ["head", "crown"],
  ajna: ["ajna", "mind"],
  throat: ["throat"],
  g: ["g", "self", "identity", "higher_self"],
  ego: ["ego", "will", "heart", "willpower"],
  sacral: ["sacral"],
  spleen: ["spleen", "splenic", "lymph"],
  solar_plexus: ["solar_plexus", "emotional", "sp"],
  root: ["root"],
};

const CENTERS_DISPLAY = [
  "Head",
  "Ajna",
  "Throat",
  "G",
  "Heart",
  "Sacral",
  "Spleen",
  "Solar Plexus",
  "Root",
];

/* ─── bodygraph geometry ─── */

type ShapeKind = "diamond" | "tri-down" | "tri-up" | "rect";

interface CenterGeo {
  key: string;          // lookup key into CENTER_ALIASES
  label: string;        // display label
  shape: ShapeKind;
  cx: number;
  cy: number;           // center-y of the shape
  w: number;
  h: number;
}

const BODY_CENTERS: CenterGeo[] = [
  { key: "head",         label: "Head",     shape: "diamond",  cx: 100, cy: 20,  w: 40, h: 28 },
  { key: "ajna",         label: "Ajna",     shape: "tri-down", cx: 100, cy: 65,  w: 40, h: 28 },
  { key: "throat",       label: "Throat",   shape: "rect",     cx: 100, cy: 112, w: 60, h: 24 },
  { key: "g",            label: "G",        shape: "diamond",  cx: 100, cy: 150, w: 44, h: 36 },
  { key: "ego",          label: "Ego",      shape: "tri-up",   cx: 148, cy: 140, w: 28, h: 24 },
  { key: "sacral",       label: "Sacral",   shape: "rect",     cx: 100, cy: 197, w: 60, h: 24 },
  { key: "spleen",       label: "Spleen",   shape: "tri-up",   cx: 48,  cy: 195, w: 28, h: 36 },
  { key: "solar_plexus", label: "SP",       shape: "tri-down", cx: 152, cy: 195, w: 28, h: 36 },
  { key: "root",         label: "Root",     shape: "rect",     cx: 100, cy: 240, w: 60, h: 24 },
];

/** Channels as [fromKey, toKey] */
const CHANNELS: [string, string][] = [
  ["head", "ajna"],
  ["ajna", "throat"],
  ["throat", "g"],
  ["throat", "ego"],
  ["g", "sacral"],
  ["g", "spleen"],
  ["sacral", "root"],
  ["sacral", "solar_plexus"],
  ["solar_plexus", "root"],
  ["spleen", "root"],
];

/* ─── center defined check ─── */

function isCenterDefined(
  key: string,
  centersObj: Record<string, unknown>,
  definedList: string[] | undefined,
): boolean {
  const aliases = CENTER_ALIASES[key];
  if (!aliases) return false;

  // Check definedList first (e.g. result.defined_centers)
  if (definedList) {
    return definedList.some((c) => {
      const lower = c.toLowerCase().replace(/[\s-]/g, "_");
      return aliases.some((a) => lower.includes(a));
    });
  }

  // Check centersObj
  for (const alias of aliases) {
    const v = centersObj[alias];
    if (v === true || v === "defined") return true;
    if (v && typeof v === "object" && (v as Record<string, unknown>).defined === true) return true;
  }
  return false;
}

/* ─── SVG shape renderers ─── */

function shapePoints(s: CenterGeo): string {
  const { cx, cy, w, h, shape } = s;
  const hw = w / 2;
  const hh = h / 2;
  switch (shape) {
    case "diamond":
      return `${cx},${cy - hh} ${cx + hw},${cy} ${cx},${cy + hh} ${cx - hw},${cy}`;
    case "tri-down":
      return `${cx - hw},${cy - hh} ${cx + hw},${cy - hh} ${cx},${cy + hh}`;
    case "tri-up":
      return `${cx},${cy - hh} ${cx + hw},${cy + hh} ${cx - hw},${cy + hh}`;
    case "rect":
      // rects use <rect>, this is a fallback
      return `${cx - hw},${cy - hh} ${cx + hw},${cy - hh} ${cx + hw},${cy + hh} ${cx - hw},${cy + hh}`;
  }
}

interface BodygraphProps {
  definedSet: Set<string>;
}

function Bodygraph({ definedSet }: BodygraphProps) {
  const lookup = new Map<string, CenterGeo>();
  for (const c of BODY_CENTERS) lookup.set(c.key, c);

  const defFill = "rgba(16, 185, 129, 0.6)";       // emerald-ish
  const undefFill = "rgba(255, 255, 255, 0.05)";
  const undefStroke = "rgba(255, 255, 255, 0.2)";
  const chanDef = "rgba(16, 181, 167, 0.4)";
  const chanUndef = "rgba(255, 255, 255, 0.1)";
  const labelColor = "rgba(255, 255, 255, 0.7)";

  return (
    <div style={styles.bodygraphWrap}>
      <svg
        viewBox="0 0 200 270"
        width="200"
        style={{ maxWidth: "200px", width: "100%", height: "auto" }}
        xmlns="http://www.w3.org/2000/svg"
      >
        {/* ── channel lines ── */}
        {CHANNELS.map(([fromKey, toKey]) => {
          const a = lookup.get(fromKey);
          const b = lookup.get(toKey);
          if (!a || !b) return null;
          const bothDefined = definedSet.has(fromKey) && definedSet.has(toKey);
          return (
            <line
              key={`${fromKey}-${toKey}`}
              x1={a.cx}
              y1={a.cy}
              x2={b.cx}
              y2={b.cy}
              stroke={bothDefined ? chanDef : chanUndef}
              strokeWidth={bothDefined ? 2 : 1}
            />
          );
        })}

        {/* ── center shapes ── */}
        {BODY_CENTERS.map((c) => {
          const def = definedSet.has(c.key);
          const fill = def ? defFill : undefFill;
          const stroke = def ? "rgba(16, 185, 129, 0.8)" : undefStroke;

          return (
            <g key={c.key}>
              {c.shape === "rect" ? (
                <rect
                  x={c.cx - c.w / 2}
                  y={c.cy - c.h / 2}
                  width={c.w}
                  height={c.h}
                  rx={3}
                  fill={fill}
                  stroke={stroke}
                  strokeWidth={1.2}
                />
              ) : (
                <polygon
                  points={shapePoints(c)}
                  fill={fill}
                  stroke={stroke}
                  strokeWidth={1.2}
                />
              )}
              <text
                x={c.cx}
                y={c.cy + 1}
                textAnchor="middle"
                dominantBaseline="central"
                fill={labelColor}
                fontSize={8}
                fontWeight={600}
                style={{ pointerEvents: "none", userSelect: "none" }}
              >
                {c.label}
              </text>
            </g>
          );
        })}
      </svg>
    </div>
  );
}

/* ─── main component ─── */

interface HumanDesignProps {
  result: Record<string, unknown>;
}

export default function HumanDesign({ result }: HumanDesignProps) {
  const profile = obj(result.profile);
  const centersObj = obj(result.centers);
  const cross = obj(result.incarnation_cross);
  const sunGate = result.sun_gate ?? cross.sun;
  const earthGate = result.earth_gate ?? cross.earth;
  const definedList = Array.isArray(result.defined_centers)
    ? (result.defined_centers as string[])
    : undefined;

  // Build set of defined center keys
  const definedSet = new Set<string>();
  for (const c of BODY_CENTERS) {
    if (isCenterDefined(c.key, centersObj, definedList)) {
      definedSet.add(c.key);
    }
  }

  return (
    <div>
      {/* ── SVG Bodygraph ── */}
      <Bodygraph definedSet={definedSet} />

      {/* ── Key stats grid ── */}
      <div style={styles.grid}>
        <div style={styles.cell}>
          <span style={styles.label}>Type</span>
          <span style={styles.value}>{str(result.type)}</span>
        </div>
        <div style={styles.cell}>
          <span style={styles.label}>Strategy</span>
          <span style={styles.value}>{str(result.strategy)}</span>
        </div>
        <div style={styles.cell}>
          <span style={styles.label}>Authority</span>
          <span style={styles.value}>{str(result.authority)}</span>
        </div>
        <div style={styles.cell}>
          <span style={styles.label}>Profile</span>
          <span style={styles.value}>
            {profile.line1 && profile.line2
              ? `${str(profile.line1)}/${str(profile.line2)}`
              : str(result.profile)}
          </span>
        </div>
        <div style={styles.cell}>
          <span style={styles.label}>Definition</span>
          <span style={styles.value}>{str(result.definition)}</span>
        </div>
        <div style={styles.cell}>
          <span style={styles.label}>Not-Self Theme</span>
          <span style={styles.value}>{str(result.not_self_theme)}</span>
        </div>
      </div>

      {/* ── Incarnation Cross ── */}
      {(sunGate != null || earthGate != null) && (
        <div style={styles.section}>
          <span style={styles.sectionTitle}>Incarnation Cross Seed</span>
          <div style={styles.grid}>
            <div style={styles.cell}>
              <span style={styles.label}>Sun Gate</span>
              <span style={styles.value}>{str(sunGate)}</span>
            </div>
            <div style={styles.cell}>
              <span style={styles.label}>Earth Gate</span>
              <span style={styles.value}>{str(earthGate)}</span>
            </div>
          </div>
        </div>
      )}

      {/* ── Centers list ── */}
      <div style={styles.section}>
        <span style={styles.sectionTitle}>Centers</span>
        {CENTERS_DISPLAY.map((name) => {
          const key = name.toLowerCase().replace(/ /g, "_");
          // Map "heart" display name → "ego" key
          const lookupKey = key === "heart" ? "ego" : key;
          const defined = definedSet.has(lookupKey);
          return (
            <div key={name} style={styles.centerRow}>
              <span>{name}</span>
              <span
                style={{
                  color: defined ? "var(--emerald)" : "var(--text-dim)",
                  fontWeight: defined ? 600 : 400,
                }}
              >
                {defined ? "Defined" : "Undefined"}
              </span>
            </div>
          );
        })}
      </div>
    </div>
  );
}
