import './engine-room.css';
import { createEngineAssemblyScene } from './engine-assembly-scene.js';

const SOURCE_ROOT = 'https://github.com/Sheshiyer/Selemene-engine/blob/';
let roomSequence = 0;

const ICONS = {
  back: '<path d="m13 5-7 7 7 7M6 12h15"/>',
  previous: '<path d="m14 6-6 6 6 6"/>',
  next: '<path d="m10 6 6 6-6 6"/>',
  recenter: '<path d="M8 3H3v5m13-5h5v5M3 16v5h5m13-5v5h-5"/><circle cx="12" cy="12" r="3"/>',
  link: '<path d="m10 13 4-4m-5 7-1 1a4 4 0 0 1-6-6l4-4a4 4 0 0 1 6 0m0 10a4 4 0 0 0 6 0l4-4a4 4 0 0 0-6-6l-1 1"/>',
  explode: '<path d="m12 2 9 5-9 5-9-5 9-5Zm-9 10 9 5 9-5M3 17l9 5 9-5"/>',
  assemble: '<path d="m12 5 9 5-9 5-9-5 9-5Zm-9 9 9 5 9-5M12 1v3m0 16v3"/>',
  assembly: '<path d="m12 3 9 5-9 5-9-5 9-5ZM3 8v9l9 5 9-5V8m-9 5v9"/>',
  external: '<path d="M14 3h7v7m0-7L10 14M10 3H3v18h18v-7"/>',
  check: '<path d="m5 12 4 4L19 6"/>',
};

const STATUS = {
  implemented: { label: 'Implemented', description: 'This responsibility is present in the inspected source.' },
  simplified: { label: 'Simplified', description: 'The source uses a bounded or simplified implementation of this responsibility.' },
  delegated: { label: 'Delegated', description: 'This responsibility is handed to another engine, service, or library.' },
  placeholder: { label: 'Placeholder', description: 'The source reserves this responsibility without a complete active implementation.' },
};

function icon(name) {
  return `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.35" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${ICONS[name] || ICONS.assembly}</svg>`;
}

function element(tag, className, text) {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text != null) node.textContent = String(text);
  return node;
}

function clamp(value, minimum = 0, maximum = 1) {
  const number = Number(value);
  return Number.isFinite(number) ? Math.min(maximum, Math.max(minimum, number)) : minimum;
}

function normalizeStatus(value) {
  const key = String(value || '').toLowerCase();
  return STATUS[key] ? { key, ...STATUS[key] } : {
    key: 'unspecified',
    label: value ? String(value) : 'Source component',
    description: 'Read the component description and source for its implementation boundaries.',
  };
}

/**
 * A source-backed, independently disposable engine dialog.
 * State callbacks are synchronous; the caller owns body selection and URL state.
 */
export function createEngineRoom({
  catalog = [], nodes = [], koshas = [], revision = '',
  onOpen, onClose, onEngineChange, onStateChange,
} = {}) {
  const engines = Array.isArray(catalog) ? catalog : [];
  const byId = new Map(engines.map(engine => [engine.id, engine]));
  const nodeById = new Map(nodes.map(node => [node.id, node]));
  const koshaById = new Map(koshas.map(kosha => [kosha.id, kosha]));
  const prefix = `engine-room-${++roomSequence}`;
  const lifetime = new AbortController();
  const motionPreference = window.matchMedia('(prefers-reduced-motion: reduce)');
  let reducedMotion = motionPreference.matches;
  let opened = false;
  let disposed = false;
  let engine = null;
  let sceneEngine = null;
  let renderer = null;
  let sceneVersion = 0;
  let frameRequest = 0;
  let opener = null;
  let noticeTimer = 0;
  let hoveredPart = null;
  let state = { engineId: null, explosion: 0, partId: null, isolated: false };

  const dialog = element('dialog', 'engine-room');
  dialog.id = prefix;
  dialog.setAttribute('aria-labelledby', `${prefix}-title`);
  dialog.setAttribute('aria-describedby', `${prefix}-summary`);
  dialog.dataset.sceneState = 'idle';
  dialog.innerHTML = `
    <div class="engine-room-shell">
      <header class="engine-room-header">
        <button type="button" class="engine-room-return" data-room-action="close">${icon('back')}<span>Return to body</span><kbd>ESC</kbd></button>
        <div class="engine-room-brand"><span class="engine-room-sigil" aria-hidden="true"></span><span>SELEMENE<small>THE INNER ARCHITECTURE</small></span></div>
        <span class="engine-room-edition">ENGINE ATLAS <span aria-hidden="true">/</span> <span data-room-ref="revision"></span></span>
      </header>

      <section class="engine-room-heading">
        <div class="engine-room-heading-copy">
          <div class="engine-room-kicker"><span data-room-ref="category"></span><span class="engine-room-category-note">EDITORIAL CORRESPONDENCE</span></div>
          <h2 id="${prefix}-title"></h2>
          <p id="${prefix}-summary" class="engine-room-summary"></p>
        </div>
        <div class="engine-room-navigation">
          <div class="engine-room-navigation-label"><label for="${prefix}-engine">EXPLORE AN ENGINE</label><span data-room-ref="position"></span></div>
          <div class="engine-room-engine-switch">
            <button type="button" class="engine-room-icon-button" data-room-action="previous" aria-label="Previous engine">${icon('previous')}</button>
            <select id="${prefix}-engine" aria-label="Choose engine"></select>
            <button type="button" class="engine-room-icon-button" data-room-action="next" aria-label="Next engine">${icon('next')}</button>
          </div>
        </div>
      </section>

      <div class="engine-room-workspace">
        <section class="engine-room-stage" aria-label="Interactive engine assembly">
          <div class="engine-room-canvas" aria-describedby="${prefix}-scene-help"></div>
          <div class="engine-room-stage-heading">
            <span class="engine-room-overline">SOURCE ASSEMBLY <span aria-hidden="true">/</span> <span data-room-ref="part-count"></span></span>
            <span class="engine-room-view-state" data-room-ref="view-state">Assembled</span>
          </div>
          <div class="engine-room-stage-tools">
            <button type="button" class="engine-room-tool" data-room-action="recenter" aria-label="Recenter engine" title="Recenter engine" disabled>${icon('recenter')}<span>Recenter</span></button>
            <button type="button" class="engine-room-tool" data-room-action="copy" aria-label="Copy engine view" title="Copy engine view">${icon('link')}<span>Copy view</span></button>
          </div>
          <button type="button" class="engine-room-context-button" data-room-action="context" hidden>${icon('assembly')}<span>Show assembly</span></button>
          <div class="engine-room-scene-message" role="status" data-room-ref="scene-message"><span class="engine-room-sigil" aria-hidden="true"></span><p>Preparing the assembly.</p></div>
          <div class="engine-room-tooltip" hidden></div>
          <div class="engine-room-stage-legend" id="${prefix}-scene-help"><span><i aria-hidden="true"></i>ONE FORM · ONE SOURCE COMPONENT</span><span data-room-ref="scene-help">DRAG TO ORBIT · SCROLL TO ZOOM</span></div>
          <p class="engine-room-geometry-note">Geometry is interpretive. Component roles follow the source.</p>
        </section>

        <aside class="engine-room-inspector" aria-label="Component insight">
          <section class="engine-room-insight" aria-live="polite" aria-atomic="true">
            <div class="engine-room-insight-top"><span class="engine-room-overline" data-room-ref="insight-label">THE ASSEMBLY</span><span class="engine-room-part-number" data-room-ref="part-number"></span></div>
            <h3 data-room-ref="part-title"></h3>
            <p class="engine-room-part-role" data-room-ref="part-role"></p>
            <p class="engine-room-part-description" data-room-ref="part-description"></p>
            <div class="engine-room-status" data-room-ref="part-status" hidden><span class="engine-room-status-badge" data-room-ref="status-label"></span><p data-room-ref="status-description"></p></div>
            <div class="engine-room-source" data-room-ref="part-source"></div>
            <div class="engine-room-relations" data-room-ref="relations" hidden></div>
            <div class="engine-room-component-navigation"><button type="button" data-room-action="next-component"><span>Next component</span>${icon('next')}</button><button type="button" data-room-action="full-assembly" hidden>Back to full assembly</button></div>
          </section>
          <section class="engine-room-parts" aria-labelledby="${prefix}-parts-title">
            <div class="engine-room-section-heading"><h4 id="${prefix}-parts-title">COMPONENTS</h4><span>SELECT TO ISOLATE</span></div>
            <ol class="engine-room-part-list"></ol>
          </section>
          <section class="engine-room-source-note"><span class="engine-room-overline">IMPLEMENTATION BOUNDARY</span><p data-room-ref="status-note"></p></section>
          <figure class="engine-room-artwork" data-room-ref="artwork" hidden><img alt="" loading="lazy" decoding="async" /><figcaption><span class="engine-room-overline">INSTRUMENT STUDY</span><span data-room-ref="artwork-label"></span><small>Brand visual · interpretive</small></figcaption></figure>
          <details class="engine-room-reconciliation" data-room-ref="reconciliation" hidden><summary>Code &amp; docs reconciliation<span aria-hidden="true">+</span></summary><div data-room-ref="reconciliation-content"></div></details>
        </aside>
      </div>

      <footer class="engine-room-controls">
        <button type="button" class="engine-room-explode" data-room-action="explode">${icon('explode')}<span>Explode engine</span></button>
        <div class="engine-room-slider-group">
          <div class="engine-room-slider-labels"><button type="button" data-room-action="assemble">ASSEMBLED</button><label for="${prefix}-explosion" class="engine-room-slider-instruction">Reveal the architecture</label><button type="button" data-room-action="expand">EXPANDED</button></div>
          <input id="${prefix}-explosion" class="engine-room-range" type="range" min="0" max="100" step="1" value="0" aria-label="Assembly expansion" aria-valuetext="0 percent expanded" />
          <div class="engine-room-slider-bottom"><span data-room-ref="motion-note">DRAG TO UNFOLD · CLICK A FORM TO INSPECT</span><span>STRUCTURE <span aria-hidden="true">──────────</span> RELATIONSHIP</span></div>
        </div>
        <div class="engine-room-expansion-readout"><output for="${prefix}-explosion" data-room-ref="expansion">00</output><span>% EXPANDED</span></div>
        <div class="engine-room-notice" role="status" aria-live="polite" data-room-ref="notice"></div>
        <input class="engine-room-share-fallback" aria-label="Current engine view link" readonly hidden />
      </footer>
    </div>`;
  document.body.append(dialog);

  const refs = Object.fromEntries([...dialog.querySelectorAll('[data-room-ref]')].map(node => [node.dataset.roomRef, node]));
  const actions = Object.fromEntries([...dialog.querySelectorAll('[data-room-action]')].map(node => [node.dataset.roomAction, node]));
  const title = dialog.querySelector(`#${prefix}-title`);
  const summary = dialog.querySelector(`#${prefix}-summary`);
  const engineSelect = dialog.querySelector(`#${prefix}-engine`);
  const range = dialog.querySelector(`#${prefix}-explosion`);
  const canvasHost = dialog.querySelector('.engine-room-canvas');
  const stage = dialog.querySelector('.engine-room-stage');
  const partList = dialog.querySelector('.engine-room-part-list');
  const inspector = dialog.querySelector('.engine-room-inspector');
  const workspace = dialog.querySelector('.engine-room-workspace');
  const tooltip = dialog.querySelector('.engine-room-tooltip');
  const shareFallback = dialog.querySelector('.engine-room-share-fallback');
  const partButtons = new Map();

  for (const item of engines) engineSelect.append(new Option(item.label || item.id, item.id));
  refs.revision.textContent = revision ? revision.slice(0, 8) : 'SOURCE STUDY';
  engineSelect.disabled = !engines.length;
  actions.previous.disabled = engines.length < 2;
  actions.next.disabled = engines.length < 2;

  function listen(target, event, callback, options = {}) {
    target.addEventListener(event, callback, { ...options, signal: lifetime.signal });
  }

  function getState() { return { ...state }; }
  function notify() { onStateChange?.(getState()); }

  function sourceLink(source, context = {}) {
    const record = typeof source === 'string' ? { path: source } : source;
    const path = record?.path || record?.file || record?.source;
    if (!path || !revision || typeof path !== 'string') return null;
    const [rawPath, fragment = ''] = path.split('#');
    if (/^[a-z]+:/i.test(rawPath) || rawPath.startsWith('/') || rawPath.split('/').includes('..')) return null;
    const line = Number(record.line || context.line);
    const anchor = Number.isInteger(line) && line > 0 ? `#L${line}` : /^L\d+(?:-L\d+)?$/.test(fragment) ? `#${fragment}` : '';
    const href = `${SOURCE_ROOT}${encodeURIComponent(revision)}/${rawPath.split('/').map(encodeURIComponent).join('/')}${anchor}`;
    const link = element('a', 'engine-room-source-link');
    link.href = href;
    link.target = '_blank';
    link.rel = 'noopener noreferrer';
    const label = element('span', '', rawPath + (anchor ? ` · ${anchor.slice(1)}` : ''));
    link.append(label);
    const glyph = element('span', 'engine-room-external-icon');
    glyph.innerHTML = icon('external');
    link.append(glyph);
    return link;
  }

  function renderSource(target, source, context) {
    target.replaceChildren();
    const link = sourceLink(source, context);
    if (!link) { target.hidden = true; return; }
    target.hidden = false;
    target.append(element('span', 'engine-room-overline', 'PINNED SOURCE'), link);
    const symbol = context?.symbol || (typeof source === 'object' && source?.symbol);
    if (symbol) target.append(element('code', 'engine-room-source-symbol', symbol));
  }

  function renderReconciliation() {
    const items = Array.isArray(engine.reconciliations) ? engine.reconciliations : [];
    const docs = Array.isArray(engine.docs) ? engine.docs : [];
    refs.reconciliation.hidden = !items.length && !docs.length;
    refs['reconciliation-content'].replaceChildren();
    refs.reconciliation.open = false;
    for (const item of items) {
      const row = element('div', 'engine-room-reconciliation-item');
      if (item.claim) row.append(element('span', 'engine-room-overline', 'DOCUMENTED CLAIM'), element('p', '', item.claim));
      if (item.truth) row.append(element('span', 'engine-room-overline', 'SOURCE TRUTH'), element('p', 'engine-room-source-truth', item.truth));
      const link = sourceLink(item.source, item);
      if (link) row.append(link);
      refs['reconciliation-content'].append(row);
    }
    if (docs.length) {
      const sources = element('div', 'engine-room-doc-links');
      sources.append(element('span', 'engine-room-overline', 'READ THE DOCUMENTS'));
      for (const doc of docs) {
        const link = sourceLink(doc);
        if (link) sources.append(link);
      }
      if (sources.childElementCount > 1) refs['reconciliation-content'].append(sources);
    }
  }

  function renderRelations(part) {
    refs.relations.replaceChildren();
    const edges = (engine.connections || []).filter(edge => edge.from === part?.id || edge.to === part?.id);
    refs.relations.hidden = !part || !edges.length;
    if (!part || !edges.length) return;
    refs.relations.append(element('span', 'engine-room-overline', 'COMPONENT FLOW'));
    const list = element('ul');
    for (const edge of edges) {
      const from = engine.parts.find(item => item.id === edge.from);
      const to = engine.parts.find(item => item.id === edge.to);
      const row = element('li');
      row.append(element('span', 'engine-room-flow-route', `${from?.label || edge.from} → ${to?.label || edge.to}`));
      if (edge.label) row.append(element('span', 'engine-room-flow-description', edge.label));
      list.append(row);
    }
    refs.relations.append(list);
  }

  function renderInsight() {
    const index = engine.parts.findIndex(part => part.id === state.partId);
    const part = engine.parts[index];
    refs['insight-label'].textContent = part ? (state.isolated ? 'ISOLATED COMPONENT' : 'SELECTED COMPONENT') : 'THE ASSEMBLY';
    refs['part-number'].textContent = part ? `${String(index + 1).padStart(2, '0')} / ${String(engine.parts.length).padStart(2, '0')}` : '';
    refs['part-title'].textContent = part?.label || 'A system, unfolded.';
    refs['part-role'].textContent = part?.role || 'Select a component to see its role.';
    refs['part-description'].textContent = part?.description || engine.summary || '';
    refs['part-status'].hidden = !part;
    if (part) {
      const status = normalizeStatus(part.status);
      refs['part-status'].dataset.status = status.key;
      refs['status-label'].textContent = status.label;
      refs['status-description'].textContent = status.description;
    }
    renderSource(refs['part-source'], part?.source || (!part && nodeById.get(engine.id)?.source), part);
    renderRelations(part);
    actions.context.hidden = !state.partId;
    actions['full-assembly'].hidden = !state.partId;
    actions['next-component'].disabled = !engine.parts.length;
    for (const [partId, button] of partButtons) {
      const selected = partId === state.partId;
      button.setAttribute('aria-pressed', String(selected));
      button.dataset.isolated = String(selected && state.isolated);
      button.title = selected && state.isolated ? 'Show the full assembly' : `Isolate ${button.dataset.label}`;
    }
    dialog.dataset.isolated = String(state.isolated);
  }

  function renderExpansion() {
    const percent = Math.round(state.explosion * 100);
    range.value = String(percent);
    range.style.setProperty('--room-progress', `${percent}%`);
    range.setAttribute('aria-valuetext', `${percent} percent expanded`);
    refs.expansion.textContent = String(percent).padStart(2, '0');
    refs['view-state'].textContent = state.isolated ? 'Component isolated' : percent === 0 ? 'Assembled' : percent === 100 ? 'Fully expanded' : 'Unfolding';
    const expanded = state.explosion >= 0.5;
    actions.explode.innerHTML = `${icon(expanded ? 'assemble' : 'explode')}<span>${expanded ? 'Reassemble' : 'Explode engine'}</span>`;
    actions.explode.dataset.expanded = String(expanded);
    actions.assemble.setAttribute('aria-pressed', String(percent === 0));
    actions.expand.setAttribute('aria-pressed', String(percent === 100));
    refs['motion-note'].textContent = reducedMotion ? 'REDUCED MOTION · POSITION CHANGES ARE IMMEDIATE' : 'DRAG TO UNFOLD · CLICK A FORM TO INSPECT';
  }

  function renderEngine() {
    const node = nodeById.get(engine.id);
    const kosha = koshaById.get(node?.kosha || node?.family) || koshas.find(item => item.engines?.includes(engine.id));
    sceneEngine = { ...engine, color: kosha?.color || engine.color || '#C5A017' };
    dialog.style.setProperty('--room-category', kosha?.color || '#C5A017');
    dialog.dataset.engine = engine.id;
    title.textContent = engine.label || node?.label || engine.id;
    summary.textContent = engine.summary || '';
    refs.category.textContent = kosha ? `${kosha.label} · ${kosha.meaning || 'Pancha Kosha'}` : 'Selemene · Engine anatomy';
    refs.position.textContent = `${String(engines.indexOf(engine) + 1).padStart(2, '0')} / ${String(engines.length).padStart(2, '0')}`;
    refs['part-count'].textContent = `${engine.parts.length} COMPONENT${engine.parts.length === 1 ? '' : 'S'}`;
    refs['status-note'].textContent = engine.statusNote || 'Inspect each component for its source-defined role and implementation boundary.';
    const artwork = typeof engine.artwork === 'string' ? engine.artwork : null;
    const artworkImage = refs.artwork.querySelector('img');
    refs.artwork.hidden = true;
    artworkImage.removeAttribute('src');
    if (artwork) {
      try {
        const artworkURL = new URL(artwork, window.location.href);
        if (artworkURL.origin === window.location.origin) {
          artworkImage.src = artworkURL.href;
          artworkImage.alt = `${engine.label || engine.id} instrument study`;
          refs['artwork-label'].textContent = engine.label || engine.id;
          refs.artwork.hidden = false;
        }
      } catch { /* Missing art never prevents source exploration. */ }
    }
    engineSelect.value = engine.id;
    canvasHost.setAttribute('aria-label', `${engine.label || engine.id} component assembly`);
    partList.replaceChildren();
    partButtons.clear();
    engine.parts.forEach((part, index) => {
      const item = element('li');
      const button = element('button', 'engine-room-part-button');
      button.type = 'button';
      button.dataset.partId = part.id;
      button.dataset.label = part.label;
      button.setAttribute('aria-pressed', 'false');
      const number = element('span', 'engine-room-part-index', String(index + 1).padStart(2, '0'));
      const name = element('span', 'engine-room-part-name', part.label);
      const status = normalizeStatus(part.status);
      const statusLabel = element('span', 'engine-room-part-status', status.label);
      statusLabel.dataset.status = status.key;
      const selectedIcon = element('span', 'engine-room-part-check');
      selectedIcon.innerHTML = icon('check');
      button.append(number, name, statusLabel, selectedIcon);
      item.append(button);
      partList.append(item);
      partButtons.set(part.id, button);
    });
    renderReconciliation();
    renderInsight();
    renderExpansion();
    hideHover();
    inspector.scrollTop = 0;
    shareFallback.hidden = true;
    refs.notice.textContent = '';
  }

  function validState(engineId, initial = {}) {
    const entry = byId.get(engineId);
    const partId = entry?.parts.some(part => part.id === initial.partId) ? initial.partId : null;
    return { engineId, explosion: clamp(initial.explosion), partId, isolated: Boolean(partId && initial.isolated) };
  }

  function syncRenderer({ immediate = false } = {}) {
    if (!renderer) return;
    renderer.setEngine(sceneEngine);
    renderer.setExplosion(state.explosion, { immediate });
    renderer.setSelectedPart(state.partId);
    renderer.setIsolatedPart(state.isolated ? state.partId : null);
    renderer.setReducedMotion(reducedMotion);
    renderer.resize();
  }

  function changeEngine(engineId, initial = {}) {
    if (!byId.has(engineId) || disposed) return false;
    engine = byId.get(engineId);
    state = validState(engineId, initial);
    renderEngine();
    syncRenderer({ immediate: true });
    onEngineChange?.(engineId, getState());
    notify();
    return true;
  }

  function selectPart(partId, { reveal = false } = {}) {
    if (!opened || !engine) return;
    if (partId != null && !engine.parts.some(part => part.id === partId)) return;
    const returning = partId === null || (state.partId === partId && state.isolated);
    const restoreContextFocus = returning && [actions.context, actions['full-assembly']].includes(document.activeElement);
    state.partId = returning ? null : partId;
    state.isolated = !returning;
    renderer?.setSelectedPart(state.partId);
    renderer?.setIsolatedPart(state.isolated ? state.partId : null);
    renderInsight();
    renderExpansion();
    hideHover();
    inspector.scrollTop = 0;
    if (reveal && getComputedStyle(inspector).overflowY === 'visible') {
      const top = inspector.getBoundingClientRect().top - workspace.getBoundingClientRect().top + workspace.scrollTop;
      workspace.scrollTo({ top, behavior: reducedMotion ? 'auto' : 'smooth' });
    }
    if (restoreContextFocus) actions['next-component'].focus({ preventScroll: true });
    notify();
  }

  function setExplosion(value, { immediate = false } = {}) {
    if (!opened) return;
    state.explosion = clamp(value);
    renderer?.setExplosion(state.explosion, { immediate: immediate || reducedMotion });
    renderExpansion();
    notify();
  }

  function hideHover() {
    hoveredPart = null;
    tooltip.hidden = true;
  }

  function showHover(partId, coordinates = {}) {
    if (!opened || !partId) { hideHover(); return; }
    coordinates ||= {};
    const part = engine.parts.find(item => item.id === partId);
    if (!part) { hideHover(); return; }
    if (hoveredPart !== partId) {
      hoveredPart = partId;
      tooltip.replaceChildren(element('strong', '', part.label), element('span', '', `${normalizeStatus(part.status).label} · Click to isolate`));
    }
    tooltip.hidden = false;
    const rect = stage.getBoundingClientRect();
    const x = Number.isFinite(coordinates.x) ? coordinates.x : Number.isFinite(coordinates.clientX) ? coordinates.clientX - rect.left : rect.width / 2;
    const y = Number.isFinite(coordinates.y) ? coordinates.y : Number.isFinite(coordinates.clientY) ? coordinates.clientY - rect.top : rect.height / 2;
    tooltip.style.left = `${clamp(x + 15, 12, Math.max(12, rect.width - tooltip.offsetWidth - 12))}px`;
    tooltip.style.top = `${clamp(y + 16, 54, Math.max(54, rect.height - tooltip.offsetHeight - 66))}px`;
  }

  function showSceneError() {
    if (!opened) return;
    dialog.dataset.sceneState = 'error';
    refs['scene-message'].hidden = false;
    refs['scene-message'].querySelector('p').textContent = 'The 3D view is unavailable here. Explore every source component in the inspector.';
    refs['scene-help'].textContent = 'COMPONENT INSIGHT REMAINS AVAILABLE';
    actions.recenter.disabled = true;
  }

  function releaseScene() {
    sceneVersion += 1;
    cancelAnimationFrame(frameRequest);
    frameRequest = 0;
    if (renderer) {
      renderer.setActive(false);
      renderer.dispose();
      renderer = null;
    }
    canvasHost.replaceChildren();
    hideHover();
    dialog.dataset.sceneState = 'idle';
    actions.recenter.disabled = true;
  }

  function initializeScene() {
    const version = ++sceneVersion;
    dialog.dataset.sceneState = 'loading';
    refs['scene-message'].hidden = false;
    refs['scene-message'].querySelector('p').textContent = 'Preparing the assembly.';
    frameRequest = requestAnimationFrame(() => {
      if (!opened || disposed || version !== sceneVersion) return;
      let created;
      try {
        created = createEngineAssemblyScene(canvasHost, {
          reducedMotion,
          onSelect: partId => { if (opened && version === sceneVersion) selectPart(partId); },
          onHover: (partId, coordinates) => { if (opened && version === sceneVersion) showHover(partId, coordinates); },
          onReady: () => {
            if (!opened || version !== sceneVersion) return;
            dialog.dataset.sceneState = 'ready';
            refs['scene-message'].hidden = true;
            refs['scene-help'].textContent = 'DRAG TO ORBIT · SCROLL TO ZOOM';
            actions.recenter.disabled = false;
          },
          onError: () => { if (version === sceneVersion) showSceneError(); },
        });
      } catch {
        showSceneError();
        return;
      }
      Promise.resolve(created).then(instance => {
        if (!instance) { if (version === sceneVersion) showSceneError(); return; }
        if (!opened || disposed || version !== sceneVersion) { instance.dispose(); return; }
        renderer = instance;
        const available = typeof instance.isAvailable === 'function' ? instance.isAvailable() : instance.isAvailable;
        if (available === false) { showSceneError(); return; }
        syncRenderer({ immediate: true });
        renderer.setActive(!document.hidden);
        dialog.dataset.sceneState = 'ready';
        refs['scene-message'].hidden = true;
        refs['scene-help'].textContent = 'DRAG TO ORBIT · SCROLL TO ZOOM';
        actions.recenter.disabled = false;
      }).catch(() => { if (version === sceneVersion) showSceneError(); });
    });
  }

  function showNotice(message) {
    clearTimeout(noticeTimer);
    refs.notice.textContent = message;
    noticeTimer = window.setTimeout(() => { refs.notice.textContent = ''; }, 4500);
  }

  function open(engineId, initialState) {
    if (disposed || !byId.has(engineId)) return false;
    if (opened) {
      if (engine.id !== engineId || initialState) changeEngine(engineId, initialState);
      return true;
    }
    opener = document.activeElement;
    opened = true;
    engine = byId.get(engineId);
    state = validState(engineId, initialState);
    renderEngine();
    try { dialog.showModal(); }
    catch {
      opened = false;
      return false;
    }
    onOpen?.(engineId, getState());
    onEngineChange?.(engineId, getState());
    notify();
    initializeScene();
    return true;
  }

  function finishClose() {
    if (!opened) return;
    const finalState = getState();
    opened = false;
    releaseScene();
    clearTimeout(noticeTimer);
    refs.notice.textContent = '';
    shareFallback.hidden = true;
    onClose?.(finalState);
    if (opener instanceof HTMLElement && opener.isConnected) opener.focus({ preventScroll: true });
    opener = null;
  }

  function close() {
    if (!opened) return;
    dialog.close();
    finishClose();
  }

  function dispose() {
    if (disposed) return;
    close();
    disposed = true;
    releaseScene();
    lifetime.abort();
    sizeObserver?.disconnect();
    dialog.remove();
  }

  listen(actions.close, 'click', close);
  listen(dialog, 'cancel', event => { event.preventDefault(); close(); });
  listen(dialog, 'close', () => { if (!dialog.open) finishClose(); });
  listen(dialog, 'click', event => {
    if (event.target !== dialog) return;
    const bounds = dialog.getBoundingClientRect();
    if (event.clientX < bounds.left || event.clientX > bounds.right || event.clientY < bounds.top || event.clientY > bounds.bottom) close();
  });
  listen(engineSelect, 'change', () => changeEngine(engineSelect.value));
  listen(actions.previous, 'click', () => changeEngine(engines[(engines.indexOf(engine) - 1 + engines.length) % engines.length].id));
  listen(actions.next, 'click', () => changeEngine(engines[(engines.indexOf(engine) + 1) % engines.length].id));
  listen(actions.context, 'click', () => selectPart(null));
  listen(actions['full-assembly'], 'click', () => selectPart(null));
  listen(actions['next-component'], 'click', () => {
    const index = engine.parts.findIndex(part => part.id === state.partId);
    if (engine.parts.length) selectPart(engine.parts[(index + 1) % engine.parts.length].id, { reveal: true });
  });
  listen(actions.explode, 'click', () => setExplosion(state.explosion < 0.5 ? 1 : 0));
  listen(actions.assemble, 'click', () => setExplosion(0));
  listen(actions.expand, 'click', () => setExplosion(1));
  listen(actions.recenter, 'click', () => renderer?.recenter());
  listen(range, 'input', () => setExplosion(Number(range.value) / 100));
  listen(partList, 'click', event => {
    const button = event.target.closest('button[data-part-id]');
    if (button && partList.contains(button)) selectPart(button.dataset.partId, { reveal: true });
  });
  listen(stage, 'pointerleave', hideHover);
  listen(refs.artwork.querySelector('img'), 'error', () => { refs.artwork.hidden = true; });
  listen(actions.copy, 'click', async () => {
    notify();
    const link = window.location.href;
    try {
      if (!navigator.clipboard?.writeText) throw new Error('Clipboard unavailable');
      await navigator.clipboard.writeText(link);
      if (opened) showNotice('Engine view copied.');
    } catch {
      if (!opened) return;
      shareFallback.value = link;
      shareFallback.hidden = false;
      shareFallback.focus();
      shareFallback.select();
      showNotice('View link selected. Use your copy shortcut.');
    }
  });
  listen(motionPreference, 'change', event => {
    reducedMotion = event.matches;
    renderer?.setReducedMotion(reducedMotion);
    if (opened) renderExpansion();
  });
  listen(document, 'visibilitychange', () => renderer?.setActive(opened && !document.hidden));
  const sizeObserver = typeof ResizeObserver === 'function' ? new ResizeObserver(() => {
    if (opened && renderer) renderer.resize();
  }) : null;
  sizeObserver?.observe(canvasHost);
  listen(window, 'resize', () => { if (opened) renderer?.resize(); });

  return { open, close, getState, isOpen: () => opened, dispose };
}
