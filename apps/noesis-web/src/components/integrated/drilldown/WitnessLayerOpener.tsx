"use client";

// ─── WitnessLayerOpener — Chapter 0 magazine-style synthesis intro ──────
// Per design v2 § 5.10 supporting cast. Renders BEFORE Part I as a
// pre-reading "what this is about" spread:
//
//   ─── WITNESS LAYER ─────────────────────────────────
//   What this reading is about
//
//   <summary>
//
//   ┃ <threshold question>
//
//                              ↓  begin
//
// Data is mocked at the witness-agents layer for now; an explicit
// witnessLayer prop can be passed when real data is wired through.

import { motion, useReducedMotion } from "motion/react";
import { DeferredSacredScene } from "../sacred-scene/DeferredSacredScene";

export interface WitnessLayerData {
  title?: string;
  summary?: string;
  question?: string;
  convergences?: string[];
  frictions?: string[];
  practice?: string;
}

interface WitnessLayerOpenerProps {
  /** Optional explicit data. When absent, an evocative default is shown so
   *  the chapter-0 spread always renders. */
  witnessLayer?: WitnessLayerData;
}

const DEFAULT_DATA: Required<
  Pick<WitnessLayerData, "title" | "summary" | "question">
> = {
  title: "What this reading is about",
  summary:
    "This is a synthesis across charts, mandalas, dasha pivots, and resonant fields. " +
    "Read it slowly — every section is a doorway. The instrument is what you already are; " +
    "what follows is a way of seeing it.",
  question:
    "What in you is being asked to stand still long enough to be witnessed?",
};

const s = {
  wrap: {
    position: "relative" as const,
    zIndex: 2,
    width: "100%",
    overflow: "hidden" as const,
  },
  sceneBackdrop: {
    position: "absolute" as const,
    inset: 0,
    zIndex: 0,
    pointerEvents: "none" as const,
  },
  sceneVeil: {
    position: "absolute" as const,
    inset: 0,
    zIndex: 1,
    pointerEvents: "none" as const,
    background:
      "linear-gradient(180deg, rgba(7,11,29,0.55) 0%, rgba(7,11,29,0.82) 100%)",
  },
  inner: {
    position: "relative" as const,
    zIndex: 2,
    width: "clamp(18rem, 64vw, 64rem)",
    margin: "0 auto",
    padding: "clamp(3rem, 8vw, 7rem) clamp(1rem, 2.4vw, 2.5rem) clamp(3rem, 6vw, 6rem)",
    display: "flex",
    flexDirection: "column" as const,
    gap: "clamp(1.5rem, 2.6vw, 2.5rem)",
  },
  eyebrow: {
    fontFamily: "var(--font-mono)",
    fontSize: "clamp(0.7rem, 0.6rem + 0.18vw, 0.82rem)",
    letterSpacing: "0.5em",
    textTransform: "uppercase" as const,
    color: "var(--c-gold, #d8b56e)",
  },
  title: {
    fontFamily: "var(--font-display)",
    fontWeight: 800,
    fontSize: "clamp(2.1rem, 1.5rem + 2.6vw, 4.5rem)",
    lineHeight: 1.04,
    letterSpacing: "-0.022em",
    color: "var(--c-parchment, #f3ead8)",
    margin: 0,
    maxWidth: "22ch",
  },
  summary: {
    fontFamily: "var(--font-body)",
    fontSize: "clamp(1.05rem, 0.95rem + 0.45vw, 1.32rem)",
    lineHeight: 1.7,
    color: "var(--text)",
    maxWidth: "44ch",
    margin: 0,
  },
  callout: {
    marginTop: "clamp(0.75rem, 1.4vw, 1.4rem)",
    padding: "clamp(1rem, 1.5vw, 1.4rem) clamp(1.2rem, 1.8vw, 1.75rem)",
    borderLeft: "2px solid var(--c-violet, #8b6dff)",
    background: "rgba(45,0,80,0.18)",
    borderRadius: "0 6px 6px 0",
    maxWidth: "48ch",
  },
  calloutLabel: {
    fontFamily: "var(--font-mono)",
    fontSize: "0.68rem",
    letterSpacing: "0.32em",
    textTransform: "uppercase" as const,
    color: "var(--c-violet, #b095ff)",
    marginBottom: "0.55rem",
    display: "block",
  },
  question: {
    fontFamily: "var(--font-display)",
    fontStyle: "italic" as const,
    fontSize: "clamp(1.15rem, 1rem + 0.55vw, 1.5rem)",
    lineHeight: 1.45,
    color: "var(--c-parchment, #f3ead8)",
    margin: 0,
  },
  beginRow: {
    marginTop: "clamp(2rem, 4vw, 4rem)",
    display: "flex",
    alignItems: "center",
    gap: "0.85rem",
    fontFamily: "var(--font-mono)",
    fontSize: "0.75rem",
    letterSpacing: "0.4em",
    textTransform: "uppercase" as const,
    color: "var(--muted, rgba(255,255,255,0.55))",
  },
  beginArrow: {
    fontSize: "1.1rem",
    color: "var(--c-gold, #d8b56e)",
  },
};

export function WitnessLayerOpener({ witnessLayer }: WitnessLayerOpenerProps) {
  const reduced = useReducedMotion();
  const title = witnessLayer?.title || DEFAULT_DATA.title;
  const summary = witnessLayer?.summary || DEFAULT_DATA.summary;
  const question = witnessLayer?.question || DEFAULT_DATA.question;

  const fadeUp = (delay: number) =>
    reduced
      ? ({ initial: false, animate: { opacity: 1, y: 0 } } as const)
      : ({
          initial: { opacity: 0, y: 18 },
          whileInView: { opacity: 1, y: 0 },
          viewport: { once: true, amount: 0.4 },
          transition: {
            duration: 0.7,
            delay,
            ease: [0.2, 0.7, 0.2, 1] as [number, number, number, number],
          },
        } as const);

  return (
    <section
      style={s.wrap}
      aria-label="Witness layer — chapter zero opener"
    >
      {/* SacredScene ambient backdrop — quiet, contemplative atmosphere
          behind the chapter-0 spread. Sits at zIndex 0 with a translucent
          veil at zIndex 1 to preserve text readability. */}
      <div style={s.sceneBackdrop} aria-hidden="true">
        <DeferredSacredScene kind="ambient" intensity={0.5} height="100%" />
      </div>
      <div style={s.sceneVeil} aria-hidden="true" />
      <div style={s.inner}>
        <motion.div style={s.eyebrow} {...fadeUp(0)}>
          Witness Layer
        </motion.div>
        <motion.h1 style={s.title} {...fadeUp(0.08)}>
          {title}
        </motion.h1>
        <motion.p style={s.summary} {...fadeUp(0.16)}>
          {summary}
        </motion.p>
        <motion.div style={s.callout} {...fadeUp(0.24)}>
          <span style={s.calloutLabel}>Threshold Question</span>
          <p style={s.question}>{question}</p>
        </motion.div>
        <motion.div style={s.beginRow} {...fadeUp(0.32)}>
          <span style={s.beginArrow}>↓</span>
          <span>begin</span>
        </motion.div>
      </div>
    </section>
  );
}
