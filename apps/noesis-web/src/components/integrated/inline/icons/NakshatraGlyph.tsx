// ─── NakshatraGlyph — constellation dots for the 27 lunar asterisms ────
// Each nakshatra rendered as 3-4 small dots arranged in an approximate
// star-pattern, with thin connecting lines (Sacred Gold).
//
// Patterns are stylized — they don't aim for astronomical accuracy, just
// give each asterism a distinct micro-shape.

import type { CSSProperties } from "react";

export type NakshatraName =
  | "ashwini"
  | "bharani"
  | "krittika"
  | "rohini"
  | "mrigashira"
  | "ardra"
  | "punarvasu"
  | "pushya"
  | "ashlesha"
  | "magha"
  | "purvaphalguni"
  | "uttaraphalguni"
  | "hasta"
  | "chitra"
  | "swati"
  | "vishakha"
  | "anuradha"
  | "jyeshtha"
  | "mool"
  | "purvashadha"
  | "uttarashadha"
  | "shravana"
  | "dhanishta"
  | "shatabhisha"
  | "purvabhadrapada"
  | "uttarabhadrapada"
  | "revati";

interface NakshatraGlyphProps {
  nakshatra: NakshatraName;
  size?: number;
  title?: string;
}

const baseStyle: CSSProperties = {
  display: "inline-block",
  verticalAlign: "middle",
  margin: "0 0.18em 0 0",
  flexShrink: 0,
};

type Dot = [number, number];

// Each nakshatra gets a small constellation pattern within 0..24 viewBox.
// Patterns are stylized; star count is approximate.
const PATTERNS: Record<NakshatraName, Dot[]> = {
  ashwini: [[6, 12], [12, 9], [18, 12]], // horse-head triad
  bharani: [[6, 6], [18, 6], [12, 18]], // yoni triangle
  krittika: [[5, 8], [10, 6], [14, 9], [19, 7], [12, 17]], // razor cluster
  rohini: [[5, 14], [10, 8], [15, 7], [19, 13], [12, 19]], // wheel
  mrigashira: [[7, 6], [12, 9], [17, 6], [12, 17]], // deer-head
  ardra: [[12, 6], [6, 14], [18, 14], [12, 19]], // teardrop
  punarvasu: [[7, 8], [17, 8], [7, 16], [17, 16]], // bow
  pushya: [[8, 8], [16, 8], [12, 14], [12, 19]], // udder triangle
  ashlesha: [[6, 8], [12, 12], [18, 8], [12, 18]], // serpent
  magha: [[5, 6], [11, 8], [19, 6], [9, 16], [16, 16]], // throne
  purvaphalguni: [[7, 7], [17, 7], [7, 17], [17, 17]], // hammock front
  uttaraphalguni: [[8, 7], [16, 7], [8, 17], [16, 17], [12, 12]], // hammock back
  hasta: [[6, 8], [10, 6], [14, 6], [18, 8], [12, 18]], // 5-finger hand
  chitra: [[12, 6], [6, 12], [18, 12], [12, 18]], // pearl diamond
  swati: [[12, 6], [12, 18]], // single sword
  vishakha: [[7, 7], [17, 7], [12, 17]], // archway
  anuradha: [[7, 9], [12, 7], [17, 9], [12, 17]], // lotus
  jyeshtha: [[6, 8], [12, 12], [18, 8]], // earring trio
  mool: [[5, 7], [9, 11], [12, 15], [15, 11], [19, 7]], // root tangle
  purvashadha: [[8, 8], [16, 8], [12, 17]], // elephant tusk
  uttarashadha: [[6, 9], [12, 7], [18, 9], [12, 16]], // planks
  shravana: [[7, 8], [12, 12], [17, 8]], // 3-step
  dhanishta: [[6, 7], [18, 7], [6, 17], [18, 17]], // drum
  shatabhisha: [[6, 6], [12, 6], [18, 6], [9, 12], [15, 12], [6, 18], [12, 18], [18, 18]], // 100-stars dense
  purvabhadrapada: [[7, 7], [17, 7], [7, 17]], // funeral cot front
  uttarabhadrapada: [[7, 7], [17, 7], [17, 17]], // funeral cot back
  revati: [[7, 9], [12, 7], [17, 9], [12, 13], [12, 18]], // fish
};

export function NakshatraGlyph({
  nakshatra,
  size = 20,
  title,
}: NakshatraGlyphProps) {
  const pattern = PATTERNS[nakshatra];
  const stroke = "var(--c-gold, #C5A017)";
  const label = title ?? nakshatra;
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      role="img"
      aria-label={label}
      style={baseStyle}
    >
      {title ? <title>{title}</title> : null}
      {/* Thin connecting strokes from each dot to the next, faded */}
      <g opacity={0.5} stroke={stroke} strokeWidth={0.6} fill="none">
        {pattern.slice(1).map(([x, y], i) => {
          const [px, py] = pattern[i];
          return <line key={i} x1={px} y1={py} x2={x} y2={y} />;
        })}
      </g>
      <g fill={stroke}>
        {pattern.map(([x, y], i) => (
          <circle key={i} cx={x} cy={y} r={1.2} />
        ))}
      </g>
    </svg>
  );
}
