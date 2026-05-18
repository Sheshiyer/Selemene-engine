// ─── DashaWaveform — iridescent ribbon for time-period tables ──────────
// Replaces dasha tables (and any time-keyed comparison) with a
// horizontal SVG waveform that flows through a mandala-ring:
//
//   ═══════════════════════════════════════════════════════
//      Rahu MD ────────╮            ╭─── Jupiter MD ───
//                       ╲          ╱
//      ◉ ◉ ◉ ◉ ◉ ◉ ◉ ◉ ◉◉────●────◉◉ ◉ ◉ ◉ ◉ ◉ ◉ ◉ ◉ ◉
//          Witness Violet  Sacred Gold  Coherence Emerald
//   ═══════════════════════════════════════════════════════
//      2008          2026 PIVOT          2042
//
// Color along the path:
//   Past   = Witness Violet (#2D0050)
//   Pivot  = Sacred Gold     (#C5A017)
//   Future = Coherence Emerald (#10B5A7)
//
// Each row in the source table becomes a marker on the ribbon. We try
// to extract a year/range from each row; if absent, markers space
// evenly. Row labels float above the markers.

import { renderInline } from "../parseBlocks";

interface DashaWaveformProps {
  headers: string[];
  rows: string[][];
  accentColor: string;
}

interface MarkerData {
  label: string;       // top label, e.g. "Rahu MD"
  subLabel?: string;   // smaller label, e.g. "Mahadasha"
  year: number | null; // extracted year for placement
  effectText?: string; // description below
}

/** Try to find a 4-digit year (19xx or 20xx) in any cell of the row. */
function extractYear(row: string[]): number | null {
  const flat = row.join(" ");
  const m = flat.match(/\b((?:19|20)\d{2})\b/);
  if (!m) return null;
  return parseInt(m[1], 10);
}

function withAlpha(hex: string, alpha: number): string {
  const a = Math.round(Math.max(0, Math.min(1, alpha)) * 255)
    .toString(16)
    .padStart(2, "0");
  return `${hex}${a}`;
}

export function DashaWaveform({ headers, rows, accentColor }: DashaWaveformProps) {
  if (rows.length === 0) return null;

  // Build marker data: first col = label, find a year, rest = description
  const markers: MarkerData[] = rows.map((row) => ({
    label: row[0] ?? "",
    subLabel: row[1] ?? undefined,
    year: extractYear(row),
    effectText: row.slice(2).join(" — ") || row[row.length - 1] || undefined,
  }));

  // Determine year span for x-position mapping
  const years = markers.map((m) => m.year).filter((y): y is number => y !== null);
  const yearMin = years.length > 0 ? Math.min(...years) : 0;
  const yearMax = years.length > 0 ? Math.max(...years) : 0;
  const yearSpan = Math.max(1, yearMax - yearMin);

  // Compute marker X positions in viewport units (0..1)
  const positions = markers.map((m, i) => {
    if (m.year === null) return (i + 0.5) / markers.length;
    return (m.year - yearMin) / yearSpan;
  });

  // SVG viewport
  const W = 1000;
  const H = 200;
  const waveAmp = 28;
  const centerY = H / 2;

  // Build the SVG path — gentle sine through all marker x positions
  const pathPoints = positions.map((p, i) => {
    const x = 40 + p * (W - 80);
    const sign = i % 2 === 0 ? -1 : 1;
    const y = centerY + Math.sin(p * Math.PI * 1.6) * waveAmp * sign;
    return { x, y };
  });

  // Catmull-Rom-ish smooth path
  const pathD = pathPoints
    .map((pt, i) => {
      if (i === 0) return `M ${pt.x} ${pt.y}`;
      const prev = pathPoints[i - 1];
      const cx1 = prev.x + (pt.x - prev.x) * 0.55;
      const cx2 = prev.x + (pt.x - prev.x) * 0.45;
      return `C ${cx1} ${prev.y}, ${cx2} ${pt.y}, ${pt.x} ${pt.y}`;
    })
    .join(" ");

  return (
    <section
      style={{ margin: "clamp(2.5rem, 5vh, 4rem) 0" }}
      aria-label="Dasha waveform — time-period flow"
    >
      {/* Eyebrow */}
      <div
        style={{
          fontFamily: "var(--font-mono, 'SF Mono', monospace)",
          fontSize: "clamp(0.6rem, 0.75vw, 0.72rem)",
          letterSpacing: "0.4em",
          textTransform: "uppercase",
          color: accentColor,
          opacity: 0.82,
          textAlign: "center",
          marginBottom: "clamp(1rem, 2vh, 1.5rem)",
        }}
      >
        {headers[0] ?? "DASHA"} · {headers.slice(1).join(" / ")}
      </div>

      <div style={{ position: "relative", width: "100%" }}>
        <svg
          viewBox={`0 0 ${W} ${H}`}
          width="100%"
          height="auto"
          preserveAspectRatio="xMidYMid meet"
          aria-hidden="true"
          style={{ display: "block" }}
        >
          <defs>
            <linearGradient id="dasha-stroke" x1="0" y1="0" x2="1" y2="0">
              <stop offset="0%"   stopColor="#2D0050" />
              <stop offset="48%"  stopColor="#C5A017" />
              <stop offset="52%"  stopColor="#C5A017" />
              <stop offset="100%" stopColor="#10B5A7" />
            </linearGradient>
            <linearGradient id="dasha-glow" x1="0" y1="0" x2="1" y2="0">
              <stop offset="0%"   stopColor="#2D0050" stopOpacity="0.35" />
              <stop offset="50%"  stopColor="#C5A017" stopOpacity="0.55" />
              <stop offset="100%" stopColor="#10B5A7" stopOpacity="0.35" />
            </linearGradient>
            {/* Mandala-ring background */}
            <radialGradient id="dasha-ring" cx="50%" cy="50%" r="50%">
              <stop offset="0%"  stopColor={withAlpha(accentColor, 0.0)} />
              <stop offset="45%" stopColor={withAlpha(accentColor, 0.18)} />
              <stop offset="55%" stopColor={withAlpha(accentColor, 0.18)} />
              <stop offset="100%" stopColor={withAlpha(accentColor, 0.0)} />
            </radialGradient>
          </defs>

          {/* Horizontal mandala ring (the carrier) */}
          <ellipse
            cx={W / 2}
            cy={centerY}
            rx={W * 0.46}
            ry={32}
            fill="none"
            stroke="url(#dasha-ring)"
            strokeWidth="14"
          />
          <line
            x1="40"
            y1={centerY}
            x2={W - 40}
            y2={centerY}
            stroke={withAlpha(accentColor, 0.22)}
            strokeWidth="1"
            strokeDasharray="2 6"
          />

          {/* Glow under-pass */}
          <path
            d={pathD}
            fill="none"
            stroke="url(#dasha-glow)"
            strokeWidth="12"
            strokeLinecap="round"
            opacity="0.7"
            filter="blur(0.7px)"
          />

          {/* Main ribbon */}
          <path
            d={pathD}
            fill="none"
            stroke="url(#dasha-stroke)"
            strokeWidth="2.6"
            strokeLinecap="round"
            strokeLinejoin="round"
          />

          {/* Markers — colored dots along the path */}
          {pathPoints.map((pt, i) => {
            const tProgress = positions[i];
            const dotColor =
              tProgress < 0.45
                ? "#2D0050"
                : tProgress > 0.55
                ? "#10B5A7"
                : "#C5A017";
            return (
              <g key={i}>
                <circle
                  cx={pt.x}
                  cy={pt.y}
                  r="6"
                  fill={dotColor}
                  stroke="#F0EDE3"
                  strokeWidth="1.25"
                />
                {markers[i].year !== null && (
                  <text
                    x={pt.x}
                    y={centerY + 60}
                    textAnchor="middle"
                    fontFamily="var(--font-mono, monospace)"
                    fontSize="11"
                    fill={withAlpha(accentColor, 0.85)}
                    letterSpacing="2"
                  >
                    {markers[i].year}
                  </text>
                )}
              </g>
            );
          })}
        </svg>

        {/* Marker labels — absolutely positioned above each dot via overlay */}
        <div
          style={{
            position: "absolute",
            inset: 0,
            pointerEvents: "none",
          }}
        >
          {pathPoints.map((pt, i) => (
            <div
              key={i}
              style={{
                position: "absolute",
                left: `${(pt.x / W) * 100}%`,
                top: `${((pt.y - 38) / H) * 100}%`,
                transform: "translate(-50%, -100%)",
                fontFamily: "var(--font-display, 'Panchang', serif)",
                fontVariationSettings: "'wght' 620",
                fontSize: "clamp(0.78rem, 0.95vw, 0.95rem)",
                color: "var(--c-parchment, #F0EDE3)",
                whiteSpace: "nowrap",
                textShadow: "0 1px 8px rgba(0,0,0,0.65)",
              }}
              dangerouslySetInnerHTML={{ __html: renderInline(markers[i].label) }}
            />
          ))}
        </div>
      </div>

      {/* Below-ribbon detail strip — effect text per marker for context */}
      {markers.some((m) => m.effectText) && (
        <div
          style={{
            marginTop: "clamp(1rem, 2vh, 1.5rem)",
            display: "grid",
            gridTemplateColumns: `repeat(${markers.length}, 1fr)`,
            gap: "clamp(0.5rem, 1vw, 1rem)",
            fontFamily: "var(--font-body, 'Satoshi', sans-serif)",
            fontSize: "clamp(0.7rem, 0.82vw, 0.82rem)",
            lineHeight: 1.45,
            color: "rgba(240, 237, 227, 0.7)",
          }}
        >
          {markers.map((m, i) => (
            <div
              key={i}
              style={{ textAlign: "center", padding: "0 0.25rem" }}
              dangerouslySetInnerHTML={{ __html: renderInline(m.effectText ?? "") }}
            />
          ))}
        </div>
      )}
    </section>
  );
}
