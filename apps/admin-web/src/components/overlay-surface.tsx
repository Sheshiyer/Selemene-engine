"use client";

import { useEffect, useId, type ReactNode } from "react";

function joinClasses(...values: Array<string | false | null | undefined>) {
  return values.filter(Boolean).join(" ");
}

interface OverlayBaseProps {
  open: boolean;
  onClose: () => void;
  eyebrow?: string;
  title?: string;
  summary?: string;
  footer?: ReactNode;
  className?: string;
  children: ReactNode;
}

function useEscapeToClose(open: boolean, onClose: () => void) {
  useEffect(() => {
    if (!open) {
      return;
    }

    function onKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") {
        onClose();
      }
    }

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [open, onClose]);
}

export function ModalSurface({
  open,
  onClose,
  eyebrow,
  title,
  summary,
  footer,
  className,
  children
}: OverlayBaseProps) {
  useEscapeToClose(open, onClose);
  const titleId = useId();

  if (!open) {
    return null;
  }

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div
        className={joinClasses("modal-card overlay-surface", className)}
        onClick={(event) => event.stopPropagation()}
        role="dialog"
        aria-modal="true"
        aria-labelledby={title ? titleId : undefined}
      >
        {eyebrow || title || summary ? (
          <header className="overlay-surface-header">
            <div>
              {eyebrow ? <div className="eyebrow">{eyebrow}</div> : null}
              {title ? <h3 id={titleId}>{title}</h3> : null}
              {summary ? <p className="helper">{summary}</p> : null}
            </div>
            <button type="button" className="overlay-close" onClick={onClose} aria-label="Close dialog">
              Close
            </button>
          </header>
        ) : null}
        <div className="overlay-surface-body">{children}</div>
        {footer ? <footer className="modal-actions overlay-surface-footer">{footer}</footer> : null}
      </div>
    </div>
  );
}

export function DrawerSurface({
  open,
  onClose,
  eyebrow,
  title,
  summary,
  footer,
  className,
  children
}: OverlayBaseProps) {
  useEscapeToClose(open, onClose);
  const titleId = useId();

  if (!open) {
    return null;
  }

  return (
    <div className="modal-overlay drawer-overlay" onClick={onClose}>
      <aside
        className={joinClasses("drawer-surface filigree-frame", className)}
        onClick={(event) => event.stopPropagation()}
        role="dialog"
        aria-modal="true"
        aria-labelledby={title ? titleId : undefined}
      >
        <header className="overlay-surface-header">
          <div>
            {eyebrow ? <div className="eyebrow">{eyebrow}</div> : null}
            {title ? <h3 id={titleId}>{title}</h3> : null}
            {summary ? <p className="helper">{summary}</p> : null}
          </div>
          <button type="button" className="overlay-close" onClick={onClose} aria-label="Close drawer">
            Close
          </button>
        </header>
        <div className="overlay-surface-body">{children}</div>
        {footer ? <footer className="modal-actions overlay-surface-footer">{footer}</footer> : null}
      </aside>
    </div>
  );
}
