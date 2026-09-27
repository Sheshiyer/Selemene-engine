import * as THREE from 'three';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';
import { mergeGeometries } from 'three/examples/jsm/utils/BufferGeometryUtils.js';

export const KOSHA_COLORS = {
  annamaya: '#C5A78A', pranamaya: '#58C4B2', manomaya: '#789DE0',
  vijnanamaya: '#B29CD6', anandamaya: '#D4B65F',
};
const TAU = Math.PI * 2;
const JADE = 0x83b8ae;
const GOLD = 0xd4b65f;
const smooth = (a, b, n) => { const t = Math.max(0, Math.min(1, (n - a) / (b - a))); return t * t * (3 - 2 * t); };

export function disposeObject(object) {
  const geometries = new Set(); const materials = new Set(); const textures = new Set();
  object.traverse((child) => {
    if (child.geometry) geometries.add(child.geometry);
    for (const material of Array.isArray(child.material) ? child.material : [child.material]) {
      if (!material) continue;
      materials.add(material);
      for (const value of Object.values(material)) if (value?.isTexture) textures.add(value);
    }
  });
  geometries.forEach((value) => value.dispose());
  materials.forEach((value) => value.dispose());
  textures.forEach((value) => value.dispose());
  object.removeFromParent(); object.clear();
}

function line(points, color, opacity, segments = false) {
  const geometry = new THREE.BufferGeometry().setFromPoints(points);
  const material = new THREE.LineBasicMaterial({ color, transparent: true, opacity, depthWrite: false });
  return segments ? new THREE.LineSegments(geometry, material) : new THREE.Line(geometry, material);
}
function ellipse(rx, ry, z = 0, samples = 160) {
  return Array.from({ length: samples + 1 }, (_, i) => {
    const a = i / samples * TAU;
    return new THREE.Vector3(Math.cos(a) * rx, Math.sin(a) * ry, z);
  });
}
function ellipsoidGrid(rx, ry, rz) {
  const points = [];
  const loop = (fn) => {
    for (let i = 0; i < 96; i += 1) points.push(fn(i / 96 * TAU), fn((i + 1) / 96 * TAU));
  };
  for (let i = 0; i < 5; i += 1) {
    const a = i / 5 * Math.PI;
    loop((t) => new THREE.Vector3(Math.cos(t) * rx * Math.cos(a), Math.sin(t) * ry, Math.cos(t) * rz * Math.sin(a)));
  }
  for (const y of [-0.64, 0, 0.64]) {
    const radius = Math.sqrt(1 - y * y);
    loop((t) => new THREE.Vector3(Math.cos(t) * rx * radius, y * ry, Math.sin(t) * rz * radius));
  }
  return points;
}

function buildReference() {
  const group = new THREE.Group();
  group.name = 'Vitruvian proportion study';
  group.add(line(ellipse(3.82, 3.82, -0.85), GOLD, 0.24));
  group.add(line([
    new THREE.Vector3(-3.4, -3.4, -0.88), new THREE.Vector3(3.4, -3.4, -0.88),
    new THREE.Vector3(3.4, 3.4, -0.88), new THREE.Vector3(-3.4, 3.4, -0.88),
    new THREE.Vector3(-3.4, -3.4, -0.88),
  ], GOLD, 0.17));
  const ticks = [];
  for (let i = 0; i < 120; i += 1) {
    const a = i / 120 * TAU;
    const length = i % 10 === 0 ? 0.12 : i % 5 === 0 ? 0.065 : 0.027;
    ticks.push(new THREE.Vector3(Math.cos(a) * 3.85, Math.sin(a) * 3.85, -0.85));
    ticks.push(new THREE.Vector3(Math.cos(a) * (3.85 + length), Math.sin(a) * (3.85 + length), -0.85));
  }
  group.add(line(ticks, GOLD, 0.28, true));
  const guides = [];
  for (const y of [-3.4, -1.7, 0, 1.7, 3.4]) guides.push(new THREE.Vector3(-0.07, y, -0.82), new THREE.Vector3(0.07, y, -0.82));
  guides.push(new THREE.Vector3(0, -3.55, -0.85), new THREE.Vector3(0, 3.55, -0.85));
  group.add(line(guides, GOLD, 0.075, true));
  const ground = line(ellipse(3.75, 2.35), JADE, 0.13);
  ground.rotation.x = Math.PI / 2; ground.position.y = -3.48;
  group.add(ground);
  return group;
}

// A smooth stand-in remains visible while the local sculpture loads.
function fallbackGeometry() {
  const parts = [];
  const add = (geometry, position, scale = [1, 1, 1], rotation = null) => {
    geometry.scale(...scale);
    if (rotation) geometry.applyQuaternion(rotation);
    geometry.translate(...position);
    const flat = geometry.index ? geometry.toNonIndexed() : geometry;
    if (flat !== geometry) geometry.dispose();
    for (const name of Object.keys(flat.attributes)) if (!['position', 'normal'].includes(name)) flat.deleteAttribute(name);
    parts.push(flat);
  };
  const oval = (p, s, detail = 20) => add(new THREE.SphereGeometry(1, detail, 14), p, s);
  const limb = (a, b, radiusA, radiusB) => {
    const from = new THREE.Vector3(...a); const to = new THREE.Vector3(...b);
    const direction = to.clone().sub(from);
    const q = new THREE.Quaternion().setFromUnitVectors(new THREE.Vector3(0, 1, 0), direction.clone().normalize());
    add(new THREE.CylinderGeometry(radiusB, radiusA, direction.length(), 16, 4), from.add(to).multiplyScalar(0.5).toArray(), [1, 1, 1], q);
  };
  const profile = [
    [0.1, -0.8], [0.56, -0.65], [0.69, -0.38], [0.52, 0.12],
    [0.59, 0.65], [0.79, 1.2], [0.82, 1.52], [0.6, 1.82], [0.22, 2.03],
  ].map(([r, y]) => new THREE.Vector2(r, y));
  add(new THREE.LatheGeometry(profile, 36), [0, 0, 0], [1, 1, 0.56]);
  limb([0, 1.88, 0], [0, 2.38, 0], 0.22, 0.19);
  oval([0, 2.76, 0], [0.43, 0.61, 0.39], 28);
  oval([0, 2.66, 0.35], [0.085, 0.16, 0.1], 12);
  for (const side of [-1, 1]) {
    oval([side * 0.79, 1.57, 0], [0.33, 0.28, 0.28]);
    limb([side * 0.8, 1.57, 0], [side * 1.83, 1.52, 0], 0.25, 0.17);
    oval([side * 1.83, 1.52, 0], [0.19, 0.18, 0.17]);
    limb([side * 1.83, 1.52, 0], [side * 2.73, 1.52, 0], 0.18, 0.105);
    oval([side * 2.91, 1.52, 0], [0.26, 0.14, 0.09]);
    for (let finger = 0; finger < 4; finger += 1) {
      const y = 1.425 + finger * 0.059;
      limb([side * 3.04, y, 0], [side * (3.33 - Math.abs(finger - 1.5) * 0.055), y, 0], 0.031, 0.023);
    }
    limb([side * 2.84, 1.41, 0.015], [side * 3.05, 1.28, 0.02], 0.047, 0.032);
    oval([side * 0.38, -0.69, 0], [0.35, 0.41, 0.32]);
    limb([side * 0.36, -0.67, 0], [side * 0.39, -1.98, 0], 0.31, 0.18);
    oval([side * 0.39, -1.97, 0.01], [0.185, 0.21, 0.19]);
    limb([side * 0.39, -2.03, 0], [side * 0.42, -3.12, 0], 0.21, 0.11);
    oval([side * 0.42, -3.23, 0.14], [0.17, 0.13, 0.34]);
  }
  const merged = mergeGeometries(parts);
  parts.forEach((geometry) => geometry.dispose());
  return normalizeGeometry(merged);
}

function normalizeGeometry(geometry) {
  geometry.computeBoundingBox();
  const center = geometry.boundingBox.getCenter(new THREE.Vector3());
  const height = geometry.boundingBox.max.y - geometry.boundingBox.min.y;
  geometry.translate(-center.x, -center.y, -center.z);
  geometry.scale(6.8 / height, 6.8 / height, 6.8 / height);
  if (!geometry.getAttribute('normal')) geometry.computeVertexNormals();
  return geometry;
}

function correctGeneratedPose(geometry) {
  // The reviewed local Meshy sculpture arrived with arms lowered by ~29 degrees.
  // A bounded shoulder blend lifts those vertices without disturbing head or torso.
  const position = geometry.getAttribute('position');
  for (let i = 0; i < position.count; i += 1) {
    const x = position.getX(i); const y = position.getY(i);
    const side = Math.sign(x);
    const weight = smooth(0.62, 1.03, Math.abs(x)) * smooth(0.1, 0.45, y);
    if (!weight) continue;
    const angle = THREE.MathUtils.degToRad(side * 29 * weight);
    const dx = x - side * 0.72; const dy = y - 1.76;
    position.setXY(i, side * 0.72 + dx * Math.cos(angle) - dy * Math.sin(angle), 1.76 + dx * Math.sin(angle) + dy * Math.cos(angle));
  }
  position.needsUpdate = true;
  geometry.computeVertexNormals();
  geometry.computeBoundingBox();
  return geometry;
}

function modelGeometry(root) {
  root.updateMatrixWorld(true);
  const geometries = [];
  root.traverse((mesh) => {
    if (!mesh.isMesh || !mesh.geometry?.getAttribute('position')) return;
    let geometry = mesh.geometry.clone().applyMatrix4(mesh.matrixWorld);
    if (geometry.index) { const indexed = geometry; geometry = indexed.toNonIndexed(); indexed.dispose(); }
    for (const name of Object.keys(geometry.attributes)) if (!['position', 'normal'].includes(name)) geometry.deleteAttribute(name);
    if (!geometry.getAttribute('normal')) geometry.computeVertexNormals();
    geometries.push(geometry);
  });
  if (!geometries.length) throw new Error('The local anatomical asset contains no mesh.');
  const merged = mergeGeometries(geometries);
  geometries.forEach((geometry) => geometry.dispose());
  return correctGeneratedPose(normalizeGeometry(merged));
}

function triangleGeometry(source, predicate) {
  const vertices = source.getAttribute('position'); const normals = source.getAttribute('normal');
  const positions = []; const directions = [];
  for (let i = 0; i < vertices.count; i += 3) {
    const x = (vertices.getX(i) + vertices.getX(i + 1) + vertices.getX(i + 2)) / 3;
    const y = (vertices.getY(i) + vertices.getY(i + 1) + vertices.getY(i + 2)) / 3;
    if (!predicate(x, y)) continue;
    for (let j = 0; j < 3; j += 1) {
      positions.push(vertices.getX(i + j), vertices.getY(i + j), vertices.getZ(i + j));
      directions.push(normals.getX(i + j), normals.getY(i + j), normals.getZ(i + j));
    }
  }
  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));
  geometry.setAttribute('normal', new THREE.Float32BufferAttribute(directions, 3));
  return geometry;
}

function sparseWire(geometry, budget = 4500) {
  const wire = new THREE.WireframeGeometry(geometry);
  const position = wire.getAttribute('position'); const count = position.count / 2;
  if (count <= budget) return wire;
  const stride = Math.ceil(count / budget); const values = [];
  for (let edge = 0; edge < count; edge += stride) {
    for (let end = 0; end < 2; end += 1) {
      const index = edge * 2 + end;
      values.push(position.getX(index), position.getY(index), position.getZ(index));
    }
  }
  wire.dispose();
  const result = new THREE.BufferGeometry();
  result.setAttribute('position', new THREE.Float32BufferAttribute(values, 3));
  return result;
}

function sculpture(geometry) {
  const group = new THREE.Group(); group.name = 'Seven anatomical strata';
  const pieces = [];
  for (let band = 0; band < 7; band += 1) {
    const section = triangleGeometry(geometry, (_, y) => Math.max(0, Math.min(6, Math.floor((y + 3.4) / 6.8 * 7))) === band);
    if (!section.getAttribute('position').count) { section.dispose(); continue; }
    const piece = new THREE.Group();
    const surface = new THREE.Mesh(section, new THREE.MeshStandardMaterial({
      color: 0x477f74, emissive: 0x0b302a, roughness: 0.55, metalness: 0.2,
      transparent: true, opacity: 0.23, depthWrite: false, side: THREE.FrontSide,
    }));
    const wire = new THREE.LineSegments(sparseWire(section), new THREE.LineBasicMaterial({
      color: JADE, transparent: true, opacity: 0.14, depthWrite: false,
    }));
    const source = section.getAttribute('position'); const samples = []; const seen = new Set();
    const step = Math.max(1, Math.floor(source.count / 2800));
    for (let i = 0; i < source.count; i += step) {
      const x = source.getX(i); const y = source.getY(i); const z = source.getZ(i);
      const key = `${Math.round(x * 500)},${Math.round(y * 500)},${Math.round(z * 500)}`;
      if (seen.has(key)) continue;
      seen.add(key); samples.push(x, y, z);
    }
    const pointsGeometry = new THREE.BufferGeometry();
    pointsGeometry.setAttribute('position', new THREE.Float32BufferAttribute(samples, 3));
    const points = new THREE.Points(pointsGeometry, new THREE.PointsMaterial({
      color: 0xafcfc4, size: 0.017, transparent: true, opacity: 0.4, depthWrite: false,
    }));
    piece.add(surface, wire, points); group.add(piece);
    pieces.push({ group: piece, surface, wire, points, band });
  }
  const ghosts = new THREE.Group(); ghosts.name = 'Vitruvian alternate limb positions';
  for (const side of [-1, 1]) {
    for (const isArm of [true, false]) {
      const segment = triangleGeometry(geometry, (x, y) => isArm ? x * side > 0.8 && y > 0.8 && y < 2.3 : x * side > 0.045 && y < -0.76);
      const pivot = new THREE.Vector3(side * (isArm ? 0.78 : 0.32), isArm ? 1.7 : -0.65, -0.06);
      segment.translate(-pivot.x, -pivot.y, -pivot.z);
      const armature = new THREE.Group(); armature.position.copy(pivot);
      armature.rotation.z = THREE.MathUtils.degToRad(side * (isArm ? 28 : 17));
      armature.add(new THREE.LineSegments(sparseWire(segment, 2300), new THREE.LineBasicMaterial({
        color: GOLD, transparent: true, opacity: 0.10, depthWrite: false,
      })));
      segment.dispose(); ghosts.add(armature);
    }
  }
  group.add(ghosts); geometry.dispose();
  return { group, pieces, ghosts };
}

export function createAnatomy({ onReady, requestRender, heartPosition = [0.04, 1.05, 0.65] } = {}) {
  const root = new THREE.Group(); root.name = 'Vitruvian anatomical atlas';
  root.add(buildReference());
  let body = sculpture(fallbackGeometry()); root.add(body.group);
  const shells = Object.entries(KOSHA_COLORS).map(([id, color], index) => {
    const group = new THREE.Group(); group.name = `${id} field`;
    const rx = 3.55 + index * 0.235; const ry = 3.75 + index * 0.21; const rz = 0.85 + index * 0.34;
    const grid = line(ellipsoidGrid(rx, ry, rz), color, 0.022, true);
    const rim = line(ellipse(rx, ry), color, 0.055);
    const skin = new THREE.Mesh(new THREE.SphereGeometry(1, 40, 24), new THREE.MeshBasicMaterial({
      color, transparent: true, opacity: 0.008, depthWrite: false, side: THREE.BackSide,
    }));
    skin.scale.set(rx, ry, rz); group.add(skin, grid, rim); root.add(group);
    return { id, index, group, grid, rim, skin };
  });
  const heart = new THREE.Group(); heart.name = 'Golden integration heart'; heart.position.fromArray(heartPosition);
  const heartCore = new THREE.Mesh(new THREE.IcosahedronGeometry(0.14, 2), new THREE.MeshBasicMaterial({ color: GOLD, transparent: true, opacity: 0.88 }));
  const heartRing = new THREE.Mesh(new THREE.TorusGeometry(0.23, 0.007, 6, 72), new THREE.MeshBasicMaterial({ color: GOLD, transparent: true, opacity: 0.57, depthWrite: false }));
  const heartOuter = new THREE.Mesh(new THREE.TorusGeometry(0.31, 0.003, 4, 72), new THREE.MeshBasicMaterial({ color: GOLD, transparent: true, opacity: 0.22, depthWrite: false }));
  heart.add(heartCore, heartRing, heartOuter); root.add(heart);
  let disposed = false; let lastState = null;

  function update({ explosion = 0, illumination = 0, time = 0, view = 'connections', kosha = 'all', reducedMotion = false }) {
    lastState = { explosion, illumination, time, view, kosha, reducedMotion };
    const anatomical = view === 'anatomy'; const field = view === 'biofield';
    for (const piece of body.pieces) {
      const n = piece.band - 3;
      piece.group.position.set((piece.band % 2 ? 1 : -1) * 0.24 * explosion, n * 0.12 * explosion, Math.abs(n) * 0.11 * explosion);
      piece.surface.material.opacity = (anatomical ? 0.39 : 0.23) * (1 - explosion * 0.35);
      piece.wire.material.opacity = (anatomical ? 0.22 : 0.135) + illumination * 0.038;
      piece.points.material.opacity = (anatomical ? 0.53 : 0.34) + illumination * 0.1;
    }
    body.ghosts.visible = explosion < 0.4;
    for (const shell of shells) {
      const active = kosha === shell.id; const dim = kosha !== 'all' && !active;
      shell.group.visible = !anatomical || active;
      const emphasis = active ? 2 : dim ? 0.2 : 1;
      shell.grid.material.opacity = (field ? 0.06 : 0.018 + explosion * 0.024) * emphasis;
      shell.rim.material.opacity = (field ? 0.16 : 0.046 + explosion * 0.036) * emphasis;
      shell.skin.material.opacity = (field ? 0.013 : 0.005) * emphasis;
      shell.group.scale.set(1 + explosion * (0.13 + shell.index * 0.025), 1 + explosion * 0.045, 1 + explosion * 0.3);
      shell.group.position.z = (shell.index - 2) * 0.35 * explosion;
      shell.group.rotation.y = (shell.index - 2) * 0.038 * explosion;
    }
    heart.scale.setScalar(reducedMotion ? 1 : 1 + Math.sin(time * Math.PI) * 0.045);
    heartCore.material.opacity = 0.8 + illumination * 0.19;
    heartRing.material.opacity = 0.42 + illumination * 0.2;
  }

  new GLTFLoader().load('/assets/vitruvian.glb', (gltf) => {
    if (disposed) { disposeObject(gltf.scene); return; }
    try {
      const next = sculpture(modelGeometry(gltf.scene));
      disposeObject(body.group); body = next; root.add(body.group);
      if (lastState) update(lastState);
      onReady?.('Meshy 7 · anatomical mesh');
    } catch {
      onReady?.('Procedural anatomical study · mesh unavailable');
    } finally { disposeObject(gltf.scene); requestRender?.(); }
  }, undefined, () => {
    if (!disposed) onReady?.('Procedural anatomical study · mesh unavailable');
  });
  return { root, update, dispose() { disposed = true; disposeObject(root); } };
}
