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
import { useRef, useMemo } from "react";
import type { ReactNode } from "react";
import type { Block } from "@/lib/integrated/parseBlocks";
import { BlockRenderer } from "./data/BlockRenderer";

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
              opacity: inView ? 1 : anchor ? 0.45 : 0.22,
              filter: inView ? "blur(0px)" : "blur(0.4px)",
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

  return <div className="verse-flow">{rendered}</div>;
}
