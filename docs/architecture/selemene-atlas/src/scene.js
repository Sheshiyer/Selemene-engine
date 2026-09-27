import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import { EffectComposer } from 'three/examples/jsm/postprocessing/EffectComposer.js';
import { RenderPass } from 'three/examples/jsm/postprocessing/RenderPass.js';
import { UnrealBloomPass } from 'three/examples/jsm/postprocessing/UnrealBloomPass.js';
import { OutputPass } from 'three/examples/jsm/postprocessing/OutputPass.js';
import { createAnatomy, disposeObject, KOSHA_COLORS } from './anatomy.js';

const DURATION = 24;
const EDGE_SEGMENTS = 40;
const VIEWS = new Set(['connections', 'anatomy', 'exploded', 'biofield']);
const DEFAULT_LABELS = new Set(['railway', 'postgres', 'redis', 'ephemeris', 'llm-proxy', 'metrics']);
const GOLD = new THREE.Color(0xe1c675);
const clamp = (v, min, max) => Math.max(min, Math.min(max, v));
const smooth = (a, b, t) => { const x = clamp((t - a) / (b - a), 0, 1); return x * x * (3 - 2 * x); };
const hash = (text) => { let n = 0; for (const c of String(text)) n = (n * 31 + c.charCodeAt(0)) >>> 0; return n / 4294967296; };
const vector = (value) => new THREE.Vector3(...(Array.isArray(value) ? value : [0, 0, 0]));
const colorValue = (value, fallback = '#92A9B4') => typeof value === 'string' ? (value.startsWith('#') ? value : `#${value}`) : value ?? fallback;

// The endpoint at 24 seconds returns to the opening arrangement.
export function shotAt(time) {
  const t = clamp(Number(time) || 0, 0, DURATION);
  return {
    explosion: smooth(10, 17, t) * (1 - smooth(21, 24, t)),
    illumination: smooth(5, 10, t) * (1 - smooth(21, 24, t)),
  };
}

function unavailable(onFallback, reason) {
  onFallback?.(reason);
  const noop = () => {};
  return {
    isAvailable: false, controls: null, camera: null,
    setViewMode: noop, setTimelineProgress: noop, setSelectedNode: noop, setKoshaFilter: noop,
    setReducedMotion: noop, setPlayState: noop, setPlaybackSpeed: noop,
    resize: noop, dispose: noop, recenter: noop, captureFrame: () => '', listNode: () => null,
  };
}

export function createScene(container, options = {}) {
  const { nodes = [], edges = [], families = [], onSelect, onHover, onProgress, onFallback, onReady } = options;
  if (!container) return unavailable(onFallback, 'The atlas canvas is unavailable. All source nodes remain available in the inspector.');
  let renderer;
  try {
    renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true, preserveDrawingBuffer: true, powerPreference: 'high-performance' });
  } catch {
    return unavailable(onFallback, 'This device could not start the 3D atlas. Explore the same source nodes using the node selector and inspector.');
  }

  let disposed = false; let animationFrame = 0; let previousFrame;
  let width = 1; let height = 1;
  let lastExplosion = -1; let lastIllumination = -1; let fitDistance = 15;
  let styleDirty = true; let renderDirty = true; let labelsDirty = true; let contextLost = false;
  const motionQuery = window.matchMedia('(prefers-reduced-motion: reduce)');
  const state = {
    view: 'connections', timeline: 0, speed: 1, playing: false,
    reducedMotion: motionQuery.matches, selectedId: nodes.find((node) => node.id === 'selemene')?.id ?? nodes[0]?.id,
    hoveredId: null, kosha: 'all', shotActive: true,
    viewExpansion: 0, viewFrom: 0, viewTo: 0, transitionElapsed: 1,
  };
  const requestRender = () => { renderDirty = true; };
  renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 1.6));
  renderer.setClearColor(0x07121a, 0);
  renderer.outputColorSpace = THREE.SRGBColorSpace;
  renderer.toneMapping = THREE.ACESFilmicToneMapping;
  renderer.toneMappingExposure = 1.05;
  renderer.domElement.className = 'atlas-webgl';
  renderer.domElement.setAttribute('aria-label', 'Interactive Vitruvian systems atlas. Drag to orbit, scroll to zoom, or use the accessible node selector.');
  renderer.domElement.setAttribute('role', 'img');
  Object.assign(renderer.domElement.style, { position: 'absolute', inset: '0', width: '100%', height: '100%', display: 'block', touchAction: 'none' });
  container.appendChild(renderer.domElement);
  if (getComputedStyle(container).position === 'static') container.style.position = 'relative';

  const scene = new THREE.Scene();
  const camera = new THREE.PerspectiveCamera(40, 1, 0.1, 160);
  camera.position.set(0, 0.4, 15);
  const controls = new OrbitControls(camera, renderer.domElement);
  controls.target.set(0, 0.04, 0);
  controls.enableDamping = !state.reducedMotion; controls.dampingFactor = 0.09;
  controls.enablePan = true; controls.rotateSpeed = 0.52; controls.zoomSpeed = 0.72; controls.panSpeed = 0.55;
  controls.minDistance = 8; controls.maxDistance = 65;
  controls.minPolarAngle = Math.PI * 0.12; controls.maxPolarAngle = Math.PI * 0.86;
  scene.add(new THREE.AmbientLight(0x9bcec5, 1.6));
  const key = new THREE.DirectionalLight(0xc5daca, 2.2); key.position.set(-4, 5, 8); scene.add(key);
  const rim = new THREE.DirectionalLight(0x568ea7, 1.7); rim.position.set(4, 2, -4); scene.add(rim);

  const composer = new EffectComposer(renderer);
  const renderPass = new RenderPass(scene, camera);
  const bloom = new UnrealBloomPass(new THREE.Vector2(1, 1), 0.17, 0.35, 0.86);
  const outputPass = new OutputPass();
  composer.addPass(renderPass); composer.addPass(bloom); composer.addPass(outputPass);
  const anatomy = createAnatomy({
    heartPosition: nodes.find((node) => node.id === 'selemene')?.body,
    onReady: (status) => { if (!disposed) { onReady?.(status); requestRender(); } },
    requestRender,
  });
  scene.add(anatomy.root);

  const labelLayer = document.createElement('div');
  labelLayer.className = 'scene-label-layer';
  labelLayer.setAttribute('aria-label', 'Visible source node labels');
  Object.assign(labelLayer.style, { position: 'absolute', inset: '0', overflow: 'hidden', pointerEvents: 'none', zIndex: '2' });
  container.appendChild(labelLayer);
  const familyColors = new Map(families.map((family) => [family.id, colorValue(family.color)]));
  const nodeColor = (node) => familyColors.get(node.family) || KOSHA_COLORS[node.kosha] || ({ core: '#D4B65F', infra: '#6689B0', connected: '#D4B785' }[node.kind]) || '#92A9B4';
  const nodeLayer = new THREE.Group(); const edgeLayer = new THREE.Group();
  const nodeLookup = new Map(); const nodeEntries = []; const pickMeshes = [];
  const sphereGeometry = new THREE.IcosahedronGeometry(1, 2);
  const hitGeometry = new THREE.IcosahedronGeometry(1, 1);
  const hitMaterial = new THREE.MeshBasicMaterial({ visible: false });
  const ringGeometry = new THREE.RingGeometry(1.65, 1.78, 40);
  const haloGeometry = new THREE.RingGeometry(2.2, 2.25, 48);
  scene.add(edgeLayer, nodeLayer);

  for (const data of nodes) {
    const color = new THREE.Color(nodeColor(data));
    const radius = data.id === 'selemene' ? 0.135 : data.kind === 'engine' ? 0.072 : data.kind === 'connected' ? 0.067 : 0.074;
    const mesh = new THREE.Mesh(sphereGeometry, new THREE.MeshBasicMaterial({ color, transparent: true, opacity: 0.9, depthWrite: false }));
    mesh.scale.setScalar(radius); mesh.renderOrder = 3;
    const ring = new THREE.Mesh(ringGeometry, new THREE.MeshBasicMaterial({ color, transparent: true, opacity: 0.32, depthWrite: false, side: THREE.DoubleSide }));
    ring.scale.setScalar(radius); ring.renderOrder = 3;
    const halo = new THREE.Mesh(haloGeometry, new THREE.MeshBasicMaterial({ color: 0xe3cb84, transparent: true, opacity: 0, depthWrite: false, side: THREE.DoubleSide }));
    halo.scale.setScalar(radius * 1.18); halo.renderOrder = 4;
    const hit = new THREE.Mesh(hitGeometry, hitMaterial);
    hit.scale.setScalar(Math.max(0.16, radius * 1.8)); hit.userData.nodeId = data.id;
    const label = document.createElement('span');
    label.className = 'node-label'; label.textContent = data.label || data.id; label.setAttribute('data-node', data.id);
    Object.assign(label.style, {
      position: 'absolute', top: '0', left: '0', whiteSpace: 'nowrap', display: 'block', visibility: 'hidden',
      color: color.getStyle(), font: '400 10px/1.3 Menlo, ui-monospace, monospace',
      letterSpacing: '0.025em', padding: '3px 5px', borderRadius: '3px',
      background: 'rgba(7, 16, 23, 0.68)', textShadow: '0 1px 8px #071017',
      willChange: 'transform', pointerEvents: 'none',
    });
    labelLayer.appendChild(label);
    const entry = {
      data, mesh, ring, halo, hit, color, radius, label,
      body: vector(data.body), field: vector(data.field || data.body), position: vector(data.body),
      projected: new THREE.Vector3(), labelWidth: Math.max(30, (data.label || data.id).length * 6.05 + 10),
    };
    nodeLookup.set(data.id, entry); nodeEntries.push(entry); pickMeshes.push(hit);
    nodeLayer.add(mesh, ring, halo, hit);
  }

  const edgeEntries = [];
  for (const data of edges) {
    const from = nodeLookup.get(data.from ?? data.source); const to = nodeLookup.get(data.to ?? data.target);
    if (!from || !to) continue;
    const positions = new Float32Array((EDGE_SEGMENTS + 1) * 3);
    const geometry = new THREE.BufferGeometry();
    geometry.setAttribute('position', new THREE.BufferAttribute(positions, 3).setUsage(THREE.DynamicDrawUsage));
    const color = to.data.kind === 'engine' ? to.color.clone() : from.color.clone().lerp(to.color, 0.5);
    const material = new THREE.LineBasicMaterial({ color, transparent: true, opacity: 0.11, depthWrite: false });
    const mesh = new THREE.Line(geometry, material); mesh.frustumCulled = false; edgeLayer.add(mesh);
    edgeEntries.push({ data, from, to, color, mesh, positions, control: new THREE.Vector3(), phase: hash(`${from.data.id}/${to.data.id}`) });
  }
  const particlePositions = new Float32Array(edgeEntries.length * 3);
  const particleColors = new Float32Array(edgeEntries.length * 3);
  const particleGeometry = new THREE.BufferGeometry();
  particleGeometry.setAttribute('position', new THREE.BufferAttribute(particlePositions, 3).setUsage(THREE.DynamicDrawUsage));
  particleGeometry.setAttribute('color', new THREE.BufferAttribute(particleColors, 3));
  const particles = new THREE.Points(particleGeometry, new THREE.PointsMaterial({ size: 0.028, vertexColors: true, transparent: true, opacity: 0.8, depthWrite: false }));
  particles.frustumCulled = false; edgeLayer.add(particles);

  function expansion() { return state.shotActive ? shotAt(state.timeline).explosion : state.viewExpansion; }
  function matchesKosha(node) { return state.kosha === 'all' || node.kosha === state.kosha || node.family === state.kosha || node.id === 'selemene'; }
  function distanceFor(amount) {
    const tangent = Math.tan(THREE.MathUtils.degToRad(camera.fov / 2));
    const halfWidth = THREE.MathUtils.lerp(5.0, 7.35, amount);
    const halfHeight = THREE.MathUtils.lerp(4.9, 5.8, amount);
    return Math.max(THREE.MathUtils.lerp(15, 19, amount), halfHeight / tangent + 0.5, halfWidth / (tangent * camera.aspect) + 0.7);
  }
  const cameraOffset = new THREE.Vector3();
  function fitCamera(amount, reset = false) {
    const next = distanceFor(amount);
    if (reset) {
      controls.target.set(0, 0.04, 0); camera.position.set(0, 0.4, next);
    } else if (Math.abs(next - fitDistance) > 0.00001) {
      cameraOffset.copy(camera.position).sub(controls.target).multiplyScalar(next / fitDistance);
      camera.position.copy(controls.target).add(cameraOffset);
    }
    fitDistance = next; controls.maxDistance = Math.max(65, next * 2.5); controls.update(); labelsDirty = true;
  }

  function updatePositions(amount) {
    for (const entry of nodeEntries) {
      entry.position.lerpVectors(entry.body, entry.field, amount);
      entry.mesh.position.copy(entry.position); entry.ring.position.copy(entry.position);
      entry.halo.position.copy(entry.position); entry.hit.position.copy(entry.position);
    }
    for (const edge of edgeEntries) {
      const a = edge.from.position; const b = edge.to.position;
      const dx = b.x - a.x; const dy = b.y - a.y;
      const bend = (edge.phase > 0.5 ? 1 : -1) * (0.13 + edge.phase * 0.12);
      edge.control.set((a.x + b.x) * 0.5 - dy * bend, (a.y + b.y) * 0.5 + dx * bend, Math.max(a.z, b.z) + 0.28 + Math.hypot(dx, dy) * 0.065);
      for (let i = 0; i <= EDGE_SEGMENTS; i += 1) {
        const t = i / EDGE_SEGMENTS; const inverse = 1 - t; const k = i * 3;
        edge.positions[k] = inverse * inverse * a.x + 2 * inverse * t * edge.control.x + t * t * b.x;
        edge.positions[k + 1] = inverse * inverse * a.y + 2 * inverse * t * edge.control.y + t * t * b.y;
        edge.positions[k + 2] = inverse * inverse * a.z + 2 * inverse * t * edge.control.z + t * t * b.z;
      }
      edge.mesh.geometry.attributes.position.needsUpdate = true;
    }
    fitCamera(amount); styleDirty = true; labelsDirty = true;
  }

  function updateStyles(illumination, amount) {
    for (const entry of nodeEntries) {
      const selected = entry.data.id === state.selectedId; const hovered = entry.data.id === state.hoveredId;
      const related = matchesKosha(entry.data); const focus = selected || hovered;
      const baseOpacity = state.view === 'anatomy' ? 0.36 : 0.79;
      entry.mesh.material.opacity = focus ? 1 : related ? baseOpacity + illumination * 0.15 : 0.09;
      entry.ring.material.opacity = focus ? 0.8 : related ? 0.25 + illumination * 0.16 : 0.025;
      entry.halo.material.opacity = selected ? 0.8 : hovered ? 0.36 : 0;
      entry.mesh.scale.setScalar(entry.radius * (focus ? 1.28 : 1));
      entry.label.style.color = focus ? '#f1dfaa' : entry.color.getStyle();
      entry.label.style.fontWeight = selected ? '500' : '400'; entry.label.classList.toggle('selected', selected);
    }
    edgeEntries.forEach((edge, index) => {
      const incident = edge.from.data.id === state.selectedId || edge.to.data.id === state.selectedId;
      const inFilteredKosha = (node) => node.kosha === state.kosha || node.family === state.kosha;
      const related = state.kosha === 'all' || inFilteredKosha(edge.from.data) || inFilteredKosha(edge.to.data);
      const activeFilter = state.kosha !== 'all';
      let opacity = incident ? 0.31 : 0.065 + illumination * 0.055 + amount * 0.024;
      if (activeFilter && !related) opacity *= 0.14;
      if (state.view === 'anatomy') opacity *= 0.14;
      if (state.view === 'biofield') opacity *= 0.55;
      edge.mesh.material.opacity = opacity; edge.mesh.material.color.copy(edge.color);
      if (incident) edge.mesh.material.color.lerp(GOLD, 0.42);
      const intensity = (activeFilter && !related ? 0.08 : incident ? 1 : 0.6) * (state.view === 'anatomy' ? 0.12 : 1);
      edge.color.toArray(particleColors, index * 3);
      for (let component = 0; component < 3; component += 1) particleColors[index * 3 + component] *= intensity;
    });
    particleGeometry.attributes.color.needsUpdate = true; styleDirty = false; labelsDirty = true;
  }

  function updateParticles() {
    edgeEntries.forEach((edge, index) => {
      // Four journeys per shot; a paused timeline never advances their positions.
      const t = (edge.phase + state.timeline / 6) % 1; const inverse = 1 - t;
      const a = edge.from.position; const b = edge.to.position; const c = edge.control;
      particlePositions[index * 3] = inverse * inverse * a.x + 2 * inverse * t * c.x + t * t * b.x;
      particlePositions[index * 3 + 1] = inverse * inverse * a.y + 2 * inverse * t * c.y + t * t * b.y;
      particlePositions[index * 3 + 2] = inverse * inverse * a.z + 2 * inverse * t * c.z + t * t * b.z;
    });
    particleGeometry.attributes.position.needsUpdate = true;
  }

  function updateLabels(amount) {
    const candidates = nodeEntries.filter((entry) => {
      const id = entry.data.id;
      const showEngineLabel = entry.data.kind === 'engine' && (amount > 0.72 || state.kosha !== 'all');
      return id === state.selectedId || id === state.hoveredId || (matchesKosha(entry.data) && (DEFAULT_LABELS.has(id) || showEngineLabel));
    }).sort((a, b) => {
      const rank = (entry) => entry.data.id === state.selectedId ? 0 : entry.data.id === state.hoveredId ? 1 : DEFAULT_LABELS.has(entry.data.id) ? 2 : 3;
      return rank(a) - rank(b);
    });
    nodeEntries.forEach((entry) => { entry.label.style.visibility = 'hidden'; });
    const occupied = [];
    for (const entry of candidates) {
      entry.projected.copy(entry.position).project(camera);
      const p = entry.projected;
      if (p.z < -1 || p.z > 1 || Math.abs(p.x) > 1.04 || Math.abs(p.y) > 1.04) continue;
      const px = (p.x * 0.5 + 0.5) * width; const py = (-p.y * 0.5 + 0.5) * height;
      const boxWidth = entry.labelWidth; const boxHeight = 19; const preferRight = px >= width / 2;
      let placement = null;
      for (const offsetY of [0, -22, 22, -42, 42]) {
        for (const right of [preferRight, !preferRight]) {
          const x = clamp(px + (right ? 12 : -boxWidth - 12), 8, Math.max(8, width - boxWidth - 8));
          const y = clamp(py - boxHeight / 2 + offsetY, 10, Math.max(10, height - boxHeight - 10));
          const box = { x, y, w: boxWidth, h: boxHeight };
          if (occupied.every((other) => box.x + box.w + 5 < other.x || other.x + other.w + 5 < box.x || box.y + box.h + 3 < other.y || other.y + other.h + 3 < box.y)) { placement = box; break; }
        }
        if (placement) break;
      }
      if (!placement) continue;
      occupied.push(placement);
      entry.label.style.transform = `translate3d(${placement.x.toFixed(1)}px, ${placement.y.toFixed(1)}px, 0)`;
      entry.label.style.visibility = 'visible';
      entry.label.style.opacity = entry.data.id === state.selectedId || entry.data.id === state.hoveredId ? '1' : '0.83';
    }
    labelsDirty = false;
  }

  function draw() {
    if (disposed || contextLost) return;
    const amount = expansion();
    const illumination = state.shotActive ? shotAt(state.timeline).illumination : state.view === 'biofield' ? 0.7 : 0.15;
    if (Math.abs(amount - lastExplosion) > 0.000001) { updatePositions(amount); lastExplosion = amount; }
    if (styleDirty || Math.abs(illumination - lastIllumination) > 0.0001) { updateStyles(illumination, amount); lastIllumination = illumination; }
    anatomy.update({ explosion: amount, illumination, time: state.timeline, view: state.view, kosha: state.kosha, reducedMotion: state.reducedMotion });
    updateParticles();
    for (const entry of nodeEntries) { entry.ring.quaternion.copy(camera.quaternion); entry.halo.quaternion.copy(camera.quaternion); }
    camera.updateMatrixWorld();
    if (labelsDirty) updateLabels(amount);
    composer.render(); renderDirty = false;
  }

  function render(now) {
    if (disposed || contextLost) return;
    const delta = previousFrame === undefined ? 0 : clamp((now - previousFrame) / 1000, 0, 0.1); previousFrame = now;
    if (state.playing && !state.reducedMotion) {
      state.timeline = Math.min(DURATION, state.timeline + delta * state.speed);
      if (state.timeline >= DURATION) state.playing = false;
      onProgress?.(state.timeline); renderDirty = true;
    }
    if (state.transitionElapsed < 0.75) {
      state.transitionElapsed = Math.min(0.75, state.transitionElapsed + delta);
      state.viewExpansion = THREE.MathUtils.lerp(state.viewFrom, state.viewTo, smooth(0, 0.75, state.transitionElapsed));
      renderDirty = true;
    }
    if (controls.update()) { renderDirty = true; labelsDirty = true; }
    if (renderDirty) draw();
    animationFrame = requestAnimationFrame(render);
  }

  function setSelectedNode(id) {
    if (!nodeLookup.has(id) || state.selectedId === id) return;
    state.selectedId = id; styleDirty = true; requestRender();
  }
  function setViewMode(view) {
    if (!VIEWS.has(view)) return;
    const current = expansion();
    state.view = view; state.shotActive = state.playing;
    state.viewFrom = current; state.viewTo = view === 'exploded' ? 1 : 0; state.viewExpansion = current;
    state.transitionElapsed = state.reducedMotion ? 0.75 : 0;
    if (state.reducedMotion) state.viewExpansion = state.viewTo;
    styleDirty = true; requestRender();
  }
  function setTimelineProgress(time) {
    state.timeline = clamp(Number(time) || 0, 0, DURATION); state.shotActive = true; requestRender();
  }
  function setKoshaFilter(id = 'all') {
    state.kosha = id === 'all' || Object.hasOwn(KOSHA_COLORS, id) ? id : 'all'; styleDirty = true; requestRender();
  }
  function setReducedMotion(value) {
    state.reducedMotion = Boolean(value); controls.enableDamping = !state.reducedMotion;
    if (state.reducedMotion) { state.playing = false; state.viewExpansion = state.viewTo; state.transitionElapsed = 0.75; }
    requestRender();
  }
  function setPlayState(playing) {
    state.playing = Boolean(playing) && !state.reducedMotion;
    if (state.playing) { state.shotActive = true; if (state.timeline >= DURATION) state.timeline = 0; previousFrame = undefined; }
    requestRender();
  }
  function resize(newWidth, newHeight) {
    if (disposed) return;
    const rect = container.getBoundingClientRect();
    width = Math.max(1, Number(newWidth) || rect.width || 1); height = Math.max(1, Number(newHeight) || rect.height || 1);
    camera.aspect = width / height; camera.updateProjectionMatrix();
    renderer.setSize(width, height, false); composer.setSize(width, height); fitCamera(expansion()); labelsDirty = true; requestRender();
  }
  function recenter() { fitCamera(expansion(), true); labelsDirty = true; requestRender(); }

  const raycaster = new THREE.Raycaster(); const pointer = new THREE.Vector2(); let pointerDown = null;
  function pick(event) {
    const rect = renderer.domElement.getBoundingClientRect();
    pointer.set((event.clientX - rect.left) / rect.width * 2 - 1, -(event.clientY - rect.top) / rect.height * 2 + 1);
    camera.updateMatrixWorld(); raycaster.setFromCamera(pointer, camera);
    const hit = raycaster.intersectObjects(pickMeshes, false)[0];
    return hit ? nodeLookup.get(hit.object.userData.nodeId) : null;
  }
  function clearHover() {
    if (!state.hoveredId) return;
    state.hoveredId = null; renderer.domElement.style.cursor = 'grab'; styleDirty = true; onHover?.(null, null); requestRender();
  }
  function onPointerDown(event) {
    if (pointerDown) { pointerDown.dragged = true; return; }
    pointerDown = { id: event.pointerId, x: event.clientX, y: event.clientY, dragged: false };
  }
  function onPointerMove(event) {
    if (pointerDown && Math.hypot(event.clientX - pointerDown.x, event.clientY - pointerDown.y) > 5) pointerDown.dragged = true;
    if (pointerDown?.dragged) { clearHover(); return; }
    const entry = pick(event);
    if (!entry) { clearHover(); return; }
    if (state.hoveredId !== entry.data.id) { state.hoveredId = entry.data.id; styleDirty = true; requestRender(); }
    renderer.domElement.style.cursor = 'pointer';
    const rect = renderer.domElement.getBoundingClientRect();
    onHover?.(entry.data, { x: event.clientX - rect.left, y: event.clientY - rect.top });
  }
  function onPointerUp(event) {
    const start = pointerDown; pointerDown = null;
    if (!start || start.id !== event.pointerId || start.dragged || Math.hypot(event.clientX - start.x, event.clientY - start.y) > 5) return;
    const entry = pick(event);
    if (entry) { setSelectedNode(entry.data.id); onSelect?.(entry.data); }
  }
  function cancelPointer() { pointerDown = null; clearHover(); }
  function onContextLost(event) {
    event.preventDefault(); contextLost = true; cancelAnimationFrame(animationFrame);
    onFallback?.('The graphics context was interrupted. All source nodes are still accessible through the inspector.');
  }
  function onContextRestored() { contextLost = false; previousFrame = undefined; renderDirty = true; animationFrame = requestAnimationFrame(render); }
  const onMotionPreference = (event) => setReducedMotion(event.matches);
  const canvas = renderer.domElement; canvas.style.cursor = 'grab';
  canvas.addEventListener('pointerdown', onPointerDown); canvas.addEventListener('pointermove', onPointerMove, { passive: true });
  canvas.addEventListener('pointerup', onPointerUp); canvas.addEventListener('pointerleave', clearHover); canvas.addEventListener('pointercancel', cancelPointer);
  canvas.addEventListener('webglcontextlost', onContextLost); canvas.addEventListener('webglcontextrestored', onContextRestored);
  motionQuery.addEventListener('change', onMotionPreference);
  const observer = new ResizeObserver((entries) => {
    const rect = entries[0]?.contentRect;
    if (rect && (Math.abs(rect.width - width) > 0.5 || Math.abs(rect.height - height) > 0.5)) resize(rect.width, rect.height);
  });
  observer.observe(container); resize(); recenter(); draw(); animationFrame = requestAnimationFrame(render);

  function dispose() {
    if (disposed) return;
    disposed = true; cancelAnimationFrame(animationFrame); observer.disconnect(); motionQuery.removeEventListener('change', onMotionPreference);
    canvas.removeEventListener('pointerdown', onPointerDown); canvas.removeEventListener('pointermove', onPointerMove);
    canvas.removeEventListener('pointerup', onPointerUp); canvas.removeEventListener('pointerleave', clearHover); canvas.removeEventListener('pointercancel', cancelPointer);
    canvas.removeEventListener('webglcontextlost', onContextLost); canvas.removeEventListener('webglcontextrestored', onContextRestored);
    controls.dispose(); anatomy.dispose(); disposeObject(scene); bloom.dispose(); outputPass.dispose(); composer.dispose(); renderer.dispose();
    labelLayer.remove(); canvas.remove(); nodeLookup.clear();
  }
  return {
    isAvailable: true, camera, controls, scene,
    setViewMode, setTimelineProgress, setSelectedNode, setKoshaFilter, setReducedMotion,
    setPlayState, setPlaybackSpeed(speed) { state.speed = clamp(Number(speed) || 1, 0.125, 4); },
    resize, recenter, dispose, listNode: (id) => nodeLookup.get(id) || null,
    captureFrame() { if (disposed || contextLost) return ''; draw(); return canvas.toDataURL('image/png'); },
  };
}

export default createScene;
