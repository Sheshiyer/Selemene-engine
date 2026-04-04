"use client";

import { useMemo, useState, type ReactNode } from "react";

function joinClasses(...values: Array<string | false | null | undefined>) {
  return values.filter(Boolean).join(" ");
}

export function useBulkSelection(visibleIds: string[]) {
  const [selectedIds, setSelectedIds] = useState<string[]>([]);
  const visibleSet = useMemo(() => new Set(visibleIds), [visibleIds]);
  const selectedVisibleIds = useMemo(
    () => selectedIds.filter((id) => visibleSet.has(id)),
    [selectedIds, visibleSet]
  );
  const selectedSet = useMemo(() => new Set(selectedVisibleIds), [selectedVisibleIds]);
  const selectedCount = selectedVisibleIds.length;
  const allVisibleSelected =
    visibleIds.length > 0 && visibleIds.every((id) => selectedSet.has(id));

  function toggle(id: string) {
    setSelectedIds((current) =>
      current.includes(id) ? current.filter((item) => item !== id) : [...current, id]
    );
  }

  function toggleVisible() {
    setSelectedIds((current) => {
      if (visibleIds.length === 0) {
        return [];
      }

      const currentSet = new Set(current);
      const next = new Set(current);

      const shouldSelectAll = visibleIds.some((id) => !currentSet.has(id));
      for (const id of visibleIds) {
        if (shouldSelectAll) {
          next.add(id);
        } else {
          next.delete(id);
        }
      }

      return Array.from(next);
    });
  }

  function clear() {
    setSelectedIds([]);
  }

  return {
    selectedIds,
    selectedSet,
    selectedCount,
    allVisibleSelected,
    toggle,
    toggleVisible,
    clear
  };
}

interface BulkActionBarProps {
  className?: string;
  itemLabel: string;
  selectedCount: number;
  visibleCount: number;
  allVisibleSelected: boolean;
  onToggleVisible: () => void;
  onClear: () => void;
  children: ReactNode;
}

export function BulkActionBar({
  className,
  itemLabel,
  selectedCount,
  visibleCount,
  allVisibleSelected,
  onToggleVisible,
  onClear,
  children
}: BulkActionBarProps) {
  return (
    <section className={joinClasses("bulk-action-bar filigree-frame", className)} aria-label={`${itemLabel} bulk actions`}>
      <div className="bulk-action-copy">
        <div className="eyebrow">Bulk selection</div>
        <h3>{selectedCount === 0 ? `No ${itemLabel} selected` : `${selectedCount} ${itemLabel} selected`}</h3>
        <p className="helper">
          {visibleCount === 0
            ? `The current filters do not expose any ${itemLabel}.`
            : `${visibleCount} visible in the current table scope. Use shared selection controls before applying bulk actions.`}
        </p>
      </div>

      <div className="bulk-action-toolbar">
        <button type="button" onClick={onToggleVisible}>
          {allVisibleSelected ? "Clear visible" : "Select visible"}
        </button>
        <button type="button" onClick={onClear} disabled={selectedCount === 0}>
          Clear selection
        </button>
        {children}
      </div>
    </section>
  );
}
