import { nodes, edges, meta, families, koshas } from './data.js';
import { createScene } from './scene.js';
import { readEngineRoomState, writeEngineRoomState } from './engine-room-state.js';

const $ = id => document.getElementById(id);
const root = document.querySelector('.atlas-shell');
const stage = $('scene-root');
const motion = window.matchMedia('(prefers-reduced-motion: reduce)');
const params = new URLSearchParams(location.search);
const validViews = ['connections', 'anatomy', 'exploded', 'biofield'];
const finiteTime = value => Number.isFinite(Number(value)) ? Math.min(24, Math.max(0, Number(value))) : 0;
const state = {
  node: nodes.find(n => n.id === params.get('node')) || nodes.find(n => n.id === 'selemene'),
  view: validViews.includes(params.get('view')) ? params.get('view') : 'connections',
  kosha: koshas.some(k => k.id === params.get('kosha')) ? params.get('kosha') : 'all',
  time: finiteTime(params.get('time')),
  speed: [0.25, 0.5, 1, 2, 4].includes(Number(params.get('speed'))) ? Number(params.get('speed')) : 1,
  playing: false,
};
let scene;
let toastTimer;
let currentExport;
let engineRoom;
let engineRoomLoading;
const initialEngineParams = new URLSearchParams(location.search);

function syncEngineURL(roomState) {
  history.replaceState(null, '', writeEngineRoomState(location.href, roomState));
}

async function loadEngineRoom() {
  if (engineRoomLoading) return engineRoomLoading;
  engineRoomLoading = Promise.all([import('./engine-room.js'), import('./engine-details.js')])
    .then(([{ createEngineRoom }, { engineDetails }]) => {
      engineRoom = createEngineRoom({
        catalog: engineDetails, nodes, koshas, revision: meta.revision,
        onOpen: (id, roomState) => {
          setPlaying(false);
          selectNode(id);
          syncEngineURL(roomState);
        },
        onEngineChange: (id, roomState) => { selectNode(id); syncEngineURL(roomState); },
        onStateChange: syncEngineURL,
        onClose: () => { syncEngineURL(null); },
      });
      return { room: engineRoom, catalog: engineDetails };
    })
    .catch(error => { engineRoomLoading = null; throw error; });
  return engineRoomLoading;
}

async function openEngine(id, restore = false) {
  const buttons = [$('explore-engines'), $('open-engine')];
  buttons.forEach(button => { button.setAttribute('aria-busy', 'true'); });
  try {
    const { room, catalog } = await loadEngineRoom();
    const initial = restore ? readEngineRoomState(initialEngineParams, catalog) : null;
    const engine = catalog.find(engine => engine.id === (initial?.engineId || id)) || catalog[0];
    room.open(engine.id, initial || undefined);
  } catch (error) {
    console.error('Engine room could not initialize', error);
    toast('The engine view could not load. Try opening it again.');
  } finally { buttons.forEach(button => { button.removeAttribute('aria-busy'); }); }
}
const sourceLink = path => `https://github.com/Sheshiyer/Selemene-engine/blob/${meta.revision}/${path}`;
function toast(message) {
  $('toast').textContent = message;
  $('toast').classList.add('visible');
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => $('toast').classList.remove('visible'), 3200);
}
function saveURL() {
  const url = new URL(location.href);
  for (const [key, value] of Object.entries({ node: state.node.id, view: state.view, kosha: state.kosha, time: state.time.toFixed(2), speed: state.speed })) url.searchParams.set(key, String(value));
  history.replaceState(null, '', url);
}
function download(blob, name) {
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url; link.download = name; link.click();
  setTimeout(() => URL.revokeObjectURL(url), 2000);
}
function previewExport(blob, name, text) {
  if (currentExport) URL.revokeObjectURL(currentExport.url);
  currentExport = { blob, name, text, url: URL.createObjectURL(blob) };
  const isText = typeof text === 'string';
  $('export-title').textContent = isText ? 'Your system briefing.' : 'A moment in the atlas.';
  $('export-description').textContent = `${state.node.label} · ${isText ? 'Source references and the five Kosha correspondences' : 'PNG of the 3D stage'}`;
  $('briefing-preview').hidden = !isText;
  $('briefing-preview').value = text || '';
  $('frame-preview').hidden = isText;
  if (!isText) $('frame-preview').src = currentExport.url;
  else $('frame-preview').removeAttribute('src');
  $('export-copy').hidden = !isText;
  $('export-download').textContent = isText ? 'Download Markdown' : 'Download PNG';
  $('export-dialog').showModal();
}
function updateTime(time) {
  state.time = finiteTime(time);
  $('timeline-range').value = String(state.time);
  $('timeline-range').style.setProperty('--progress', `${state.time / 24 * 100}%`);
  $('time-value').textContent = state.time.toFixed(2).padStart(5, '0');
  const chapter = state.time < 5 ? 0 : state.time < 10 ? 1 : state.time < 21 ? 2 : 3;
  $('chapter-title').textContent = ['01 — Embody', '02 — Connect', '03 — Unfold', '04 — Return'][chapter];
  document.querySelectorAll('[data-chapter]').forEach((b, i) => b.classList.toggle('active', i === chapter));
  root.dataset.time = state.time.toFixed(2);
  $('timeline-range').setAttribute('aria-valuetext', `${state.time.toFixed(1)} seconds, ${['Embody', 'Connect', 'Unfold', 'Return'][chapter]}`);
}
function setPlaying(value) {
  state.playing = Boolean(value) && !motion.matches && Boolean(scene?.isAvailable);
  scene?.setPlayState(state.playing);
  $('play-toggle').textContent = state.playing ? 'Ⅱ' : '▷';
  $('play-toggle').setAttribute('aria-label', state.playing ? 'Pause cinematic shot' : 'Play cinematic shot');
  $('play-toggle').setAttribute('aria-pressed', String(state.playing));
  root.dataset.playing = String(state.playing);
  if (!state.playing) saveURL();
}
function renderViews() {
  document.querySelectorAll('button[data-view]').forEach(b => b.setAttribute('aria-pressed', String(b.dataset.view === state.view)));
  root.dataset.view = state.view;
}
function setView(view, resetTime = true) {
  if (!validViews.includes(view)) return;
  setPlaying(false);
  if (resetTime) { updateTime(0); scene?.setTimelineProgress(0); }
  state.view = view;
  scene?.setViewMode(view);
  renderViews();
  saveURL();
}
function renderFilter() {
  document.querySelectorAll('button[data-kosha]').forEach(b => b.setAttribute('aria-pressed', String(b.dataset.kosha === state.kosha)));
  const k = koshas.find(k => k.id === state.kosha);
  $('stage-state').textContent = k ? `${k.label} / ${k.meaning} · ${k.engines.length} engines` : 'Vitruvian anatomy · five sheaths';
  root.dataset.kosha = state.kosha;
  scene?.setKoshaFilter?.(state.kosha);
}
function setFilter(id) {
  if (id !== 'all' && !koshas.some(k => k.id === id)) return;
  state.kosha = id;
  renderFilter();
  if (id !== 'all' && state.node.kosha !== id) selectNode(koshas.find(k => k.id === id).engines[0]);
  saveURL();
}
function selectNode(id) {
  const node = nodes.find(n => n.id === id);
  if (!node) return;
  state.node = node;
  if (state.kosha !== 'all' && node.kosha !== state.kosha) { state.kosha = node.kosha || 'all'; renderFilter(); }
  document.querySelector('.inspector').dataset.node = id;
  document.querySelector('.inspector-index').textContent = `${String(nodes.indexOf(node) + 1).padStart(2, '0')} / ${nodes.length}`;
  $('node-select').value = id;
  $('node-kind').textContent = { engine: 'ENGINE / SYMBOLIC MIRROR', core: 'INTEGRATION CORE', infra: 'INFRASTRUCTURE', connected: 'CONNECTED SYSTEM' }[node.kind];
  $('node-name').textContent = node.label;
  $('node-anatomy').textContent = node.anatomy;
  $('node-description').textContent = node.description;
  $('open-engine').hidden = node.kind !== 'engine';
  if (node.kind === 'engine') {
    $('engine-artwork').src = `/assets/engines/${node.id}.png`;
    $('engine-artwork').alt = `${node.label} — sculptural instrument from the Tryambakam collection`;
    $('open-engine-name').textContent = `Inside ${node.label}`;
  } else { $('engine-artwork').removeAttribute('src'); }
  $('node-runtime').textContent = node.runtime;
  $('node-evidence').textContent = node.evidence;
  $('node-source').href = sourceLink(node.source);
  $('node-source').title = node.source;
  $('node-source').setAttribute('aria-label', `View repository source for ${node.label}`);
  const k = koshas.find(k => k.id === node.kosha);
  $('node-kosha').textContent = k ? `${k.number} / ${k.label}` : node.kind === 'core' ? 'Across the five sheaths' : 'The supporting field';
  $('node-kosha').style.color = k?.color || '#D4B65F';
  $('node-rationale').textContent = node.koshaRationale || (node.kind === 'core' ? 'Integration connects the five engine categories. Its position at the heart is a visual correspondence.' : 'Infrastructure and consumer applications form the surrounding system field; they are not assigned a classical kosha.');
  const links = edges.filter(e => e.from === id || e.to === id);
  $('connection-count').textContent = String(links.length).padStart(2, '0');
  $('connected-list').replaceChildren();
  for (const edge of links) {
    const other = nodes.find(n => n.id === (edge.from === id ? edge.to : edge.from));
    const b = document.createElement('button'); b.type = 'button'; b.dataset.node = other.id;
    b.textContent = other.label; b.title = `${edge.label} · ${edge.source}`;
    const arrow = document.createElement('span'); arrow.textContent = edge.from === id ? '↗' : '↙'; b.append(arrow);
    b.addEventListener('click', () => selectNode(other.id)); $('connected-list').append(b);
  }
  if (!links.length) { const p = document.createElement('p'); p.className = 'eyebrow'; p.textContent = 'No admitted connection in this snapshot'; $('connected-list').append(p); }
  scene?.setSelectedNode(id);
  saveURL();
}
for (const k of koshas) {
  const button = document.createElement('button'); button.className = 'kosha-choice'; button.dataset.kosha = k.id; button.style.setProperty('--kosha-color', k.color); button.setAttribute('aria-pressed', 'false'); button.setAttribute('aria-label', `${k.label} — ${k.meaning}, ${k.engines.length} engines`);
  const index = document.createElement('span'); index.className = 'kosha-index'; index.textContent = k.number;
  const text = document.createElement('span'); const title = document.createElement('strong'); title.textContent = k.label; const small = document.createElement('small'); small.textContent = `${k.meaning.toUpperCase()} / ${String(k.engines.length).padStart(2,'0')}`; text.append(title, small); button.append(index, text); $('kosha-options').append(button);
}
for (const [kind, label] of [['core','Integration'], ['engine','Engines'], ['infra','Infrastructure'], ['connected','Connected systems']]) {
  const group = document.createElement('optgroup'); group.label = label;
  nodes.filter(n => n.kind === kind).forEach(n => { const o = document.createElement('option'); o.value = n.id; o.textContent = n.label; group.append(o); }); $('node-select').append(group);
}
$('engine-count').textContent = meta.engineCount;
$('component-count').textContent = nodes.length;
$('methods-counts').textContent = `${meta.engineCount} declared runtime identities, ${meta.publicMirrors} public mirrors, ${meta.workflowCount} declared workflows, ${nodes.length} components and ${edges.length} source relationships. A declared workflow is not evidence of runtime availability.`;
$('source-revision').textContent = `SOURCE ${meta.revision} / ${meta.snapshot}`;
try {
  scene = createScene(stage, { nodes, edges, families,
    onSelect: node => { if (node?.id) selectNode(node.id); },
    onHover: (node, coords) => {
      $('scene-tooltip').hidden = !node;
      if (node) { $('scene-tooltip').textContent = `${node.label} · ${koshas.find(k=>k.id===node.kosha)?.label || node.kind}`; if (coords) { $('scene-tooltip').style.left = `${Math.min(stage.clientWidth - 220, Math.max(10,coords.x + 12))}px`; $('scene-tooltip').style.top = `${Math.min(stage.clientHeight - 70,Math.max(10,coords.y + 12))}px`; } }
    },
    onProgress: time => { updateTime(time); if (time >= 24) setPlaying(false); },
    onReady: message => { $('model-status').textContent = typeof message === 'string' ? message : 'Meshy 7 · anatomical mesh'; root.dataset.model = 'ready'; },
    onFallback: message => { $('model-status').textContent = message; root.dataset.model = 'fallback'; },
  });
} catch (error) {
  console.error('Atlas scene could not initialize', error);
  const fallback = document.createElement('p'); fallback.className = 'node-description'; fallback.style.cssText = 'position:absolute;top:35%;left:15%;right:15%;z-index:4'; fallback.textContent = 'The 3D scene is unavailable. You can still explore every component, kosha correspondence and source in the inspector.'; stage.append(fallback);
  $('model-status').textContent = '3D unavailable · inspector remains usable'; root.dataset.model = 'fallback';
}
scene?.setViewMode(state.view);
if (state.view === 'connections') scene?.setTimelineProgress(state.time);
else state.time = 0;
scene?.setPlaybackSpeed(state.speed);
scene?.setReducedMotion(motion.matches);
setPlaying(false);
selectNode(state.node.id); renderViews(); renderFilter(); updateTime(state.time);
$('play-speed').value = String(state.speed);
function applyMotion() {
  scene?.setReducedMotion(motion.matches);
  if (motion.matches) setPlaying(false);
  $('play-toggle').disabled = motion.matches || !scene?.isAvailable;
  $('play-blast').disabled = motion.matches || !scene?.isAvailable;
  root.dataset.reducedMotion = String(motion.matches);
  $('mapping-note').textContent = motion.matches ? 'Reduced motion enabled · use the timeline and sheath controls' : 'Pancha Kosha assignments are interpretive · source topology is verifiable';
}
applyMotion(); motion.addEventListener('change', applyMotion);
$('node-select').addEventListener('change', e => selectNode(e.target.value));
document.querySelectorAll('button[data-kosha]').forEach(b => b.addEventListener('click', () => setFilter(b.dataset.kosha)));
document.querySelectorAll('button[data-view]').forEach(b => b.addEventListener('click', () => setView(b.dataset.view)));
$('play-toggle').addEventListener('click', () => {
  if (state.playing) { setPlaying(false); return; }
  if (state.view !== 'connections') setView('connections', true);
  if (state.time >= 24) { updateTime(0); scene?.setTimelineProgress(0); }
  setPlaying(true);
});
$('play-blast').addEventListener('click', () => { setFilter('all'); setView('connections'); updateTime(0); scene?.setTimelineProgress(0); setPlaying(true); });
$('timeline-range').addEventListener('input', e => { setPlaying(false); if (state.view !== 'connections') setView('connections', false); updateTime(e.target.value); scene?.setTimelineProgress(state.time); });
$('timeline-range').addEventListener('change', saveURL);
document.querySelectorAll('[data-chapter]').forEach(b => b.addEventListener('click', () => { setView('connections', false); updateTime(b.dataset.chapter); scene?.setTimelineProgress(state.time); saveURL(); }));
$('play-speed').addEventListener('change', e => { state.speed = Number(e.target.value); scene?.setPlaybackSpeed(state.speed); saveURL(); });
$('recenter').addEventListener('click', () => scene?.recenter());
$('open-engine').addEventListener('click', () => openEngine(state.node.id));
$('explore-engines').addEventListener('click', () => openEngine(state.node.kind === 'engine' ? state.node.id : 'panchanga'));
$('methods-open').addEventListener('click', () => $('methods-dialog').showModal());
$('share-view').addEventListener('click', async () => { saveURL(); try { await navigator.clipboard.writeText(location.href); toast('View link copied'); } catch { toast('Copy the current address to share this view'); } });
$('save-frame').addEventListener('click', async () => {
  if (!scene?.isAvailable) { toast('A 3D frame is not available'); return; }
  setPlaying(false);
  try { const data = scene.captureFrame(); if (!data) throw new Error('No frame'); const response = await fetch(data); previewExport(await response.blob(), `selemene-${state.view}-${state.node.id}.png`); } catch { toast('Frame could not be prepared'); }
});
$('briefing-export').addEventListener('click', () => {
  const n = state.node; const k = koshas.find(k => k.id === n.kosha);
  const lines = ['# Selemene — The Fivefold Atlas', '', `## ${n.label}`, '', n.description, '', `- View: ${state.view}`, `- Shot position: ${state.time.toFixed(2)} / 24 seconds`, `- Body correspondence: ${n.anatomy}`, `- Kosha: ${k?.label || 'Supporting system field'} (proposed mapping)`, `- Rationale: ${n.koshaRationale || 'System infrastructure or integration surrounding the five sheaths.'}`, `- Runtime: ${n.runtime}`, `- Evidence: ${n.evidence}`, `- Source: ${sourceLink(n.source)}`, '', '## Direct relationships', '', ...edges.filter(e=>e.from===n.id||e.to===n.id).map(e=>`- ${e.from} → ${e.to}: ${e.label} ([source](${sourceLink(e.source)}))`), '', '## The five sheaths', '', ...koshas.map(k=>`- ${k.label} (${k.meaning}): ${k.engines.map(id=>nodes.find(n=>n.id===id).label).join(', ')}`), '', `Snapshot: ${meta.snapshot} · ${meta.revision}`, `${meta.engineCount} runtime identities · ${nodes.length} components · ${edges.length} relationships.`, '', 'Anatomical position, kosha assignments and motion are conceptual. No physiological, traffic or health measurements are represented.'];
  const text = lines.join('\n');
  previewExport(new Blob([text], {type:'text/markdown;charset=utf-8'}), `selemene-briefing-${n.id}.md`, text);
});
$('export-copy').addEventListener('click', async () => {
  try { await navigator.clipboard.writeText(currentExport.text); toast('Briefing copied'); }
  catch { $('briefing-preview').focus(); $('briefing-preview').select(); toast('Briefing selected — use your copy shortcut'); }
});
$('export-download').addEventListener('click', () => {
  if (currentExport) { download(currentExport.blob, currentExport.name); toast('Download requested'); }
});
document.addEventListener('visibilitychange', () => { if (document.hidden) setPlaying(false); });
document.addEventListener('keydown', e => { if (e.key === ' ' && e.target === document.body && !$('methods-dialog').open && !engineRoom?.isOpen()) { e.preventDefault(); $('play-toggle').click(); } });
window.addEventListener('pagehide', () => { engineRoom?.dispose(); scene?.dispose(); motion.removeEventListener('change',applyMotion); if (currentExport) URL.revokeObjectURL(currentExport.url); });
if (nodes.some(node => node.kind === 'engine' && node.id === initialEngineParams.get('engine'))) {
  openEngine(initialEngineParams.get('engine'), true);
}
