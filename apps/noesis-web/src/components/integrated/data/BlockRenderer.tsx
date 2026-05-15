"use client";

// ─── BlockRenderer — dispatches typed Block to the right component ─────
// HtmlBlocks flow through the existing VerseFlow logic (split & illuminate
// per top-level element). Specialised block kinds dispatch to W3
// components.

import type { Block } from "@/lib/integrated/parseBlocks";
import { HexagonTrio } from "./HexagonTrio";
import { SigilCascade } from "./SigilCascade";
import { DecisionPlate } from "./DecisionPlate";

interface BlockRendererProps {
  block: Block;
}

export function BlockRenderer({ block }: BlockRendererProps) {
  switch (block.kind) {
    case "hex-trio":
      return (
        <HexagonTrio
          subjects={block.subjects}
          columns={block.columns}
          rows={block.rows}
        />
      );
    case "cascade":
      return <SigilCascade entries={block.entries} />;
    case "decision":
      return <DecisionPlate marker={block.marker} />;
    case "html":
      // html blocks are handled by VerseFlow in its block-mode path
      return null;
    default:
      return null;
  }
}
