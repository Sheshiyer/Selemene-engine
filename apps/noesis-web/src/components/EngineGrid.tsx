"use client";

/**
 * EngineGrid — hex honeycomb engine selector.
 *
 * Replaces the previous 5-column rounded-square bento layout.
 * Renders 17 engines (or any subset returned by a compass workflow)
 * as a staggered hex grid that collapses cleanly on mobile.
 *
 * Architecture:
 *   - Each cell uses CSS clip-path: polygon(...) for the hex shape.
 *   - Rows of 4 hexes alternate with rows of 3 hexes, with the
 *     3-row offset by half a cell so the geometry interlocks
 *     (3-4-3-4-3 for 17 cells; falls back to flat-flex for smaller sets).
 *   - The center cell (when 17 engines, index 8) is the WITNESS hub,
 *     rendered slightly larger with sacred-gold border.
 *   - States: inactive (parchment 60%), hover (coherence-emerald edge),
 *     active (sacred-gold edge + flow-indigo glow), has-data (emerald dot).
 *
 * Mobile (<= 640px) collapses to a flat 3-column grid (still hex cells)
 * because staggered hex math gets brittle below ~360px.
 *
 * No bento, no rectangles. Sacred geometry as load-bearing architecture.
 */

import { useId } from "react";
import type { CompassMode } from "@/components/CompassSelector";

export interface EngineGridItem {
  id: string;
  label: string;
  sigil: string;
  direction: Exclude<CompassMode, "full-spectrum">;
}

const directionColor: Record<EngineGridItem["direction"], string> = {
  stabilize: "var(--c-violet)",
  heal: "var(--c-indigo)",
  create: "var(--c-emerald)",
  mutate: "var(--signal)",
};

/* ── Hex layout planning ────────────────────────────────
 * For exactly 17 engines, lay out as 3-4-3-4-3 staggered rows.
 * The middle cell (row 2, col 1 of the second 4-row) is the
 * WITNESS hub by convention — index 8 in zero-based ordering.
 * For other lengths (compass workflows), fall back to a single
 * flex row that wraps — same hex cells, just no staggering.
 * ──────────────────────────────────────────────────────── */

interface RowSpec {
  count: number;
  offset: boolean; // half-cell horizontal offset
}

const HONEYCOMB_17: RowSpec[] = [
  { count: 3, offset: true },
  { count: 4, offset: false },
  { count: 3, offset: true },
  { count: 4, offset: false },
  { count: 3, offset: true },
];

function planRows(items: EngineGridItem[]): RowSpec[] | null {
  if (items.length === 17) return HONEYCOMB_17;
  return null;
}

/* ── Component ──────────────────────────────────────── */

interface EngineGridProps {
  engines: EngineGridItem[];
  activeEngineId: string;
  availableEngineIds: Set<string>;
  onSelect: (engineId: string) => void;
}

export default function EngineGrid({
  engines,
  activeEngineId,
  availableEngineIds,
  onSelect,
}: EngineGridProps) {
  const labelId = useId();
  const honeycomb = planRows(engines);

  return (
    <section
      className="engine-honeycomb"
      aria-labelledby={labelId}
    >
      <header className="engine-honeycomb-header">
        <h2 id={labelId} className="engine-honeycomb-title">
          Engine Mandala
        </h2>
        <span className="engine-honeycomb-meta">
          {engines.length} {engines.length === 1 ? "engine" : "engines"}
        </span>
      </header>

      {honeycomb ? (
        <div className="hex-honeycomb" role="tablist" aria-label="Engine selector">
          {(() => {
            let cursor = 0;
            return honeycomb.map((row, rowIdx) => {
              const slice = engines.slice(cursor, cursor + row.count);
              const startIdx = cursor;
              cursor += row.count;
              return (
                <div
                  key={rowIdx}
                  className={`hex-row ${row.offset ? "hex-row-offset" : ""}`}
                >
                  {slice.map((engine, i) => {
                    const cellIdx = startIdx + i;
                    const isCenter = cellIdx === 8; // WITNESS hub for 17
                    return (
                      <HexCell
                        key={engine.id}
                        engine={engine}
                        active={activeEngineId === engine.id}
                        hasData={availableEngineIds.has(engine.id)}
                        hub={isCenter}
                        delayMs={cellIdx * 28}
                        onSelect={onSelect}
                      />
                    );
                  })}
                </div>
              );
            });
          })()}
        </div>
      ) : (
        <div className="hex-flow" role="tablist" aria-label="Engine selector">
          {engines.map((engine, idx) => (
            <HexCell
              key={engine.id}
              engine={engine}
              active={activeEngineId === engine.id}
              hasData={availableEngineIds.has(engine.id)}
              hub={false}
              delayMs={idx * 30}
              onSelect={onSelect}
            />
          ))}
        </div>
      )}
    </section>
  );
}

/* ── HexCell ────────────────────────────────────────── */

interface HexCellProps {
  engine: EngineGridItem;
  active: boolean;
  hasData: boolean;
  hub: boolean;
  delayMs: number;
  onSelect: (id: string) => void;
}

function HexCell({
  engine,
  active,
  hasData,
  hub,
  delayMs,
  onSelect,
}: HexCellProps) {
  const sigilColor = active ? "var(--signal)" : directionColor[engine.direction];

  return (
    <button
      type="button"
      role="tab"
      aria-selected={active}
      aria-label={engine.label}
      onClick={() => onSelect(engine.id)}
      className={[
        "hex-cell",
        active ? "is-active" : "",
        hub ? "is-hub" : "",
      ]
        .filter(Boolean)
        .join(" ")}
      style={{ animationDelay: `${delayMs}ms` }}
    >
      <span className="hex-cell-frame" aria-hidden>
        <span className="hex-cell-inner">
          <span
            className="hex-cell-sigil"
            style={{ color: sigilColor }}
          >
            {engine.sigil}
          </span>
        </span>
        {hasData && <span className="hex-cell-pulse" aria-hidden />}
      </span>
      <span className="hex-cell-label">{engine.label}</span>
    </button>
  );
}
