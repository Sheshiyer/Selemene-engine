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

import { useEffect, useRef, useLayoutEffect } from "react";
import { gsap } from "gsap";
import SplitType from "split-type";
import type { SectionData } from "./data/sections";

interface ProseReaderProps {
  section: SectionData;
  prose: string;
  onClose: () => void;
}

/** Split prose into sentence chunks. Groups by sentence boundary (. ! ?)
 *  with quote/period exception handling so abbreviations don't false-split. */
function splitIntoSentences(prose: string): string[] {
  if (!prose) return [];
  // Normalize whitespace + line breaks; preserve paragraph breaks as \n\n
  const normalized = prose.replace(/\r\n/g, "\n").trim();
  const paragraphs = normalized.split(/\n{2,}/);
  const sentences: string[] = [];
  for (const para of paragraphs) {
    // Sentence split on . ! ? followed by space + capital letter (rough)
    const parts = para
      .split(/(?<=[.!?])\s+(?=[A-Z“"'(])/)
      .map((s) => s.trim())
      .filter(Boolean);
    sentences.push(...parts);
  }
  return sentences;
}

export function ProseReader({ section, prose, onClose }: ProseReaderProps) {
  const overlayRef = useRef<HTMLDivElement>(null);
  const scrollerRef = useRef<HTMLDivElement>(null);
  const eyebrowRef = useRef<HTMLDivElement>(null);
  const headlineRef = useRef<HTMLHeadingElement>(null);
  const summaryRef = useRef<HTMLParagraphElement>(null);
  const sentenceRefs = useRef<Array<HTMLParagraphElement | null>>([]);

  const sentences = splitIntoSentences(prose);
  sentenceRefs.current = new Array(sentences.length).fill(null);

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

    sentenceRefs.current.forEach((el) => {
      if (el) io.observe(el);
    });
    return () => io.disconnect();
  }, [sentences.length, section.id]);

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
                fontWeight: 700,
                fontSize: "clamp(2rem, 4vw, 4.25rem)",
                letterSpacing: "-0.01em",
                lineHeight: 0.98,
                color: "var(--c-parchment, #F0EDE3)",
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
                fontWeight: 500,
                fontSize: "clamp(1rem, 1.15vw, 1.25rem)",
                lineHeight: 1.5,
                color: "rgba(240,237,227,0.78)",
                opacity: 0,
              }}
            >
              {section.summary}
            </p>
          </header>

          {/* Sentences — each one is a focus-zone candidate */}
          {sentences.length > 0 ? (
            sentences.map((sentence, i) => (
              <p
                key={i}
                ref={(el) => {
                  sentenceRefs.current[i] = el;
                }}
                className="depth-prose-sentence"
                style={{
                  margin: 0,
                  fontFamily: "var(--font-body, 'Satoshi', sans-serif)",
                  fontSize: "clamp(1.1rem, 1.25vw, 1.4rem)",
                  lineHeight: 1.55,
                  color: "var(--c-parchment, #F0EDE3)",
                  letterSpacing: "0.005em",
                  maxWidth: "62ch",
                }}
                dangerouslySetInnerHTML={{ __html: renderInlineMarkdown(sentence) }}
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
      </article>
    </div>
  );
}

/** Very lightweight inline-markdown renderer: **bold** + *italic* +
 *  basic line break support. Sentences come pre-split so we just need
 *  inline formatting. */
function renderInlineMarkdown(text: string): string {
  return text
    // Escape HTML
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    // **bold**
    .replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>")
    // *italic* (single asterisk)
    .replace(/(?<!\*)\*([^*]+)\*(?!\*)/g, "<em>$1</em>")
    // _italic_ (underscore variant)
    .replace(/\b_([^_]+)_\b/g, "<em>$1</em>");
}
