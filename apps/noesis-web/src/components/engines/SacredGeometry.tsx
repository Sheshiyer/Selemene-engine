import { type ReactElement } from "react";
import GenericEngineView from "./GenericEngineView";

interface SacredGeometryProps {
  result: Record<string, unknown>;
}

const STROKE = "rgba(16,181,167,0.7)";
const STROKE_DIM = "rgba(16,181,167,0.35)";
const SW = 0.8;
const CX = 100;
const CY = 100;

/* ── Pattern Detection ────────────────────────────────────── */

type FormKey =
  | "flower_of_life"
  | "metatrons_cube"
  | "sri_yantra"
  | "fibonacci"
  | "seed_of_life"
  | "vesica_piscis"
  | "merkaba"
  | "torus"
  | "default";

function detectForm(name: string): FormKey {
  const n = name.toLowerCase();
  if (n.includes("metatron")) return "metatrons_cube";
  if (n.includes("flower of life") || n.includes("flower_of_life")) return "flower_of_life";
  if (n.includes("sri yantra") || n.includes("shri yantra")) return "sri_yantra";
  if (n.includes("fibonacci") || n.includes("golden spiral")) return "fibonacci";
  if (n.includes("seed of life") || n.includes("seed_of_life")) return "seed_of_life";
  if (n.includes("vesica piscis") || n.includes("vesica_piscis")) return "vesica_piscis";
  if (n.includes("merkaba") || n.includes("star tetrahedron")) return "merkaba";
  if (n.includes("torus")) return "torus";
  return "default";
}

/* ── Geometry Helpers ─────────────────────────────────────── */

function circleAt(cx: number, cy: number, r: number, key: string, stroke = STROKE): ReactElement {
  return <circle key={key} cx={cx} cy={cy} r={r} fill="none" stroke={stroke} strokeWidth={SW} />;
}

function hexRing(cx: number, cy: number, dist: number, r: number, prefix: string): ReactElement[] {
  const els: ReactElement[] = [];
  for (let i = 0; i < 6; i++) {
    const angle = (i * 60) * (Math.PI / 180);
    const px = cx + dist * Math.cos(angle);
    const py = cy + dist * Math.sin(angle);
    els.push(circleAt(px, py, r, `${prefix}-${i}`));
  }
  return els;
}

function hexCenters(cx: number, cy: number, dist: number): Array<[number, number]> {
  const pts: Array<[number, number]> = [[cx, cy]];
  for (let i = 0; i < 6; i++) {
    const angle = (i * 60) * (Math.PI / 180);
    pts.push([cx + dist * Math.cos(angle), cy + dist * Math.sin(angle)]);
  }
  return pts;
}

/* ── Pattern Renderers ────────────────────────────────────── */

function renderFlowerOfLife(): ReactElement[] {
  const R = 28;
  return [
    circleAt(CX, CY, R, "fol-c"),
    ...hexRing(CX, CY, R, R, "fol"),
  ];
}

function renderMetatronsCube(): ReactElement[] {
  const R = 28;
  const centers = hexCenters(CX, CY, R);
  // Flower of life base
  const circles = [
    circleAt(CX, CY, R, "mc-c"),
    ...hexRing(CX, CY, R, R, "mc"),
  ];
  // Lines connecting all center points
  const lines: ReactElement[] = [];
  let idx = 0;
  for (let i = 0; i < centers.length; i++) {
    for (let j = i + 1; j < centers.length; j++) {
      lines.push(
        <line
          key={`mc-l-${idx++}`}
          x1={centers[i][0]}
          y1={centers[i][1]}
          x2={centers[j][0]}
          y2={centers[j][1]}
          stroke={STROKE_DIM}
          strokeWidth={0.5}
        />
      );
    }
  }
  return [...lines, ...circles];
}

function renderSriYantra(): ReactElement[] {
  const els: ReactElement[] = [];
  // Outer concentric rings
  els.push(circleAt(CX, CY, 85, "sy-ring-o", STROKE_DIM));
  els.push(circleAt(CX, CY, 78, "sy-ring-i", STROKE_DIM));
  // Upward-pointing triangles (3 nested)
  const upTriSizes = [62, 44, 26];
  upTriSizes.forEach((sz, i) => {
    const top = CY - sz * 0.85;
    const bot = CY + sz * 0.5;
    const halfW = sz * 0.7;
    els.push(
      <polygon
        key={`sy-up-${i}`}
        points={`${CX},${top} ${CX - halfW},${bot} ${CX + halfW},${bot}`}
        fill="none"
        stroke={STROKE}
        strokeWidth={SW}
      />
    );
  });
  // Downward-pointing triangles (3 nested)
  const downTriSizes = [56, 38, 20];
  downTriSizes.forEach((sz, i) => {
    const bot = CY + sz * 0.85;
    const top = CY - sz * 0.5;
    const halfW = sz * 0.7;
    els.push(
      <polygon
        key={`sy-dn-${i}`}
        points={`${CX},${bot} ${CX - halfW},${top} ${CX + halfW},${top}`}
        fill="none"
        stroke={STROKE}
        strokeWidth={SW}
      />
    );
  });
  // Center bindu
  els.push(circleAt(CX, CY, 2, "sy-bindu"));
  return els;
}

function renderFibonacci(): ReactElement[] {
  const els: ReactElement[] = [];
  // Golden rectangle spiral – approximate with quarter-circle arcs
  // Fibonacci sequence segments scaled to fit viewBox
  const fibs = [3, 5, 8, 13, 21, 34, 55];
  const scale = 1.4;
  let x = CX;
  let y = CY;
  fibs.forEach((f, i) => {
    const r = f * scale;
    // Rotate the sweep direction each step
    const startAngle = (i * 90) * (Math.PI / 180);
    const endAngle = ((i + 1) * 90) * (Math.PI / 180);
    const sx = x + r * Math.cos(startAngle);
    const sy = y - r * Math.sin(startAngle);
    const ex = x + r * Math.cos(endAngle);
    const ey = y - r * Math.sin(endAngle);
    els.push(
      <path
        key={`fib-${i}`}
        d={`M ${sx} ${sy} A ${r} ${r} 0 0 0 ${ex} ${ey}`}
        fill="none"
        stroke={STROKE}
        strokeWidth={SW}
      />
    );
    // Move pivot for next arc
    x += (r * 0.3) * Math.cos(endAngle + Math.PI / 2);
    y -= (r * 0.3) * Math.sin(endAngle + Math.PI / 2);
  });
  // Golden rectangle guide
  els.push(
    <rect
      key="fib-rect"
      x={CX - 70}
      y={CY - 55}
      width={140}
      height={110}
      rx={2}
      fill="none"
      stroke={STROKE_DIM}
      strokeWidth={0.4}
    />
  );
  return els;
}

function renderSeedOfLife(): ReactElement[] {
  const R = 24;
  return [
    circleAt(CX, CY, R, "sol-c"),
    ...hexRing(CX, CY, R, R, "sol"),
  ];
}

function renderVesicaPiscis(): ReactElement[] {
  const R = 40;
  const offset = R * 0.5;
  return [
    circleAt(CX - offset, CY, R, "vp-l"),
    circleAt(CX + offset, CY, R, "vp-r"),
    // Highlight intersection – a vertical lens shape via path
    <path
      key="vp-lens"
      d={`M ${CX} ${CY - 34.6} A ${R} ${R} 0 0 1 ${CX} ${CY + 34.6} A ${R} ${R} 0 0 1 ${CX} ${CY - 34.6}`}
      fill="rgba(16,181,167,0.06)"
      stroke={STROKE}
      strokeWidth={0.5}
    />,
  ];
}

function renderMerkaba(): ReactElement[] {
  const R = 60;
  // Upward triangle
  const upPts = [0, 1, 2].map((i) => {
    const angle = (i * 120 - 90) * (Math.PI / 180);
    return `${CX + R * Math.cos(angle)},${CY + R * Math.sin(angle)}`;
  }).join(" ");
  // Downward triangle
  const downPts = [0, 1, 2].map((i) => {
    const angle = (i * 120 + 90) * (Math.PI / 180);
    return `${CX + R * Math.cos(angle)},${CY + R * Math.sin(angle)}`;
  }).join(" ");
  return [
    <polygon key="mk-up" points={upPts} fill="none" stroke={STROKE} strokeWidth={SW} />,
    <polygon key="mk-dn" points={downPts} fill="none" stroke={STROKE} strokeWidth={SW} />,
    circleAt(CX, CY, R + 6, "mk-ring", STROKE_DIM),
  ];
}

function renderTorus(): ReactElement[] {
  const els: ReactElement[] = [];
  // Top-view torus: concentric ellipses with varying tilt
  const steps = 12;
  for (let i = 0; i < steps; i++) {
    const angle = (i * 180 / steps) * (Math.PI / 180);
    const rx = 55;
    const ry = 55 * Math.abs(Math.cos(angle));
    const opacity = 0.3 + 0.5 * (i / steps);
    els.push(
      <ellipse
        key={`tor-${i}`}
        cx={CX}
        cy={CY}
        rx={rx}
        ry={Math.max(ry, 4)}
        fill="none"
        stroke={STROKE}
        strokeWidth={0.5}
        opacity={opacity}
        transform={`rotate(${(i * 180) / steps}, ${CX}, ${CY})`}
      />
    );
  }
  // Outer ring
  els.push(circleAt(CX, CY, 58, "tor-outer", STROKE_DIM));
  // Inner hole
  els.push(circleAt(CX, CY, 12, "tor-inner", STROKE));
  return els;
}

function renderDefaultMandala(): ReactElement[] {
  const els: ReactElement[] = [];
  // Concentric circles
  [20, 35, 50, 65].forEach((r, i) => {
    els.push(circleAt(CX, CY, r, `mand-c-${i}`, i % 2 === 0 ? STROKE : STROKE_DIM));
  });
  // 8 radial lines
  for (let i = 0; i < 8; i++) {
    const angle = (i * 45) * (Math.PI / 180);
    els.push(
      <line
        key={`mand-l-${i}`}
        x1={CX + 18 * Math.cos(angle)}
        y1={CY + 18 * Math.sin(angle)}
        x2={CX + 65 * Math.cos(angle)}
        y2={CY + 65 * Math.sin(angle)}
        stroke={STROKE_DIM}
        strokeWidth={0.5}
      />
    );
  }
  return els;
}

const RENDERERS: Record<FormKey, () => ReactElement[]> = {
  flower_of_life: renderFlowerOfLife,
  metatrons_cube: renderMetatronsCube,
  sri_yantra: renderSriYantra,
  fibonacci: renderFibonacci,
  seed_of_life: renderSeedOfLife,
  vesica_piscis: renderVesicaPiscis,
  merkaba: renderMerkaba,
  torus: renderTorus,
  default: renderDefaultMandala,
};

/* ── SVG Wrapper ──────────────────────────────────────────── */

function GeometryPattern({ formName, svgPreview }: { formName: string; svgPreview?: string }): ReactElement {
  // If server returned an svg_preview string, use it directly
  if (svgPreview) {
    return (
      <div
        style={{
          display: "flex",
          justifyContent: "center",
          padding: "0.5rem 0",
        }}
        dangerouslySetInnerHTML={{ __html: svgPreview }}
      />
    );
  }

  const key = detectForm(formName);
  const elements = RENDERERS[key]();

  return (
    <div style={{ display: "flex", justifyContent: "center", padding: "0.5rem 0" }}>
      <svg
        viewBox="0 0 200 200"
        xmlns="http://www.w3.org/2000/svg"
        style={{
          maxWidth: 200,
          width: "100%",
          background: "rgba(7,11,29,0.4)",
          borderRadius: "var(--radius, 8px)",
        }}
      >
        {elements}
      </svg>
    </div>
  );
}

/* ── Styles ───────────────────────────────────────────────── */

const s = {
  grid: { display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(200px, 1fr))", gap: "0.75rem" },
  cell: { background: "var(--field)", borderRadius: "var(--radius)", padding: "0.75rem", display: "flex", flexDirection: "column" as const, gap: "0.25rem" },
  label: { fontSize: "0.7rem", color: "var(--text-muted)", textTransform: "uppercase" as const, letterSpacing: "0.06em", fontWeight: 600 },
  value: { fontSize: "1rem", fontWeight: 600, color: "var(--gold)" },
  sub: { fontSize: "0.8rem", color: "var(--text-muted)" },
  desc: { fontSize: "0.875rem", color: "var(--text)", lineHeight: 1.65, padding: "0.75rem", background: "var(--field)", borderRadius: "var(--radius)" },
  meditation: { padding: "0.75rem 1rem", background: "var(--gold-soft)", border: "1px solid var(--line-gold)", borderRadius: "var(--radius)", fontSize: "0.875rem", color: "var(--text)", lineHeight: 1.65, fontStyle: "italic" },
};

function str(v: unknown): string { return v == null ? "—" : String(v); }

/* ── Component ────────────────────────────────────────────── */

export default function SacredGeometry({ result }: SacredGeometryProps) {
  const form = result.form as Record<string, unknown> | undefined;
  const meditation = result.meditation_guidance as string | undefined;

  if (!form && !result.form_id && !result.primary_form) return <GenericEngineView result={result} />;

  const name = form?.name ?? result.form_name ?? result.primary_form;
  const category = form?.category ?? result.category;
  const elements = (form?.elements ?? result.elements) as string[] | undefined;
  const symbolism = form?.symbolism ?? result.symbolism;
  const ratio = (form?.golden_ratio_present ?? result.golden_ratio) as boolean | undefined;
  const description = form?.description ?? result.description;
  const intention = result.intention as string | undefined;
  const svgPreview = result.svg_preview as string | undefined;

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "1rem" }}>
      {/* SVG Pattern — rendered from form name or server svg_preview */}
      {name != null && (
        <GeometryPattern formName={str(name)} svgPreview={svgPreview} />
      )}

      <div style={s.grid}>
        <div style={s.cell}>
          <span style={s.label}>Form</span>
          <span style={s.value}>{str(name)}</span>
          {category != null && <span style={s.sub}>{str(category)}</span>}
        </div>
        {symbolism != null && (
          <div style={s.cell}>
            <span style={s.label}>Symbolism</span>
            <span style={s.value}>{str(symbolism)}</span>
          </div>
        )}
        {ratio != null && (
          <div style={s.cell}>
            <span style={s.label}>Golden Ratio</span>
            <span style={s.value}>{ratio ? "Present ◈" : "Absent"}</span>
          </div>
        )}
        {elements != null && elements.length > 0 && (
          <div style={s.cell}>
            <span style={s.label}>Elements</span>
            <span style={{ fontSize: "0.85rem", color: "var(--text)" }}>{elements.join(", ")}</span>
          </div>
        )}
      </div>

      {intention != null && (
        <div style={{ fontSize: "0.8rem", color: "var(--text-muted)", fontStyle: "italic" }}>
          Held intention: &ldquo;{intention}&rdquo;
        </div>
      )}

      {description != null && <div style={s.desc}>{str(description)}</div>}

      {meditation != null && (
        <div style={s.meditation}>
          <div style={{ fontSize: "0.7rem", color: "var(--gold)", fontWeight: 700, marginBottom: "0.5rem", textTransform: "uppercase" as const, letterSpacing: "0.06em", fontStyle: "normal" }}>Meditation Guidance</div>
          {meditation}
        </div>
      )}
    </div>
  );
}
