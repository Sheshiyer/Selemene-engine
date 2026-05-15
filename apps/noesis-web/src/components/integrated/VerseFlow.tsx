"use client";

// ─── VerseFlow — scroll-illuminated reading column ──────────────────────
// Per design MD § 3.3. Takes parsed HTML (from marked) and wraps each
// top-level block (<p>, headings, lists, blockquotes, tables) as a
// <Verse>. Verses default to dim opacity 0.22 + 0.4px blur; illuminate
// to full opacity as they enter viewport-center.
//
// Long paragraphs (>160 words, 3+ sentences) split into sentence-grouped
// sub-verses for the 2-3 line focus rhythm the user asked for.

import { motion, useInView } from "motion/react";
import { useRef, useMemo } from "react";
import type { ReactNode } from "react";

interface VerseProps {
  children: ReactNode;
  anchor?: boolean;
}

function Verse({ children, anchor = false }: VerseProps) {
  const ref = useRef<HTMLDivElement>(null);
  const inView = useInView(ref, { margin: "-35% 0% -35% 0%", once: false });
  return (
    <motion.div
      ref={ref}
      style={{
        margin: "0 0 22px",
        willChange: "opacity, filter",
      }}
      initial={false}
      animate={{
        opacity: inView ? 1 : (anchor ? 0.45 : 0.22),
        filter: inView ? "blur(0px)" : "blur(0.4px)",
      }}
      transition={{ duration: 0.6, ease: [0.2, 0.7, 0.2, 1] }}
    >
      {children}
    </motion.div>
  );
}

interface VerseFlowProps {
  /** Pre-parsed HTML (from marked or similar) */
  html: string;
}

/**
 * Server-side splitting: take rendered HTML and break it into top-level
 * blocks. We do this with a permissive regex that matches paragraph-level
 * elements. Long <p> blocks get sentence-split into multiple verses.
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

export function VerseFlow({ html }: VerseFlowProps) {
  const verses = useMemo(() => splitIntoVerses(html), [html]);
  return (
    <div className="verse-flow">
      {verses.map((v, i) => (
        <Verse key={i} anchor={v.kind === "anchor"}>
          <div dangerouslySetInnerHTML={{ __html: v.html }} />
        </Verse>
      ))}
    </div>
  );
}
