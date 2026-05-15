"use client";

// ─── SubjectCompendium — opening bento spread, one card per native ──────
// Replaces the front-of-reading "who are these subjects" data dump.
// Each subject becomes a BentoCard with:
//   eyebrow: "Native A" / "Native B" / ...
//   title: subject name (massive Panchang display)
//   description: 1-line summary (birth meta)
//   featured: vertical stack of BentoChips (Lagna / AK / Nakshatra / ...)
//
// Layout: 3-up on desktop, 2-up on tablet, stacked on mobile.

import { BentoCard } from "./BentoCard";
import { BentoChip } from "./BentoChip";
import { BentoGrid } from "./BentoGrid";

interface SubjectSummary {
  name: string;
  birth_date?: string;
  birth_place?: string;
  lagna?: string;
  atmakaraka?: string;
  birth_nakshatra?: string;
  current_dasha?: string;
}

interface SubjectCompendiumProps {
  subjects: SubjectSummary[];
  title?: string;
  eyebrow?: string;
}

const NATIVE_LETTER = ["A", "B", "C", "D", "E", "F"];

export function SubjectCompendium({
  subjects,
  title = "The Native Field",
  eyebrow = "Compendium",
}: SubjectCompendiumProps) {
  return (
    <section
      style={{
        position: "relative",
        width: "100%",
        zIndex: 3,
        padding: "clamp(2rem, 4vw, 4rem) 0 clamp(1.5rem, 3vw, 3rem)",
      }}
    >
      <div
        style={{
          width: "min(96rem, 96vw)",
          margin: "0 auto",
          padding: "0 clamp(0.75rem, 1.5vw, 1.5rem)",
          marginBottom: "clamp(1.5rem, 3vw, 2.5rem)",
        }}
      >
        <div
          style={{
            fontFamily: "var(--font-mono)",
            fontSize: "clamp(0.7rem, 0.65rem + 0.12vw, 0.82rem)",
            letterSpacing: "0.4em",
            textTransform: "uppercase",
            color: "var(--c-gold, #C5A017)",
            marginBottom: "clamp(0.5rem, 1vw, 1rem)",
          }}
        >
          {eyebrow}
        </div>
        <h2
          style={{
            fontFamily: "var(--font-display)",
            fontWeight: 800,
            fontSize: "clamp(2rem, 1.5rem + 2.4vw, 4.5rem)",
            lineHeight: 1.0,
            letterSpacing: "-0.035em",
            margin: 0,
            color: "var(--c-parchment, #F0EDE3)",
          }}
        >
          {title}
        </h2>
      </div>

      <BentoGrid>
        {subjects.map((s, i) => (
          <BentoCard
            key={s.name}
            span={1}
            eyebrow={`Native ${NATIVE_LETTER[i] ?? i + 1}`}
            title={s.name}
            description={
              [s.birth_date, s.birth_place].filter(Boolean).join(" · ") || undefined
            }
            status="LIVE"
            tone={(["violet", "indigo", "emerald"] as const)[i % 3]}
            hasFeature
          >
            <div
              style={{
                display: "flex",
                flexWrap: "wrap",
                gap: "clamp(0.5rem, 1vw, 0.85rem)",
              }}
            >
              {s.lagna && (
                <BentoChip label="Lagna" variant="gold">
                  {s.lagna}
                </BentoChip>
              )}
              {s.atmakaraka && (
                <BentoChip label="Atmakaraka" variant="emerald">
                  {s.atmakaraka}
                </BentoChip>
              )}
              {s.birth_nakshatra && (
                <BentoChip label="Nakshatra" variant="indigo">
                  {s.birth_nakshatra}
                </BentoChip>
              )}
              {s.current_dasha && (
                <BentoChip label="Mahadasha" variant="violet">
                  {s.current_dasha}
                </BentoChip>
              )}
            </div>
          </BentoCard>
        ))}
      </BentoGrid>
    </section>
  );
}
