// Engine mandala: a bronze bench plate milled with a 19-cell honeycomb (1 + 6 + 12).
// node build.mjs          -> engine-mandala.html (dark, on void) + preview.html
// node build.mjs --light  -> engine-mandala-light.html (parchment)
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import * as k from "../kit/iso-kit.mjs";
import { kitScript } from "../.scripts/inline-kit.mjs";

const HERE = dirname(fileURLToPath(import.meta.url));
const theme = process.argv.includes("--light") ? "light" : "dark";

// ---------------------------------------------------------------------------
// Truth: the seventeen engine ids, in the order docs/ENGINES.md lists them.
const ENGINES_MD = readFileSync(join(HERE, "..", "..", "..", "ENGINES.md"), "utf8");
const ENGINES = [...ENGINES_MD.matchAll(/POST \/api\/v1\/engines\/([a-z-]+)\/calculate/g)].map((m) => m[1]);
if (ENGINES.length !== 17) throw new Error(`expected 17 engines in docs/ENGINES.md, found ${ENGINES.length}: ${ENGINES.join(", ")}`);

// ---------------------------------------------------------------------------
// World. x, y on the bench, z up. Units are about 1 px of the real thing.
const WIDTH = 600;
const HEIGHT = 390;
const AZIMUTH = 45;
const ELEVATION = 36;

const CELL_R = 12; // circumradius of a cell pocket (vertices on ±x)
const APOTHEM = CELL_R * Math.cos(Math.PI / 6);
const WEB = 3; // bronze left between pockets
const PITCH = 2 * APOTHEM + WEB; // centre to centre, flat to flat
const POCKET_DEPTH = 4;
const TRAY_R = CELL_R - 0.9; // the tray sits in its pocket with a 0.9 gap
const TRAY_H = 6; // from the pocket floor to 2 above the plate
const TRAY_TOP = -POCKET_DEPTH + TRAY_H;
const WELL_R = TRAY_R - 2.4;
const WELL_DEPTH = 1;
const TILE_R = WELL_R - 1.1;
const TILE_H = 1.6;
const WITNESS_H = 9; // the centre tray is taller
const WITNESS_TOP = -POCKET_DEPTH + WITNESS_H;
const BOSS_R = 2.6;
const BOSS_H = 1.4;
const LENS_R = 2;
const LIFT = 3.2; // a lifted tray keeps its base inside the pocket (bottom at -0.8)

const PLATE_APOTHEM = 74;
const PLATE_R = PLATE_APOTHEM / Math.cos(Math.PI / 6);
const PLATE_H = 12;
const FOOT_H = 5;
const Z0 = -PLATE_H - FOOT_H;

const deg = (a) => (a * Math.PI) / 180;
const polar = (r, a) => [r * Math.cos(deg(a)), r * Math.sin(deg(a))];
const hexRing = (cx, cy, r, rot = 0) => Array.from({ length: 6 }, (_, i) => [cx + r * Math.cos(deg(rot + 60 * i)), cy + r * Math.sin(deg(rot + 60 * i))]);

// Spiral: Witness at the centre, the inner ring clockwise on screen from the back-left
// cell, then the outer ring from the cell beside the last inner one, closing on the
// corner next to where it began. Inner neighbours sit on the flat normals (30 + 60k).
const INNER = [210, 270, 330, 30, 90, 150].map((a) => polar(PITCH, a));
const OUTER = [180, 210, 240, 270, 300, 330, 0, 30, 60, 90, 120, 150].map((a) => polar(a % 60 === 0 ? Math.sqrt(3) * PITCH : 2 * PITCH, a));
const CELLS = [
  { id: "witness", kind: "witness", at: [0, 0] },
  ...ENGINES.map((id, n) => ({ id, kind: "engine", n: n + 1, at: n < 6 ? INNER[n] : OUTER[n - 6] })),
  { id: "the question", kind: "question", at: OUTER[11] },
];
if (CELLS.length !== 19) throw new Error("the honeycomb needs 19 cells");

const PLATE_RING = hexRing(0, 0, PLATE_R);
const P = k.fitProjection(
  [
    ...PLATE_RING.map(([x, y]) => [x, y, Z0]),
    ...PLATE_RING.map(([x, y]) => [x, y, 0]),
    [0, 0, WITNESS_TOP + BOSS_H + LIFT + 2],
  ],
  WIDTH,
  HEIGHT,
  { pad: 26, azimuth: AZIMUTH, elevation: ELEVATION },
);

const at = (point) => k.iso(point, P);
const S = (paths, style) => k.solidSvg(paths, style);
const L = (d, style) => k.lineSvg(d, style);
const D = (points, style) => k.dotsSvg(points, style);
const depth = ([x, y]) => k.depthOf([x, y, 0], P);
const byDepth = (points) => [...points].sort((a, b) => depth(a) - depth(b));
const signedArea = (pts) => pts.reduce((sum, [x, y], i) => { const [nx, ny] = pts[(i + 1) % pts.length]; return sum + x * ny - nx * y; }, 0) / 2;

// A milled recess: a hole of outline `ring` in a horizontal face at zTop, `depth` deep.
// Draws the floor, the inner walls you can see from above (the far ones), the rim on
// the face, the floor edges of those walls and the vertical corner edges where a wall
// turns out of view. Every open line ends on the rim or on a corner edge.
function recessSvg(ring, zTop, depth, { rim = "lo", rimClass = "", floorShade = 0 } = {}) {
  const count = ring.length;
  const ccw = signedArea(ring) > 0;
  const [vx, vy] = k.towardViewer(P);
  const { sinA, cosA } = k.cameraOf(P);
  const [lx, ly] = [-sinA, cosA];
  const top = ring.map(([x, y]) => at([x, y, zTop]));
  const bottom = ring.map(([x, y]) => at([x, y, zTop - depth]));
  const parts = [`<path class="iso-shade" data-shade="${floorShade}" d="${k.pathOf(bottom, true)}"/>`];
  const seen = [];
  for (let i = 0; i < count; i++) {
    const [ax, ay] = ring[i];
    const [bx, by] = ring[(i + 1) % count];
    const dx = bx - ax, dy = by - ay;
    const len = Math.hypot(dx, dy) || 1;
    const [nx, ny] = ccw ? [dy / len, -dx / len] : [-dy / len, dx / len]; // outward
    seen[i] = nx * vx + ny * vy < -1e-9; // we look at the inside of the walls that face away
    if (!seen[i]) continue;
    const lit = (-nx * lx - ny * ly + 1) / 2;
    const shade = Math.min(3, Math.max(0, Math.floor(lit * 4)));
    parts.push(`<path class="iso-shade" data-shade="${shade}" d="${k.pathOf([top[i], top[(i + 1) % count], bottom[(i + 1) % count], bottom[i]], true)}"/>`);
  }
  const edges = [];
  const corners = [];
  for (let i = 0; i < count; i++) {
    if (seen[i]) edges.push(k.pathOf([bottom[i], bottom[(i + 1) % count]]));
    if (seen[(i + count - 1) % count] !== seen[i]) corners.push(k.pathOf([top[i], bottom[i]]));
  }
  parts.push(L(edges.join(""), { tone: "faint" }));
  parts.push(L(corners.join(""), { tone: "faint" }));
  parts.push(L(k.pathOf(top, true), { tone: rim, className: rimClass || undefined }));
  return parts.join("");
}

// ---------------------------------------------------------------------------
// The bench plate: a flat-to-flat bronze hexagon on six feet, gold-edged, with a
// groove, six corner screws, a gold rule strip and a label plate on the near flat.
function plate() {
  const out = [`<path class="iso-halo" d="${k.extrude(PLATE_RING, Z0, PLATE_H, P).fill}"/>`];
  const feet = byDepth(Array.from({ length: 6 }, (_, i) => polar(PLATE_R - 15, 60 * i)));
  for (const [x, y] of feet) {
    out.push(S(k.cylinder(x, y, 6.5, Z0, 1.6, P, 28), { tone: "lo" }));
    out.push(S(k.cylinder(x, y, 5, Z0 + 1.6, FOOT_H - 1.6, P, 28), { tone: "lo" }));
  }
  out.push(S(k.extrude(PLATE_RING, -PLATE_H, PLATE_H, P, { bevel: 1.6 }), { tone: "mid", crease: "lo", className: "bench" }));
  out.push(L(k.onTop(hexRing(0, 0, PLATE_R - 4), 0, P, true), { tone: "faint" }));
  const screws = Array.from({ length: 6 }, (_, i) => polar(PLATE_R - 11, 60 * i));
  out.push(L(screws.map(([x, y]) => k.ring(x, y, 2.2, 0, P, 18)).join(""), { tone: "lo" }));
  out.push(D(screws.map(([x, y]) => at([x, y, 0])), { size: 0.5 }));
  // gold rule strip along the near flat edge (y = +apothem), ticks hung from its far edge
  const strip = { x: -34, y: 61.5, w: 40, d: 7, r: 1 };
  out.push(S(k.slabOf(strip, 0, 0.6, P, 3), { tone: "hi", crease: "none" }));
  out.push(L(k.planOutline(k.insetPlan(strip, 0.3), 0.6, P, 3), { tone: "lo" })); // the engraved rule the ticks hang from
  const ticks = k.topTicks(strip.x + 4, strip.x + strip.w - 4, 4, 5, strip.y + 0.3, 0.6, [2.4, strip.d - 0.6], P);
  out.push(L(ticks.minor, { tone: "lo" }), L(ticks.major, { tone: "mid" }));
  out.push(D([at([strip.x + 2, strip.y + 3.5, 0.6]), at([strip.x + strip.w - 2, strip.y + 3.5, 0.6])], { size: 0.45 }));
  // label plate
  const label = { x: 14, y: 61.5, w: 18, d: 7, r: 1 };
  out.push(S(k.slabOf(label, 0, 0.8, P, 4), { tone: "mid", crease: "none" }));
  out.push(L(k.planOutline(k.insetPlan(label, 1.4), 0.8, P, 4), { tone: "faint" }));
  const rules = [[3, 11.5], [3, 9], [3, 7.5]].map(([from, to], i) => k.lineOnTop([label.x + from, label.y + 2 + i * 1.5], [label.x + from + to, label.y + 2 + i * 1.5], 0.8, P));
  out.push(L(rules.join(""), { tone: "lo", free: true }));
  out.push(D([at([label.x + 1.6, label.y + 3.5, 0.8]), at([label.x + label.w - 1.6, label.y + 3.5, 0.8])], { size: 0.4 }));
  // the spiral engraved into the webs: a groove from each pocket rim to the next cell's rim
  const groove = [];
  for (let i = 1; i < CELLS.length; i++) {
    const [ax, ay] = CELLS[i - 1].at, [bx, by] = CELLS[i].at;
    const len = Math.hypot(bx - ax, by - ay);
    const [ux, uy] = [(bx - ax) / len, (by - ay) / len];
    const from = [ax + ux * APOTHEM, ay + uy * APOTHEM], to = [bx - ux * APOTHEM, by - uy * APOTHEM];
    groove.push(k.lineOnTop(from, to, 0, P));
  }
  out.push(L(groove.join(""), { tone: "lo" }));
  return out.join("");
}

// ---------------------------------------------------------------------------
// One cell: the pocket milled into the plate, a gold glow in the pocket while the tray
// is lifted, then the tray clipped to its pocket opening so the part of it below the
// bench top hides behind the near web, as it would.
function cellSvg(cell, index) {
  const [cx, cy] = cell.at;
  const pocket = hexRing(cx, cy, CELL_R);
  const out = [`<g data-pocket="${index}">${recessSvg(pocket, 0, POCKET_DEPTH, { rim: "lo", rimClass: "pocket-rim" })}${L(k.onTop(hexRing(cx, cy, CELL_R + 0.7), 0, P, true), { tone: "faint" })}</g>`];
  if (cell.kind === "question") return out.join("");
  out.push(`<path class="iso-halo cell-glow" data-glow="${index}" d="${k.pathOf(hexRing(cx, cy, TRAY_R).map(([x, y]) => at([x, y, -POCKET_DEPTH])), true)}"/>`);
  const clipId = `pocket-${index}`;
  const trayRing = hexRing(cx, cy, TRAY_R);
  const tray = [];
  if (cell.kind === "witness") {
    tray.push(S(k.extrude(trayRing, -POCKET_DEPTH, WITNESS_H, P, { bevel: 0.6 }), { tone: "mid", crease: "lo", inner: L(k.ringSeam(trayRing, TRAY_TOP, P), { tone: "faint" }) }));
    tray.push(L(k.onTop(hexRing(cx, cy, TRAY_R - 1), WITNESS_TOP, P, true), { tone: "hi", className: "tray-rim" }));
    // the trinity: three interlocking gold rings, the lens where they overlap
    const rings = [90, 210, 330].map((a) => { const [ox, oy] = polar(2.2, a); return k.ring(cx + ox, cy + oy, 4.4, WITNESS_TOP, P, 48); });
    tray.push(L(rings.join(""), { tone: "mid" }));
    tray.push(S(k.cylinder(cx, cy, BOSS_R, WITNESS_TOP, BOSS_H, P, 28, 0.3), { tone: "hi", crease: "lo" }));
    tray.push(L(k.sideArc(cx, cy, BOSS_R, WITNESS_TOP + 0.5, P, 16), { tone: "faint" }));
    tray.push(`<path class="lens" d="${k.ring(cx, cy, LENS_R, WITNESS_TOP + BOSS_H, P, 36)}"/>`);
  } else {
    tray.push(S(k.extrude(trayRing, -POCKET_DEPTH, TRAY_H, P, { bevel: 0.5 }), { tone: "mid", crease: "lo" }));
    tray.push(L(k.onTop(hexRing(cx, cy, TRAY_R - 1), TRAY_TOP, P, true), { tone: "hi", className: "tray-rim" }));
    const well = hexRing(cx, cy, WELL_R);
    tray.push(recessSvg(well, TRAY_TOP, WELL_DEPTH, { rim: "lo" }));
    const wellClip = `well-${index}`;
    const tile = [];
    tile.push(S(k.extrude(hexRing(cx, cy, TILE_R), TRAY_TOP - WELL_DEPTH, TILE_H, P, { bevel: 0.3 }), { tone: "hi", crease: "lo", className: "tile" }));
    tile.push(L(k.ring(cx, cy, 3.2, TRAY_TOP - WELL_DEPTH + TILE_H, P, 32), { tone: "faint" }));
    tile.push(D([at([cx, cy, TRAY_TOP - WELL_DEPTH + TILE_H])], { size: 0.45, tone: "lo" }));
    tray.push(`<clipPath id="${wellClip}"><path d="${k.extrude(well, TRAY_TOP, 12, P).fill}"/></clipPath><g clip-path="url(#${wellClip})">${tile.join("")}</g>`);
  }
  out.push(`<clipPath id="${clipId}"><path d="${k.extrude(pocket, 0, 24, P).fill}"/></clipPath>`);
  out.push(`<g clip-path="url(#${clipId})"><g class="tray" data-tray="${index}">${tray.join("")}</g></g>`);
  return out.join("");
}

const order = CELLS.map((cell, index) => ({ cell, index })).sort((a, b) => depth(a.cell.at) - depth(b.cell.at));
const body = [plate(), ...order.map(({ cell, index }) => cellSvg(cell, index))];

const label =
  "A bronze bench plate milled with nineteen hexagonal pockets in a honeycomb of one, six and twelve. Each pocket holds a shallow bronze tray with a gold rim. The centre tray is Witness, taller, with three interlocking rings and an emerald lens; the seventeen engine trays spiral outward in registry order; the last pocket is empty, the question. One tray at a time lifts in its pocket and its rim lights.";

const svg = k.figureSvg({ width: WIDTH, height: HEIGHT, label, body });

// Screen centres of the cells, for the pointer.
const CENTRES = CELLS.map((cell) => at([cell.at[0], cell.at[1], cell.kind === "witness" ? WITNESS_TOP : cell.kind === "engine" ? TRAY_TOP : -POCKET_DEPTH]));
const CELL_PX = k.screenRise(1, P) / Math.cos(deg(ELEVATION)) * CELL_R; // one world unit on screen, times the cell radius

const live = `
const P = ${JSON.stringify(P)};
const CELLS = ${JSON.stringify(CELLS.map(({ id, kind, n }) => ({ id, kind, n })))};
const CENTRES = ${JSON.stringify(CENTRES.map(([x, y]) => [Math.round(x * 10) / 10, Math.round(y * 10) / 10]))};
const W = ${WIDTH}, H = ${HEIGHT}, LIFT = ${LIFT}, REACH = ${(CELL_PX * 1.02).toFixed(2)};
const ENGINES = ${ENGINES.length};
const stage = document.querySelector(".iso-stage");
const svg = stage.querySelector("svg");
const readout = document.querySelector("[data-readout]");
const trays = CELLS.map((_, i) => svg.querySelector('[data-tray="' + i + '"]'));
const pockets = CELLS.map((_, i) => svg.querySelector('[data-pocket="' + i + '"]'));
const glows = CELLS.map((_, i) => svg.querySelector('[data-glow="' + i + '"]'));
const still = matchMedia("(prefers-reduced-motion: reduce)");
const lifts = CELLS.map(() => 0);
const drawn = CELLS.map(() => -1);
let selected = 0, touring = !still.matches, clock = 0, frame = 0, last = 0, visible = false, idleTimer = 0;
const DWELL = 1.5;
const tourAt = (t) => Math.floor(t / DWELL) % CELLS.length;
const nameOf = (i) => { const c = CELLS[i]; return c.kind === "engine" ? "engine:" + c.id + " · " + c.n + "/" + ENGINES : c.id; };
const wordsOf = (i) => { const c = CELLS[i]; return c.kind === "engine" ? "Engine " + c.n + " of " + ENGINES + ", " + c.id + ", lifted" : c.kind === "witness" ? "Witness, the centre cell, lifted" : "The question, the empty cell"; };
function select(i) {
  if (i === selected) return;
  const was = selected;
  selected = i;
  for (const j of [was, i]) {
    const on = j === selected;
    (trays[j] || pockets[j]).toggleAttribute("data-on", on);
    if (CELLS[j].kind === "question") pockets[j].toggleAttribute("data-on", on);
  }
  readout.textContent = nameOf(i);
  stage.setAttribute("aria-valuenow", String(i));
  stage.setAttribute("aria-valuetext", wordsOf(i));
}
function draw() {
  for (let i = 0; i < CELLS.length; i++) {
    const lift = Math.round(lifts[i] * 100) / 100;
    if (lift === drawn[i] || !trays[i]) continue;
    drawn[i] = lift;
    if (lift > 0) trays[i].setAttribute("transform", translateAlong(P, [0, 0, lift]));
    else trays[i].removeAttribute("transform");
    glows[i].style.opacity = (lift / LIFT * 0.6).toFixed(3);
  }
}
function tick(now) {
  const dt = last ? Math.min((now - last) / 1000, 1 / 30) : 1 / 60;
  last = now;
  const calm = still.matches;
  if (touring) { clock += dt; select(tourAt(clock)); }
  let moving = false;
  for (let i = 0; i < CELLS.length; i++) {
    const want = i === selected ? LIFT : 0;
    lifts[i] = calm ? want : lifts[i] + (want - lifts[i]) * (1 - Math.exp(-dt / 0.22));
    if (Math.abs(want - lifts[i]) < 0.005) lifts[i] = want;
    if (lifts[i] !== want) moving = true;
  }
  draw();
  frame = visible && (touring || moving) ? requestAnimationFrame(tick) : 0;
}
const run = () => { if (!frame && visible) { last = 0; frame = requestAnimationFrame(tick); } };
const steer = (i) => {
  touring = false;
  select(Math.max(0, Math.min(CELLS.length - 1, i)));
  clearTimeout(idleTimer);
  idleTimer = setTimeout(() => { if (still.matches) return; touring = true; clock = selected * DWELL; run(); }, 3600);
  run();
};
new IntersectionObserver((entries) => { visible = entries[entries.length - 1].isIntersecting; if (visible) run(); }, { rootMargin: "120px 0px" }).observe(stage);
const cellAt = (event) => {
  const box = svg.getBoundingClientRect();
  const x = (event.clientX - box.left) / box.width * W, y = (event.clientY - box.top) / box.height * H;
  let best = -1, near = REACH * REACH;
  CENTRES.forEach(([cx, cy], i) => { const d = (cx - x) ** 2 + (cy - y) ** 2; if (d < near) { near = d; best = i; } });
  return best;
};
stage.addEventListener("pointermove", (event) => { const i = cellAt(event); if (i >= 0) steer(i); });
stage.addEventListener("pointerdown", (event) => { const i = cellAt(event); if (i >= 0) steer(i); });
stage.addEventListener("keydown", (event) => {
  let to = null;
  if (event.key === "ArrowRight" || event.key === "ArrowDown") to = (selected + 1) % CELLS.length;
  if (event.key === "ArrowLeft" || event.key === "ArrowUp") to = (selected + CELLS.length - 1) % CELLS.length;
  if (event.key === "Home") to = 0;
  if (event.key === "End") to = CELLS.length - 1;
  if (to === null) return;
  event.preventDefault();
  steer(to);
});
(trays[0] || pockets[0]).setAttribute("data-on", "");
readout.textContent = nameOf(0);
stage.setAttribute("aria-valuetext", wordsOf(0));
lifts[0] = still.matches ? LIFT : 0;
draw();
`;

const plateBody = k.plateHtml({
  fig: "Fig 1",
  title: "Engine mandala",
  hint: "Hover a cell · arrows walk the ring",
  readout: "witness",
  keys: [
    { mark: "lit", label: "Lifted tray: the cell under the pointer, rim lit" },
    { mark: "raised", label: "Nineteen pockets, 1 + 6 + 12, flat to flat" },
    { mark: "dim", label: "Witness at the centre, the question at the end of the spiral" },
  ],
  caption: `A bronze bench plate milled with nineteen hexagonal pockets, one at the centre, six around it, twelve around those. The centre tray is Witness, the dyad service behind /api/v1/witness/interpret, taller and carrying an emerald lens where three gold rings overlap. The ${ENGINES.length} engine trays spiral outward in the order of docs/ENGINES.md, ${ENGINES[0]} to ${ENGINES[ENGINES.length - 1]}, and the last pocket is empty: the question. Lift a tray and its gold rim lights; the readout names the engine under it.`,
  body: `<div class="iso-stage" tabindex="0" role="slider" aria-label="Engine mandala: arrow keys walk the honeycomb from Witness through the seventeen engines to the question" aria-valuemin="0" aria-valuemax="18" aria-valuenow="0">${svg}</div>`,
});

// Brand tokens over the kit's: patinated bronze with gold as line, on void; parchment and ink for Folio.
const BRAND_CSS = `
.iso{
  --anatomy-page:#070B1D;--anatomy-card:#070B1D;--anatomy-text:#F0EDE3;--anatomy-strong:#F0EDE3;--anatomy-muted:#AEBBC4;
  --anatomy-paper:#211B11;--anatomy-face:#2C2417;--anatomy-top:#6E5B36;
  --anatomy-shade-0:#15110B;--anatomy-shade-1:#2C2417;--anatomy-shade-2:#5A4A2E;--anatomy-shade-3:#64522F;
  --anatomy-lit-top:#A08E5C;--anatomy-lit-shade:#6A5A36;
  --anatomy-hi:#C5A017;--anatomy-mid:#A38518;--anatomy-lo:#7A6516;--anatomy-faint:#4A3E18;--anatomy-lit:#E6B84D;
  --anatomy-halo:#C5A017;--anatomy-halo-opacity:.22;--anatomy-dot:#C5A017;
  --lens:#10B5A7;--lens-core:#7FE3D6;
}
.iso[data-theme="light"]{
  --anatomy-page:#F0EDE3;--anatomy-card:#F0EDE3;--anatomy-text:#1A1A24;--anatomy-strong:#1A1A24;--anatomy-muted:#5A4A2E;
  --anatomy-paper:#F0EDE3;--anatomy-face:#FBF9F3;--anatomy-top:#F7F3E8;
  --anatomy-shade-0:#C9BFA6;--anatomy-shade-1:#D8CFB8;--anatomy-shade-2:#E3DBC7;--anatomy-shade-3:#ECE6D6;
  --anatomy-lit-top:#FFFDF6;--anatomy-lit-shade:#E9E1CC;
  --anatomy-hi:#1A1A24;--anatomy-mid:#5A4A2E;--anatomy-lo:#9A8A68;--anatomy-faint:#CDC3A8;--anatomy-lit:#76652A;
  --anatomy-halo:#5A4A2E;--anatomy-halo-opacity:.26;--anatomy-dot:#5A4A2E;
  --lens:#0E9A8E;--lens-core:#5CCFC2;
}
.iso-plate{--anatomy-paper:#211B11;box-shadow:0 0 0 1px var(--anatomy-faint)}
.bench>.iso-top{fill:#7C6B46}
.iso[data-theme="light"] .bench>.iso-top{fill:#EDE7D8}
.tile>.iso-top{fill:#968458}
.iso[data-theme="light"] .tile>.iso-top{fill:#FFFDF7}
.iso[data-theme="light"] .iso-plate{--anatomy-paper:#F0EDE3}
.iso-plate-corner[data-corner="title"]{font-family:"Cinzel",Georgia,serif;font-weight:500;font-size:12px;letter-spacing:.14em;color:var(--anatomy-text)}
.iso-stage{outline:none;touch-action:pan-y}
.iso-stage:focus-visible{outline:1px solid var(--anatomy-muted);outline-offset:6px}
.tray{transition:none}
.tray .tray-rim,.tray .iso-edge{transition:stroke 420ms var(--anatomy-ease)}
.tray[data-on] .tray-rim,.tray[data-on]>.iso-solid>.iso-edge{stroke:var(--anatomy-lit)}
.tray[data-on]>.iso-solid>.iso-top{fill:var(--anatomy-lit-top)}
[data-pocket][data-on] .pocket-rim{stroke:var(--anatomy-lit)}
.cell-glow{filter:blur(2.5px);transform:none;opacity:0;--anatomy-halo-opacity:1}
.lens{fill:var(--lens);stroke:var(--lens-core);stroke-width:.35;opacity:.72;transition:opacity 420ms var(--anatomy-ease)}
.tray[data-on] .lens{opacity:1}
@media (prefers-reduced-motion:reduce){.tray .tray-rim,.tray .iso-edge,.lens{transition:none}}
`;
const FONT = `<link rel="preconnect" href="https://fonts.googleapis.com"><link rel="preconnect" href="https://fonts.gstatic.com" crossorigin><link href="https://fonts.googleapis.com/css2?family=Cinzel:wght@500&display=swap" rel="stylesheet">`;

let html = k.pageHtml({ title: "Engine mandala", theme, body: plateBody, script: kitScript() + live });
html = html.replace("</title>", "</title>" + FONT).replace("</style>", BRAND_CSS + "</style>");

if (theme === "light") {
  writeFileSync(join(HERE, "engine-mandala-light.html"), html);
  console.log("wrote engine-mandala-light.html");
} else {
  writeFileSync(join(HERE, "engine-mandala.html"), html);
  writeFileSync(join(HERE, "preview.html"), `<!-- @dsCard group="Figures" height=520 -->\n` + html);
  console.log("wrote engine-mandala.html, preview.html");
}
const solids = (body.join("").match(/class="iso-solid/g) || []).length;
const paths = (body.join("").match(/<path/g) || []).length;
console.log(`${ENGINES.length} engines · 19 cells · ${solids} solids · ${paths} paths`);
