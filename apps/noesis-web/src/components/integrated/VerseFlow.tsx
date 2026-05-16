"use client";

// ─── VerseFlow — scroll-illuminated reading column ──────────────────────
// Takes parsed HTML (from marked) or a typed Block[] (from parseBlocks)
// and renders each top-level block as a <Verse> — dim opacity 0.22 +
// 0.4px blur, illuminating to full opacity as it enters viewport-center.
//
// Long paragraphs (>160 words, 3+ sentences) split into sentence-grouped
// sub-verses for the 2-3 line focus rhythm.
//
// Specialised block kinds (hex-trio, cascade, decision) are dispatched to
// the W3 BlockRenderer instead of being rendered as html.

import { motion, useInView, useReducedMotion } from "motion/react";
import { useRef, useMemo, useEffect } from "react";
import type { ReactNode } from "react";
import { createRoot, type Root } from "react-dom/client";
import type { Block } from "@/lib/integrated/parseBlocks";
import { BlockRenderer } from "./data/BlockRenderer";
import { useDrilldown } from "./drilldown/DrilldownContext";
import {
  MicroYantra,
  type MicroYantraKind,
} from "./inline/MicroYantra";
import { decorateHeadings } from "@/lib/integrated/decorateHeadings";

interface VerseProps {
  children: ReactNode;
  anchor?: boolean;
}

function Verse({ children, anchor = false }: VerseProps) {
  const ref = useRef<HTMLDivElement>(null);
  const inView = useInView(ref, { margin: "-35% 0% -35% 0%", once: false });
  const reduced = useReducedMotion();
  return (
    <motion.div
      ref={ref}
      style={{
        margin: "0 0 22px",
        willChange: "opacity, filter",
      }}
      initial={false}
      animate={
        reduced
          ? { opacity: 1, filter: "blur(0px)" }
          : {
              // Default opacity raised from 0.22 → 0.55 so prose is
              // always legibly visible (not "missing"). Illumination
              // still bumps to 1.0 at viewport-center for focus rhythm.
              opacity: inView ? 1 : anchor ? 0.75 : 0.55,
              filter: inView ? "blur(0px)" : "blur(0.2px)",
            }
      }
      transition={{ duration: 0.6, ease: [0.2, 0.7, 0.2, 1] }}
    >
      {children}
    </motion.div>
  );
}

interface VerseFlowProps {
  /** Legacy: pre-parsed HTML (from marked). Used when no `blocks` prop. */
  html?: string;
  /** Preferred: parsed typed Block[] from parseMarkdownBlocks. */
  blocks?: Block[];
}

/**
 * Take rendered HTML and break it into top-level blocks. Long <p> blocks
 * get sentence-split into multiple verses.
 */
function splitIntoVerses(html: string): Array<{ kind: "block" | "anchor"; html: string }> {
  const blockTags = ["p", "h2", "h3", "h4", "ul", "ol", "blockquote", "table", "pre"];
  const blockRe = new RegExp(`<(?:${blockTags.join("|")})[^>]*>[\\s\\S]*?</(?:${blockTags.join("|")})>`, "g");
  const out: Array<{ kind: "block" | "anchor"; html: string }> = [];

  const matches = [...html.matchAll(blockRe)];
  for (const m of matches) {
    const block = m[0];
    const tag = block.match(/^<([a-z0-9]+)/i)?.[1].toLowerCase() ?? "p";
    const isAnchor = tag === "h2" || tag === "h3" || tag === "h4";

    if (tag === "p") {
      const text = block.replace(/<\/?p[^>]*>/g, "");
      const words = text.split(/\s+/).filter(Boolean).length;
      const sentenceEnds = (text.match(/[.!?]\s+(?=[A-Z“"])/g) || []).length;
      if (words > 160 && sentenceEnds >= 3) {
        const parts = text.split(/(?<=[.!?])\s+(?=[A-Z“"])/);
        let current: string[] = [];
        let currentWords = 0;
        for (const p of parts) {
          const w = p.split(/\s+/).filter(Boolean).length;
          current.push(p);
          currentWords += w;
          if (currentWords >= 55 || current.length >= 3) {
            out.push({ kind: "block", html: `<p>${current.join(" ")}</p>` });
            current = [];
            currentWords = 0;
          }
        }
        if (current.length) {
          out.push({ kind: "block", html: `<p>${current.join(" ")}</p>` });
        }
        continue;
      }
    }
    out.push({ kind: isAnchor ? "anchor" : "block", html: block });
  }
  return out;
}

// ─── Term-link hydration ────────────────────────────────────────────────
// enhanceTermsInHtml() wraps known technical terms with
//   <span data-engine-term data-engine-id=… data-term-definition=…>
// On mount, walk the rendered prose for these markers and attach a click
// handler that opens the drill-down panel via DrilldownContext. Hover +
// focus reveal a tooltip element inserted next to the span.
function useEngineTermHydration(
  rootRef: React.RefObject<HTMLDivElement | null>,
) {
  const { open, engineOutputs } = useDrilldown();
  useEffect(() => {
    const root = rootRef.current;
    if (!root) return;

    const spans = Array.from(
      root.querySelectorAll<HTMLElement>("span[data-engine-term]"),
    );
    if (spans.length === 0) return;

    type Cleanup = () => void;
    const cleanups: Cleanup[] = [];

    for (const span of spans) {
      const engineId = span.getAttribute("data-engine-id");
      const definition = span.getAttribute("data-term-definition") || "";
      if (!engineId) continue;

      // Style: dotted gold underline, color shift on hover/focus.
      span.style.borderBottom = "1px dotted var(--c-gold, #d8b56e)";
      span.style.color = "var(--c-gold, #d8b56e)";
      span.style.cursor = "help";
      span.style.transition = "color 0.15s ease, border-color 0.15s ease";
      span.setAttribute("role", "button");
      span.setAttribute("tabindex", "0");
      span.setAttribute(
        "aria-label",
        `Open ${engineId.replace(/-/g, " ")} drilldown`,
      );

      let tooltip: HTMLSpanElement | null = null;
      const showTooltip = () => {
        span.style.color = "var(--c-parchment, #f3ead8)";
        span.style.borderBottomColor = "var(--c-parchment, #f3ead8)";
        if (tooltip) return;
        tooltip = document.createElement("span");
        tooltip.setAttribute("role", "tooltip");
        tooltip.textContent = "";
        // Build content: monospace label + definition
        const label = document.createElement("span");
        label.textContent = engineId
          .split("-")
          .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
          .join(" ");
        Object.assign(label.style, {
          fontFamily: "var(--font-mono)",
          fontSize: "0.62rem",
          letterSpacing: "0.18em",
          textTransform: "uppercase",
          color: "var(--c-gold, #d8b56e)",
          marginBottom: "0.25rem",
          display: "block",
        });
        const body = document.createElement("span");
        body.textContent = definition;
        tooltip.appendChild(label);
        tooltip.appendChild(body);
        Object.assign(tooltip.style, {
          position: "absolute",
          bottom: "calc(100% + 8px)",
          left: "50%",
          transform: "translateX(-50%)",
          minWidth: "200px",
          maxWidth: "280px",
          padding: "0.55rem 0.75rem",
          background: "rgba(7,11,29,0.96)",
          border: "1px solid rgba(216,181,110,0.35)",
          borderRadius: "6px",
          fontSize: "0.78rem",
          lineHeight: "1.45",
          color: "rgba(255,255,255,0.92)",
          fontFamily: "var(--font-body)",
          boxShadow: "0 8px 24px rgba(0,0,0,0.45)",
          whiteSpace: "normal",
          zIndex: "60",
          pointerEvents: "none",
          textAlign: "left",
        });
        // Position parent must be relative so tooltip floats correctly.
        if (getComputedStyle(span).position === "static") {
          span.style.position = "relative";
          span.style.display = "inline-block";
        }
        span.appendChild(tooltip);
      };
      const hideTooltip = () => {
        span.style.color = "var(--c-gold, #d8b56e)";
        span.style.borderBottomColor = "var(--c-gold, #d8b56e)";
        if (tooltip && tooltip.parentNode === span) {
          span.removeChild(tooltip);
        }
        tooltip = null;
      };
      const onClick = (e: Event) => {
        e.preventDefault();
        e.stopPropagation();
        open({ engineId, result: engineOutputs[engineId] });
        hideTooltip();
      };
      const onKey = (e: KeyboardEvent) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          onClick(e);
        }
      };

      span.addEventListener("mouseenter", showTooltip);
      span.addEventListener("mouseleave", hideTooltip);
      span.addEventListener("focus", showTooltip);
      span.addEventListener("blur", hideTooltip);
      span.addEventListener("click", onClick);
      span.addEventListener("keydown", onKey);

      cleanups.push(() => {
        span.removeEventListener("mouseenter", showTooltip);
        span.removeEventListener("mouseleave", hideTooltip);
        span.removeEventListener("focus", showTooltip);
        span.removeEventListener("blur", hideTooltip);
        span.removeEventListener("click", onClick);
        span.removeEventListener("keydown", onKey);
        hideTooltip();
      });
    }

    return () => {
      for (const c of cleanups) c();
    };
  }, [open, engineOutputs, rootRef]);
}

// ─── Micro-yantra hydration ─────────────────────────────────────────────
// microYantraEnhance.ts injects placeholder spans of the form
//   <span data-micro-yantra="<kind>" data-arg="<arg>" aria-hidden></span>
// into the rendered HTML. After mount we walk the verse content, locate
// each placeholder, mount a real <MicroYantra> React tree into it via
// React 18 createRoot, and tear them down on unmount.
//
// Defensive: if a placeholder is the first child of a [data-engine-term]
// wrapper, that's fine — we just hydrate in place; the glyph sits inside
// the W6 link surface visually.
function useMicroYantraHydration(
  rootRef: React.RefObject<HTMLDivElement | null>,
  deps: ReadonlyArray<unknown>,
) {
  useEffect(() => {
    const root = rootRef.current;
    if (!root) return;
    if (typeof window === "undefined") return;

    const placeholders = Array.from(
      root.querySelectorAll<HTMLElement>("span[data-micro-yantra]"),
    );
    if (placeholders.length === 0) return;

    const mounted: Root[] = [];
    for (const el of placeholders) {
      // Skip already-hydrated placeholders (re-runs over the same DOM).
      if (el.dataset.yantraHydrated === "1") continue;
      const kindAttr = el.getAttribute("data-micro-yantra");
      const arg = el.getAttribute("data-arg");
      if (!kindAttr || !arg) continue;
      // Clear the SSR spacer styling — React component manages its own.
      el.removeAttribute("style");
      el.removeAttribute("aria-hidden");
      el.dataset.yantraHydrated = "1";
      try {
        const root = createRoot(el);
        root.render(
          <MicroYantra
            kind={kindAttr as MicroYantraKind}
            arg={arg}
          />,
        );
        mounted.push(root);
      } catch {
        // Hydration failure on this node is non-fatal; skip silently.
      }
    }

    return () => {
      for (const r of mounted) {
        // Defer unmount past commit phase per React 18 guidance.
        Promise.resolve().then(() => r.unmount());
      }
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, deps);
}

// ─── Heading decoration ────────────────────────────────────────────────
// On mount, prepend a small sacred-geometry sigil to each h2/h3 in the
// verse content. Idempotent via [data-yantra-decorated].
function useHeadingDecoration(
  rootRef: React.RefObject<HTMLDivElement | null>,
  deps: ReadonlyArray<unknown>,
) {
  useEffect(() => {
    decorateHeadings(rootRef.current);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, deps);
}

export function VerseFlow({ html, blocks }: VerseFlowProps) {
  // Mode A: typed blocks (preferred). HTML chunks are split & illuminated;
  // specialised blocks dispatch to BlockRenderer.
  const rendered = useMemo(() => {
    if (blocks) {
      const nodes: ReactNode[] = [];
      blocks.forEach((b, bi) => {
        if (b.kind === "html") {
          const verses = splitIntoVerses(b.html);
          verses.forEach((v, vi) => {
            nodes.push(
              <Verse key={`b${bi}-v${vi}`} anchor={v.kind === "anchor"}>
                <div dangerouslySetInnerHTML={{ __html: v.html }} />
              </Verse>,
            );
          });
        } else {
          nodes.push(<BlockRenderer key={`b${bi}`} block={b} />);
        }
      });
      return nodes;
    }
    // Mode B (legacy): full html string
    if (html) {
      const verses = splitIntoVerses(html);
      return verses.map((v, i) => (
        <Verse key={i} anchor={v.kind === "anchor"}>
          <div dangerouslySetInnerHTML={{ __html: v.html }} />
        </Verse>
      ));
    }
    return null;
  }, [html, blocks]);

  const rootRef = useRef<HTMLDivElement>(null);
  useEngineTermHydration(rootRef);
  // Re-run hydration whenever the rendered content changes.
  useMicroYantraHydration(rootRef, [html, blocks]);
  useHeadingDecoration(rootRef, [html, blocks]);

  return (
    <div ref={rootRef} className="verse-flow">
      {rendered}
    </div>
  );
}
