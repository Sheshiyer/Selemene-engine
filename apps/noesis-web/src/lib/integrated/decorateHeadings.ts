// ─── decorateHeadings — leading sacred-geometry marks on h2/h3 ─────────
// DOM walker, invoked from VerseFlow on mount, that prepends a small
// inline-SVG sigil to each <h2> and <h3> within the verse content. The
// h2 mark is a larger interlocking-triangle (10px), the h3 mark a
// smaller dot-trine cousin (8px). Idempotent — a heading already
// marked is skipped via a data-decorated flag.
//
// Per integrated-reading-design-v2.md § 5.5.

const STROKE = "var(--c-gold, #C5A017)";

function h2SigilSVG(): string {
  // Two interlocking triangles (Star of David / shatkona) — stylized
  // sacred geometry for section breaks.
  return `<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" aria-hidden="true" style="display:inline-block;vertical-align:0.06em;margin-right:0.5em;flex-shrink:0;opacity:0.85"><g stroke="${STROKE}" stroke-width="1.2" fill="none" stroke-linejoin="round"><path d="M12 3 L21 19 L3 19 Z"/><path d="M12 21 L3 5 L21 5 Z"/></g></svg>`;
}

function h3SigilSVG(): string {
  // Single trine — three dots in a triangular cluster.
  return `<svg xmlns="http://www.w3.org/2000/svg" width="11" height="11" viewBox="0 0 24 24" aria-hidden="true" style="display:inline-block;vertical-align:0.05em;margin-right:0.45em;flex-shrink:0;opacity:0.8"><g fill="${STROKE}"><circle cx="12" cy="5" r="2"/><circle cx="5" cy="18" r="2"/><circle cx="19" cy="18" r="2"/></g><g stroke="${STROKE}" stroke-width="0.6" opacity="0.4" fill="none"><path d="M12 5 L5 18 L19 18 Z"/></g></svg>`;
}

const SELECTOR = "h2:not([data-yantra-decorated]), h3:not([data-yantra-decorated])";

/**
 * Walk the given root and decorate each h2/h3 with a leading sigil.
 * Safe to re-run; already-decorated headings are skipped.
 *
 * @returns the number of headings decorated.
 */
export function decorateHeadings(root: HTMLElement | Document | null): number {
  if (!root) return 0;
  if (typeof window === "undefined") return 0;

  const headings = root.querySelectorAll<HTMLElement>(SELECTOR);
  let count = 0;
  headings.forEach((h) => {
    // Skip if already decorated
    if (h.dataset.yantraDecorated === "1") return;
    const tag = h.tagName.toLowerCase();
    const svg = tag === "h2" ? h2SigilSVG() : h3SigilSVG();

    // Insert as the first child so it appears just before the text.
    const wrap = document.createElement("span");
    wrap.className = "heading-yantra-mark";
    wrap.setAttribute("aria-hidden", "true");
    wrap.style.display = "inline-block";
    wrap.innerHTML = svg;
    h.insertBefore(wrap, h.firstChild);
    h.dataset.yantraDecorated = "1";
    count += 1;
  });
  return count;
}
