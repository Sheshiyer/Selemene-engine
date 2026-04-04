"use client";

import { useEffect, useMemo, useRef, useState, type KeyboardEvent as ReactKeyboardEvent } from "react";
import { ModalSurface } from "@/components/overlay-surface";

function joinClasses(...values: Array<string | false | null | undefined>) {
  return values.filter(Boolean).join(" ");
}

export interface CommandPaletteItem {
  id: string;
  title: string;
  description?: string;
  section: string;
  keywords?: string[];
  shortcutHint?: string;
  danger?: boolean;
  onSelect: () => void;
}

interface CommandPaletteProps {
  open: boolean;
  onClose: () => void;
  items: CommandPaletteItem[];
}

function isTextEntryTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) {
    return false;
  }

  const tagName = target.tagName.toLowerCase();
  return (
    tagName === "input" ||
    tagName === "textarea" ||
    tagName === "select" ||
    target.isContentEditable
  );
}

export function useCommandPaletteToggle(onOpen: () => void) {
  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      if (isTextEntryTarget(event.target)) {
        return;
      }

      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        onOpen();
      }
    }

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [onOpen]);
}

export function CommandPalette({
  open,
  onClose,
  items
}: CommandPaletteProps) {
  const inputRef = useRef<HTMLInputElement | null>(null);
  const [query, setQuery] = useState("");
  const [selectedIndex, setSelectedIndex] = useState(0);

  useEffect(() => {
    if (!open) {
      return;
    }
    window.requestAnimationFrame(() => inputRef.current?.focus());
  }, [open]);

  const filteredItems = useMemo(() => {
    const normalized = query.trim().toLowerCase();
    if (!normalized) {
      return items;
    }

    return items.filter((item) => {
      const haystack = [
        item.title,
        item.description ?? "",
        item.section,
        ...(item.keywords ?? [])
      ]
        .join(" ")
        .toLowerCase();

      return haystack.includes(normalized);
    });
  }, [items, query]);

  const groupedItems = useMemo(() => {
    const groups: Array<{ section: string; items: Array<CommandPaletteItem & { index: number }> }> = [];
    filteredItems.forEach((item, index) => {
      const existing = groups.find((group) => group.section === item.section);
      if (existing) {
        existing.items.push({ ...item, index });
        return;
      }
      groups.push({ section: item.section, items: [{ ...item, index }] });
    });
    return groups;
  }, [filteredItems]);

  const activeIndex =
    filteredItems.length === 0 ? 0 : Math.min(selectedIndex, filteredItems.length - 1);

  function handleClose() {
    setQuery("");
    setSelectedIndex(0);
    onClose();
  }

  function activate(index: number) {
    const selected = filteredItems[index];
    if (!selected) {
      return;
    }

    handleClose();
    selected.onSelect();
  }

  function onListKeyDown(event: ReactKeyboardEvent<HTMLInputElement>) {
    if (event.key === "ArrowDown") {
      event.preventDefault();
      setSelectedIndex((current) =>
        filteredItems.length === 0 ? 0 : (Math.min(current, filteredItems.length - 1) + 1) % filteredItems.length
      );
      return;
    }

    if (event.key === "ArrowUp") {
      event.preventDefault();
      setSelectedIndex((current) =>
        filteredItems.length === 0
          ? 0
          : (Math.min(current, filteredItems.length - 1) - 1 + filteredItems.length) % filteredItems.length
      );
      return;
    }

    if (event.key === "Enter") {
      event.preventDefault();
      activate(activeIndex);
      return;
    }

    if (event.key === "Escape") {
      event.preventDefault();
      handleClose();
    }
  }

  return (
    <ModalSurface
      open={open}
      onClose={handleClose}
      eyebrow="Quick actions"
      title="Command Palette"
      summary="Navigate routes and run high-frequency admin actions from one keyboard-first surface."
      className="command-palette-modal"
      footer={
        <div className="command-palette-footer">
          <span>Use ↑ ↓ to move, Enter to run, Esc to close.</span>
          <span>Cmd/Ctrl + K to open.</span>
        </div>
      }
    >
      <div className="command-palette-body">
        <label className="command-palette-search">
          <span className="sr-only">Search commands</span>
          <input
            ref={inputRef}
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            onKeyDown={onListKeyDown}
            placeholder="Search routes and actions"
          />
        </label>

        {filteredItems.length === 0 ? (
          <div className="state-table-empty command-palette-empty">
            <div className="telemetry-caption">Empty</div>
            <div className="state-table-empty-title">No matching commands</div>
            <div className="helper">Try a route name, action, or keyword like audit, refresh, or sign out.</div>
          </div>
        ) : (
          <div className="command-palette-results" role="listbox" aria-label="Available commands">
            {groupedItems.map((group) => (
              <section key={group.section} className="command-palette-group">
                <div className="telemetry-caption command-palette-group-label">{group.section}</div>
                <div className="command-palette-group-items">
                  {group.items.map((item) => {
                    const active = item.index === activeIndex;
                    return (
                      <button
                        key={item.id}
                        type="button"
                        role="option"
                        aria-selected={active}
                        className={joinClasses(
                          "command-palette-item",
                          active && "active",
                          item.danger && "danger"
                        )}
                        onMouseEnter={() => setSelectedIndex(item.index)}
                        onClick={() => activate(item.index)}
                      >
                        <div className="command-palette-copy">
                          <div className="command-palette-title">{item.title}</div>
                          {item.description ? (
                            <div className="helper command-palette-description">{item.description}</div>
                          ) : null}
                        </div>
                        {item.shortcutHint ? (
                          <span className="command-palette-shortcut">{item.shortcutHint}</span>
                        ) : null}
                      </button>
                    );
                  })}
                </div>
              </section>
            ))}
          </div>
        )}
      </div>
    </ModalSurface>
  );
}
