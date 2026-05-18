"use client";

// ─── ProseReader — modal text-reveal with 3-4 sentence focus highlight ──
// Combines three codrops effects for the reading modal:
//
//   1. OnScrollTextHighlight effect-1 — headline 3D char-stagger reveal
//      on modal mount (chars fade in from z=300 with rotationX -45° → 0)
//   2. ScrollBlurTypography effect-2 — body sentences blur-to-focus
//      scrub tied to MODAL scroll (chars start at blur(10px) brightness(30%);
//      sharpen as scroll brings them into focus zone)
//   3. Custom focus-zone logic per user spec: 3-4 sentences highlighted
//      at any moment; the rest dim+blurred to deliver impact-per-beat
//
// Per user directive: "not more than three or four sentences should be
// highlighted while reading, so that way we can deliver the impact of
// what is being said through the highlight."

import { useEffect, useRef, useLayoutEffect, useState } from "react";
import { gsap } from "gsap";
import SplitType from "split-type";
import type { SectionData } from "./data/sections";
import {
  parseProseBlocks,
  splitIntoSentences,
  renderInline,
  type ProseBlock,
} from "./prose/parseBlocks";
import { YantraLattice } from "./prose/carriers/YantraLattice";
import { SigilCascade } from "./prose/carriers/SigilCascade";
import { BentoTrio } from "./prose/carriers/BentoTrio";
import { DashaWaveform } from "./prose/carriers/DashaWaveform";
import { WitnessPulse } from "./prose/carriers/WitnessPulse";
import { DecisionPlate } from "./prose/carriers/DecisionPlate";

interface ProseReaderProps {
  section: SectionData;
  prose: string;
  onClose: () => void;
  /** Advance to the next reading section. When omitted, next-affordances
   *  are hidden. */
  onNext?: () => void;
  /** Go back to the previous reading section. When omitted, prev-
   *  affordances are hidden. */
  onPrev?: () => void;
}

// ─── Edge-scroll "radial progress" tuning ──────────────────────────────
// At-edge wheel events accumulate `progress` (-100 → +100). +100 fires
// onNext, -100 fires onPrev. Decay returns progress toward 0 when the
// user pauses, so accidental brushes don't trigger.
const EDGE_PROGRESS_THRESHOLD = 100;
const EDGE_WHEEL_GAIN = 0.55;       // % progress per deltaY pixel
const EDGE_DECAY_PER_SECOND = 90;   // % decays away per second of idle

export function ProseReader({ section, prose, onClose, onNext, onPrev }: ProseReaderProps) {
  const overlayRef = useRef<HTMLDivElement>(null);
  const scrollerRef = useRef<HTMLDivElement>(null);
  const eyebrowRef = useRef<HTMLDivElement>(null);
  const headlineRef = useRef<HTMLHeadingElement>(null);
  const summaryRef = useRef<HTMLParagraphElement>(null);
  const sentenceRefs = useRef<Array<HTMLParagraphElement | null>>([]);

  // Parse the prose into typed blocks. Paragraph blocks split into
  // sentences for the focus-zone highlight; non-paragraph blocks (tables,
  // headings, blockquotes, lists) dispatch to geometric carriers below.
  const blocks: ProseBlock[] = parseProseBlocks(prose);
  // Flat sentence list across all paragraph blocks — used to seed refs
  // array length only; the IO observer just selects `.depth-prose-sentence`
  // descendants of the scroller so we don't need per-sentence refs.
  const totalSentences = blocks
    .filter((b) => b.kind === "paragraph")
    .reduce((n, b) => n + splitIntoSentences((b as { text: string }).text).length, 0);
  sentenceRefs.current = new Array(totalSentences).fill(null);

  // ── ESC to close + body-scroll lock ────────────────────────────────
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    const prevOverflow = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    return () => {
      window.removeEventListener("keydown", onKey);
      document.body.style.overflow = prevOverflow;
    };
  }, [onClose]);

  // ── Mount: split headline + animate 3D char-stagger reveal ─────────
  useLayoutEffect(() => {
    const ctx = gsap.context(() => {
      // 1. Headline char-stagger reveal (OnScrollTextHighlight effect-1)
      if (headlineRef.current) {
        const split = new SplitType(headlineRef.current, { types: "chars" });
        if (split.chars && split.chars.length > 0) {
          gsap.set(headlineRef.current, { perspective: 500 });
          gsap.fromTo(
            split.chars,
            { opacity: 0, z: 300, rotationX: -45 },
            {
              opacity: 1,
              z: 0,
              rotationX: 0,
              duration: 0.8,
              stagger: 0.035,
              ease: "power2.out",
              delay: 0.15,
            },
          );
        }
      }

      // 2. Eyebrow + summary fade in
      if (eyebrowRef.current) {
        gsap.fromTo(
          eyebrowRef.current,
          { opacity: 0, y: 8 },
          { opacity: 1, y: 0, duration: 0.7, ease: "power2.out" },
        );
      }
      if (summaryRef.current) {
        gsap.fromTo(
          summaryRef.current,
          { opacity: 0, y: 12 },
          { opacity: 0.78, y: 0, duration: 0.9, delay: 0.6, ease: "power2.out" },
        );
      }
    }, overlayRef);

    return () => ctx.revert();
  }, [section.id]);

  // ── Body sentences: 3-4 sentence focus zone via IntersectionObserver
  //    (modal is its own scroll container; observer rootMargin defines
  //    the focus band — sentences inside it are sharp + bright; outside
  //    are dim + slightly blurred). This is the user-spec "highlight
  //    3-4 sentences at a time for storytelling impact." ──────────────
  useEffect(() => {
    if (!scrollerRef.current) return;

    // The focus band sits in the vertical center of the modal viewport.
    // Sentences crossing this band get a "lit" class; the IO unsets it
    // when they leave. We use a moderate band (35-65% of viewport)
    // so 3-4 sentences sit inside it at once at typical scroll speeds.
    const io = new IntersectionObserver(
      (entries) => {
        for (const e of entries) {
          if (e.isIntersecting) {
            (e.target as HTMLElement).classList.add("lit");
          } else {
            (e.target as HTMLElement).classList.remove("lit");
          }
        }
      },
      {
        root: scrollerRef.current,
        rootMargin: "-35% 0% -35% 0%",
        threshold: 0,
      },
    );

    // Observe every `.depth-prose-sentence` descendant of the scroller.
    // Carrier blocks (tables, headings, blockquotes) render at full
    // opacity always — they're geometric anchors, not part of the focus
    // cycle. Only verse sentences get the lit/unlit treatment.
    const sentenceEls = scrollerRef.current.querySelectorAll(
      ".depth-prose-sentence",
    );
    sentenceEls.forEach((el) => io.observe(el));
    return () => io.disconnect();
  }, [totalSentences, section.id]);

  // ── Edge-scroll radial progress: aggressive scroll past the end fills
  //    a circle; 100% auto-advances to next. Scrolling past the top
  //    fills the reverse circle, auto-advances to prev. Decays on idle. ──
  const [edgeProgress, setEdgeProgress] = useState(0); // -100..+100
  const lastWheelAtRef = useRef<number>(0);

  useEffect(() => {
    const scroller = scrollerRef.current;
    if (!scroller) return;

    const isAtBottom = () =>
      scroller.scrollTop + scroller.clientHeight >= scroller.scrollHeight - 1;
    const isAtTop = () => scroller.scrollTop <= 0;

    const onWheel = (e: WheelEvent) => {
      const atBottom = isAtBottom();
      const atTop = isAtTop();
      const goingDown = e.deltaY > 0;
      const goingUp = e.deltaY < 0;
      // Only accumulate when scrolling AT the edge in the OUTWARD direction
      if ((atBottom && goingDown && onNext) || (atTop && goingUp && onPrev)) {
        e.preventDefault();
        const delta = Math.abs(e.deltaY) * EDGE_WHEEL_GAIN;
        const direction = goingDown ? 1 : -1;
        lastWheelAtRef.current = performance.now();
        setEdgeProgress((p) => {
          const next = p + direction * delta;
          return Math.max(-EDGE_PROGRESS_THRESHOLD, Math.min(EDGE_PROGRESS_THRESHOLD, next));
        });
      }
    };

    // Decay loop — runs every animation frame, lowers |progress| if the
    // user paused. Triggers nav at threshold.
    let rafId = 0;
    let lastTs = performance.now();
    const tick = () => {
      const now = performance.now();
      const dt = (now - lastTs) / 1000;
      lastTs = now;
      const idleSec = (now - lastWheelAtRef.current) / 1000;
      setEdgeProgress((p) => {
        if (p === 0) return 0;
        // If progress reached threshold this frame, fire the nav
        if (p >= EDGE_PROGRESS_THRESHOLD && onNext) {
          // Defer to escape the setState callback
          queueMicrotask(() => {
            onNext();
          });
          return 0;
        }
        if (p <= -EDGE_PROGRESS_THRESHOLD && onPrev) {
          queueMicrotask(() => {
            onPrev();
          });
          return 0;
        }
        // Decay back to 0 if user has paused > 0.15s
        if (idleSec > 0.15) {
          const decay = EDGE_DECAY_PER_SECOND * dt;
          if (p > 0) return Math.max(0, p - decay);
          if (p < 0) return Math.min(0, p + decay);
        }
        return p;
      });
      rafId = requestAnimationFrame(tick);
    };
    rafId = requestAnimationFrame(tick);

    scroller.addEventListener("wheel", onWheel, { passive: false });
    return () => {
      scroller.removeEventListener("wheel", onWheel);
      cancelAnimationFrame(rafId);
    };
  }, [onNext, onPrev]);

  // ── Click backdrop to close ─────────────────────────────────────────
  const handleBackdropClick = (e: React.MouseEvent) => {
    if (e.target === e.currentTarget) onClose();
  };

  return (
    <div
      ref={overlayRef}
      onClick={handleBackdropClick}
      style={{
        position: "fixed",
        inset: 0,
        zIndex: 200,
        background: "radial-gradient(ellipse at center, rgba(7,11,29,0.92) 0%, rgba(7,11,29,0.97) 70%)",
        backdropFilter: "blur(14px)",
        display: "flex",
        alignItems: "stretch",
        justifyContent: "center",
        padding: "clamp(1rem, 4vh, 4rem) clamp(1rem, 4vw, 4rem)",
        animation: "depthReaderFadeIn 0.4s ease",
      }}
    >
      {/* Embedded styles — keyframes + lit-state filters */}
      <style>{`
        @keyframes depthReaderFadeIn {
          from { opacity: 0; }
          to   { opacity: 1; }
        }
        .depth-prose-sentence {
          /* Default state: out of focus zone — dim + soft blur */
          opacity: 0.30;
          filter: blur(2.5px) brightness(55%);
          transform: translateY(0.25rem);
          transition:
            opacity 600ms cubic-bezier(0.2, 0.7, 0.2, 1),
            filter 600ms cubic-bezier(0.2, 0.7, 0.2, 1),
            transform 600ms cubic-bezier(0.2, 0.7, 0.2, 1);
          will-change: opacity, filter, transform;
        }
        .depth-prose-sentence.lit {
          /* In focus zone — sharp, full opacity, slightly forward */
          opacity: 1;
          filter: blur(0px) brightness(110%);
          transform: translateY(0);
        }
        .depth-prose-sentence + .depth-prose-sentence {
          margin-top: 1.1em;
        }
        .depth-prose-sentence strong {
          color: var(--accent, var(--c-gold, #C5A017));
          font-weight: 600;
        }
        .depth-prose-sentence em {
          color: var(--accent, var(--c-emerald, #10B5A7));
          font-style: italic;
        }
        @media (prefers-reduced-motion: reduce) {
          .depth-prose-sentence {
            opacity: 1 !important;
            filter: none !important;
            transform: none !important;
            transition: none !important;
          }
        }
      `}</style>

      {/* Modal frame */}
      <article
        onClick={(e) => e.stopPropagation()}
        style={
          {
            position: "relative",
            width: "min(60rem, 92vw)",
            height: "min(92vh, 100%)",
            background: "var(--c-void, #070B1D)",
            border: `1px solid ${section.accentColor}33`,
            borderRadius: "10px",
            boxShadow: `0 0 120px ${section.accentColor}22, 0 30px 80px rgba(0,0,0,0.6)`,
            display: "flex",
            flexDirection: "column",
            overflow: "hidden",
            // Expose accent as CSS var so embedded styles can pick it up
            ["--accent" as string]: section.accentColor,
          } as React.CSSProperties
        }
      >
        {/* Close button — fixed top-right inside modal */}
        <button
          onClick={onClose}
          aria-label="Close reading"
          style={{
            position: "absolute",
            top: "1rem",
            right: "1rem",
            zIndex: 20,
            background: "rgba(7,11,29,0.7)",
            border: `1px solid ${section.accentColor}55`,
            color: "var(--c-parchment, #F0EDE3)",
            width: "2.4rem",
            height: "2.4rem",
            borderRadius: "50%",
            cursor: "pointer",
            fontSize: "1.1rem",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            backdropFilter: "blur(8px)",
            transition: "all 0.2s ease",
          }}
          onMouseEnter={(e) => {
            e.currentTarget.style.background = section.accentColor;
            e.currentTarget.style.color = "var(--c-void, #070B1D)";
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.background = "rgba(7,11,29,0.7)";
            e.currentTarget.style.color = "var(--c-parchment, #F0EDE3)";
          }}
        >
          ×
        </button>

        {/* SCROLL CONTAINER */}
        <div
          ref={scrollerRef}
          style={{
            flex: 1,
            overflowY: "auto",
            overflowX: "hidden",
            scrollBehavior: "smooth",
            padding: "clamp(2rem, 5vh, 4.5rem) clamp(2rem, 5vw, 5rem) 15vh",
          }}
        >
          {/* Header — eyebrow + headline + summary */}
          <header
            style={{
              marginBottom: "clamp(2.5rem, 5vh, 4rem)",
              paddingBottom: "clamp(1.5rem, 3vh, 2.5rem)",
              borderBottom: `1px solid ${section.accentColor}33`,
            }}
          >
            <div
              ref={eyebrowRef}
              style={{
                fontFamily: "var(--font-mono, 'SF Mono', monospace)",
                fontSize: "clamp(0.7rem, 0.8vw, 0.85rem)",
                letterSpacing: "0.45em",
                color: section.accentColor,
                textTransform: "uppercase",
                marginBottom: "1.25rem",
              }}
            >
              {section.numeral} · {section.direction ?? "WITNESS"}
            </div>
            <h1
              ref={headlineRef}
              style={{
                margin: 0,
                fontFamily: "var(--font-display, 'Panchang', serif)",
                fontVariationSettings: "'wght' 720",
                // Fluid sizing capped by BOTH viewport-width AND
                // viewport-height. On tall narrow windows the title
                // shrinks with vw; on short wide windows it caps at
                // 6vh so it never devours vertical space. The em max
                // (3.2em) prevents oversize on 4K monitors.
                fontSize: "clamp(1.6em, min(4.2vw, 6vh), 3.2em)",
                letterSpacing: "-0.015em",
                // Bumped from 0.98 → 1.06 so long titles that wrap
                // don't have descenders kiss the next line's ascenders
                lineHeight: 1.06,
                color: "var(--c-parchment, #F0EDE3)",
                wordBreak: "normal",
                overflowWrap: "break-word",
                hyphens: "auto",
                maxWidth: "100%",
              }}
            >
              {section.title}
            </h1>
            <p
              ref={summaryRef}
              style={{
                margin: "1.25rem 0 0",
                maxWidth: "44ch",
                fontFamily: "var(--font-display, 'Panchang', serif)",
                fontStyle: "italic",
                fontVariationSettings: "'wght' 500",
                fontSize: "clamp(1em, 1.15vw, 1.2em)",
                lineHeight: 1.5,
                color: "rgba(240,237,227,0.78)",
                opacity: 0,
              }}
            >
              {section.summary}
            </p>
          </header>

          {/* Blocks — paragraphs flow as verse sentences (focus-zone lit);
              tables/headings/blockquotes/lists dispatch to geometric
              carriers (Yantra Lattice / Sigil Cascade / Bento Trio /
              Dasha Waveform / Witness Pulse / Decision Plate). */}
          {blocks.length > 0 ? (
            blocks.map((block, bi) => (
              <BlockRenderer
                key={bi}
                block={block}
                accentColor={section.accentColor}
                cardinal={section.direction}
              />
            ))
          ) : (
            <p
              style={{
                fontStyle: "italic",
                color: "rgba(240,237,227,0.5)",
              }}
            >
              [No prose available for this section yet.]
            </p>
          )}

          {/* End-of-reading marker */}
          <div
            style={{
              marginTop: "clamp(3rem, 6vh, 5rem)",
              paddingTop: "clamp(1.5rem, 3vh, 2.5rem)",
              borderTop: `1px solid ${section.accentColor}33`,
              textAlign: "center",
              fontFamily: "var(--font-mono, 'SF Mono', monospace)",
              fontSize: "0.7rem",
              letterSpacing: "0.5em",
              color: section.accentColor,
              opacity: 0.7,
            }}
          >
            ∴ END OF {section.numeral}
          </div>
        </div>

        {/* Bottom gradient fade — visual end-of-reading cue */}
        <div
          aria-hidden="true"
          style={{
            position: "absolute",
            bottom: 0,
            left: 0,
            right: 0,
            height: "12vh",
            background:
              "linear-gradient(180deg, transparent 0%, var(--c-void, #070B1D) 100%)",
            pointerEvents: "none",
          }}
        />

        {/* ─── NEXT / PREV navigation pills + radial-progress rings ─── */}
        {onPrev && (
          <NavPill
            direction="prev"
            accentColor={section.accentColor}
            progress={Math.max(0, -edgeProgress) / EDGE_PROGRESS_THRESHOLD}
            onClick={onPrev}
            label="PREV"
          />
        )}
        {onNext && (
          <NavPill
            direction="next"
            accentColor={section.accentColor}
            progress={Math.max(0, edgeProgress) / EDGE_PROGRESS_THRESHOLD}
            onClick={onNext}
            label="NEXT"
          />
        )}
      </article>
    </div>
  );
}

/** NEXT / PREV pill button with a circular fill ring that visualizes
 *  edge-scroll progress. The ring fills from 0 → 1 as the user pushes
 *  past the scroll edge; at 1.0 the ProseReader's scroll handler auto-
 *  fires the corresponding callback. Clicking the pill is the instant
 *  alternative. */
function NavPill({
  direction,
  accentColor,
  progress,
  onClick,
  label,
}: {
  direction: "next" | "prev";
  accentColor: string;
  progress: number; // 0..1
  onClick: () => void;
  label: string;
}) {
  const isNext = direction === "next";
  // Circle ring math — circumference = 2π · r
  const R = 22;
  const C = 2 * Math.PI * R;
  const dashOffset = C * (1 - Math.min(1, Math.max(0, progress)));

  return (
    <button
      onClick={onClick}
      aria-label={isNext ? "Next section" : "Previous section"}
      style={{
        position: "absolute",
        bottom: "clamp(1.5rem, 4vh, 3rem)",
        [isNext ? "right" : "left"]: "clamp(1.5rem, 4vw, 3rem)",
        zIndex: 25,
        display: "flex",
        alignItems: "center",
        gap: "0.75rem",
        padding: "0.6rem 0.6rem",
        paddingLeft: isNext ? "1.1rem" : "0.6rem",
        paddingRight: isNext ? "0.6rem" : "1.1rem",
        background: "rgba(7,11,29,0.85)",
        border: `1px solid ${accentColor}55`,
        color: accentColor,
        borderRadius: "999px",
        cursor: "pointer",
        backdropFilter: "blur(8px)",
        fontFamily: "var(--font-mono, 'SF Mono', monospace)",
        fontSize: "clamp(0.62rem, 0.75vw, 0.78rem)",
        letterSpacing: "0.3em",
        textTransform: "uppercase",
        transition: "all 0.2s ease",
        flexDirection: isNext ? "row" : "row-reverse",
        boxShadow: progress > 0.05
          ? `0 0 ${24 + progress * 30}px ${accentColor}${Math.round(progress * 0.8 * 255).toString(16).padStart(2, "0")}`
          : "none",
      }}
      onMouseEnter={(e) => {
        e.currentTarget.style.background = `${accentColor}`;
        e.currentTarget.style.color = "var(--c-void, #070B1D)";
      }}
      onMouseLeave={(e) => {
        e.currentTarget.style.background = "rgba(7,11,29,0.85)";
        e.currentTarget.style.color = accentColor;
      }}
    >
      <span>{label}</span>
      <span
        style={{
          position: "relative",
          width: "2.6rem",
          height: "2.6rem",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
        }}
      >
        {/* SVG circular progress ring */}
        <svg
          width="100%"
          height="100%"
          viewBox="0 0 50 50"
          style={{ position: "absolute", inset: 0, transform: "rotate(-90deg)" }}
        >
          {/* Track */}
          <circle
            cx="25"
            cy="25"
            r={R}
            fill="none"
            stroke="currentColor"
            strokeOpacity="0.25"
            strokeWidth="2"
          />
          {/* Progress arc */}
          <circle
            cx="25"
            cy="25"
            r={R}
            fill="none"
            stroke="currentColor"
            strokeWidth="2.5"
            strokeDasharray={C}
            strokeDashoffset={dashOffset}
            strokeLinecap="round"
            style={{ transition: "stroke-dashoffset 80ms linear" }}
          />
        </svg>
        {/* Arrow */}
        <span
          aria-hidden="true"
          style={{
            position: "relative",
            fontSize: "1.1rem",
            lineHeight: 1,
            transform: isNext ? "translateX(1px)" : "translateX(-1px)",
          }}
        >
          {isNext ? "→" : "←"}
        </span>
      </span>
    </button>
  );
}

/** Dispatch a single ProseBlock to its appropriate renderer.
 *  - paragraphs: split into sentences, each carrying `.depth-prose-sentence`
 *    so the IO focus-zone lights 3-4 at a time.
 *  - headings: WitnessPulse (h2) or compact pulse (h3/h4).
 *  - blockquote: DecisionPlate.
 *  - table: routed by classification to Yantra Lattice / Sigil Cascade /
 *           Bento Trio / Dasha Waveform.
 *  - list: SigilCascade in list-mode.
 *  - code/hr: simple primitives. */
function BlockRenderer({
  block,
  accentColor,
  cardinal,
}: {
  block: ProseBlock;
  accentColor: string;
  cardinal?: string;
}) {
  switch (block.kind) {
    case "paragraph": {
      const sentences = splitIntoSentences(block.text);
      return (
        <div style={{ display: "grid", gap: "1.1em", margin: "1.5rem 0" }}>
          {sentences.map((s, si) => (
            <p
              key={si}
              className="depth-prose-sentence"
              style={{
                margin: 0,
                fontFamily: "var(--font-body, 'Satoshi', sans-serif)",
                fontSize: "clamp(1.05em, 1.2vw, 1.3em)",
                lineHeight: 1.55,
                color: "var(--c-parchment, #F0EDE3)",
                letterSpacing: "0.005em",
                maxWidth: "62ch",
              }}
              dangerouslySetInnerHTML={{ __html: renderInline(s) }}
            />
          ))}
        </div>
      );
    }

    case "heading":
      return (
        <WitnessPulse
          text={block.text}
          cardinal={block.level === 2 ? cardinal : undefined}
          accentColor={accentColor}
          level={block.level === 1 ? 2 : (block.level as 2 | 3 | 4)}
        />
      );

    case "blockquote":
      return <DecisionPlate lines={block.lines} accentColor={accentColor} />;

    case "hr":
      return (
        <div
          aria-hidden="true"
          style={{
            margin: "clamp(2rem, 4vh, 3.5rem) auto",
            width: "min(160px, 35%)",
            height: "1px",
            background: `linear-gradient(90deg, transparent, ${accentColor}66, transparent)`,
          }}
        />
      );

    case "list":
      return (
        <SigilCascade
          rows={block.items.map((it) => [it])}
          accentColor={accentColor}
          mode="list"
        />
      );

    case "code":
      return (
        <pre
          style={{
            margin: "clamp(1rem, 2vh, 1.5rem) 0",
            padding: "1rem 1.25rem",
            background: "rgba(7,11,29,0.7)",
            border: `1px solid ${accentColor}30`,
            borderRadius: "8px",
            fontFamily: "var(--font-mono, 'SF Mono', monospace)",
            fontSize: "0.85rem",
            color: "rgba(240,237,227,0.92)",
            overflow: "auto",
          }}
        >
          <code>{block.text}</code>
        </pre>
      );

    case "table":
      switch (block.classification) {
        case "yantra-lattice":
          return (
            <YantraLattice
              headers={block.headers}
              rows={block.rows}
              accentColor={accentColor}
            />
          );
        case "sigil-cascade":
          return (
            <SigilCascade
              headers={block.headers}
              rows={block.rows}
              accentColor={accentColor}
            />
          );
        case "bento-trio":
          return (
            <BentoTrio
              headers={block.headers}
              rows={block.rows}
              accentColor={accentColor}
            />
          );
        case "dasha-waveform":
          return (
            <DashaWaveform
              headers={block.headers}
              rows={block.rows}
              accentColor={accentColor}
            />
          );
        default:
          return null;
      }
  }
}
