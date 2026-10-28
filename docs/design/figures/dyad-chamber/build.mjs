// Dyad chamber: two facing mirrors on a bronze rail with a lamp between them.
// node build.mjs            -> dyad-chamber.html (dark, on void) + preview.html
// node build.mjs --light    -> dyad-chamber-light.html (parchment)
// node build.mjs --audit    -> proves the geometry at rest (kit/audit.mjs)
import { writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import * as k from "../kit/iso-kit.mjs";
import * as G from "../kit/lathe.mjs";
import * as A from "../kit/audit.mjs";
import { kitScript } from "../.scripts/inline-kit.mjs";

const HERE = dirname(fileURLToPath(import.meta.url));
const theme = process.argv.includes("--light") ? "light" : "dark";

const WIDTH = 600;
const HEIGHT = 380;

// ---------------------------------------------------------------- the world
// x runs along the rail (Pichet at -x, Aletheios at +x), y across the bench, z up.
const PLATE = { x: -164, y: -46, w: 328, d: 92, r: 8 };
const PLATE_H = 10;
const FOOT_H = 5;
const BED = { x: -140, y: -9, w: 280, d: 18, r: 1.5 };
const BED_H = 4;
const RAIL = { x: -140, y: -4, w: 280, d: 8, r: 1.5 };
const RAIL_TOP = BED_H + 5;
const STOP_W = 6;
const STRIP = { x: -122, y: 35, w: 244, d: 7, r: 1 };
const LABEL = { x: 124, y: 17, w: 26, d: 13, r: 1.2 };

const MIRROR_X = 118; // carriage centres at ±MIRROR_X
const MIRROR_T = 2; // half thickness of a mirror
const ZC = 42; // mirror centre height
const FACE_P = -MIRROR_X + MIRROR_T; // Pichet's face (looks toward +x)
const FACE_A = MIRROR_X - MIRROR_T; // Aletheios' face (looks toward -x)
const REACH = [-75, 75]; // lamp travel
const LAMP_Y = 2; // the lamp hangs here, over the rail
const X0 = -48; // rest pose

// Pichet: a hexagon, flat top and bottom, 46 wide.
const HEX_R = 23;
const hexagon = (r) => Array.from({ length: 6 }, (_, i) => [r * Math.cos((i * Math.PI) / 3), r * Math.sin((i * Math.PI) / 3)]);
// Aletheios: a vesica, the lens of two circles of radius 24 whose centres sit 14 apart; 34 wide, 46 tall.
const VES_R = 24;
const VES_D = 14;
function vesica(scale = 1, per = 13) {
  const half = Math.acos(VES_D / 2 / VES_R);
  const right = Array.from({ length: per + 1 }, (_, i) => -half + (2 * half * i) / per).map((a) => [-VES_D / 2 + VES_R * Math.cos(a), VES_R * Math.sin(a)]);
  const left = Array.from({ length: per + 1 }, (_, i) => -half + (2 * half * i) / per).map((a) => [VES_D / 2 - VES_R * Math.cos(a), -VES_R * Math.sin(a)]);
  return [...right.slice(0, -1), ...left.slice(0, -1)].map(([u, v]) => [u * scale, v * scale]);
}
const scaled = (poly, s) => poly.map(([u, v]) => [u * s, v * s]);
// Clip a polygon to v >= floor (Sutherland-Hodgman against one half-plane).
function above(poly, floor) {
  const out = [];
  poly.forEach((a, i) => {
    const b = poly[(i + 1) % poly.length];
    const inA = a[1] >= floor, inB = b[1] >= floor;
    if (inA) out.push(a);
    if (inA !== inB) { const t = (floor - a[1]) / (b[1] - a[1]); out.push([a[0] + (b[0] - a[0]) * t, floor]); }
  });
  return out;
}
const HEX = hexagon(HEX_R);
const VES = vesica(1);

const P = k.fitProjection(
  [
    ...k.boxCorners(PLATE, -PLATE_H - FOOT_H, 0),
    [-MIRROR_X, -HEX_R, ZC + HEX_R],
    [-MIRROR_X, HEX_R, ZC + HEX_R],
    [MIRROR_X, -VES_R, ZC + VES_R],
    [MIRROR_X, VES_R, ZC + VES_R],
    [REACH[0], -14, 66],
    [REACH[1], 14, 66],
  ],
  WIDTH,
  HEIGHT,
  { pad: 26, azimuth: 38, elevation: 30 },
);

const at = (p) => k.iso(p, P);
const S = (paths, style) => k.solidSvg(paths, style);
const L = (d, style) => k.lineSvg(d, style);
const D = (points, style) => k.dotsSvg(points, style);
const byDepth = (points) => [...points].sort((a, b) => k.depthOf([a[0], a[1], 0], P) - k.depthOf([b[0], b[1], 0], P));
const f1 = (v) => Math.round(v * 10) / 10;

// ---------------------------------------------------------------- the model
// Pichet's share of the lamp is the lamp's distance from Aletheios' face over the whole gap, so the
// nearer mirror owns the larger share. At the midpoint both hold 0.5.
const GAP = FACE_A - FACE_P;
const shareOf = (x) => (FACE_A - x) / GAP;
const EDGE = shareOf(REACH[0]); // the largest share the lamp can give, 0.82
function stateOf(x) {
  const p = shareOf(x);
  const v = Math.max(-1, Math.min(1, (p - 0.5) / (EDGE - 0.5)));
  const glowP = 0.5 + 0.5 * v;
  const glowA = 0.5 - 0.5 * v;
  const dimP = 1 - 0.65 * Math.max(0, -v);
  const dimA = 1 - 0.65 * Math.max(0, v);
  const held = Math.abs(p - 0.5) < 0.02;
  const text = held ? "held · both" : p > 0.5 ? "pichet speaks · " + p.toFixed(2) : "aletheios speaks · " + (1 - p).toFixed(2);
  return { p, glowP, glowA, dimP, dimA, held, text };
}

// ---------------------------------------------------------------- recorder
const R = A.recorder(P);
const record = R.solid;
const put = (name, ref, svg, shapes = [], extra = {}) => R.put({ name, at: ref, svg, shapes, ...extra });
const centreOf = (plan, z, h) => [plan.x + plan.w / 2, plan.y + plan.d / 2, z + h / 2];
const slab = (name, plan, z, h, style = {}, { steps = 6, bevel = 0, inner = "", extra = "", bias = 0, owner } = {}) =>
  put(name, centreOf(plan, z, h), S(k.slabOf(plan, z, h, P, steps, bevel), { tone: "mid", ...style, inner }) + extra, [record(A.slab(plan, z, h, { name, owner }, steps))], { bias });
const cyl = (name, cx, cy, r, z, h, style = {}, { steps = 28, bevel = 0, inner = "", extra = "", bias = 0, owner } = {}) =>
  put(name, [cx, cy, z + h / 2], S(k.cylinder(cx, cy, r, z, h, P, steps, bevel), { tone: "mid", ...style, inner }) + extra, [record(A.cylinder(cx, cy, r, z, h, { name, owner }))], { bias });

// ---------------------------------------------------------------- base
function base() {
  put("halo", [0, 0, -PLATE_H - FOOT_H], `<path class="iso-halo" d="${k.haloOf(PLATE, -PLATE_H - FOOT_H, PLATE_H, P)}"/>`, [record(A.slab(PLATE, -PLATE_H - FOOT_H - 0.01, 0.01, { name: "halo", owner: "stand.plate" }, 8))], { bias: -60 });
  for (const [x, y] of byDepth(k.corners(PLATE, 20))) {
    cyl("stand.foot.flange", x, y, 7, -PLATE_H - FOOT_H, 1.6, { tone: "lo" }, { steps: 28 });
    cyl("stand.foot", x, y, 5.4, -PLATE_H - FOOT_H + 1.6, FOOT_H - 1.6, { tone: "lo" }, { steps: 28 });
  }
  const screws = k.corners(PLATE, 10);
  slab("stand.plate", PLATE, -PLATE_H, PLATE_H, { tone: "mid" }, {
    steps: 8,
    bevel: 1.6,
    extra:
      L(k.planOutline(k.insetPlan(PLATE, 5), 0, P), { tone: "faint" }) +
      L(screws.map(([x, y]) => k.ring(x, y, 2.2, 0, P, 16)).join(""), { tone: "lo" }) +
      D(screws.map(([x, y]) => at([x, y, 0])), { size: 0.5 }),
  });
  // The ruler strip reads the lamp's position: a tick every 5 units, a major every 25, the zero mark lit.
  const ticks = k.topTicks(STRIP.x + 2, STRIP.x + STRIP.w - 2, 5, 5, STRIP.y + 0.8, 0.6, [2, 3.6], P);
  slab("strip", STRIP, 0, 0.6, { tone: "lo", crease: "none" }, {
    steps: 3,
    extra:
      L(ticks.minor, { tone: "lo" }) +
      L(ticks.major, { tone: "mid" }) +
      L(k.lineOnTop([0, STRIP.y + 0.8], [0, STRIP.y + 5.6], 0.6, P), { tone: "hi" }) +
      D([at([STRIP.x + 1.2, STRIP.y + 3.5, 0.6]), at([STRIP.x + STRIP.w - 1.2, STRIP.y + 3.5, 0.6])], { size: 0.45 }),
  });
  // Label plate with three rule lines standing for the engraved name.
  const rules = [[3, 20], [3, 16], [3, 12]].map(([from, to], i) => k.lineOnTop([LABEL.x + from, LABEL.y + 3.2 + i * 3.2], [LABEL.x + to, LABEL.y + 3.2 + i * 3.2], 0.8, P));
  slab("label", LABEL, 0, 0.8, { tone: "lo", crease: "none" }, {
    steps: 3,
    extra:
      L(k.planOutline(k.insetPlan(LABEL, 1.6), 0.8, P, 3), { tone: "faint" }) +
      L(rules.join(""), { tone: "lo", free: true }) +
      D([at([LABEL.x + 2.2, LABEL.y + LABEL.d - 2.2, 0.8]), at([LABEL.x + LABEL.w - 2.2, LABEL.y + 2.2, 0.8])], { size: 0.45 }),
  });
}

// ---------------------------------------------------------------- rail
function rail() {
  const bedScrews = [-120, -80, -40, 0, 40, 80, 120].map((x) => at([x, BED.y + BED.d - 2.2, BED_H]));
  slab("bed", BED, 0, BED_H, { tone: "mid" }, { steps: 3, bevel: 0.5, extra: D(bedScrews, { size: 0.42, tone: "lo" }) });
  slab("rail", RAIL, BED_H, RAIL_TOP - BED_H, { tone: "mid" }, { steps: 3, bevel: 0.6, inner: L(k.sideSeam(RAIL, BED_H + 2.4, P, 3), { tone: "faint" }) });
  for (const x of [BED.x - STOP_W, BED.x + BED.w]) {
    const plan = { x, y: -12, w: STOP_W, d: 24, r: 1.2 };
    slab("stop", plan, 0, 13, { tone: "mid" }, { steps: 3, bevel: 0.5, extra: D([at([x + 3, -7, 13]), at([x + 3, 7, 13])], { size: 0.45 }) });
  }
}

// ---------------------------------------------------------------- a carriage on the rail
// Two bushings ride the bed either side of the rail; the block bridges them over the rail.
function carriage(name, cx, w, d, { rivets = 4, knob = true, back = null } = {}) {
  const block = { x: cx - w / 2, y: -d / 2, w, d, r: 2.5 };
  for (const y of [-d / 2 + 1, RAIL.y + RAIL.d + 1]) {
    const hold = R.put;
    if (back && y < 0) R.put = (item) => back.push(item);
    slab(`${name}.bushing`, { x: cx - w / 2 + 2, y, w: w - 4, d: d / 2 - RAIL.d / 2 - 2, r: 1 }, BED_H, RAIL_TOP - BED_H, { tone: "mid", crease: "none" }, { steps: 3 });
    R.put = hold;
  }
  const rivetDots = Array.from({ length: rivets }, (_, i) => at([cx - w / 2 + 4 + ((w - 8) * i) / (rivets - 1), d / 2 - 2.4, 15]));
  slab(`${name}.block`, block, RAIL_TOP, 6, { tone: "mid" }, { steps: 5, bevel: 0.6, extra: D(rivetDots, { size: 0.42, tone: "lo" }) });
  if (knob) {
    // Knurled lock knob on the front face, with a pointer from its centre dot to its rim.
    const neck = { x: cx - 2, y: d / 2 - 0.3, w: 4, d: 3.4, r: 0.8 };
    slab(`${name}.neck`, neck, RAIL_TOP + 1, 4, { tone: "mid", crease: "none" }, { steps: 2, owner: `${name}.knob` });
    const ky = d / 2 + 3.1 + 2.9;
    cyl(`${name}.knob`, cx, ky, 3.2, RAIL_TOP, 6, { tone: "mid" }, {
      steps: 24,
      extra:
        L(k.knurl(cx, ky, 3.2, RAIL_TOP, RAIL_TOP + 6, 22, P), { tone: "lo" }) +
        L(k.lineOnTop([cx, ky], [cx + 3.2, ky], RAIL_TOP + 6, P), { tone: "hi" }) +
        D([at([cx, ky, RAIL_TOP + 6])], { size: 0.5, tone: "hi" }),
    });
  }
  return block;
}

// ---------------------------------------------------------------- a mirror on its carriage
// `side` is -1 for Pichet (far, faces +x) and +1 for Aletheios (near, faces -x).
function mirror(side) {
  const name = side < 0 ? "pichet" : "aletheios";
  const cx = side * MIRROR_X;
  carriage(name, cx, 30, 34);
  // A yoke: a bar across the carriage top, a post either side of the mirror, a trunnion block on each
  // post that holds the mirror at its widest point, and a knurled tilt screw on the near block.
  const half = side < 0 ? HEX_R : VES_R - VES_D / 2;
  const py = half + 4;
  slab(`${name}.yoke`, { x: cx - 4, y: -(py + 3.4), w: 8, d: 2 * (py + 3.4), r: 1.5 }, 15, 3, { tone: "mid" }, { steps: 3, bevel: 0.4 });
  for (const sign of [-1, 1]) {
    const y = sign * py;
    cyl(`${name}.post`, cx, y, 2.4, 18, ZC - 3 - 18, { tone: "mid", crease: "none" }, { steps: 20, extra: L(k.sideArc(cx, y, 2.4, 27, P, 12), { tone: "faint" }) });
    const block = sign > 0 ? { x: cx - 3, y: half - 0.3, w: 6, d: py + 2.4 - (half - 0.3), r: 0.8 } : { x: cx - 3, y: -(py + 2.4), w: 6, d: py + 2.4 - (half - 0.3), r: 0.8 };
    const tilt = sign > 0 ? L(k.knurl(cx, half + 2.6, 1.6, ZC + 3, ZC + 5.6, 12, P), { tone: "lo" }) : "";
    slab(`${name}.trunnion`, block, ZC - 3, 6, { tone: "mid" }, { steps: 2, bevel: 0.3, extra: D([at([cx, y, ZC + 3])], { size: 0.5, tone: "hi" }) });
    if (sign > 0) cyl(`${name}.tilt`, cx, half + 2.6, 1.6, ZC + 3, 2.6, { tone: "mid" }, { steps: 14, extra: tilt });
  }

  const F = G.frameOf([cx, 0, ZC], [1, 0, 0], [0, 1, 0], [0, 0, 1]);
  const poly = side < 0 ? HEX : VES;
  const face = (s, pts) => k.pathOf(pts.map(([u, v]) => at([cx + s, u, ZC + v])), true);
  const shown = MIRROR_T; // the +x face is the one the camera sees: Pichet's mirror, Aletheios' back
  const litColour = side < 0 ? "var(--anatomy-lit)" : "var(--dy-emerald)";
  let overlay = "";
  if (side < 0) {
    // Pichet: a gold rim inset from the edge, a bronze mirror face with two reflection streaks.
    const inner = scaled(HEX, 0.89);
    const chordA = k.pathOf([at([cx + shown, inner[3][0], ZC + inner[3][1]]), at([cx + shown, inner[1][0], ZC + inner[1][1]])]);
    const chordB = k.pathOf([at([cx + shown, inner[4][0], ZC + inner[4][1]]), at([cx + shown, inner[0][0], ZC + inner[0][1]])]);
    overlay =
      `<path class="dy-sheen" d="${face(shown, inner)}"/>` +
      L(chordA + chordB, { tone: "faint" }) +
      `<path class="dy-veil" d="${face(shown, inner)}"/>` +
      L(face(shown, inner), { tone: "hi", className: "dy-rim" }) +
      `<path class="dy-rim-lit" d="${face(shown, inner)}" style="stroke:${litColour}"/>`;
  } else {
    // Aletheios: a vesica frame with an emerald glass pane in its aperture, seen from behind.
    const rim = vesica(0.94);
    const aperture = vesica(0.86);
    const core = vesica(0.6);
    overlay =
      `<path class="dy-glass" d="${face(shown, aperture)}"/>` +
      `<path class="dy-glass-core" d="${face(shown, core)}"/>` +
      L(face(shown, core), { tone: "faint" }) +
      `<path class="dy-veil" d="${face(shown, aperture)}"/>` +
      L(face(shown, aperture), { tone: "lo" }) +
      L(face(shown, rim), { tone: "hi", className: "dy-rim" }) +
      `<path class="dy-rim-lit" d="${face(shown, rim)}" style="stroke:${litColour}"/>`;
  }
  // A soft light around the mirror when it speaks, behind the glass.
  const glowPoly = scaled(poly, 1.25);
  put(`${name}.glow`, [cx - 0.5, 0, ZC], `<path class="dy-glow" data-witness="${name}" d="${face(0, glowPoly)}" style="fill:${litColour}"/>`, [record(A.prism(F, poly, -MIRROR_T, MIRROR_T, { name: `${name}.glow`, owner: `${name}.mirror` }))], { bias: -0.3 });
  put(`${name}.mirror`, [cx, 0, ZC], `<g class="dy-mirror" data-witness="${name}">${S(G.prismOf(F, poly, -MIRROR_T, MIRROR_T, P), { tone: "hi", crease: "lo" })}${overlay}</g>`, [record(A.prism(F, poly, -MIRROR_T, MIRROR_T, { name: `${name}.mirror` }))]);
}

// ---------------------------------------------------------------- the lamp on its carriage, built at X0
function lamp() {
  const items = [];
  const back = [];
  const hold = R.put;
  R.put = (item) => items.push(item);
  carriage("lamp", X0, 24, 32, { rivets: 3, back });
  const py = -14;
  cyl("lamp.collar", X0, py, 4, 15, 3, { tone: "hi", crease: "lo" }, { steps: 28 });
  cyl("lamp.post", X0, py, 2.6, 18, 23, { tone: "hi", crease: "none" }, { steps: 24, extra: L([26, 34].map((z) => k.sideArc(X0, py, 2.6, z, P, 14)).join(""), { tone: "faint" }) });
  // Two knuckles joined by a raked boom: the arm's joints, each with a pivot pin on top.
  const K1 = [X0, py, 46];
  const K2 = [X0, LAMP_Y, 62]; // the boom rakes at 45° and lands high on the second knuckle, clear of the lamp cap
  cyl("lamp.knuckle1", X0, py, 3.6, 41, 9, { tone: "hi", crease: "lo" }, { steps: 24, extra: D([at([X0, py, 50])], { size: 0.5, tone: "hi" }) + L(k.sideArc(X0, py, 3.6, 45.5, P, 14), { tone: "lo" }) });
  const BF = G.frameAlong(K1, G.sub3(K2, K1));
  const reach = G.len3(G.sub3(K2, K1));
  const boomPoly = [[-2.2, -2], [2.2, -2], [2.2, 2], [-2.2, 2]];
  put("lamp.boom", G.lerp3(K1, K2, 0.5), S(G.prismOf(BF, boomPoly, 3.3, reach - 3.1, P), { tone: "hi", crease: "lo" }), [record(A.prism(BF, boomPoly, 3.3, reach - 3.1, { name: "lamp.boom", owner: "lamp.knuckle1" }))]);
  cyl("lamp.knuckle2", X0, LAMP_Y, 3.4, 56, 9, { tone: "hi", crease: "lo" }, { steps: 24, extra: D([at([X0, LAMP_Y, 65])], { size: 0.5, tone: "hi" }) + L(k.sideArc(X0, LAMP_Y, 3.4, 60.5, P, 14), { tone: "lo" }), owner: "lamp.boom" });
  cyl("lamp.stem", X0, LAMP_Y, 1.8, 54.5, 1.5, { tone: "hi", crease: "none" }, { steps: 16 });
  cyl("lamp.cap", X0, LAMP_Y, 3.8, 53, 1.5, { tone: "hi", crease: "lo" }, { steps: 32 });
  cyl("lamp.housing", X0, LAMP_Y, 4.8, 46, 7, { tone: "hi", crease: "lo" }, { steps: 32, extra: L(k.knurl(X0, LAMP_Y, 4.8, 47.5, 51.5, 30, P), { tone: "mid" }) });
  cyl("lamp.shade", X0, LAMP_Y, 6.6, 44, 2, { tone: "hi", crease: "lo" }, { steps: 36, bevel: 0.4 });
  cyl("lamp.bead", X0, LAMP_Y, 2.8, 39, 5, { tone: "hi", crease: "lo", lit: true }, { steps: 24 });
  R.put = hold;
  // The light pool lies on the rail top under the lamp, as a flat projected ellipse.
  const pool = `<path class="dy-pool" d="${k.pathOf(k.circleRing(0, 0, 1, 40).map(([u, v]) => at([X0 + u * 22, v * 4.5, RAIL_TOP])), true)}"/>`;
  put("lamp.back", [X0, -13, 6.5], `<g data-part="lamp-back">${back.map((item) => item.svg).join("")}</g>`, back.flatMap((item) => item.shapes));
  put("lamp.pool", [X0, 0, RAIL_TOP], `<g data-part="pool">${pool}</g>`, [record(A.slab({ x: X0 - 22, y: -4.5, w: 44, d: 9, r: 4.5 }, RAIL_TOP, 0.01, { name: "lamp.pool", owner: "rail" }, 10))], { bias: 0 });
  put("lamp", [X0, 0, 30], `<g data-part="lamp">${items.map((item) => item.svg).join("")}</g>`, items.flatMap((item) => item.shapes), { bias: 0.4 });
}

base();
rail();
mirror(-1);
lamp();
mirror(1);

A.settle(R, P);
// The light pool is markup only, so the solver never sees it: seat it just above the rail bench it lies on.
const keyOf = (name) => R.items.find((item) => item.name === name).key;
R.items.find((item) => item.name === "lamp.pool").key = Math.max(keyOf("stand.plate"), keyOf("bed"), keyOf("rail"), keyOf("strip")) + 0.001;
R.items.sort((a, b) => a.key - b.key);

const rest = stateOf(X0);
const svg = k.figureSvg({
  width: WIDTH,
  height: HEIGHT,
  label:
    "The dyad chamber: a bronze rail on a base plate carries two facing mirrors on carriages at its ends, a hexagonal mirror with a gold rim for Pichet and a vesica-shaped mirror with an emerald glass face for Aletheios. Between them a small lamp hangs from a jointed arm on its own carriage. Wherever the lamp slides, the mirror it is nearer lights and the other dims; at the midpoint both hold half.",
  body: R.items.map((item) => item.svg),
  className: "dy-svg",
});

const live = `
const P = ${JSON.stringify(P)};
const REACH = ${JSON.stringify(REACH)};
const FACE_P = ${FACE_P}, FACE_A = ${FACE_A}, GAP = ${GAP}, EDGE = ${EDGE}, X0 = ${X0}, RAIL_TOP = ${RAIL_TOP}, LAMP_Y = ${LAMP_Y};
const shareOf = ${shareOf.toString()};
const stateOf = ${stateOf.toString()};
const stage = document.querySelector(".iso-stage");
const svg = stage.querySelector("svg");
const lamp = svg.querySelector('[data-part="lamp"]');
const pool = svg.querySelector('[data-part="pool"]');
const lampBack = svg.querySelector('[data-part="lamp-back"]');
const readout = document.querySelector("[data-readout]");
const still = matchMedia("(prefers-reduced-motion: reduce)");
const clamp = (value, low, high) => Math.min(high, Math.max(low, value));
let x = X0, target = X0;
let frame = 0, last = 0, clock = 0, touring = !still.matches, visible = false, idleTimer = 0, sayTimer = 0, dragging = false;
const AMP = (REACH[1] - REACH[0]) / 2, PERIOD = 11;
const phase = Math.asin(clamp(X0 / AMP, -1, 1));
const tourAt = (t) => AMP * Math.sin((t / PERIOD) * Math.PI * 2 + phase);
const f2 = (v) => Math.round(v * 100) / 100;
function draw() {
  const move = translateAlong(P, [x - X0, 0, 0]);
  lamp.setAttribute("transform", move);
  pool.setAttribute("transform", move);
  lampBack.setAttribute("transform", move);
  const s = stateOf(x);
  svg.style.setProperty("--glow-p", f2(s.glowP));
  svg.style.setProperty("--glow-a", f2(s.glowA));
  svg.style.setProperty("--dim-p", f2(s.dimP));
  svg.style.setProperty("--dim-a", f2(s.dimA));
  if (readout.textContent !== s.text) readout.textContent = s.text;
}
function describe() {
  const s = stateOf(x);
  stage.setAttribute("aria-valuenow", String(Math.round(s.p * 100)));
  stage.setAttribute("aria-valuetext", s.held ? "Held, both witnesses at half" : s.p > 0.5 ? "Pichet speaks at " + s.p.toFixed(2) : "Aletheios speaks at " + (1 - s.p).toFixed(2));
}
function tick(now) {
  const dt = last ? Math.min((now - last) / 1000, 1 / 30) : 1 / 60;
  last = now;
  const calm = still.matches;
  if (touring) { clock += dt; target = tourAt(clock); }
  x = calm ? target : x + (target - x) * (1 - Math.exp(-dt / 0.3));
  if (Math.abs(target - x) < 0.01) x = target;
  draw();
  const settled = !touring && x === target;
  frame = visible && !settled ? requestAnimationFrame(tick) : 0;
}
const run = () => { if (!frame && visible) { last = 0; frame = requestAnimationFrame(tick); } };
const steer = (next) => {
  touring = false;
  target = clamp(next, REACH[0], REACH[1]);
  clearTimeout(idleTimer);
  idleTimer = setTimeout(() => { touring = !still.matches; clock = Math.asin(clamp(target / AMP, -1, 1)) * PERIOD / (Math.PI * 2) - phase * PERIOD / (Math.PI * 2); run(); }, 3600);
  clearTimeout(sayTimer);
  sayTimer = setTimeout(describe, 700);
  run();
};
new IntersectionObserver((entries) => { visible = entries[entries.length - 1].isIntersecting; if (visible) run(); }, { rootMargin: "120px 0px" }).observe(stage);
// The pointer maps onto the rail: its position projected onto the screen vector of one world unit along x.
const origin = iso([0, LAMP_Y, RAIL_TOP + 20], P);
const along = axisVector(P, [1, 0, 0]);
const alongLength = along[0] * along[0] + along[1] * along[1];
const railAt = (event) => {
  const box = svg.getBoundingClientRect();
  const vx = ((event.clientX - box.left) / box.width) * ${WIDTH};
  const vy = ((event.clientY - box.top) / box.height) * ${HEIGHT};
  return ((vx - origin[0]) * along[0] + (vy - origin[1]) * along[1]) / alongLength;
};
stage.addEventListener("pointerdown", (event) => { dragging = true; stage.setPointerCapture(event.pointerId); steer(railAt(event)); });
stage.addEventListener("pointermove", (event) => { if (dragging || event.pointerType === "mouse") steer(railAt(event)); });
const release = (event) => { if (!dragging) return; dragging = false; try { stage.releasePointerCapture(event.pointerId); } catch {} steer(target); };
stage.addEventListener("pointerup", release);
stage.addEventListener("pointercancel", release);
stage.addEventListener("pointerleave", () => { if (!dragging) steer(target); });
stage.addEventListener("keydown", (event) => {
  const from = touring ? x : target;
  const step = event.shiftKey ? 20 : 5;
  let to = null;
  if (event.key === "ArrowLeft" || event.key === "ArrowDown") to = from - step;
  if (event.key === "ArrowRight" || event.key === "ArrowUp") to = from + step;
  if (event.key === "Home") to = REACH[0];
  if (event.key === "End") to = REACH[1];
  if (event.key === " " || event.key === "Enter" || event.key === "0") to = 0;
  if (to === null) return;
  event.preventDefault();
  steer(to);
});
draw();
describe();
`;

const body = k.plateHtml({
  fig: "Fig 2",
  title: "Dyad chamber",
  hint: "Drag the lamp along the rail",
  readout: rest.text,
  keys: [
    { mark: "lit", label: "The lamp: the one light, on a jointed arm" },
    { mark: "raised", label: "Pichet, hexagon, gold rim · Aletheios, vesica, emerald glass" },
  ],
  caption:
    "Two witnesses face each other across a rail. The lamp is the question; the mirror it is nearer speaks, lit in its own colour, while the other dims to 35%. At the midpoint both hold half and the chamber reads held · both. The readout's share is the lamp's distance from the far mirror over the whole gap.",
  body: `<div class="iso-stage" tabindex="0" role="slider" aria-label="Dyad chamber: drag or use the arrow keys to move the lamp between the two mirrors; Home and End go to either mirror, Space to the midpoint" aria-valuemin="0" aria-valuemax="100" aria-valuenow="${Math.round(rest.p * 100)}">${svg}</div>`,
});

// Brand tokens over the kit's: patinated bronze, gold as line, one lit accent, emerald only for Aletheios.
const THEME_CSS = `
.iso{
  --anatomy-page:#070B1D;--anatomy-card:#070B1D;--anatomy-text:#F0EDE3;--anatomy-strong:#F0EDE3;--anatomy-muted:#AEBBC4;
  --anatomy-paper:#211B11;--anatomy-face:#2C2417;--anatomy-top:#6E5B36;
  --anatomy-shade-0:#15110B;--anatomy-shade-1:#2C2417;--anatomy-shade-2:#5A4A2E;--anatomy-shade-3:#64522F;
  --anatomy-lit-top:#D9AE45;--anatomy-lit-shade:#A8862E;
  --anatomy-hi:#C5A017;--anatomy-mid:#B08F1E;--anatomy-lo:#4A3E22;--anatomy-faint:#6A5A36;--anatomy-lit:#E6B84D;
  --anatomy-halo:#C5A017;--anatomy-halo-opacity:.22;--anatomy-dot:#E6B84D;
  --dy-emerald:#10B5A7;--dy-glass:#10B5A7;--dy-glass-core:#7FE3D8;--dy-veil:#070B1D;--dy-pool:#E6B84D;
}
.iso[data-theme="light"]{
  --anatomy-page:#F0EDE3;--anatomy-card:#F0EDE3;--anatomy-text:#1A1A24;--anatomy-strong:#1A1A24;--anatomy-muted:#6B6358;
  --anatomy-paper:#F0EDE3;--anatomy-face:#FBF9F2;--anatomy-top:#F7F4EA;
  --anatomy-shade-0:#D3CBB3;--anatomy-shade-1:#DFD8C3;--anatomy-shade-2:#E8E2D0;--anatomy-shade-3:#EFEADB;
  --anatomy-lit-top:#FFF6D6;--anatomy-lit-shade:#E8D9A6;
  --anatomy-hi:#5A4A2E;--anatomy-mid:#8A7652;--anatomy-lo:#B3A684;--anatomy-faint:#D6CDB4;--anatomy-lit:#76652A;
  --anatomy-halo:#6E5B36;--anatomy-halo-opacity:.3;--anatomy-dot:#5A4A2E;
  --dy-emerald:#0E9A8E;--dy-glass:#10B5A7;--dy-glass-core:#9BE7DE;--dy-veil:#F0EDE3;--dy-pool:#C5A017;
}
.iso-plate{--anatomy-paper:#211B11}
.iso[data-theme="light"] .iso-plate{--anatomy-paper:#F0EDE3}
.iso-plate-corner[data-corner="title"]{font-family:Cinzel,Georgia,serif;letter-spacing:.18em;font-size:11.5px}
.iso-stage{outline:none;touch-action:pan-y;cursor:ew-resize}
.iso-stage:focus-visible{outline:1px solid var(--anatomy-muted);outline-offset:6px}
.dy-svg{--glow-p:${rest.glowP};--glow-a:${rest.glowA};--dim-p:${rest.dimP};--dim-a:${rest.dimA}}
.dy-sheen{fill:var(--anatomy-lit-top);stroke:none;opacity:calc(.18 + .5 * var(--glow-p))}
.dy-glass{fill:var(--dy-glass);stroke:none;opacity:calc(.22 + .58 * var(--glow-a))}
.dy-glass-core{fill:var(--dy-glass-core);stroke:none;opacity:calc(.05 + .3 * var(--glow-a))}
.dy-veil{fill:var(--dy-veil);stroke:none}
.dy-mirror[data-witness="pichet"] .dy-veil{opacity:calc(1 - var(--dim-p))}
.dy-mirror[data-witness="aletheios"] .dy-veil{opacity:calc(1 - var(--dim-a))}
.dy-rim-lit{fill:none;stroke-width:1.1px;stroke-linejoin:round;vector-effect:non-scaling-stroke}
.dy-mirror[data-witness="pichet"] .dy-rim-lit{opacity:var(--glow-p)}
.dy-mirror[data-witness="aletheios"] .dy-rim-lit{opacity:var(--glow-a)}
.dy-mirror[data-witness="pichet"] .dy-rim{opacity:calc(.35 + .65 * var(--dim-p))}
.dy-mirror[data-witness="aletheios"] .dy-rim{opacity:calc(.35 + .65 * var(--dim-a))}
.dy-glow{stroke:none;filter:blur(7px)}
.dy-glow[data-witness="pichet"]{opacity:calc(.55 * var(--glow-p))}
.dy-glow[data-witness="aletheios"]{opacity:calc(.5 * var(--glow-a))}
.dy-pool{fill:var(--dy-pool);stroke:none;opacity:.34;filter:blur(5px)}
.iso[data-theme="light"] .dy-glow,.iso[data-theme="light"] .dy-pool,.iso[data-theme="light"] .dy-glass,.iso[data-theme="light"] .dy-sheen{mix-blend-mode:multiply}
.iso[data-theme="light"] .dy-glow{opacity:calc(.28 * var(--glow-p))}
.iso[data-theme="light"] .dy-glow[data-witness="aletheios"]{opacity:calc(.26 * var(--glow-a))}
.iso[data-theme="light"] .dy-pool{opacity:.2}
.iso[data-theme="light"] .dy-sheen{fill:#E6B84D;opacity:calc(.08 + .3 * var(--glow-p))}
@media (prefers-reduced-motion:reduce){.dy-glow,.dy-pool{filter:blur(6px)}}
`;
const FONT = `<link rel="preconnect" href="https://fonts.googleapis.com"><link rel="preconnect" href="https://fonts.gstatic.com" crossorigin><link href="https://fonts.googleapis.com/css2?family=Cinzel:wght@500&display=swap" rel="stylesheet">`;

let html = k.pageHtml({ title: "Dyad chamber", theme, body, script: kitScript() + live });
html = html.replace("</style>", THEME_CSS + "</style>").replace("<title>", FONT + "<title>");

const file = theme === "light" ? "dyad-chamber-light.html" : "dyad-chamber.html";
writeFileSync(join(HERE, file), html);
if (theme === "dark") writeFileSync(join(HERE, "preview.html"), `<!-- @dsCard group="Figures" height=520 -->\n` + html);
console.log("wrote", file, "·", R.items.length, "items ·", R.solids.length, "solids · rest:", rest.text);

A.auditOrExit(R, P, {});
