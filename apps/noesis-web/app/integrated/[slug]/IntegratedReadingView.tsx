"use client";

// ─── IntegratedReadingView — client wrapper for animated rendering ─────
// Composes: ConstellationGrid backdrop, OrbitalCover hero, VerseFlow body
// for each Part, LaArcFade between Parts.
//
// Per design MD § 4 (page composition).

import { ConstellationGrid } from "@/components/integrated/ConstellationGrid";
import { CoverScene } from "@/components/integrated/cover/CoverScene";
import { VerseFlow } from "@/components/integrated/VerseFlow";
import { LaArcFade } from "@/components/integrated/LaArcFade";
import type { IntegratedReading, PassMetric } from "@/lib/integrated/loader";

type PassWithHtml = PassMetric & { markdown: string; html: string };

interface ViewProps {
  reading: Omit<IntegratedReading, "passes"> & { passes: PassWithHtml[] };
}

function toRoman(n: number): string {
  const map: Array<[number, string]> = [
    [1000, "M"], [900, "CM"], [500, "D"], [400, "CD"], [100, "C"],
    [90, "XC"], [50, "L"], [40, "XL"], [10, "X"], [9, "IX"],
    [5, "V"], [4, "IV"], [1, "I"],
  ];
  let s = "";
  let r = n;
  for (const [v, sym] of map) {
    while (r >= v) { s += sym; r -= v; }
  }
  return s;
}

export function IntegratedReadingView({ reading }: ViewProps) {
  const coverTitle =
    reading.subjects.length >= 2 ? "COMPOSITE FIELD" : "INTEGRATED READING";
  const birthMeta = `${reading.mode.toUpperCase().replace(/-/g, " · ")}  ·  ${reading.registerBand.toUpperCase().replace("_", "-")}  ·  ${reading.totalWords.toLocaleString()} WORDS`;

  return (
    <>
      <ConstellationGrid />

      <CoverScene
        title={coverTitle}
        birthMeta={birthMeta}
        subjects={reading.subjects}
        topologySvg={reading.topologySvg}
      />

      <article
        style={{
          position: "relative",
          width: "100%",
          background: "linear-gradient(180deg, rgba(7,11,29,0.55) 0%, rgba(7,11,29,0.85) 100%)",
          backdropFilter: "blur(2px)",
          zIndex: 2,
        }}
      >
        <div
          style={{
            width: "clamp(18rem, 72vw, 80rem)",
            margin: "0 auto",
            padding: "clamp(2rem, 4vw, 5rem) clamp(1rem, 2.4vw, 2.5rem) clamp(3rem, 6vw, 6rem)",
            fontSize: "clamp(1rem, 0.85rem + 0.45vw, 1.22rem)",
            lineHeight: 1.65,
            color: "var(--text)",
          }}
        >
          {reading.passes.map((pass, i) => {
            const isLast = i === reading.passes.length - 1;
            return (
              <div key={pass.id} style={{ marginBottom: "clamp(2rem, 6vw, 6rem)" }}>
                <header
                  id={`part-${i + 1}`}
                  style={{ margin: "clamp(2rem, 4vw, 4rem) 0 clamp(1.5rem, 3vw, 3rem)" }}
                >
                  <div
                    style={{
                      fontFamily: "var(--font-mono)",
                      fontSize: "clamp(0.72rem, 0.65rem + 0.15vw, 0.85rem)",
                      letterSpacing: "0.45em",
                      textTransform: "uppercase",
                      color: "var(--c-gold)",
                      marginBottom: "0.85rem",
                    }}
                  >
                    Part {toRoman(i + 1)}
                  </div>
                  <h1
                    style={{
                      fontFamily: "var(--font-display)",
                      fontWeight: 800,
                      fontSize: "clamp(2rem, 1.4rem + 2.4vw, 4.5rem)",
                      lineHeight: 1.02,
                      letterSpacing: "-0.022em",
                      color: "var(--c-parchment)",
                      margin: 0,
                    }}
                  >
                    {pass.title}
                  </h1>
                  <div
                    style={{
                      marginTop: "0.85rem",
                      fontFamily: "var(--font-mono)",
                      fontSize: "0.85rem",
                      color: "var(--c-emerald)",
                      letterSpacing: "0.12em",
                    }}
                  >
                    {pass.words.toLocaleString()} words · {pass.xrefs} cross-references
                  </div>
                </header>

                <VerseFlow html={pass.html} />

                {!isLast && <LaArcFade />}
              </div>
            );
          })}

          <LaArcFade />

          <footer
            style={{
              marginTop: "clamp(3rem, 6vw, 6rem)",
              paddingTop: "clamp(1.5rem, 3vw, 3rem)",
              borderTop: "1px solid var(--line-faint)",
              fontFamily: "var(--font-mono)",
              fontSize: "0.85rem",
              color: "var(--muted)",
              textAlign: "center" as const,
              letterSpacing: "0.12em",
            }}
          >
            <div
              style={{
                fontFamily: "var(--font-display)",
                fontStyle: "italic",
                fontWeight: 500,
                fontSize: "1.05rem",
                color: "var(--c-gold)",
                marginBottom: "0.75rem",
              }}
            >
              The Anatomist Who Sees Fractals
            </div>
            <div>TRYAMBAKAM NOESIS · 1331.TRYAMBAKAM.SPACE</div>
            <div style={{ marginTop: "1rem", maxWidth: "48ch", margin: "1rem auto 0", fontStyle: "italic", color: "var(--muted)" }}>
              This document is documentation of an instrument. The instrument is what
              you already are. The Quine principle: the system succeeds when you no
              longer need it.
            </div>
          </footer>
        </div>
      </article>
    </>
  );
}
