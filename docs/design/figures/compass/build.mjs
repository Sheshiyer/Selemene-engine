// The compass: a four-pan balance on a hexagonal bronze plinth. `node build.mjs` writes compass.html,
// `--light` writes compass-light.html, `--audit` proves the rest-pose geometry.
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import * as k from "../kit/iso-kit.mjs";
import * as G from "../kit/lathe.mjs";
import * as A from "../kit/audit.mjs";
import { kitScript } from "../.scripts/inline-kit.mjs";
import * as B from "./beam.mjs";

const HERE = dirname(fileURLToPath(import.meta.url));
const theme = process.argv.includes("--light") ? "light" : "dark";

const WIDTH = 600;
const HEIGHT = 400;
const AZIMUTH = 52;
const RAD = Math.PI / 180;

// Plinth: a hexagon, flat-to-flat, one flat facing the viewer.
const HEX_R = 78;
const PLINTH_H = 8;
const FOOT_H = 5;
const hexRing = (r, z = 0) => Array.from({ length: 6 }, (_, i) => [r * Math.cos((AZIMUTH + 30 + 60 * i) * RAD), r * Math.sin((AZIMUTH + 30 + 60 * i) * RAD)]);
const polar = (rho, deg) => [rho * Math.cos(deg * RAD), rho * Math.sin(deg * RAD)];

// Column.
const FOOT = { r: 12, h: 2.5 };
const SHAFT_R = 3.4;
const TABLE = { r: 18, z: 29, h: 2.2 };
const COLLAR = { r: 4.8, z: 44, h: 2.4 };
const CUP = { r: 6.2, z: 56, h: 3 };
const BALL_R = 3;

// The pans at rest and the weight's seat.
const PAN_Z = B.PIVOT[2] - B.MODEL.e - B.CHAIN; // the top of each pan
const SEAT = { at: polar(38, 60), r: 3.6, h: 1 };
const WEIGHT = { r: 3, h: 3, knob: 1.3, knobH: 1.8 };

const P = k.fitProjection(
  [
    ...hexRing(HEX_R).map(([x, y]) => [x, y, -PLINTH_H - FOOT_H]),
    ...hexRing(HEX_R).map(([x, y]) => [x, y, 0]),
    [0, 0, B.PIVOT[2] + B.HUB.s1 + B.HUB.capH + 2],
    ...[[1, 0], [-1, 0], [0, 1], [0, -1]].map(([x, y]) => [B.MODEL.L * x, B.MODEL.L * y, B.PIVOT[2] + B.ARM_S + B.END_R + 13]),
  ],
  WIDTH,
  HEIGHT,
  { pad: 36, azimuth: AZIMUTH },
);

// The four directions, assigned by where they land on screen: the Render Bible puts STABILIZE on the
// left pan, HEAL near, CREATE right and MUTATE far.
const AXES = [[1, 0], [-1, 0], [0, 1], [0, -1]];
const screenOf = (d) => k.iso([B.MODEL.L * d[0], B.MODEL.L * d[1], PAN_Z], P);
const pick = (score) => AXES.reduce((best, d) => (score(screenOf(d), d) > score(screenOf(best), best) ? d : best), AXES[0]);
const DIRS = [pick(([sx]) => -sx), pick(([, sy]) => sy), pick(([sx]) => sx), pick(([, sy]) => -sy)];
const NAMES = ["STABILIZE", "HEAL", "CREATE", "MUTATE"];
const PLACES = ["left", "near", "right", "far"];
if (new Set(DIRS.map(String)).size !== 4) throw new Error("the four pans must land on four different sides of the screen");

const at = (point) => k.iso(point, P);
const S = (paths, style) => k.solidSvg(paths, style);
const L = (d, style) => k.lineSvg(d, style);
const D = (points, style) => k.dotsSvg(points, style);
const byDepth = (points) => [...points].sort((a, b) => k.depthOf([a[0], a[1], 0], P) - k.depthOf([b[0], b[1], 0], P));
const FLOOR = { o: [0, 0, 0], a: [0, 0, 1], u: [1, 0, 0], v: [0, 1, 0] };

// Every drawn piece is an audit item with its true shape; A.settle measures the painter's order.
const R = A.recorder(P);
const record = R.solid;
const items = [];
function put(name, ref, svg, { shapes = [], bias = 0, data = {} } = {}) {
  const item = R.put({ name, at: ref, bias, svg, shapes });
  item.data = data;
  items.push(item);
  return item;
}
const tubeOf = (name, points, r, options = {}) => R.route(name, points, r, { owner: options.owner ?? name, ...options });

// ---------------------------------------------------------------- base
function plinth() {
  const ring = hexRing(HEX_R);
  const halo = `<path class="iso-halo" d="${k.extrude(ring, -PLINTH_H - FOOT_H, 4, P).fill}"/>`;
  for (const [x, y] of byDepth(hexRing(HEX_R - 14))) {
    put("stand.foot", [x, y, -PLINTH_H - FOOT_H + 2], S(k.cylinder(x, y, 7, -PLINTH_H - FOOT_H, 1.6, P, 28), { tone: "lo" }) + S(k.cylinder(x, y, 5.4, -PLINTH_H - FOOT_H + 1.6, FOOT_H - 1.6, P, 28), { tone: "mid" }), {
      shapes: [record(A.cylinder(x, y, 7, -PLINTH_H - FOOT_H, 1.6, { name: "stand.foot.flange", ground: true })), record(A.cylinder(x, y, 5.4, -PLINTH_H - FOOT_H + 1.6, FOOT_H - 1.6, { name: "stand.foot", ground: true }))],
    });
  }
  const screws = hexRing(HEX_R - 7);
  const top = [
    S(k.extrude(ring, -PLINTH_H, PLINTH_H, P, { bevel: 1.2 }), { tone: "lo", inner: L(k.ringSeam(ring, -PLINTH_H + 2.6, P), { tone: "faint" }) }),
    L(k.onTop(hexRing(HEX_R - 3.6), 0, P, true), { tone: "hi" }),
    L(k.onTop(hexRing(HEX_R - 11), 0, P, true), { tone: "faint" }),
    rose(),
    L(screws.map(([x, y]) => k.ring(x, y, 2.1, 0, P, 16)).join(""), { tone: "lo" }),
    D(screws.map(([x, y]) => at([x, y, 0])), { size: 0.5 }),
  ];
  put("stand.plinth", [0, 0, -PLINTH_H / 2], halo + top.join(""), { shapes: [record(A.extruded(ring, -PLINTH_H, PLINTH_H, { name: "stand.plinth", ground: true }))], bias: -0.5 });
  // A soft halo under each pan.
  const haloes = DIRS.map((d) => `<path class="iso-halo" d="${k.cylinder(B.MODEL.L * d[0], B.MODEL.L * d[1], B.PAN_R + 1, 0, 0.2, P, 40).fill}" style="filter:blur(5px)"/>`).join("");
  put("stand.pan-shadows", [0, 0, 0.1], haloes, { shapes: [record(A.cylinder(0, 0, 1, 0, 0.1, { name: "stand.pan-shadows", ground: true }))], bias: 0.02 });
}

// The compass rose engraved on the plinth: a ring round the column foot, a gold rule out to each
// direction's index and a dot on each diagonal.
function rose() {
  const ringR = 30;
  const out = [L(k.ring(0, 0, ringR, 0, P, 64), { tone: "faint" })];
  const rules = DIRS.map((d) => {
    const edge = edgeAt(d);
    return k.lineOnTop([FOOT.r * d[0], FOOT.r * d[1]], [(edge - 10) * d[0], (edge - 10) * d[1]], 0, P);
  });
  out.push(L(rules.join(""), { tone: "lo" }));
  out.push(D([45, 135, 225, 315].map((deg) => at([...polar(ringR, deg), 0])), { size: 0.5, tone: "mid" }));
  return out.join("");
}
const edgeAt = (d) => (HEX_R * Math.cos(30 * RAD)) / Math.cos(((((Math.atan2(d[1], d[0]) / RAD - AZIMUTH) % 60) + 90) % 60) * RAD - 30 * RAD);

function indices() {
  DIRS.forEach((d, index) => {
    const edge = edgeAt(d);
    const tip = [(edge - 2) * d[0], (edge - 2) * d[1]];
    const base = [(edge - 10) * d[0], (edge - 10) * d[1]];
    const side = [-d[1] * 3, d[0] * 3];
    const tri = [tip, [base[0] + side[0], base[1] + side[1]], [base[0] - side[0], base[1] - side[1]]];
    const inner = tri.map(([x, y]) => {
      const cx = (tri[0][0] + tri[1][0] + tri[2][0]) / 3;
      const cy = (tri[0][1] + tri[1][1] + tri[2][1]) / 3;
      return [cx + (x - cx) * 0.6, cy + (y - cy) * 0.6];
    });
    put(`stand.index.${NAMES[index]}`, [(edge - 6) * d[0], (edge - 6) * d[1], 0.45], S(k.extrude(tri, 0, 0.9, P, { bevel: 0.3 }), { tone: "hi", crease: "lo" }) + L(k.onTop(inner, 0.9, P, true), { tone: "hi" }), {
      shapes: [record(A.extruded(tri, 0, 0.9, { name: `stand.index.${NAMES[index]}`, ground: true }))],
      bias: 0.05,
    });
  });
}

function labelPlate() {
  const [cx, cy] = polar(57, 150);
  const plan = { x: cx - 11, y: cy - 4.5, w: 22, d: 9, r: 1 };
  const svg = [
    S(k.slabOf(plan, 0, 0.8, P, 4, 0.3), { tone: "lo" }),
    L(k.planOutline(k.insetPlan(plan, 1.6), 0.8, P, 4), { tone: "faint" }),
    L([13, 8, 11].map((len, row) => k.lineOnTop([plan.x + 3.5, plan.y + 2.4 + row * 2], [plan.x + 3.5 + len, plan.y + 2.4 + row * 2], 0.8, P)).join(""), { tone: "lo", free: true }),
    D([at([plan.x + 1.6, plan.y + plan.d / 2, 0.8]), at([plan.x + plan.w - 1.6, plan.y + plan.d / 2, 0.8])], { size: 0.45 }),
  ].join("");
  put("stand.label", [cx, cy, 0.4], svg, { shapes: [record(A.slab(plan, 0, 0.8, { name: "stand.label", ground: true }, 4))], bias: 0.05 });
}

function seat() {
  const [x, y] = SEAT.at;
  put("stand.seat", [x, y, SEAT.h / 2], S(k.cylinder(x, y, SEAT.r, 0, SEAT.h, P, 32, 0.3), { tone: "lo" }) + L(k.ring(x, y, SEAT.r - 1, SEAT.h, P, 32), { tone: "faint" }), {
    shapes: [record(A.cylinder(x, y, SEAT.r, 0, SEAT.h, { name: "stand.seat", ground: true }))],
    bias: 0.05,
  });
}

// ---------------------------------------------------------------- column
function column() {
  const owner = "column";
  const screws = [45, 135, 225, 315].map((deg) => polar(9.4, AZIMUTH + deg));
  put("column.foot", [0, 0, FOOT.h / 2], S(k.cylinder(0, 0, FOOT.r, 0, FOOT.h, P, 56, 0.6), { tone: "mid" }) + L(screws.map(([x, y]) => k.ring(x, y, 1.3, FOOT.h, P, 12)).join(""), { tone: "lo" }) + D(screws.map(([x, y]) => at([x, y, FOOT.h])), { size: 0.45 }), {
    shapes: [record(A.cylinder(0, 0, FOOT.r, 0, FOOT.h, { name: "column.foot", owner }))],
  });
  put("column.shaft-low", [0, 0, (FOOT.h + TABLE.z) / 2], S(k.cylinder(0, 0, SHAFT_R, FOOT.h, TABLE.z - FOOT.h, P, 28), { tone: "mid", crease: "none", inner: L(k.sideArc(0, 0, SHAFT_R, 14, P, 14), { tone: "faint" }) }), {
    shapes: [record(A.cylinder(0, 0, SHAFT_R, FOOT.h, TABLE.z - FOOT.h, { name: "column.shaft-low", owner }))],
  });
  put("column.table", [0, 0, TABLE.z + TABLE.h / 2], S(k.cylinder(0, 0, TABLE.r, TABLE.z, TABLE.h, P, 72, 0.6), { tone: "mid", inner: L(k.sideArc(0, 0, TABLE.r, TABLE.z + 0.7, P, 40), { tone: "faint" }) }) + L(k.ring(0, 0, TABLE.r - 1.6, TABLE.z + TABLE.h, P, 72), { tone: "faint" }) + scale(), {
    shapes: [record(A.cylinder(0, 0, TABLE.r, TABLE.z, TABLE.h, { name: "column.table", owner }))],
  });
  const upperTop = TABLE.z + TABLE.h;
  put("column.shaft-mid", [0, 0, (upperTop + COLLAR.z) / 2], S(k.cylinder(0, 0, SHAFT_R, upperTop, COLLAR.z - upperTop, P, 28), { tone: "mid", crease: "none", inner: L(k.sideArc(0, 0, SHAFT_R, 38, P, 14), { tone: "faint" }) }), {
    shapes: [record(A.cylinder(0, 0, SHAFT_R, upperTop, COLLAR.z - upperTop, { name: "column.shaft-mid", owner }))],
  });
  put("column.collar", [0, 0, COLLAR.z + COLLAR.h / 2], S(k.cylinder(0, 0, COLLAR.r, COLLAR.z, COLLAR.h, P, 32), { tone: "mid" }) + L(k.knurl(0, 0, COLLAR.r, COLLAR.z, COLLAR.z + COLLAR.h, 26, P), { tone: "lo" }), {
    shapes: [record(A.cylinder(0, 0, COLLAR.r, COLLAR.z, COLLAR.h, { name: "column.collar", owner }))],
  });
  const collarTop = COLLAR.z + COLLAR.h;
  put("column.shaft-high", [0, 0, (collarTop + CUP.z) / 2], S(k.cylinder(0, 0, SHAFT_R, collarTop, CUP.z - collarTop, P, 28), { tone: "mid", crease: "none" }), {
    shapes: [record(A.cylinder(0, 0, SHAFT_R, collarTop, CUP.z - collarTop, { name: "column.shaft-high", owner }))],
  });
  const cupTop = CUP.z + CUP.h;
  put("column.cup", [0, 0, CUP.z + CUP.h / 2], S(k.cylinder(0, 0, CUP.r, CUP.z, CUP.h, P, 36, 0.5), { tone: "mid", inner: L(k.sideArc(0, 0, CUP.r, CUP.z + 1.2, P, 18), { tone: "faint" }) }) + L(k.ring(0, 0, BALL_R + 0.3, cupTop, P, 24), { tone: "faint" }), {
    shapes: [record(A.cylinder(0, 0, CUP.r, CUP.z, CUP.h, { name: "column.cup", owner }))],
  });
  const ballAt = [0, 0, cupTop + BALL_R];
  if (Math.abs(ballAt[2] - B.PIVOT[2]) > 1e-9) throw new Error(`the pivot ball rests at ${ballAt[2]}, but the beam model pivots at ${B.PIVOT[2]}`);
  put("column.pivot", ballAt, S(G.sphereOf(ballAt, BALL_R, P, { steps: 36 }), { tone: "hi", lit: true }), { shapes: [record(A.ball(ballAt, BALL_R, { name: "column.pivot", owner }))], bias: 0.1 });
}

// The engraved arc scale on the table, centred under the pointer's rest tip: arcs at 5°, 10° and 15°
// of tilt, two bounding radials, azimuth ticks on the outer arc and the zero dot.
function scale() {
  const z = TABLE.z + TABLE.h;
  const [cx, cy] = polar(B.POINTER.hang, B.POINTER.angle / RAD);
  const out = B.POINTER.angle / RAD;
  const from = out - 110;
  const to = out + 110;
  const radii = [5, 10, 15].map((deg) => B.POINTER.len * Math.sin(deg * RAD));
  const arc = (r) => Array.from({ length: 45 }, (_, i) => polar(r, from + ((to - from) * i) / 44)).map(([x, y]) => [cx + x, cy + y]);
  const parts = [
    L(k.onTop(arc(radii[0]), z, P), { tone: "lo" }),
    L(k.onTop(arc(radii[1]), z, P), { tone: "lo" }),
    L(k.onTop(arc(radii[2]), z, P), { tone: "mid" }),
  ];
  const radial = (deg, r0, r1) => k.lineOnTop([cx + polar(r0, deg)[0], cy + polar(r0, deg)[1]], [cx + polar(r1, deg)[0], cy + polar(r1, deg)[1]], z, P);
  parts.push(L(radial(from, radii[0], radii[2]) + radial(to, radii[0], radii[2]), { tone: "lo" }));
  const minor = [];
  const major = [];
  for (let deg = from; deg <= to + 1e-9; deg += 15) {
    const isPan = DIRS.some((d) => Math.abs(((Math.atan2(d[1], d[0]) / RAD - deg + 540) % 360) - 180) < 1e-6);
    (isPan ? major : minor).push(radial(deg, radii[2], radii[2] + (isPan ? 1.8 : 1.1)));
  }
  parts.push(L(minor.join(""), { tone: "lo" }), L(major.join(""), { tone: "mid" }));
  parts.push(D([at([cx, cy, z])], { size: 0.5, tone: "hi" }));
  return parts.join("");
}

// ---------------------------------------------------------------- beam, pans, weight
const REST = B.rotationOf([0, 0]);

function beam() {
  const parts = B.beamParts(P, [0, 0], DIRS);
  const F = B.hubFrame(REST);
  const hubAt = [0, 0, B.PIVOT[2] + B.ARM_S];
  put("beam.hub", hubAt, parts.hub, {
    shapes: [record(A.disc(F, B.HUB.s0, B.HUB.s1, B.HUB.r, { name: "beam.hub", owner: "beam" })), record(A.disc(F, B.HUB.s1, B.HUB.s1 + B.HUB.capH, B.HUB.cap, { name: "beam.cap", owner: "beam" }))],
    data: { beam: "hub" },
  });
  DIRS.forEach((d, index) => {
    const FA = B.armFrame(REST, d);
    const mid = G.add3(FA.o, G.mul3(FA.a, (B.ARM.s0 + B.ARM_S1) / 2));
    put(`beam.arm${index}`, mid, parts[`arm${index}`], { shapes: [record(A.prism(FA, B.RECT, B.ARM.s0, B.ARM_S1, { name: `beam.arm${index}`, owner: "beam" }))], data: { beam: `arm${index}` } });
    const end = B.endOf(REST, d);
    put(`beam.end${index}`, end, parts[`end${index}`], { shapes: [record(A.ball(end, B.END_R, { name: `beam.end${index}`, owner: "beam" }))], data: { beam: `end${index}` }, bias: 0.05 });
    const stem = B.stemOf(REST, d);
    put(`beam.stem${index}`, G.lerp3(stem[0], stem[1], 0.5), parts[`stem${index}`], { shapes: tubeOf(`beam.stem${index}`, stem, 0.4, { owner: `beam.stem${index}` }), data: { beam: `stem${index}` } });
  });
  const { hang, bob, top } = B.pointerOf(REST);
  put("beam.pointer", G.lerp3(hang, bob, 0.5), parts.pointer, { shapes: tubeOf("beam.pointer", [hang, top], 0.35, { owner: "beam.pointer" }), data: { beam: "pointer" }, bias: 0.2 });
  put("beam.bob", bob, parts.bob, { shapes: [record(A.ball(bob, B.POINTER.bob, { name: "beam.bob", owner: "beam" }))], data: { beam: "bob" }, bias: 0.25 });
}

function pans() {
  DIRS.forEach((d, index) => {
    const owner = `pan${index}`;
    const hook = B.hookOf(REST, d);
    const centre = [B.MODEL.L * d[0], B.MODEL.L * d[1]];
    const base = Math.atan2(d[1], d[0]) + Math.PI / 2;
    const eyes = [0, 1, 2].map((i) => [centre[0] + B.EYE_RING * Math.cos(base + (i * 2 * Math.PI) / 3), centre[1] + B.EYE_RING * Math.sin(base + (i * 2 * Math.PI) / 3), PAN_Z + B.EYE_R]);
    const chains = eyes.map((eye, i) => {
      const dir = G.unit3(G.sub3(eye, hook));
      const a = G.add3(hook, G.mul3(dir, B.JOIN_R));
      const b = G.sub3(eye, G.mul3(dir, B.EYE_R));
      put(`${owner}.chain${i}`, G.lerp3(a, b, 0.5), L(k.segment(a, b, P), { tone: "mid" }), { shapes: tubeOf(`${owner}.chain${i}`, [a, b], 0.35, { owner: `${owner}.chain${i}` }), data: { pan: index } });
      return [a, b];
    });
    put(`${owner}.join`, hook, S(G.sphereOf(hook, B.JOIN_R, P, { steps: 20 }), { tone: "hi" }), { shapes: [record(A.ball(hook, B.JOIN_R, { name: `${owner}.join`, owner }))], data: { pan: index }, bias: 0.3 });
    const dish =
      S(k.cylinder(centre[0], centre[1], B.PAN_R, PAN_Z - B.PAN_H, B.PAN_H, P, 64, 0.6), { tone: "hi", crease: "lo" }) +
      L(k.ring(centre[0], centre[1], B.PAN_R - 2.2, PAN_Z, P, 56), { tone: "faint" }) +
      L(k.ring(centre[0], centre[1], 3.2, PAN_Z, P, 24), { tone: "faint" }) +
      D([at([centre[0], centre[1], PAN_Z])], { size: 0.45, tone: "lo" });
    put(`${owner}.dish`, [centre[0], centre[1], PAN_Z - B.PAN_H / 2], dish, { shapes: [record(A.cylinder(centre[0], centre[1], B.PAN_R, PAN_Z - B.PAN_H, B.PAN_H, { name: `${owner}.dish`, owner }))], data: { pan: index }, bias: 0.4 });
    eyes.forEach((eye, i) => {
      put(`${owner}.eye${i}`, eye, S(G.sphereOf(eye, B.EYE_R, P, { steps: 14 }), { tone: "hi" }), { shapes: [record(A.ball(eye, B.EYE_R, { name: `${owner}.eye${i}`, owner }))], data: { pan: index }, bias: 0.5 });
    });
    void chains;
  });
}

function weight() {
  const [x, y] = SEAT.at;
  const z = SEAT.h;
  const svg =
    S(k.cylinder(0, 0, WEIGHT.r, 0, WEIGHT.h, P, 28, 0.4), { tone: "hi", crease: "lo" }) +
    L(k.ring(0, 0, WEIGHT.r - 0.8, WEIGHT.h, P, 24), { tone: "faint" }) +
    S(k.cylinder(0, 0, WEIGHT.knob, WEIGHT.h, WEIGHT.knobH, P, 18, 0.3), { tone: "hi", crease: "lo" }) +
    L(k.knurl(0, 0, WEIGHT.knob, WEIGHT.h, WEIGHT.h + WEIGHT.knobH, 10, P, 0.3), { tone: "lo" });
  put("weight", [x, y, z + WEIGHT.h / 2], `<g transform="${k.translateAlong(P, [x, y, z])}">${svg}</g>`, {
    shapes: [record(A.cylinder(x, y, WEIGHT.r, z, WEIGHT.h, { name: "weight.body", owner: "weight" })), record(A.cylinder(x, y, WEIGHT.knob, z + WEIGHT.h, WEIGHT.knobH, { name: "weight.knob", owner: "weight" }))],
    data: { weight: "" },
    bias: 0.35,
  });
}

plinth();
indices();
labelPlate();
seat();
column();
beam();
pans();
weight();

A.settle(R, P);
items.sort((a, b) => a.key - b.key);
A.auditOrExit(R, P, { crowd: [["beam.pointer", "beam.arm*"]] });

const dataAttrs = (item) =>
  Object.entries(item.data)
    .map(([key, value]) => ` data-${key}="${value}"`)
    .join("");
const body = items.map((item) => `<g class="it" data-name="${item.name}" data-k="${item.key.toFixed(3)}"${dataAttrs(item)}>${item.svg}</g>`);

const eq = B.tiltEq(DIRS[1]);
const EQ_DEG = B.degreesOf(eq);
const DATA = {
  P,
  DIRS,
  NAMES,
  PLACES,
  PAN_Z,
  SEAT: { at: SEAT.at, z: SEAT.h },
  WEIGHT_H: WEIGHT.h,
  TOUR: [0, 1, 2, 3, null],
};

const svg = k.figureSvg({
  width: WIDTH,
  height: HEIGHT,
  label: `A four-pan balance on a hexagonal bronze plinth. A column carries a gold ball pivot; a cross beam rests on it with a pan hung on three chains from each arm end, one per compass direction: ${NAMES.map((n, i) => `${n} ${PLACES[i]}`).join(", ")}. A pointer hangs from the hub and reads the tilt on an arc scale engraved on the column's table. Placing the one weight in a pan tilts the beam ${EQ_DEG.toFixed(1)} degrees toward it.`,
  body,
});

const CSS = `
.iso{--anatomy-page:#070B1D;--anatomy-card:#070B1D;--anatomy-text:#AEBBC4;--anatomy-strong:#F0EDE3;--anatomy-muted:#8A96A0;
  --anatomy-paper:#211B11;--anatomy-face:#211B11;--anatomy-top:#6E5B36;
  --anatomy-shade-0:#15110B;--anatomy-shade-1:#2C2417;--anatomy-shade-2:#5A4A2E;--anatomy-shade-3:#64522F;
  --anatomy-lit-top:#B39542;--anatomy-lit-shade:#8A7030;
  --anatomy-hi:#C5A017;--anatomy-mid:#9A8030;--anatomy-lo:#6A5A2A;--anatomy-faint:#3E3620;--anatomy-lit:#E6B84D;
  --anatomy-halo:#C5A017;--anatomy-halo-opacity:.22;--anatomy-dot:#E6B84D}
.iso[data-theme="light"]{--anatomy-page:#F0EDE3;--anatomy-card:#F0EDE3;--anatomy-text:#5A4A2E;--anatomy-strong:#1A1A24;--anatomy-muted:#8A7F66;
  --anatomy-paper:#F0EDE3;--anatomy-face:#F7F4EA;--anatomy-top:#FAF7EE;
  --anatomy-shade-0:#D6CDB6;--anatomy-shade-1:#DFD7C2;--anatomy-shade-2:#E7E0CD;--anatomy-shade-3:#EDE7D7;
  --anatomy-lit-top:#FFFCF2;--anatomy-lit-shade:#EADFBE;
  --anatomy-hi:#1A1A24;--anatomy-mid:#5A4A2E;--anatomy-lo:#9A8C6E;--anatomy-faint:#CFC7B2;--anatomy-lit:#76652A;
  --anatomy-halo:#5A4A2E;--anatomy-halo-opacity:.3;--anatomy-dot:#1A1A24}
.iso .iso-plate{--anatomy-paper:var(--anatomy-card);box-shadow:0 0 0 1px rgba(197,160,23,.22)}
.iso[data-theme="light"] .iso-plate{box-shadow:0 0 0 1px rgba(90,74,46,.28)}
.iso-plate-corner[data-corner="title"]{font-family:Cinzel,"Times New Roman",serif;font-size:11.5px;letter-spacing:.16em;color:var(--anatomy-strong)}
.iso-stage{outline:none;touch-action:pan-y}
.iso-stage:focus-visible{outline:1px solid var(--anatomy-muted);outline-offset:6px}
`;

const latheSource = readFileSync(join(HERE, "..", "kit", "lathe.mjs"), "utf8")
  .replace(/^import .*$/gm, "")
  .replace(/^export /gm, "");
const latheScript = `const G = (() => {${latheSource}\nreturn { viewOf, scaleOf, frameAlong, frameOf, pointOf, disc, sphereOf, prismOf, circleOf, arcOf, dot3, add3, sub3, mul3, unit3, cross3, len3, lerp3 };})();\n`;
const beamScript = readFileSync(join(HERE, "beam.mjs"), "utf8")
  .replace(/^import .*$/gm, "")
  .replace(/^export /gm, "");
const LIVE = readFileSync(join(HERE, "live.js"), "utf8").replace("__DATA__", JSON.stringify(DATA));

const card = k.plateHtml({
  fig: "Fig 4",
  title: "Compass",
  hint: "Hover over a pan to place the weight",
  readout: "level",
  keys: [
    { mark: "lit", label: "Pivot, pointer and bob: the beam's tilt, read on the arc scale" },
    { mark: "raised", label: `Four pans: ${NAMES.map((n, i) => `${n} ${PLACES[i]}`).join(", ")}` },
  ],
  caption: `A four-pan balance. Each pan weighs ${B.MODEL.pan} g and hangs ${B.MODEL.e} units below a pivot ${B.MODEL.L} units away; the ${B.MODEL.beam} g beam carries its mass ${-B.MODEL.hb} above the ball. Put the ${B.MODEL.weight} g weight in a pan and the beam settles ${EQ_DEG.toFixed(1)}° toward it, swinging at ${(Math.sqrt((B.MODEL.g * B.stiffnessOf(true)) / B.inertiaOf(true)) / (2 * Math.PI)).toFixed(2)} Hz under ${B.MODEL.zeta} of critical damping. The pans follow the Render Bible: STABILIZE left, HEAL near, CREATE right, MUTATE far.`,
  body: `<div class="iso-stage" tabindex="0" role="slider" aria-label="Compass balance: arrow keys put the weight in the left, near, right or far pan; Home takes it out" aria-valuemin="0" aria-valuemax="4" aria-valuenow="0" aria-valuetext="Level: the weight is on its seat and no pan is loaded">${svg}</div>`,
});

const fonts = `<link rel="preconnect" href="https://fonts.googleapis.com"><link rel="preconnect" href="https://fonts.gstatic.com" crossorigin><link href="https://fonts.googleapis.com/css2?family=Cinzel:wght@500&display=swap" rel="stylesheet"><style>${CSS}</style>`;
const page = k.pageHtml({ title: "Compass", theme, body: fonts + card, script: kitScript() + latheScript + beamScript + LIVE, width: 650 });
const file = theme === "light" ? "compass-light.html" : "compass.html";
writeFileSync(join(HERE, file), page);
if (theme === "dark") writeFileSync(join(HERE, "preview.html"), `<!-- @dsCard group="Figures" height=520 -->\n${page}`);
console.log(`wrote ${file} · ${items.length} items · ${R.solids.length} solids · ${R.tubes.length} tubes · settles at ${EQ_DEG.toFixed(2)}°`);
