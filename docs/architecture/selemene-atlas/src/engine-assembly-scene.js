import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import { EffectComposer } from 'three/examples/jsm/postprocessing/EffectComposer.js';
import { RenderPass } from 'three/examples/jsm/postprocessing/RenderPass.js';
import { UnrealBloomPass } from 'three/examples/jsm/postprocessing/UnrealBloomPass.js';
import { OutputPass } from 'three/examples/jsm/postprocessing/OutputPass.js';
import { RoomEnvironment } from 'three/examples/jsm/environments/RoomEnvironment.js';

// These are deliberately schematic instruments. Their parts describe software
// responsibilities; their dimensions and motion do not encode measurements.
const COLORS = { void: 0x070b1d, surface: 0x122b30, silver: 0xbc9472, paper: 0xf0ede3, gold: 0xc99d61, emerald: 0x47796f, indigo: 0x477e99, bronze: 0xb3764e, amber: 0xffab46 };
const TAU = Math.PI * 2;
const clamp = (value, low = 0, high = 1) => Math.max(low, Math.min(high, Number(value) || 0));
const ease = (value) => { const t = clamp(value); return t * t * (3 - 2 * t); };
const hash = (text) => { let value = 2166136261; for (const character of String(text)) value = Math.imul(value ^ character.charCodeAt(0), 16777619); return (value >>> 0) / 4294967296; };
const SVG_NS = 'http://www.w3.org/2000/svg';
const PROFILES = {
  orrery: ['dial', 'rings', 'orrery', 'prism', 'ring', 'lens', 'orrery'],
  dial: ['dial', 'ring', 'dial', 'lattice', 'rings', 'lens', 'dial'],
  rings: ['ring', 'rings', 'lens', 'spiral', 'rings', 'prism', 'ring'],
  helix: ['dial', 'helix', 'lattice', 'helix', 'prism', 'rings', 'lens'],
  wave: ['ring', 'wave', 'spiral', 'lens', 'wave', 'rings', 'prism'],
  cards: ['cards', 'lattice', 'cards', 'prism', 'cards', 'lens', 'ring'],
  geometry: ['ring', 'geometry', 'rings', 'prism', 'geometry', 'lens', 'dial'],
  lens: ['dial', 'lens', 'lattice', 'lens', 'rings', 'prism', 'ring'],
  lattice: ['lattice', 'cards', 'lattice', 'prism', 'ring', 'lens', 'lattice'],
  spiral: ['dial', 'spiral', 'helix', 'rings', 'spiral', 'lens', 'prism'],
  prism: ['ring', 'prism', 'lens', 'geometry', 'lattice', 'prism', 'rings'],
};
const SHAPE_ALIASES = {
  disc: 'dial', disk: 'dial', plate: 'cards', base: 'dial', clock: 'dial', rotor: 'dial', core: 'core',
  annulus: 'ring', torus: 'ring', orbit: 'orrery', orbital: 'orrery', planets: 'orrery',
  coil: 'helix', dna: 'helix', waveform: 'wave', ribbon: 'wave', sound: 'wave',
  crystal: 'prism', pyramid: 'prism', diamond: 'prism', polyhedron: 'geometry',
  sphere: 'geometry', orb: 'geometry', cage: 'geometry', memory: 'lattice',
  board: 'lattice', chip: 'lattice', processor: 'lattice', grid: 'lattice',
  sensor: 'lens', camera: 'lens', glass: 'lens', display: 'cards', screen: 'cards', card: 'cards',
};
const PROFILE_ALIASES = {
  'calendar-rings': 'orrery', 'temporal-dials': 'dial', 'period-lattice': 'rings',
  'orbital-comparator': 'orrery', 'wave-bank': 'wave', 'bodygraph-assembly': 'bodygraph',
  'fourfold-key': 'fourfold', 'reduction-stack': 'lattice', 'field-computation': 'helix',
  'facial-lens': 'lens', 'resonance-selector': 'wave', 'card-assembly': 'cards',
  'hexagram-scaffold': 'hexagram', 'ninefold-assessment': 'enneagram', 'form-library': 'geometry',
  'glyph-forge': 'glyph', 'musical-assembly': 'wave', 'reflection-composite': 'lattice',
  'capture-record': 'lens', 'record-capture': 'lens',
};

function surfaceTextures() {
  const size = 128; const patina = new Uint8Array(size * size * 4); const grain = new Uint8Array(size * size * 4); const stone = new Uint8Array(size * size * 4);
  const noise = (x, y) => { const value = Math.sin(x * 127.1 + y * 311.7 + 19.37) * 43758.5453; return value - Math.floor(value); };
  for (let y = 0; y < size; y += 1) {
    for (let x = 0; x < size; x += 1) {
      const index = (y * size + x) * 4; const n = noise(x, y);
      const pool = Math.sin(x * 0.099 + Math.sin(y * 0.08) * 2) * Math.cos(y * 0.117 + Math.sin(x * 0.043) * 3) * 0.5 + 0.5;
      const oxidation = clamp((pool + n * 0.1 - 0.72) * 2.7) * 0.48;
      const copper = [149 + n * 35, 93 + n * 25, 63 + n * 22]; const verdigris = [70 + n * 18, 111 + n * 19, 101 + n * 17];
      for (let channel = 0; channel < 3; channel += 1) patina[index + channel] = Math.round(THREE.MathUtils.lerp(copper[channel], verdigris[channel], oxidation));
      patina[index + 3] = 255;
      const g = Math.round(135 + n * 87 + Math.sin(x * 1.7) * 8); grain[index] = grain[index + 1] = grain[index + 2] = g; grain[index + 3] = 255;
      const speckle = noise(Math.floor(x / 2), Math.floor(y / 2)); const fragment = speckle > 0.91 ? 0.58 : speckle > 0.85 ? 0.82 : 1;
      stone[index] = (224 + n * 12) * fragment; stone[index + 1] = (213 + n * 13) * fragment; stone[index + 2] = (190 + n * 17) * fragment; stone[index + 3] = 255;
    }
  }
  const texture = (pixels, color = false) => {
    const value = new THREE.DataTexture(pixels, size, size, THREE.RGBAFormat); value.needsUpdate = true; value.wrapS = value.wrapT = THREE.RepeatWrapping;
    value.magFilter = THREE.LinearFilter; value.minFilter = THREE.LinearMipmapLinearFilter; value.generateMipmaps = true;
    if (color) value.colorSpace = THREE.SRGBColorSpace; return value;
  };
  const result = { patina: texture(patina, true), grain: texture(grain), stone: texture(stone, true) };
  result.patina.repeat.set(2.1, 2.1); result.grain.repeat.set(3, 3);
  return result;
}

function unavailable(onError, message) {
  queueMicrotask(() => onError?.(message));
  const noop = () => {};
  return { isAvailable: false, setEngine: noop, setExplosion: noop, setSelectedPart: noop, setIsolatedPart: noop, setReducedMotion: noop, recenter: noop, resize: noop, captureFrame: () => '', dispose: noop, setActive: noop };
}

function tintFor(engine) {
  const value = engine?.color ?? engine?.tint ?? engine?.categoryColor ?? engine?.category?.color ?? engine?.koshaColor;
  if (value instanceof THREE.Color) return value.clone();
  if (typeof value === 'number' || (typeof value === 'string' && /^#?[0-9a-f]{6}$/i.test(value))) {
    return new THREE.Color(typeof value === 'string' && !value.startsWith('#') ? `#${value}` : value);
  }
  return new THREE.Color(COLORS.emerald).lerp(new THREE.Color(COLORS.indigo), hash(engine?.id || 'engine') * 0.65);
}

/**
 * Self-contained, demand-rendered engine assembly. Programmatic setters never
 * dispatch onSelect. Only a canvas or numbered-label click selects a part.
 * onHover(partId|null, {x,y}) uses coordinates local to the supplied container.
 */
export function createEngineAssemblyScene(container, options = {}) {
  const { onSelect, onHover, onReady, onError } = options;
  if (!container) return unavailable(onError, 'The engine assembly stage is unavailable.');
  let renderer;
  try {
    renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true, preserveDrawingBuffer: true, powerPreference: 'high-performance' });
  } catch {
    return unavailable(onError, 'This device could not start the 3D assembly. The component descriptions remain available.');
  }

  const originalPosition = container.style.position;
  const changedPosition = getComputedStyle(container).position === 'static';
  if (changedPosition) container.style.position = 'relative';
  const canvas = renderer.domElement;
  canvas.className = 'engine-assembly-webgl';
  canvas.setAttribute('role', 'img');
  canvas.setAttribute('aria-label', 'Interactive schematic engine assembly. Drag to orbit, scroll to zoom, or choose a numbered component.');
  Object.assign(canvas.style, { position: 'absolute', inset: '0', display: 'block', width: '100%', height: '100%', touchAction: 'none' });
  container.appendChild(canvas);
  renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 1.65));
  renderer.setClearColor(COLORS.void, 0);
  renderer.outputColorSpace = THREE.SRGBColorSpace;
  renderer.toneMapping = THREE.ACESFilmicToneMapping;
  renderer.toneMappingExposure = 1.12;
  const textures = surfaceTextures();

  const scene = new THREE.Scene();
  const camera = new THREE.PerspectiveCamera(34, 1, 0.05, 100);
  const cameraDirection = new THREE.Vector3(0.76, 0.49, 1).normalize();
  camera.position.copy(cameraDirection).multiplyScalar(11);
  const controls = new OrbitControls(camera, canvas);
  controls.enableDamping = !options.reducedMotion;
  controls.dampingFactor = 0.095;
  controls.rotateSpeed = 0.57;
  controls.zoomSpeed = 0.65;
  controls.panSpeed = 0.46;
  controls.minDistance = 3.5;
  controls.maxDistance = 38;
  controls.minPolarAngle = Math.PI * 0.12;
  controls.maxPolarAngle = Math.PI * 0.84;
  scene.add(new THREE.HemisphereLight(COLORS.paper, COLORS.surface, 1.65));
  const keyLight = new THREE.DirectionalLight(0xf2ecd8, 3.4); keyLight.position.set(-4, 7, 5); scene.add(keyLight);
  const rimLight = new THREE.DirectionalLight(0xa4c1e5, 2.9); rimLight.position.set(5, 2, -4); scene.add(rimLight);
  const frontLight = new THREE.DirectionalLight(0xc8d5dd, 1.1); frontLight.position.set(2, -2, 6); scene.add(frontLight);
  let environmentTarget; let environmentRoom; let pmrem;
  try {
    environmentRoom = new RoomEnvironment();
    pmrem = new THREE.PMREMGenerator(renderer);
    environmentTarget = pmrem.fromScene(environmentRoom, 0.025);
    scene.environment = environmentTarget.texture;
  } catch {
    // Direct studio lights still provide a complete scene on limited devices.
  } finally { environmentRoom?.dispose(); pmrem?.dispose(); }
  const composer = new EffectComposer(renderer);
  const renderPass = new RenderPass(scene, camera);
  const bloom = new UnrealBloomPass(new THREE.Vector2(1, 1), 0.23, 0.42, 0.96);
  const outputPass = new OutputPass();
  composer.addPass(renderPass); composer.addPass(bloom); composer.addPass(outputPass);

  const labelLayer = document.createElement('div');
  labelLayer.className = 'engine-assembly-labels';
  labelLayer.setAttribute('aria-label', 'Assembly components');
  Object.assign(labelLayer.style, { position: 'absolute', inset: '0', overflow: 'hidden', pointerEvents: 'none', zIndex: '2' });
  const leaderLayer = document.createElementNS(SVG_NS, 'svg');
  leaderLayer.setAttribute('aria-hidden', 'true');
  Object.assign(leaderLayer.style, { position: 'absolute', inset: '0', width: '100%', height: '100%', overflow: 'hidden', pointerEvents: 'none' });
  labelLayer.appendChild(leaderLayer); container.appendChild(labelLayer);

  let disposed = false; let active = true; let contextLost = false;
  let width = 1; let height = 1; let frame = 0; let inFrame = false; let dirty = true; let lastTime = 0;
  let engineRoot = null; let foundation = null; let spine = null;
  let engineData = null; let engineSeed = 0; let assembledHeight = 1; let currentHalfHeight = 1; let family = 'orrery';
  let entries = []; let links = []; let pickMeshes = []; let entryMap = new Map();
  let engineGeometries = new Set(); let engineMaterials = new Set(); let geometryCache = new Map();
  let currentExplosion = 0; let targetExplosion = 0; let selectedId = null; let isolatedId = null; let isolationPartId = null; let hoveredId = null;
  let isolationMix = 0; let reducedMotion = Boolean(options.reducedMotion); let lastFitDistance = 11;
  let pointerDown = null;
  const raycaster = new THREE.Raycaster(); const pointer = new THREE.Vector2();
  const tempPosition = new THREE.Vector3(); const cameraOffset = new THREE.Vector3();
  const gold = new THREE.Color(COLORS.gold); const paper = new THREE.Color(COLORS.paper);

  function invalidate() {
    dirty = true;
    if (!disposed && active && !contextLost && !frame && !inFrame) frame = requestAnimationFrame(renderFrame);
  }
  function geometry(key, make) {
    if (!geometryCache.has(key)) { const value = make(); engineGeometries.add(value); geometryCache.set(key, value); }
    return geometryCache.get(key);
  }
  function material(entry, kind = 'body', extra = {}) {
    const color = kind === 'gold' ? COLORS.gold : kind === 'silver' ? COLORS.silver : kind === 'tint' ? new THREE.Color(COLORS.emerald).lerp(entry.tint, 0.11) : kind === 'dark' ? COLORS.surface : 0xf3d8bf;
    let value;
    if (kind === 'glass') {
      value = new THREE.MeshPhysicalMaterial({ color: entry.tint.clone().lerp(new THREE.Color(0x9d7748), 0.3), metalness: 0.04, roughness: 0.28, envMapIntensity: 0.3, transmission: 0.18, thickness: 0.28, ior: 1.45, clearcoat: 0.4, transparent: true, opacity: 0.64, depthWrite: false, side: THREE.DoubleSide, ...extra });
    } else {
      value = new THREE.MeshStandardMaterial({ color, metalness: kind === 'dark' ? 0.44 : kind === 'tint' ? 0.54 : 0.79, roughness: kind === 'gold' ? 0.31 : 0.48, envMapIntensity: kind === 'body' ? 0.72 : kind === 'tint' ? 0.58 : 0.9, map: kind === 'body' ? textures.patina : null, bumpMap: textures.grain, bumpScale: kind === 'body' ? 0.011 : 0.006, transparent: true, opacity: 1, ...extra });
    }
    value.userData.baseOpacity = value.opacity;
    value.userData.baseDepthWrite = value.depthWrite;
    value.userData.baseEmissive = value.emissive?.clone();
    value.userData.baseEmissiveIntensity = value.emissiveIntensity || 0;
    entry.materials.push(value); engineMaterials.add(value); return value;
  }
  function strokeMaterial(entry, color = COLORS.gold, opacity = 0.6) {
    const value = new THREE.LineBasicMaterial({ color, transparent: true, opacity, depthWrite: false });
    value.userData.baseOpacity = opacity; entry.materials.push(value); engineMaterials.add(value); return value;
  }
  function mesh(entry, shape, finish, position = [0, 0, 0], scale) {
    const object = new THREE.Mesh(shape, finish);
    object.position.set(...position); if (scale) object.scale.set(...scale);
    object.userData.partId = entry.id; entry.group.add(object); pickMeshes.push(object); return object;
  }
  function cylinder(entry, radius, depth, finish, position = [0, 0, 0], topRadius = radius) {
    return mesh(entry, geometry(`cylinder-${radius}-${topRadius}-${depth}`, () => new THREE.CylinderGeometry(topRadius, radius, depth, 80)), finish, position);
  }
  function sphere(entry, radius, finish, position = [0, 0, 0], scale) {
    return mesh(entry, geometry(`sphere-${radius}`, () => new THREE.SphereGeometry(radius, 28, 18)), finish, position, scale);
  }
  function torus(entry, radius, tube, finish, y = 0) {
    const object = mesh(entry, geometry(`torus-${radius}-${tube}`, () => new THREE.TorusGeometry(radius, tube, 8, 96)), finish, [0, y, 0]);
    object.rotation.x = Math.PI / 2; return object;
  }
  function annulus(entry, radius, innerRadius, depth, finish, y = 0) {
    const shape = geometry(`annulus-${radius}-${innerRadius}-${depth}`, () => {
      const outline = new THREE.Shape(); outline.absarc(0, 0, radius, 0, TAU, false);
      const hole = new THREE.Path(); hole.absarc(0, 0, innerRadius, 0, TAU, true); outline.holes.push(hole);
      return new THREE.ExtrudeGeometry(outline, { depth, bevelEnabled: true, bevelSegments: 2, steps: 1, bevelSize: 0.015, bevelThickness: 0.014, curveSegments: 64 });
    });
    const object = mesh(entry, shape, finish, [0, y - depth / 2, 0]); object.rotation.x = -Math.PI / 2; return object;
  }
  function line(entry, points, finish, closed = false) {
    const shape = new THREE.BufferGeometry().setFromPoints(points.map((point) => Array.isArray(point) ? new THREE.Vector3(...point) : point));
    engineGeometries.add(shape);
    const object = closed ? new THREE.LineLoop(shape, finish) : new THREE.Line(shape, finish); entry.group.add(object); return object;
  }
  function etching(entry, radius, y, count = 60, opacity = 0.54) {
    const vertices = [];
    for (let index = 0; index < count; index += 1) {
      const angle = index / count * TAU;
      const inner = radius - (index % 5 === 0 ? 0.12 : 0.047);
      vertices.push(Math.cos(angle) * inner, y, Math.sin(angle) * inner, Math.cos(angle) * radius, y, Math.sin(angle) * radius);
    }
    const shape = geometry(`ticks-${radius}-${y}-${count}`, () => new THREE.BufferGeometry().setAttribute('position', new THREE.Float32BufferAttribute(vertices, 3)));
    entry.group.add(new THREE.LineSegments(shape, strokeMaterial(entry, COLORS.paper, opacity)));
  }
  function circleLine(entry, radius, y, finish) {
    const shape = geometry(`circle-${radius}-${y}`, () => new THREE.BufferGeometry().setFromPoints(Array.from({ length: 96 }, (_, index) => new THREE.Vector3(Math.cos(index / 96 * TAU) * radius, y, Math.sin(index / 96 * TAU) * radius))));
    entry.group.add(new THREE.LineLoop(shape, finish));
  }
  function bolts(entry, radius, y, count = 6) {
    const shape = geometry('bolt', () => new THREE.CylinderGeometry(0.028, 0.028, 0.027, 10));
    const object = new THREE.InstancedMesh(shape, material(entry, 'gold'), count);
    const matrix = new THREE.Matrix4();
    for (let index = 0; index < count; index += 1) {
      const angle = TAU * index / count + Math.PI / 6;
      matrix.makeTranslation(Math.cos(angle) * radius, y, Math.sin(angle) * radius); object.setMatrixAt(index, matrix);
    }
    object.instanceMatrix.needsUpdate = true; object.userData.partId = entry.id; entry.group.add(object); pickMeshes.push(object);
  }
  function roundedPlate(entry, w, d, depth, finish, y = 0, yaw = 0) {
    const shape = geometry(`rounded-${w}-${d}-${depth}`, () => {
      const x = -w / 2; const z = -d / 2; const r = 0.12; const outline = new THREE.Shape();
      outline.moveTo(x + r, z); outline.lineTo(x + w - r, z); outline.quadraticCurveTo(x + w, z, x + w, z + r);
      outline.lineTo(x + w, z + d - r); outline.quadraticCurveTo(x + w, z + d, x + w - r, z + d);
      outline.lineTo(x + r, z + d); outline.quadraticCurveTo(x, z + d, x, z + d - r);
      outline.lineTo(x, z + r); outline.quadraticCurveTo(x, z, x + r, z);
      return new THREE.ExtrudeGeometry(outline, { depth, bevelEnabled: true, bevelSize: 0.018, bevelThickness: 0.012, bevelSegments: 2, curveSegments: 8, steps: 1 });
    });
    const object = mesh(entry, shape, finish, [0, y - depth / 2, 0]); object.rotation.set(-Math.PI / 2, 0, yaw); return object;
  }
  function tube(entry, points, radius, finish) {
    const curve = new THREE.CatmullRomCurve3(points.map((point) => new THREE.Vector3(...point)));
    const shape = new THREE.TubeGeometry(curve, Math.max(60, points.length * 2), radius, 6, false); engineGeometries.add(shape);
    return mesh(entry, shape, finish);
  }

  function makePart(entry, type) {
    const body = material(entry, 'body'); const metal = material(entry, 'silver'); const brass = material(entry, 'gold');
    const accent = material(entry, 'tint', { emissive: entry.tint, emissiveIntensity: 0.09 });
    const hairline = strokeMaterial(entry, COLORS.gold, 0.72);
    const ticks = 48 + Math.floor(engineSeed * 4) * 12;
    entry.group.rotation.y = (engineSeed - 0.5) * 0.65;

    if (type === 'core') {
      annulus(entry, 0.93, 0.69, 0.11, body, -0.18); torus(entry, 0.91, 0.025, brass, -0.11);
      const heart = material(entry, 'gold', { color: COLORS.amber, emissive: COLORS.amber, emissiveIntensity: 2.3, roughness: 0.31, metalness: 0.35 });
      sphere(entry, 0.255, heart, [0, 0.02, 0]);
      const teeth = new THREE.InstancedMesh(geometry('core-teeth', () => new THREE.BoxGeometry(0.09, 0.13, 0.1)), brass, 24);
      const matrix = new THREE.Matrix4(); const rotor = new THREE.Object3D();
      for (let index = 0; index < 24; index += 1) { const angle = index / 24 * TAU; rotor.position.set(Math.cos(angle) * 0.76, -0.14, Math.sin(angle) * 0.76); rotor.rotation.y = -angle; rotor.updateMatrix(); matrix.copy(rotor.matrix); teeth.setMatrixAt(index, matrix); }
      teeth.instanceMatrix.needsUpdate = true; teeth.userData.partId = entry.id; entry.group.add(teeth); pickMeshes.push(teeth);
      for (let index = 0; index < 3; index += 1) { const orbit = torus(entry, 0.49, 0.019, index === 1 ? accent : brass, 0.025); orbit.rotation.set(Math.PI / 2 + (index - 1) * 0.7, index * 0.9, index * 0.42); }
      entry.partHeight = 0.75;
    } else if (type === 'star') {
      const tetra = geometry('star-tetrahedron', () => new THREE.TetrahedronGeometry(0.94));
      const edges = geometry('star-tetrahedron-edges', () => new THREE.EdgesGeometry(tetra));
      const vertices = edges.attributes.position; const rodShape = geometry('star-rod', () => new THREE.CylinderGeometry(0.03, 0.03, 1, 8));
      const up = new THREE.Vector3(0, 1, 0); const start = new THREE.Vector3(); const end = new THREE.Vector3(); const direction = new THREE.Vector3();
      for (const inversion of [-1, 1]) {
        for (let index = 0; index < vertices.count; index += 2) {
          start.fromBufferAttribute(vertices, index).multiply(new THREE.Vector3(1, inversion, 1)); end.fromBufferAttribute(vertices, index + 1).multiply(new THREE.Vector3(1, inversion, 1));
          direction.subVectors(end, start);
          const rod = mesh(entry, rodShape, inversion < 0 ? brass : accent); rod.position.copy(start).add(end).multiplyScalar(0.5); rod.scale.y = direction.length(); rod.quaternion.setFromUnitVectors(up, direction.normalize());
        }
      }
      sphere(entry, 0.16, material(entry, 'gold', { color: COLORS.amber, emissive: COLORS.amber, emissiveIntensity: 1.9, metalness: 0.35 }), [0, 0, 0]);
      annulus(entry, 1.1, 1.01, 0.055, body, -0.35); etching(entry, 1.075, -0.31, 48, 0.52); entry.partHeight = 1.28;
    } else if (type === 'hexagram') {
      roundedPlate(entry, 1.88, 2.06, 0.085, body, -0.035);
      const solid = geometry('hexagram-solid', () => new THREE.BoxGeometry(1.32, 0.065, 0.14));
      const broken = geometry('hexagram-broken', () => new THREE.BoxGeometry(0.54, 0.065, 0.14));
      for (let row = 0; row < 6; row += 1) {
        const z = (row - 2.5) * 0.265; const yang = (row + entry.index) % 3 !== 1;
        if (yang) mesh(entry, solid, brass, [0, 0.06, z]);
        else { mesh(entry, broken, accent, [-0.4, 0.06, z]); mesh(entry, broken, accent, [0.4, 0.06, z]); }
      }
      entry.partHeight = 0.29;
    } else if (type === 'enneagram') {
      annulus(entry, 1.25, 1.06, 0.1, body); torus(entry, 1.23, 0.016, brass, 0.07);
      const points = Array.from({ length: 9 }, (_, index) => { const angle = index / 9 * TAU - Math.PI / 2; return [Math.cos(angle) * 0.96, 0.06, Math.sin(angle) * 0.96]; });
      for (const cycle of [[0, 3, 6], [1, 4, 7, 5, 2, 8]]) line(entry, cycle.map((index) => points[index]), hairline, true);
      points.forEach((point, index) => sphere(entry, index === entry.index % 9 ? 0.085 : 0.042, index === entry.index % 9 ? brass : accent, point));
      etching(entry, 1.19, 0.07, 54, 0.4); entry.partHeight = 0.28;
    } else if (type === 'bodygraph') {
      roundedPlate(entry, 1.84, 2.33, 0.085, body, -0.055);
      const positions = [[0, -0.91], [0, -0.58], [0, -0.18], [0, 0.24], [-0.55, 0.17], [0.54, 0.32], [-0.56, 0.68], [0.57, 0.72], [0, 0.95]];
      for (const [from, to] of [[0, 1], [1, 2], [2, 3], [2, 4], [2, 5], [3, 5], [3, 8], [4, 6], [6, 8], [5, 7], [7, 8]]) line(entry, [[positions[from][0], 0.025, positions[from][1]], [positions[to][0], 0.025, positions[to][1]]], hairline);
      for (let index = 0; index < positions.length; index += 1) { const [x, z] = positions[index]; const shape = geometry('bodygraph-node', () => new THREE.OctahedronGeometry(0.13)); mesh(entry, shape, index % 3 ? accent : brass, [x, 0.06, z], [1, 0.48, 1]); }
      entry.partHeight = 0.32;
    } else if (type === 'fourfold') {
      annulus(entry, 1.2, 1.09, 0.08, body); circleLine(entry, 1.17, 0.05, hairline);
      for (let index = 0; index < 4; index += 1) {
        const angle = index / 4 * TAU + Math.PI / 4; const x = Math.cos(angle) * 0.7; const z = Math.sin(angle) * 0.7;
        line(entry, [[0, 0.02, 0], [x, 0.02, z]], hairline);
        const key = mesh(entry, geometry('fourfold-key', () => new THREE.OctahedronGeometry(0.26)), index % 2 ? brass : accent, [x, 0.07, z], [1, 0.55, 1]); key.rotation.y = angle;
      }
      sphere(entry, 0.12, brass); entry.partHeight = 0.39;
    } else if (type === 'glyph') {
      cylinder(entry, 1.21, 0.085, body); torus(entry, 1.19, 0.015, brass, 0.055);
      const glyph = Array.from({ length: 7 }, (_, index) => { const angle = hash(`${engineData?.id}/${entry.id}/${index}`) * TAU; return [Math.cos(angle) * (index % 2 ? 0.82 : 0.55), 0.057, Math.sin(angle) * (index % 2 ? 0.82 : 0.55)]; });
      tube(entry, glyph, 0.012, brass); circleLine(entry, 0.89, 0.057, hairline); etching(entry, 1.13, 0.057, 60, 0.52); entry.partHeight = 0.26;
    } else if (type === 'ring' || type === 'rings') {
      annulus(entry, 1.28, type === 'rings' ? 0.95 : 0.66, 0.115, body);
      torus(entry, 1.275, 0.021, brass, 0.06); etching(entry, 1.235, 0.075, ticks);
      if (type === 'rings') {
        annulus(entry, 0.89, 0.66, 0.065, metal, 0.035);
        annulus(entry, 0.58, 0.4, 0.07, accent, 0.055);
        circleLine(entry, 0.43, 0.1, hairline);
        const spokeVertices = [];
        for (let index = 0; index < 3; index += 1) {
          const angle = index * TAU / 3; spokeVertices.push([Math.cos(angle) * 0.35, -0.005, Math.sin(angle) * 0.35], [Math.cos(angle) * 1.23, -0.005, Math.sin(angle) * 1.23]);
        }
        const shape = new THREE.BufferGeometry().setFromPoints(spokeVertices.map((point) => new THREE.Vector3(...point))); engineGeometries.add(shape);
        entry.group.add(new THREE.LineSegments(shape, hairline));
      } else { torus(entry, 0.68, 0.018, brass, 0.07); bolts(entry, 1.1, 0.095, 4); }
      entry.partHeight = 0.23;
    } else if (type === 'lens') {
      annulus(entry, 1.18, 1.0, 0.15, body);
      const glass = material(entry, 'glass'); sphere(entry, 1, glass, [0, 0.06, 0], [1, 0.14, 1]);
      torus(entry, 1.02, 0.017, brass, 0.08); torus(entry, 1.155, 0.014, metal, 0.08);
      circleLine(entry, 0.76, 0.185, strokeMaterial(entry, COLORS.paper, 0.38));
      const cross = 0.1;
      line(entry, [[-cross, 0.205, 0], [cross, 0.205, 0]], hairline); line(entry, [[0, 0.205, -cross], [0, 0.205, cross]], hairline);
      etching(entry, 1.13, 0.105, 36, 0.45); bolts(entry, 1.105, 0.12, 3); entry.partHeight = 0.4;
    } else if (type === 'prism' || type === 'geometry') {
      annulus(entry, 1.12, 0.89, 0.08, body, -0.12); torus(entry, 1.11, 0.012, brass, -0.065);
      const shape = geometry(type === 'prism' ? 'prism' : 'icosahedron', () => type === 'prism' ? new THREE.OctahedronGeometry(1, 0) : new THREE.IcosahedronGeometry(1, 0));
      const crystal = mesh(entry, shape, material(entry, 'glass', { opacity: 0.84, roughness: 0.19, transmission: 0.22 }), [0, 0.01, 0], type === 'prism' ? [0.77, 0.31, 0.77] : [0.66, 0.33, 0.66]);
      crystal.rotation.y = Math.PI / 4;
      const edges = geometry(`${type}-edges`, () => new THREE.EdgesGeometry(shape));
      const cage = new THREE.LineSegments(edges, strokeMaterial(entry, COLORS.paper, 0.64)); cage.position.copy(crystal.position); cage.scale.copy(crystal.scale); cage.rotation.copy(crystal.rotation); entry.group.add(cage);
      cylinder(entry, 0.16, 0.07, brass, [0, -0.3, 0]); etching(entry, 1.07, -0.057, 48, 0.34);
      entry.partHeight = 0.76;
    } else if (type === 'helix') {
      annulus(entry, 1.16, 0.97, 0.07, body, -0.2); annulus(entry, 1.16, 0.97, 0.07, metal, 0.2);
      const turns = 2.75 + Math.floor(engineSeed * 2) * 0.25;
      for (let strand = 0; strand < 2; strand += 1) {
        const points = Array.from({ length: 110 }, (_, index) => {
          const t = index / 109; const angle = t * turns * TAU + strand * Math.PI;
          return [Math.cos(angle) * 0.84, (t - 0.5) * 0.38, Math.sin(angle) * 0.84];
        });
        tube(entry, points, 0.022, strand ? accent : brass);
      }
      circleLine(entry, 1.11, 0.245, hairline); bolts(entry, 1.065, 0.245, 4); entry.partHeight = 0.58;
    } else if (type === 'wave') {
      roundedPlate(entry, 2.22, 1.75, 0.095, body, -0.07);
      for (let row = 0; row < 5; row += 1) {
        const points = Array.from({ length: 42 }, (_, index) => {
          const x = index / 41 * 1.98 - 0.99;
          return [x, 0.055 + Math.sin(index / 41 * TAU * (1.5 + engineSeed)) * 0.095 * Math.sin(index / 41 * Math.PI), (row - 2) * 0.28];
        });
        tube(entry, points, row === 2 ? 0.018 : 0.01, row === 2 ? brass : accent);
      }
      line(entry, [[-1.02, 0.01, -0.68], [-1.02, 0.01, 0.68]], hairline); line(entry, [[1.02, 0.01, -0.68], [1.02, 0.01, 0.68]], hairline);
      entry.partHeight = 0.36;
    } else if (type === 'cards') {
      for (let index = 0; index < 3; index += 1) roundedPlate(entry, 1.79, 2.18, 0.052, index === 2 ? metal : body, (index - 1) * 0.076, (index - 1) * 0.07);
      const y = 0.122;
      line(entry, [[-0.64, y, -0.83], [0.64, y, -0.83], [0.64, y, 0.83], [-0.64, y, 0.83]], hairline, true);
      circleLine(entry, 0.49, y + 0.006, strokeMaterial(entry, COLORS.surface, 0.85));
      const diamond = [[0, y + 0.011, -0.4], [0.31, y + 0.011, 0], [0, y + 0.011, 0.4], [-0.31, y + 0.011, 0]];
      line(entry, diamond, hairline, true); cylinder(entry, 0.07, 0.03, accent, [0, y + 0.02, 0]); entry.partHeight = 0.34;
    } else if (type === 'lattice') {
      roundedPlate(entry, 2.2, 2.2, 0.105, body, -0.075);
      const tiles = new THREE.InstancedMesh(geometry('lattice-tile', () => new THREE.BoxGeometry(0.25, 0.085, 0.25)), accent, 16);
      const matrix = new THREE.Matrix4();
      for (let index = 0; index < 16; index += 1) { matrix.makeTranslation((index % 4 - 1.5) * 0.42, 0.035, (Math.floor(index / 4) - 1.5) * 0.42); tiles.setMatrixAt(index, matrix); }
      tiles.instanceMatrix.needsUpdate = true; tiles.userData.partId = entry.id; entry.group.add(tiles); pickMeshes.push(tiles);
      const traces = [];
      for (let index = 0; index < 5; index += 1) {
        const position = (index - 2) * 0.42;
        traces.push([-0.94, -0.012, position], [0.94, -0.012, position], [position, -0.012, -0.94], [position, -0.012, 0.94]);
      }
      const traceShape = new THREE.BufferGeometry().setFromPoints(traces.map((point) => new THREE.Vector3(...point))); engineGeometries.add(traceShape);
      entry.group.add(new THREE.LineSegments(traceShape, strokeMaterial(entry, COLORS.gold, 0.66)));
      roundedPlate(entry, 0.37, 0.37, 0.12, brass, 0.075, Math.PI / 4); entry.partHeight = 0.34;
    } else if (type === 'spiral') {
      annulus(entry, 1.24, 1.12, 0.1, body); torus(entry, 1.21, 0.016, metal, 0.063);
      const points = Array.from({ length: 160 }, (_, index) => {
        const t = index / 159; const angle = t * TAU * (3.5 + engineSeed * 0.5); const radius = 0.12 + t * 0.92;
        return [Math.cos(angle) * radius, Math.sin(t * Math.PI) * 0.075, Math.sin(angle) * radius];
      });
      tube(entry, points, 0.024, brass); sphere(entry, 0.095, accent, points[0]);
      etching(entry, 1.195, 0.065, 48, 0.47); entry.partHeight = 0.29;
    } else if (type === 'orrery') {
      annulus(entry, 1.28, 1.07, 0.1, body); torus(entry, 1.27, 0.018, brass, 0.07);
      const orbit = torus(entry, 0.83, 0.016, metal, 0.05); orbit.rotation.z = 0.13;
      torus(entry, 0.48, 0.013, brass, 0.04);
      sphere(entry, 0.21, brass, [0, 0.065, 0], [1, 0.83, 1]);
      for (let index = 0; index < 3; index += 1) {
        const angle = index * 2.4 + engineSeed * TAU; const radius = index === 1 ? 0.48 : 0.83;
        sphere(entry, index === 1 ? 0.078 : 0.103, index === 2 ? metal : accent, [Math.cos(angle) * radius, 0.075, Math.sin(angle) * radius]);
      }
      etching(entry, 1.235, 0.07, ticks); entry.partHeight = 0.51;
    } else {
      cylinder(entry, 1.28, 0.13, body); cylinder(entry, 1.17, 0.018, metal, [0, 0.079, 0]);
      torus(entry, 1.265, 0.018, brass, 0.075); etching(entry, 1.12, 0.095, ticks, 0.82);
      circleLine(entry, 0.91, 0.098, strokeMaterial(entry, COLORS.surface, 0.66)); circleLine(entry, 0.65, 0.102, hairline);
      cylinder(entry, 0.14, 0.046, brass, [0, 0.12, 0]);
      const angle = engineSeed * TAU;
      line(entry, [[Math.cos(angle + Math.PI) * 0.2, 0.145, Math.sin(angle + Math.PI) * 0.2], [Math.cos(angle) * 0.85, 0.145, Math.sin(angle) * 0.85]], strokeMaterial(entry, COLORS.surface, 0.9));
      bolts(entry, 1.205, 0.107, 3); entry.partHeight = 0.31;
    }
    // A fine, non-blooming selection contour keeps material texture legible.
    entry.selectionRing = torus(entry, 1.39, 0.008, material(entry, 'gold', { opacity: 0, emissive: COLORS.gold, emissiveIntensity: 0.13 }), -entry.partHeight / 2 + 0.025);
    entry.selectionRing.material.userData.selectionContour = true;
  }

  function makeLabel(entry) {
    const button = document.createElement('button'); button.type = 'button'; button.className = 'engine-assembly-label';
    button.dataset.part = entry.id; button.setAttribute('aria-label', `Component ${entry.index + 1}: ${entry.data.label || entry.id}`); button.setAttribute('aria-pressed', 'false');
    Object.assign(button.style, { position: 'absolute', top: '0', left: '0', display: 'flex', alignItems: 'center', gap: '8px', maxWidth: '170px', height: '34px', border: '1px solid rgba(138,155,168,.2)', borderRadius: '3px', padding: '0 9px 0 6px', color: '#B4BFCA', background: 'rgba(7,11,29,.82)', font: '400 10px/1.25 ui-monospace, SFMono-Regular, Menlo, monospace', letterSpacing: '.01em', cursor: 'pointer', pointerEvents: 'auto', textAlign: 'left', whiteSpace: 'nowrap', boxShadow: '0 3px 18px rgba(0,0,0,.09)', visibility: 'hidden' });
    const number = document.createElement('span'); number.textContent = String(entry.index + 1).padStart(2, '0');
    Object.assign(number.style, { display: 'grid', placeItems: 'center', width: '21px', height: '21px', flex: '0 0 21px', color: '#C5A017', borderRight: '1px solid rgba(197,160,23,.24)', paddingRight: '5px', fontSize: '9px', letterSpacing: '.03em' });
    const name = document.createElement('span'); name.textContent = entry.data.label || entry.id; Object.assign(name.style, { overflow: 'hidden', textOverflow: 'ellipsis' });
    button.append(number, name);
    const leader = document.createElementNS(SVG_NS, 'polyline'); leader.setAttribute('fill', 'none'); leader.setAttribute('stroke-width', '0.7'); leader.setAttribute('stroke', '#8A9BA8'); leader.setAttribute('stroke-opacity', '.32');
    const dot = document.createElementNS(SVG_NS, 'circle'); dot.setAttribute('r', '2'); dot.setAttribute('fill', '#C5A017'); dot.setAttribute('fill-opacity', '.78');
    leaderLayer.append(leader, dot); labelLayer.appendChild(button);
    const hover = (event) => { const rect = container.getBoundingClientRect(); setHover(entry.id, { x: event.clientX - rect.left, y: event.clientY - rect.top }); };
    const leave = () => setHover(null, null);
    const click = (event) => { event.stopPropagation(); if (!disposed && active) onSelect?.(entry.id); };
    const focus = () => { button.style.outline = '1px solid #C5A017'; button.style.outlineOffset = '3px'; setHover(entry.id, null); };
    const blur = () => { button.style.outline = ''; button.style.outlineOffset = ''; setHover(null, null); };
    button.addEventListener('click', click); button.addEventListener('pointerenter', hover); button.addEventListener('pointermove', hover); button.addEventListener('pointerleave', leave); button.addEventListener('focus', focus); button.addEventListener('blur', blur);
    entry.label = button; entry.leader = leader; entry.dot = dot;
    entry.removeLabel = () => { button.removeEventListener('click', click); button.removeEventListener('pointerenter', hover); button.removeEventListener('pointermove', hover); button.removeEventListener('pointerleave', leave); button.removeEventListener('focus', focus); button.removeEventListener('blur', blur); button.remove(); leader.remove(); dot.remove(); };
  }

  function clearEngine() {
    entries.forEach((entry) => entry.removeLabel?.());
    if (engineRoot) scene.remove(engineRoot);
    engineGeometries.forEach((value) => value.dispose()); engineMaterials.forEach((value) => value.dispose());
    engineGeometries = new Set(); engineMaterials = new Set(); geometryCache = new Map();
    entries = []; links = []; pickMeshes = []; entryMap = new Map(); engineRoot = null; foundation = null; spine = null;
  }

  function setEngine(engine) {
    if (disposed) return;
    clearEngine(); engineData = engine || null; selectedId = null; isolatedId = null; isolationPartId = null; hoveredId = null; isolationMix = 0;
    currentExplosion = targetExplosion = 0;
    if (!engine || !Array.isArray(engine.parts) || !engine.parts.length) { invalidate(); return; }
    engineSeed = hash(engine.id || engine.label);
    const profileNames = Object.keys(PROFILES);
    const profile = String(engine.profile || '').toLowerCase();
    family = PROFILE_ALIASES[profile] || profile;
    const shapes = PROFILES[family] || PROFILES[profileNames[Math.floor(engineSeed * profileNames.length)]];
    const tint = tintFor(engine);
    engineRoot = new THREE.Group(); scene.add(engineRoot);
    const usedIds = new Set();
    for (const part of engine.parts) {
      if (!part || part.id == null || usedIds.has(String(part.id))) continue;
      const id = String(part.id); usedIds.add(id);
      const entry = { id, data: part, index: entries.length, group: new THREE.Group(), materials: [], tint: tint.clone(), partHeight: 0.3, assembledY: 0, position: new THREE.Vector3(), projected: new THREE.Vector3(), selectionMix: 0, dimMix: 0, baseScale: 1, restX: 0, restZ: 0, currentHalfHeight: 0.2 };
      const requestedShape = String(part.shape || '').toLowerCase().replace(/[ _]/g, '-');
      let shape = PROFILES[requestedShape] ? requestedShape : SHAPE_ALIASES[requestedShape] || shapes[entry.index % shapes.length];
      if (family === 'orrery' && shape !== 'core') {
        if (shape === 'dial') shape = 'ring'; else if (shape === 'cards') shape = 'orrery';
        if (shape === 'ring' || shape === 'rings' || shape === 'orrery') {
          entry.restX = [0.1, 1.22, -0.47, -1.13, 0.7, 0.21, 1.45][entry.index % 7];
          entry.restZ = entry.index % 2 ? 0.3 : -0.3; entry.baseScale = 0.86 + entry.index * 0.055;
        }
      }
      if (family === 'geometry') {
        if (shape === 'prism' || shape === 'geometry') {
          shape = 'star';
          entry.baseScale = entry.index === 0 ? 1.2 : 0.9;
          entry.restZ = entry.index % 2 ? 0.18 : 0;
        } else entry.baseScale = 0.4;
      }
      if (family === 'bodygraph' && (shape === 'helix' || shape === 'cards')) shape = 'bodygraph';
      if (family === 'hexagram' && (shape === 'cards' || shape === 'dial')) shape = 'hexagram';
      if (family === 'enneagram' && (shape === 'ring' || shape === 'rings')) shape = 'enneagram';
      if (family === 'fourfold' && (shape === 'cards' || shape === 'prism')) shape = 'fourfold';
      if (family === 'glyph' && (shape === 'cards' || shape === 'dial')) shape = 'glyph';
      makePart(entry, shape); engineRoot.add(entry.group); entries.push(entry); entryMap.set(id, entry); makeLabel(entry);
    }
    if (!entries.length) { invalidate(); return; }
    const gap = 0.12;
    assembledHeight = entries.reduce((total, entry) => total + entry.partHeight, 0) + gap * (entries.length - 1);
    let level = -assembledHeight / 2;
    for (const entry of entries) {
      // Nested software instruments become distinct responsibilities as they open.
      entry.assembledY = family === 'orrery'
        ? (entry.index - (entries.length - 1) / 2) * 0.1
        : (level + entry.partHeight / 2) * (family === 'geometry' ? 0.08 : 0.36);
      level += entry.partHeight + gap;
    }
    assembledHeight = family === 'orrery' ? 3.6 : Math.max(...entries.map(entry => Math.abs(entry.assembledY) + entry.partHeight * entry.baseScale / 2)) * 2;
    currentHalfHeight = assembledHeight / 2;

    // An engraved drafting plinth and alignment axis belong to the schematic,
    // never to an engine's component inventory or its pick targets.
    foundation = new THREE.Group(); engineRoot.add(foundation);
    const stoneMaterial = new THREE.MeshStandardMaterial({ color: 0x9b8870, map: textures.stone, bumpMap: textures.grain, bumpScale: 0.012, roughness: 0.98, metalness: 0.02, envMapIntensity: 0.22 }); engineMaterials.add(stoneMaterial);
    const plinthShape = geometry('stone-plinth', () => new THREE.CylinderGeometry(1.78, 1.8, 0.24, 96));
    const plinth = new THREE.Mesh(plinthShape, stoneMaterial); plinth.position.y = -0.19; foundation.add(plinth);
    const rimMaterial = new THREE.MeshStandardMaterial({ color: COLORS.gold, metalness: 0.8, roughness: 0.39 }); engineMaterials.add(rimMaterial);
    const rimShape = geometry('plinth-rim', () => new THREE.TorusGeometry(1.79, 0.015, 8, 96));
    const plinthRim = new THREE.Mesh(rimShape, rimMaterial); plinthRim.rotation.x = Math.PI / 2; plinthRim.position.y = -0.064; foundation.add(plinthRim);
    const guideMaterial = new THREE.LineBasicMaterial({ color: COLORS.gold, transparent: true, opacity: 0.22, depthWrite: false }); engineMaterials.add(guideMaterial);
    for (const radius of [1.7, 1.79, 2.08]) {
      const shape = geometry(`plinth-${radius}`, () => new THREE.BufferGeometry().setFromPoints(Array.from({ length: 128 }, (_, index) => new THREE.Vector3(Math.cos(index / 128 * TAU) * radius, 0, Math.sin(index / 128 * TAU) * radius))));
      foundation.add(new THREE.LineLoop(shape, guideMaterial));
    }
    const marks = [];
    for (let index = 0; index < 48; index += 1) {
      const angle = index / 48 * TAU; const inner = index % 4 === 0 ? 1.88 : 2.02;
      marks.push(Math.cos(angle) * inner, 0, Math.sin(angle) * inner, Math.cos(angle) * 2.1, 0, Math.sin(angle) * 2.1);
    }
    const markShape = geometry('plinth-ticks', () => new THREE.BufferGeometry().setAttribute('position', new THREE.Float32BufferAttribute(marks, 3)));
    foundation.add(new THREE.LineSegments(markShape, guideMaterial));
    const axisShape = new THREE.BufferGeometry().setFromPoints([new THREE.Vector3(0, -1, 0), new THREE.Vector3(0, 1, 0)]); engineGeometries.add(axisShape);
    const axisMaterial = new THREE.LineDashedMaterial({ color: COLORS.gold, transparent: true, opacity: 0.14, dashSize: 0.07, gapSize: 0.085, depthWrite: false }); engineMaterials.add(axisMaterial);
    spine = new THREE.Line(axisShape, axisMaterial); spine.computeLineDistances(); engineRoot.add(spine);
    for (const connection of engine.connections || []) {
      const from = entryMap.get(String(connection.from)); const to = entryMap.get(String(connection.to));
      if (!from || !to || from === to) continue;
      const positions = new Float32Array(21 * 3);
      const shape = new THREE.BufferGeometry().setAttribute('position', new THREE.BufferAttribute(positions, 3).setUsage(THREE.DynamicDrawUsage)); engineGeometries.add(shape);
      const finish = new THREE.LineBasicMaterial({ color: COLORS.gold, transparent: true, opacity: 0.17, depthWrite: false }); engineMaterials.add(finish);
      const object = new THREE.Line(shape, finish); object.frustumCulled = false; engineRoot.add(object);
      links.push({ from, to, positions, object, side: hash(`${connection.from}/${connection.to}`) > 0.5 ? 1 : -1 });
    }
    canvas.setAttribute('aria-label', `${engine.label || engine.id} schematic assembly, ${entries.length} components. Drag to orbit, scroll to zoom, or choose a numbered component.`);
    resize(); recenter(); updateTransforms(); invalidate();
    queueMicrotask(() => { if (!disposed && engineData === engine) onReady?.({ engineId: engine.id, partCount: entries.length, schematic: true }); });
  }

  function fitDistance() {
    const halfHeight = currentHalfHeight;
    const tan = Math.tan(THREE.MathUtils.degToRad(camera.fov / 2));
    const usableAspect = camera.aspect * (width < 600 ? 0.8 : 0.62);
    const full = Math.max((halfHeight * 0.93 + 0.7) / tan, 1.75 / (tan * Math.max(0.2, usableAspect))) + 1.8;
    const isolated = Math.max(6.6, 1.6 / (tan * Math.max(0.3, usableAspect))) + 0.5;
    return THREE.MathUtils.lerp(full, isolated, isolationMix);
  }
  function fitCamera(reset = false) {
    const distance = fitDistance();
    if (reset) { controls.target.set(0, 0, 0); camera.position.copy(cameraDirection).multiplyScalar(distance); }
    else if (Math.abs(distance - lastFitDistance) > 0.00001) {
      cameraOffset.copy(camera.position).sub(controls.target).multiplyScalar(distance / lastFitDistance); camera.position.copy(controls.target).add(cameraOffset);
    }
    lastFitDistance = distance; controls.minDistance = Math.max(2.7, distance * 0.33); controls.maxDistance = Math.max(32, distance * 3);
  }

  function updateTransforms() {
    if (!entries.length) return;
    engineRoot.position.y = 0.25 * (1 - isolationMix);
    const middle = (entries.length - 1) / 2;
    for (const entry of entries) {
      const stagger = entry.index / Math.max(1, entries.length - 1) * 0.09;
      const spread = ease((currentExplosion - stagger) / (1 - stagger));
      const offset = entry.index - middle;
      entry.position.set(0, entry.assembledY + offset * 0.98 * spread, 0);
      if (entry.id === isolationPartId) entry.position.multiplyScalar(1 - isolationMix);
      entry.group.position.copy(entry.position);
      const orientation = Math.max(spread, entry.id === isolationPartId ? isolationMix * 0.65 : 0);
      entry.group.rotation.set(entry.restX * (1 - orientation), (engineSeed - 0.5) * 0.65, entry.restZ * (1 - orientation));
      const focusScale = THREE.MathUtils.lerp(entry.baseScale, 1.25, entry.id === isolationPartId ? isolationMix : 0);
      entry.group.scale.setScalar(focusScale * (1 + entry.selectionMix * 0.035));
      entry.currentHalfHeight = THREE.MathUtils.lerp(family === 'orrery' ? 1.58 * entry.baseScale : entry.partHeight * entry.baseScale / 2, entry.partHeight * entry.baseScale / 2, spread);
      const fade = 1 - entry.dimMix * 0.965;
      entry.group.visible = entry.dimMix < 0.995;
      for (const finish of entry.materials) {
        const base = finish.userData.baseOpacity ?? 1;
        finish.opacity = finish.userData.selectionContour ? entry.selectionMix * 0.66 * fade : base * fade;
        if (finish.isMeshStandardMaterial) finish.depthWrite = entry.dimMix > 0.05 ? false : finish.userData.baseDepthWrite;
        if (finish.emissive && !finish.userData.selectionContour) {
          finish.emissive.copy(finish.userData.baseEmissive || new THREE.Color(0)).lerp(gold, entry.selectionMix * 0.075);
          finish.emissiveIntensity = (finish.userData.baseEmissiveIntensity || 0) + entry.selectionMix * 0.1;
        }
      }
      entry.label.style.color = entry.selectionMix > 0.15 ? '#F0EDE3' : '#AFBDC9';
      entry.label.style.borderColor = entry.selectionMix > 0.15 ? 'rgba(197,160,23,.65)' : 'rgba(138,155,168,.2)';
      entry.label.style.background = entry.selectionMix > 0.15 ? 'rgba(25,27,30,.94)' : 'rgba(7,11,29,.83)';
      entry.label.setAttribute('aria-pressed', String(entry.id === selectedId));
      entry.leader.setAttribute('stroke', entry.selectionMix > 0.15 ? '#C5A017' : '#8A9BA8');
      entry.leader.setAttribute('stroke-opacity', String(0.24 + entry.selectionMix * 0.4));
    }
    const bottom = Math.min(...entries.map((entry) => entry.position.y - entry.currentHalfHeight)) - 0.23;
    const top = Math.max(...entries.map((entry) => entry.position.y + entry.currentHalfHeight)) + 0.22;
    currentHalfHeight = Math.max(Math.abs(bottom), Math.abs(top));
    foundation.position.y = bottom; foundation.visible = isolationMix < 0.999;
    foundation.traverse((object) => {
      const finish = object.material; if (!finish) return;
      if (finish.userData.foundationOpacity === undefined) finish.userData.foundationOpacity = finish.opacity;
      if (finish.userData.foundationDepthWrite === undefined) finish.userData.foundationDepthWrite = finish.depthWrite;
      finish.transparent = true; finish.opacity = finish.userData.foundationOpacity * (1 - isolationMix); finish.depthWrite = finish.userData.foundationDepthWrite && isolationMix < 0.05;
    });
    spine.position.y = (top + bottom) / 2; spine.scale.y = (top - bottom) / 2; spine.material.opacity = (0.11 + currentExplosion * 0.12) * (1 - isolationMix);
    for (const link of links) {
      const selected = selectedId === link.from.id || selectedId === link.to.id;
      link.object.material.opacity = (0.07 + currentExplosion * 0.12 + (selected ? 0.13 : 0)) * (1 - isolationMix);
      const a = link.from.position; const b = link.to.position;
      const bend = 1.43 + Math.abs(a.y - b.y) * 0.15;
      for (let index = 0; index <= 20; index += 1) {
        const t = index / 20; const k = index * 3; const arc = Math.sin(t * Math.PI);
        link.positions[k] = THREE.MathUtils.lerp(a.x, b.x, t) + arc * bend * link.side;
        link.positions[k + 1] = THREE.MathUtils.lerp(a.y, b.y, t);
        link.positions[k + 2] = THREE.MathUtils.lerp(a.z, b.z, t) - arc * 0.64;
      }
      link.object.geometry.attributes.position.needsUpdate = true;
    }
    fitCamera();
  }

  function updateLabels() {
    if (width < 2 || height < 2) return;
    camera.updateMatrixWorld(); engineRoot?.updateMatrixWorld(true);
    const margin = width < 480 ? 10 : 22;
    const labelWidth = width < 480 ? 105 : width < 740 ? 132 : 164;
    const sides = [[], []];
    for (const entry of entries) {
      const visible = !isolatedId || entry.id === isolatedId || isolationMix < 0.1;
      entry.group.getWorldPosition(tempPosition); entry.projected.copy(tempPosition).project(camera);
      const x = (entry.projected.x * 0.5 + 0.5) * width; const y = (0.5 - entry.projected.y * 0.5) * height;
      const inFrame = visible && entry.projected.z > -1 && entry.projected.z < 1 && x > -20 && x < width + 20 && y > -40 && y < height + 40;
      entry.label.style.visibility = inFrame ? 'visible' : 'hidden'; entry.leader.style.visibility = inFrame ? 'visible' : 'hidden'; entry.dot.style.visibility = inFrame ? 'visible' : 'hidden';
      entry.label.tabIndex = inFrame ? 0 : -1;
      if (!inFrame) continue;
      entry.label.style.width = `${labelWidth}px`; entry.label.style.maxWidth = `${labelWidth}px`;
      const side = entry.index % 2;
      sides[side].push({ entry, x, y, top: clamp(y - 17, margin, height - margin - 34) });
    }
    for (let side = 0; side < 2; side += 1) {
      const labels = sides[side].sort((a, b) => a.top - b.top);
      const gap = Math.min(43, (height - margin * 2 - 34) / Math.max(1, labels.length - 1));
      for (let index = 1; index < labels.length; index += 1) labels[index].top = Math.max(labels[index].top, labels[index - 1].top + gap);
      if (labels.length) {
        labels[labels.length - 1].top = Math.min(labels[labels.length - 1].top, height - margin - 34);
        for (let index = labels.length - 2; index >= 0; index -= 1) labels[index].top = Math.min(labels[index].top, labels[index + 1].top - gap);
      }
      for (const label of labels) {
        const { entry, x, y } = label;
        const left = side ? width - margin - labelWidth : margin;
        const joinX = side ? left : left + labelWidth; const joinY = label.top + 17;
        const anchorX = clamp(x + (side ? 17 : -17), 0, width);
        const kneeX = side ? joinX - 12 : joinX + 12;
        entry.label.style.transform = `translate3d(${Math.round(left)}px,${Math.round(label.top)}px,0)`;
        entry.leader.setAttribute('points', `${anchorX.toFixed(1)},${y.toFixed(1)} ${kneeX.toFixed(1)},${joinY.toFixed(1)} ${joinX.toFixed(1)},${joinY.toFixed(1)}`);
        entry.dot.setAttribute('cx', anchorX.toFixed(1)); entry.dot.setAttribute('cy', y.toFixed(1));
      }
    }
  }

  function renderFrame(time) {
    frame = 0;
    if (disposed || !active || contextLost) return;
    inFrame = true;
    const dt = Math.min(0.05, lastTime ? Math.max(0.001, (time - lastTime) / 1000) : 1 / 60); lastTime = time;
    const blend = reducedMotion ? 1 : 1 - Math.exp(-dt * 12.5);
    let moving = false;
    const nextExplosion = currentExplosion + (targetExplosion - currentExplosion) * blend;
    currentExplosion = Math.abs(nextExplosion - targetExplosion) < 0.0002 ? targetExplosion : nextExplosion;
    moving ||= currentExplosion !== targetExplosion;
    const isolationTarget = isolatedId ? 1 : 0;
    isolationMix += (isolationTarget - isolationMix) * blend;
    if (Math.abs(isolationMix - isolationTarget) < 0.0005) isolationMix = isolationTarget; else moving = true;
    if (!isolationMix && !isolatedId) isolationPartId = null;
    for (const entry of entries) {
      const selectionTarget = entry.id === selectedId || entry.id === hoveredId || entry.id === isolatedId ? 1 : 0;
      entry.selectionMix += (selectionTarget - entry.selectionMix) * blend;
      if (Math.abs(entry.selectionMix - selectionTarget) < 0.001) entry.selectionMix = selectionTarget; else moving = true;
      const dimTarget = isolatedId && entry.id !== isolatedId ? 1 : 0;
      entry.dimMix += (dimTarget - entry.dimMix) * blend;
      if (Math.abs(entry.dimMix - dimTarget) < 0.001) entry.dimMix = dimTarget; else moving = true;
    }
    updateTransforms();
    const cameraChanged = controls.update();
    if (dirty || moving || cameraChanged) { composer.render(); updateLabels(); dirty = false; }
    inFrame = false;
    if (moving || cameraChanged) invalidate();
    else lastTime = 0;
  }

  function localPoint(event) {
    const rect = container.getBoundingClientRect(); return { x: event.clientX - rect.left, y: event.clientY - rect.top };
  }
  function pick(event) {
    if (!entries.length || width < 2 || height < 2) return null;
    const point = localPoint(event); pointer.set(point.x / width * 2 - 1, 1 - point.y / height * 2);
    scene.updateMatrixWorld(true); camera.updateMatrixWorld(); raycaster.setFromCamera(pointer, camera);
    const available = isolatedId ? pickMeshes.filter((object) => object.userData.partId === isolatedId) : pickMeshes;
    return raycaster.intersectObjects(available, false)[0]?.object.userData.partId || null;
  }
  function setHover(id, point) {
    if (disposed || !active) return;
    if (hoveredId !== id) { hoveredId = id; canvas.style.cursor = id ? 'pointer' : 'grab'; invalidate(); }
    onHover?.(id, point);
  }
  function pointerMove(event) {
    if (pointerDown) { if (Math.hypot(event.clientX - pointerDown.x, event.clientY - pointerDown.y) > 5) pointerDown.moved = true; return; }
    setHover(pick(event), localPoint(event));
  }
  function pointerStart(event) {
    if (!active || event.button !== 0) return;
    pointerDown = { x: event.clientX, y: event.clientY, moved: false, id: event.pointerId };
    canvas.style.cursor = 'grabbing';
  }
  function pointerEnd(event) {
    const start = pointerDown; pointerDown = null; canvas.style.cursor = hoveredId ? 'pointer' : 'grab';
    if (!active || !start || start.id !== event.pointerId || start.moved || Math.hypot(event.clientX - start.x, event.clientY - start.y) > 5) return;
    const id = pick(event); if (id) onSelect?.(id);
  }
  function pointerCancel() { pointerDown = null; canvas.style.cursor = 'grab'; setHover(null, null); }
  function pointerLeave() { if (!pointerDown) setHover(null, null); }
  function loseContext(event) { event.preventDefault(); contextLost = true; if (frame) cancelAnimationFrame(frame); frame = 0; onError?.('The 3D graphics context was interrupted. Component descriptions remain available.'); }
  function restoreContext() { contextLost = false; invalidate(); }
  canvas.addEventListener('pointermove', pointerMove); canvas.addEventListener('pointerdown', pointerStart); canvas.addEventListener('pointerup', pointerEnd);
  canvas.addEventListener('pointercancel', pointerCancel); canvas.addEventListener('pointerleave', pointerLeave);
  canvas.addEventListener('webglcontextlost', loseContext); canvas.addEventListener('webglcontextrestored', restoreContext);
  controls.addEventListener('change', invalidate); controls.addEventListener('start', invalidate); controls.addEventListener('end', invalidate);

  function resize() {
    if (disposed) return;
    const rect = container.getBoundingClientRect(); const nextWidth = Math.floor(rect.width); const nextHeight = Math.floor(rect.height);
    if (nextWidth < 2 || nextHeight < 2) return;
    if (width === nextWidth && height === nextHeight) { invalidate(); return; }
    width = nextWidth; height = nextHeight; camera.aspect = width / height; camera.updateProjectionMatrix();
    renderer.setSize(width, height, false); composer.setSize(width, height); leaderLayer.setAttribute('viewBox', `0 0 ${width} ${height}`);
    fitCamera(); invalidate();
  }
  const observer = typeof ResizeObserver !== 'undefined' ? new ResizeObserver(resize) : null;
  observer?.observe(container); window.addEventListener('resize', resize);

  function setExplosion(value, settings = {}) {
    if (disposed) return;
    targetExplosion = clamp(value);
    if (settings?.immediate || reducedMotion) currentExplosion = targetExplosion;
    invalidate();
  }
  function setSelectedPart(id) {
    if (disposed) return;
    selectedId = id != null && entryMap.has(String(id)) ? String(id) : null; invalidate();
  }
  function setIsolatedPart(id) {
    if (disposed) return;
    isolatedId = id != null && entryMap.has(String(id)) ? String(id) : null;
    if (isolatedId) { selectedId = isolatedId; isolationPartId = isolatedId; }
    if (reducedMotion) isolationMix = isolatedId ? 1 : 0;
    invalidate();
  }
  function setReducedMotion(value) {
    reducedMotion = Boolean(value); controls.enableDamping = !reducedMotion;
    if (reducedMotion) { currentExplosion = targetExplosion; isolationMix = isolatedId ? 1 : 0; }
    invalidate();
  }
  function recenter() { if (disposed) return; fitCamera(true); controls.update(); invalidate(); }
  function setActive(value) {
    if (disposed) return;
    active = Boolean(value); controls.enabled = active; lastTime = 0;
    labelLayer.style.display = active ? '' : 'none';
    if (!active) { if (frame) cancelAnimationFrame(frame); frame = 0; pointerDown = null; hoveredId = null; }
    else { resize(); invalidate(); }
  }
  function captureFrame() {
    if (disposed || contextLost) return '';
    try { updateTransforms(); controls.update(); composer.render(); return canvas.toDataURL('image/png'); }
    catch { onError?.('The current assembly frame could not be captured.'); return ''; }
  }
  function dispose() {
    if (disposed) return;
    disposed = true; if (frame) cancelAnimationFrame(frame); frame = 0;
    observer?.disconnect(); window.removeEventListener('resize', resize);
    controls.removeEventListener('change', invalidate); controls.removeEventListener('start', invalidate); controls.removeEventListener('end', invalidate); controls.dispose();
    canvas.removeEventListener('pointermove', pointerMove); canvas.removeEventListener('pointerdown', pointerStart); canvas.removeEventListener('pointerup', pointerEnd);
    canvas.removeEventListener('pointercancel', pointerCancel); canvas.removeEventListener('pointerleave', pointerLeave);
    canvas.removeEventListener('webglcontextlost', loseContext); canvas.removeEventListener('webglcontextrestored', restoreContext);
    clearEngine(); scene.environment = null; environmentTarget?.dispose(); Object.values(textures).forEach((texture) => texture.dispose());
    renderPass.dispose?.(); bloom.dispose(); outputPass.dispose(); composer.dispose(); renderer.dispose(); renderer.forceContextLoss();
    labelLayer.remove(); canvas.remove();
    if (changedPosition && container.style.position === 'relative') container.style.position = originalPosition;
  }

  resize(); invalidate();
  return { isAvailable: true, setEngine, setExplosion, setSelectedPart, setIsolatedPart, setReducedMotion, recenter, resize, captureFrame, dispose, setActive };
}
