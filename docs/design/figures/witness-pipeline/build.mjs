// Witness pipeline: a bronze bench with a flat belt and four stations.
// node build.mjs            → witness-pipeline.html (dark, on void)
// node build.mjs --light    → witness-pipeline-light.html (parchment)
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import * as k from "../kit/iso-kit.mjs";
import * as G from "../kit/lathe.mjs";
import { kitScript } from "../.scripts/inline-kit.mjs";
import { makeParts } from "./parts.mjs";

const HERE = dirname(fileURLToPath(import.meta.url));
const THEME = process.argv.includes("--light") ? "light" : "dark";

// ── Truth: the stages a request passes through in crates/noesis-witness/src ──
// lib.rs      generate_witness_prompt(engine_id, level, ctx)      engine output in, rule-based prompt
// routing.rs  routing_for_engine(engine_id) → RoutingMode          AletheiosPrimary | PichetPrimary | DyadSynthesis
// interpret.rs interpret_with_llm: build_context_message, then tokio::join!(aletheios, pichet)
// interpret.rs … then complete_for_role("synthesis") → parse_synthesis_json → WitnessDyadLlm
// llm.rs      LlmClient::for_tier, model_for_tier(tier, role)      the models behind the pens and the press
const STAGES = ["engine output", "routing", "interpret", "synthesis"];
// Engines in docs/ENGINES.md order (1–17); route from routing.rs routing_for_engine.
// A = AletheiosPrimary, P = PichetPrimary, AP = DyadSynthesis (raaga is unmapped → None → the dyad bin).
const ENGINES = [
  ["panchanga", "AP"], ["human-design", "A"], ["gene-keys", "AP"], ["vimshottari", "A"], ["numerology", "A"],
  ["biorhythm", "P"], ["vedic-clock", "P"], ["biofield", "P"], ["face-reading", "P"], ["nadabrahman", "P"],
  ["transits", "AP"], ["tarot", "AP"], ["i-ching", "A"], ["enneagram", "A"], ["sacred-geometry", "AP"],
  ["sigil-forge", "AP"], ["raaga", "AP"],
].map(([id, route], index) => ({ id, route, index: index + 1 }));

// ── World ────────────────────────────────────────────────────────────────────
const W = 600;
const H = 350;
const BASE = { x: -176, y: -42, w: 352, d: 84, r: 10 };
const BASE_H = 12;
const FOOT_H = 5;
const DRUM_X = 128;
const DRUM_Z = 21;
const DRUM_R = 7;
const DRUM_S = 20;
const BELT_T = 1.2;
const WRAP_R = DRUM_R + BELT_T;
const BELT_TOP = DRUM_Z + WRAP_R;
const BELT_W = 34;
const BED = { x: -120, y: -19, w: 240, d: 38, r: 1 };
const RAIL_Z = [9, 26];
const X = [-84, -28, 28, 84];
const POST_Z = 52;
const BAR_Z = [52, 58];
const PLATE_H = 1.6;
const LOAD = -110;
const OUT = 108;
const ROUTE_Y = { A: 5, AP: 0, P: -5 };
const PEN_Y = { A: 4.5, P: -4.5 };
const PLATEN_H = 3;
const CRANK_S = 34;

const D = {
  beltTop: BELT_TOP,
  plateH: PLATE_H,
  drumX: DRUM_X,
  drumZ: DRUM_Z,
  drumS: DRUM_S,
  crankS: CRANK_S,
  routeY: ROUTE_Y,
  penY: PEN_Y,
  wheel: [X[0], 0, 68.4],
  lever: [X[1], 0, 61],
  folioX: X[3],
  rodZ0: 32,
  rodZ1: BAR_Z[0],
  platenH: PLATEN_H,
};

const P = k.fitProjection(
  [
    ...k.boxCorners(BASE, -BASE_H - FOOT_H, 0),
    [X[0] - 9, -9, 70], [X[0] + 9, 9, 70],
    [X[1], -12, 77], [X[1], 12, 77],
    [X[3] + 11, 3, 63.4],
    [X[2], 0, 66],
  ],
  W,
  H,
  { pad: 26, azimuth: 38, elevation: 27 },
);
const at = (p) => k.iso(p, P);
const S = (paths, style) => k.solidSvg(paths, style);
const L = (d, style) => k.lineSvg(d, style);
const Dots = (points, style) => k.dotsSvg(points, style);
const byDepth = (points) => [...points].sort((a, b) => k.depthOf([a[0], a[1], 0], P) - k.depthOf([b[0], b[1], 0], P));
const YF = (x, z) => G.frameOf([x, 0, z], [0, 1, 0], [1, 0, 0], [0, 0, 1]); // along +y, u = +x, v = +z
const XF = (y, z) => G.frameOf([0, y, z], [1, 0, 0], [0, 1, 0], [0, 0, 1]); // along +x, u = +y, v = +z
const PARTS = makeParts(k, G, P, D);
let count = 0;
const part = (svg, n = 1) => {
  count += n;
  return svg;
};

// ── Base plate on feet, groove, screws, ruler strip, label plate ─────────────
function base() {
  const out = [`<path class="iso-halo" d="${k.haloOf(BASE, -BASE_H - FOOT_H, BASE_H, P)}"/>`];
  for (const [x, y] of byDepth(k.corners(BASE, 24))) {
    out.push(part(S(k.cylinder(x, y, 7, -BASE_H - FOOT_H, 1.6, P, 28), { tone: "lo" })));
    out.push(part(S(k.cylinder(x, y, 5.4, -BASE_H - FOOT_H + 1.6, FOOT_H - 1.6, P, 28), { tone: "mid" })));
  }
  out.push(part(S(k.slabOf(BASE, -BASE_H, BASE_H, P, 8, 1.6), { tone: "mid", inner: L(k.sideSeam(BASE, -4, P, 8), { tone: "faint" }) })));
  out.push(L(k.planOutline(k.insetPlan(BASE, 6), 0, P), { tone: "lo" }));
  const screws = k.corners(BASE, 11);
  out.push(part(L(screws.map(([x, y]) => k.ring(x, y, 2.2, 0, P, 16)).join(""), { tone: "lo" }), 4));
  out.push(Dots(screws.map(([x, y]) => at([x, y, 0])), { size: 0.5 }));
  const strip = { x: -150, y: BASE.y + BASE.d - 11, w: 300, d: 6, r: 1 };
  out.push(part(S(k.slabOf(strip, 0, 0.6, P, 3), { tone: "lo", crease: "none" })));
  const ticks = k.topTicks(strip.x + 5, strip.x + strip.w - 5, 5, 5, strip.y + 0.6, 0.6, [1.8, 3.4], P);
  out.push(L(ticks.minor, { tone: "lo" }), L(ticks.major, { tone: "mid" }));
  out.push(Dots([at([strip.x + 2.4, strip.y + 3, 0.6]), at([strip.x + strip.w - 2.4, strip.y + 3, 0.6])], { size: 0.45 }));
  const label = { x: 104, y: 27, w: 26, d: 11, r: 1.2 };
  out.push(part(S(k.slabOf(label, 0, 0.8, P, 3), { tone: "lo", crease: "none" })));
  out.push(L(k.planOutline(k.insetPlan(label, 1.4), 0.8, P, 3), { tone: "faint" }));
  out.push(L([16, 11, 14].map((w, i) => k.lineOnTop([label.x + 3.6, label.y + 2.8 + i * 2.7], [label.x + 3.6 + w, label.y + 2.8 + i * 2.7], 0.8, P)).join(""), { tone: "lo", free: true }));
  out.push(Dots([at([label.x + label.w - 2.2, label.y + 2.2, 0.8]), at([label.x + label.w - 2.2, label.y + label.d - 2.2, 0.8])], { size: 0.45 }));
  return out.join("");
}

// ── Hopper of blank plates over the head of the belt ─────────────────────────
const HOP_X = -126;
function hopperPost(side) {
  const y = side > 0 ? 33 : -33;
  const out = [];
  out.push(part(S(k.cylinder(HOP_X, y, 4.2, 0, 1.6, P, 24), { tone: "lo" })));
  out.push(part(S(k.cylinder(HOP_X, y, 2.2, 1.6, 44.4, P, 20), { tone: "mid", crease: "none", inner: L(k.sideArc(HOP_X, y, 2.2, 24, P, 12) + k.sideArc(HOP_X, y, 2.2, 40, P, 12), { tone: "faint" }) })));
  return out.join("");
}
function hopper() {
  const out = [];
  const arm = { x: HOP_X - 3, y: -34.5, w: 6, d: 47.5, r: 0.8 };
  out.push(part(S(G.prismOf(XF(0, 0), [[-30.8, 22], [-30.8, 46], [-12, 46]], HOP_X - 1.6, HOP_X + 1.6, P), { tone: "mid" })));
  out.push(part(S(k.slabOf(arm, 46, 4, P, 2, 0.4), { tone: "mid" })));
  out.push(Dots([at([HOP_X, -31, 50]), at([HOP_X, 10, 50])], { size: 0.45 }));
  const frame = [
    { x: HOP_X - 14.6, y: -11.6, w: 29.2, d: 1.6, r: 0.3 },
    { x: HOP_X - 14.6, y: 10, w: 29.2, d: 1.6, r: 0.3 },
    { x: HOP_X - 14.6, y: -10, w: 1.6, d: 20, r: 0.3 },
    { x: HOP_X + 13, y: -10, w: 1.6, d: 20, r: 0.3 },
  ];
  const corners = [[HOP_X - 14.6, -11.6], [HOP_X + 13, -11.6], [HOP_X - 14.6, 10], [HOP_X + 13, 10]];
  for (const [cx, cy] of byDepth(corners)) {
    out.push(part(S(k.slabOf({ x: cx, y: cy, w: 1.6, d: 1.6, r: 0.2 }, 34, 10, P, 1), { tone: "mid", crease: "none" })));
    const tab = { x: cx < HOP_X ? cx + 1.6 : cx - 3, y: cy < 0 ? cy + 1.6 : cy - 2, w: 3, d: 2, r: 0.2 };
    out.push(part(S(k.slabOf(tab, 34, 1, P, 1), { tone: "lo", crease: "none" })));
  }
  for (let i = 0; i < 3; i++) {
    const plan = { x: HOP_X - 11, y: -8, w: 22, d: 16, r: 1.2 };
    out.push(part(S(k.slabOf(plan, 35 + i * PLATE_H, PLATE_H, P, 3, 0.3), { tone: "hi", crease: "lo" })));
  }
  for (const plan of frame) out.push(part(S(k.slabOf(plan, 44, 2, P, 1), { tone: "mid", crease: "none" })));
  out.push(Dots(corners.map(([cx, cy]) => at([cx + 0.8, cy + 0.8, 46])), { size: 0.35, tone: "lo" }));
  return out.join("");
}

// ── Outfeed tray with folded pages at the far end ────────────────────────────
function outfeed() {
  const out = [];
  const tray = { x: 145, y: -12, w: 26, d: 24, r: 1 };
  out.push(`<path class="iso-halo" d="${k.haloOf(tray, 0, 0.5, P)}" style="filter:blur(4px)"/>`);
  out.push(part(S(k.slabOf(tray, 0, 1.6, P, 3, 0.3), { tone: "lo" })));
  for (let i = 0; i < 3; i++) {
    const plan = { x: 152.5, y: -8, w: 11, d: 16, r: 1 };
    const z = 1.6 + i * PLATE_H * 2;
    out.push(part(S(k.slabOf(plan, z, PLATE_H * 2, P, 3, 0.3), { tone: "hi", crease: "lo", inner: L(k.sideSeam(plan, z + PLATE_H, P, 3), { tone: "lo" }) })));
  }
  out.push(part(S(k.slabOf({ x: tray.x, y: tray.y + tray.d, w: tray.w, d: 1.4, r: 0.4 }, 0, 4.2, P, 2), { tone: "mid" })));
  out.push(part(S(k.slabOf({ x: tray.x + tray.w, y: tray.y, w: 1.4, d: tray.d + 1.4, r: 0.4 }, 0, 4.2, P, 2), { tone: "mid" })));
  return out.join("");
}

// ── Belt bed: legs, rails, slider bed, belt ──────────────────────────────────
function legs(side) {
  const out = [];
  for (const x of [-78, 0, 78]) {
    const plan = { x: x - 5, y: side > 0 ? 18 : -24, w: 10, d: 6, r: 0.8 };
    out.push(part(S(k.slabOf(plan, 0, RAIL_Z[0], P, 2, 0.3), { tone: "lo" })));
    out.push(Dots([at([x - 3, plan.y + 3, RAIL_Z[0]]), at([x + 3, plan.y + 3, RAIL_Z[0]])], { size: 0.4, tone: "lo" }));
  }
  return out.join("");
}
function rail(side) {
  const plan = { x: -118, y: side > 0 ? 19 : -23, w: 236, d: 4, r: 0.8 };
  const seam = L(k.sideSeam(plan, RAIL_Z[0] + 4, P, 2), { tone: "faint" });
  const bolts = Dots(Array.from({ length: 11 }, (_, i) => at([-110 + i * 22, plan.y + 2, RAIL_Z[1]])), { size: 0.45, tone: "lo" });
  return part(S(k.slabOf(plan, RAIL_Z[0], RAIL_Z[1] - RAIL_Z[0], P, 2, 0.4), { tone: "mid", inner: seam })) + bolts;
}
function bed() {
  const out = [];
  out.push(part(S(k.slabOf(BED, 14, 14, P, 2), { tone: "mid", crease: "none" })));
  const belt = { x: -DRUM_X, y: -BELT_W / 2, w: DRUM_X * 2, d: BELT_W, r: 0.2 };
  out.push(part(S(k.slabOf(belt, BELT_TOP - BELT_T, BELT_T, P, 1), { tone: "mid", crease: "none" })));
  out.push(L(k.lineOnTop([-DRUM_X + 2, -BELT_W / 2 + 3], [DRUM_X - 2, -BELT_W / 2 + 3], BELT_TOP, P) + k.lineOnTop([-DRUM_X + 2, BELT_W / 2 - 3], [DRUM_X - 2, BELT_W / 2 - 3], BELT_TOP, P), { tone: "faint", free: true }));
  return out.join("");
}

// ── Drums with shafts, pillow blocks and the crank ───────────────────────────
function pedestal(x, side) {
  const out = [];
  const plan = { x: x - 6, y: side > 0 ? 23 : -29, w: 12, d: 6, r: 0.8 };
  out.push(part(S(k.slabOf(plan, 0, 16, P, 2, 0.4), { tone: "lo" })));
  const F = YF(x, DRUM_Z);
  const [s0, s1] = side > 0 ? [23, 29] : [-29, -23];
  out.push(part(S(G.disc(s0, s1, 5.5, F, P, { steps: 36 }), { tone: "mid", inner: L(G.arcOf(F, side > 0 ? s0 + 1.2 : s1 - 1.2, 5.5, P), { tone: "faint" }) })));
  out.push(Dots([at([x - 4, plan.y + 3, 16]), at([x + 4, plan.y + 3, 16])], { size: 0.4, tone: "lo" }));
  return out.join("");
}
function shaft(x, s0, s1) {
  return part(S(G.disc(s0, s1, 2.2, YF(x, DRUM_Z), P, { steps: 20 }), { tone: "mid", crease: "none" }));
}
function drum(x) {
  const F = YF(x, DRUM_Z);
  const out = [];
  out.push(part(S(G.disc(-DRUM_S, DRUM_S, DRUM_R, F, P, { steps: 44 }), { tone: "mid" })));
  out.push(part(S(G.ringBand(-BELT_W / 2, BELT_W / 2, DRUM_R, WRAP_R, F, P, { steps: 44 }), { tone: "mid", crease: "none" })));
  return out.join("");
}
function drumAssembly(x, near) {
  const out = [];
  out.push(pedestal(x, -1));
  out.push(shaft(x, -32, -29), shaft(x, -23, -DRUM_S));
  out.push(drum(x));
  if (!near) out.push(`<g data-part="drum-spokes">${PARTS.drumSpokes(0)}</g>`);
  out.push(shaft(x, DRUM_S, 23));
  out.push(pedestal(x, 1));
  if (near) {
    out.push(shaft(x, 29, 31));
    out.push(part(S(G.disc(31, CRANK_S, 3.5, YF(x, DRUM_Z), P, { steps: 24 }), { tone: "mid" })));
    out.push(`<g data-part="crank">${PARTS.crank(0)}</g>`);
    count += 3;
  } else {
    out.push(shaft(x, 29, 32));
    out.push(part(S(G.disc(32, 33.2, 3.2, YF(x, DRUM_Z), P, { steps: 24 }), { tone: "lo" })));
  }
  return out.join("");
}

// ── Stations: posts, crossbars and the machine at each ───────────────────────
function post(x, side) {
  const out = [];
  const foot = { x: x - 5.5, y: side > 0 ? 24.5 : -35.5, w: 11, d: 11, r: 1.2 };
  const plan = { x: x - 4, y: side > 0 ? 26 : -34, w: 8, d: 8, r: 1 };
  out.push(part(S(k.slabOf(foot, 0, 1.6, P, 3), { tone: "lo", crease: "none" })));
  out.push(Dots(k.corners(foot, 2).map(([fx, fy]) => at([fx, fy, 1.6])), { size: 0.4, tone: "lo" }));
  out.push(part(S(k.slabOf(plan, 1.6, POST_Z - 1.6, P, 3, 0.5), { tone: "mid", inner: L(k.sideSeam(plan, 40, P, 3), { tone: "faint" }) })));
  return out.join("");
}
function crossbar(x) {
  const plan = { x: x - 5, y: -35, w: 10, d: 49, r: 1.2 };
  const out = [];
  out.push(part(S(G.prismOf(XF(0, 0), [[-26, 24], [-26, BAR_Z[0]], [-8, BAR_Z[0]]], x - 2, x + 2, P), { tone: "mid" })));
  out.push(part(S(k.slabOf(plan, BAR_Z[0], BAR_Z[1] - BAR_Z[0], P, 3, 0.6), { tone: "mid", inner: L(k.sideSeam(plan, BAR_Z[0] + 1.6, P, 3), { tone: "faint" }) })));
  out.push(Dots([[x - 2.6, -31.5], [x + 2.6, -31.5], [x - 2.6, 10.5], [x + 2.6, 10.5]].map(([px, py]) => at([px, py, BAR_Z[1]])), { size: 0.45 }));
  return out.join("");
}

// Station 1: die press. The ram carries a die that stamps the engine's number.
function diePress() {
  const x = X[0];
  const under = [];
  const die = { x: x - 8, y: -6, w: 16, d: 12, r: 1 };
  const head = part(S(k.slabOf(die, 36, 4, P, 3, 0.4), { tone: "hi", crease: "lo" })) + part(S(k.cylinder(x, 0, 3, 40, 9.2, P, 20), { tone: "mid", crease: "none" }));
  under.push(`<g data-part="die">${head}</g>`);
  under.push(part(S(k.cylinder(x, 0, 5.4, 44, BAR_Z[0] - 44, P, 28), { tone: "mid", inner: L(k.sideArc(x, 0, 5.4, 46, P, 16), { tone: "faint" }) })));
  const over = [];
  over.push(part(S(k.cylinder(x, 0, 1.8, BAR_Z[1], 8, P, 16), { tone: "mid", crease: "none" })));
  over.push(part(S(k.cylinder(x, 0, 9, 66, 2.4, P, 48, 0.4), { tone: "mid" })));
  over.push(L(k.knurl(x, 0, 9, 66, 68.4, 48, P), { tone: "lo" }));
  over.push(L(k.ring(x, 0, 7.6, 68.4, P, 48), { tone: "lo" }));
  over.push(`<g data-part="wheel">${PARTS.wheelSpokes(0)}</g>`);
  over.push(part(S(k.cylinder(x, 0, 2.2, 68.4, 1.6, P, 16), { tone: "mid" })));
  over.push(Dots([at([x, 0, 70])], { size: 0.45, tone: "hi" }));
  return { under: under.join(""), over: over.join("") };
}

// Station 2: switch gate. The gold lever tilts to the route; the punch marks it on the plate.
function switchGate() {
  const x = X[1];
  const under = [];
  under.push(`<g data-part="carriage">${part(S(k.cylinder(x, 0, 1.2, 32, 8, P, 14), { tone: "mid", crease: "none" }))}${part(S(k.slabOf({ x: x - 4.5, y: -4, w: 9, d: 8, r: 0.8 }, 40, 6, P, 2, 0.3), { tone: "hi", crease: "lo" }))}</g>`);
  const rail = { x: x - 3, y: -12, w: 6, d: 24, r: 0.6 };
  under.push(part(S(k.slabOf(rail, 46, BAR_Z[0] - 46, P, 2, 0.3), { tone: "mid", inner: L(k.sideSeam(rail, 49, P, 2), { tone: "faint" }) })));
  const over = [];
  over.push(part(S(k.slabOf({ x: x - 4, y: -4, w: 8, d: 8, r: 0.8 }, BAR_Z[1], 3, P, 2, 0.3), { tone: "mid" })));
  const F = XF(0, 61);
  const fan = [[-3, -3], [3, -3]];
  for (let i = 0; i <= 10; i++) {
    const a = ((50 + 8 * i) * Math.PI) / 180;
    fan.push([Math.cos(a) * 15, Math.sin(a) * 15]);
  }
  const rays = [50, 90, 130].map((deg) => {
    const a = (deg * Math.PI) / 180;
    return k.pathOf([at([x - 2.6, Math.cos(a) * 3.2, 61 + Math.sin(a) * 3.2]), at([x - 2.6, Math.cos(a) * 14.6, 61 + Math.sin(a) * 14.6])]);
  });
  over.push(part(S(G.prismOf(F, fan.map(([u, v]) => [u, v - 3]).map(([u, v]) => [u, v + 3]), x - 3.2, x - 2.6, P), { tone: "lo", inner: L(rays.join(""), { tone: "mid" }) })));
  over.push(part(S(G.disc(x - 2.6, x - 1, 3, F, P, { steps: 24 }), { tone: "mid" })));
  over.push(`<g data-part="lever">${PARTS.lever(0)}</g>`);
  count += 2;
  over.push(part(S(G.disc(x + 1, x + 3, 3, F, P, { steps: 24 }), { tone: "mid" })));
  over.push(Dots([at([x + 3, 0, 61])], { size: 0.45, tone: "hi" }));
  return { under: under.join(""), over: over.join("") };
}

// Station 3: reading lens with two pen arms, Aletheios near (left) and Pichet far (right).
function readingLens() {
  const x = X[2];
  const under = [];
  for (const side of [-1, 1]) {
    const y = side > 0 ? PEN_Y.A : PEN_Y.P;
    under.push(`<g data-part="pen-${side > 0 ? "a" : "p"}">${part(S(k.cylinder(x, y, 1.1, 36, 9, P, 14), { tone: "hi", crease: "none" }))}</g>`);
  }
  const barrel = part(S(k.cylinder(x, 0, 7, 41, BAR_Z[0] - 41, P, 40), { tone: "hi", crease: "lo", inner: L(k.sideArc(x, 0, 7, 44.6, P, 24) + k.sideArc(x, 0, 7, 48.4, P, 24), { tone: "faint" }) }));
  under.push(barrel);
  under.push(L(k.knurl(x, 0, 7, 41.4, 44.2, 40, P, 0.3), { tone: "lo" }));
  for (const side of [-1, 1]) {
    const holder = { x: x - 2, y: side > 0 ? 2.5 : -6.5, w: 4, d: 4, r: 0.6 };
    under.push(part(S(k.slabOf(holder, 44, BAR_Z[0] - 44, P, 2, 0.3), { tone: "mid" })));
    under.push(Dots([at([x, side > 0 ? 4.5 : -4.5, BAR_Z[0]])], { size: 0.35, tone: "lo" }));
    const arm = side > 0 ? { x: x - 2, y: 6.5, w: 4, d: 4.5, r: 0.6 } : { x: x - 2, y: -26, w: 4, d: 19.5, r: 0.6 };
    under.push(part(S(k.slabOf(arm, 46, 3, P, 2, 0.3), { tone: "mid" })));
    const bracket = side > 0 ? { x: x - 3, y: 9, w: 6, d: 4, r: 0.6 } : { x: x - 3, y: -30, w: 6, d: 4, r: 0.6 };
    under.push(part(S(k.slabOf(bracket, 44, side > 0 ? BAR_Z[0] - 44 : 6, P, 2, 0.3), { tone: "mid" })));
    if (side < 0) under.push(Dots([at([x - 1.6, -28, 50]), at([x + 1.6, -28, 50])], { size: 0.4, tone: "lo" }));
  }
  const over = [];
  over.push(part(S(k.cylinder(x, 0, 3.2, BAR_Z[1], 6, P, 24), { tone: "mid", crease: "none", inner: L(k.sideArc(x, 0, 3.2, 61, P, 16), { tone: "faint" }) })));
  over.push(part(S(k.cylinder(x, 0, 4.2, 64, 2, P, 28, 0.3), { tone: "hi", crease: "lo" })));
  over.push(L(k.ring(x, 0, 2.6, 66, P, 24), { tone: "lo" }));
  return { under: under.join(""), over: over.join("") };
}

// Station 4: folio press. The platen descends on two guide rods and the blade folds the plate.
function folioPress() {
  const x = X[3];
  const under = [];
  const rods = PARTS.rods(38);
  under.push(`<g data-part="rods-lower">${rods.lower}</g>`);
  count += 2;
  const platen = { x: x - 12, y: -9, w: 24, d: 18, r: 1.2 };
  under.push(`<g data-part="platen">${part(S(k.slabOf({ x: x - 0.7, y: -8, w: 1.4, d: 16, r: 0.3 }, 35, 3, P, 1), { tone: "hi", crease: "none" }))}${part(S(k.slabOf(platen, 38, PLATEN_H, P, 3, 0.5), { tone: "hi", crease: "lo", inner: L(k.sideSeam(platen, 39, P, 3), { tone: "faint" }) }))}${Dots([at([x - 8, 0, 41]), at([x + 8, 0, 41])], { size: 0.4, tone: "lo" })}</g>`);
  under.push(`<g data-part="rods-upper">${rods.upper}</g>`);
  const over = [];
  over.push(part(S(k.slabOf({ x: x - 11, y: -3, w: 22, d: 6, r: 0.8 }, BAR_Z[1], 1.6, P, 2, 0.3), { tone: "mid" })));
  for (const dx of [-8, 8]) {
    over.push(part(S(k.cylinder(x + dx, 0, 1.4, BAR_Z[1] + 1.6, 1.4, P, 14), { tone: "mid", crease: "none" })));
    over.push(part(S(k.cylinder(x + dx, 0, 2.6, 61, 2.4, P, 6), { tone: "mid" })));
    over.push(Dots([at([x + dx, 0, 63.4])], { size: 0.4, tone: "hi" }));
  }
  return { under: over.length ? under.join("") : "", over: over.join("") };
}

// ── Assemble the layers, back to front ───────────────────────────────────────
const machines = [diePress(), switchGate(), readingLens(), folioPress()];
const back = [base(), legs(-1), rail(-1), hopperPost(-1), X.map((x) => post(x, -1)).join(""), drumAssembly(-DRUM_X, false), bed()];
const firstPlate = PARTS.plate(ENGINES[3], {});
count += 1;
const plateSlot = `<g data-part="plate" transform="${k.translateAlong(P, [LOAD, 0, 0])}">${firstPlate}</g>`;
const front = [
  rail(1),
  legs(1),
  hopper(),
  drumAssembly(DRUM_X, true),
  ...X.map((x, i) => machines[i].under + crossbar(x) + machines[i].over),
  outfeed(),
];

const LABEL =
  "A bronze bench with a flat belt over two drums and a crank. Four C-frame stations reach over the belt from the far side: a die press that stamps a number, a switch gate with a gold lever and a punch, a reading lens with two pen arms, and a folio press that folds the plate. A bronze plate rides the belt from a hopper of blanks over the head of the belt to a tray of folded pages at the right, gaining one impression at each station.";
const svg = k.figureSvg({ width: W, height: H, label: LABEL, body: [...back, plateSlot, ...front] });

// ── Page ─────────────────────────────────────────────────────────────────────
const CSS = readFileSync(join(HERE, "theme.css"), "utf8");
const latheSource = readFileSync(join(HERE, "..", "kit", "lathe.mjs"), "utf8").replace(/^import .*$/m, "").replace(/^export /gm, "");
const latheScript = `const G = (() => {${latheSource}\nreturn { frameOf, pointOf, disc, prismOf, spokesOf, ribsOf };})();\n`;
const partsScript = readFileSync(join(HERE, "parts.mjs"), "utf8").replace(/^export /gm, "");
const DATA = { P, D, X, LOAD, OUT, STAGES, ENGINES, ROUTE_Y, WRAP_R, BELT_TOP, PLATE_H, PLATEN_H, firstEngine: 3 };
const LIVE = readFileSync(join(HERE, "live.js"), "utf8").replace("__DATA__", JSON.stringify(DATA));

const body =
  `<style>${CSS}</style>` +
  k.plateHtml({
    fig: "Fig 3",
    title: "Witness pipeline",
    hint: "Hover to scrub · ← → · ↑ ↓ engine",
    readout: "engine output · vimshottari · blank",
    keys: [
      { mark: "lit", label: "Gold lever: the route routing_for_engine picks for the plate's engine" },
      { mark: "raised", label: "Four stations: die press, switch gate, reading lens, folio press" },
    ],
    caption:
      "A request through crates/noesis-witness. An engine's output arrives as a blank plate from the magazine; the die press stamps its registry number (engine output); the switch gate's lever tilts to the route routing_for_engine returns, Aletheios-primary to the left, Pichet-primary to the right, dyad-synthesis straight on, and punches it into the plate (routing); under the reading lens two pens write at once, as interpret_with_llm runs the Aletheios and Pichet pillars in parallel (interpret); the folio press folds the plate into a page carrying the synthesis line and the witness question, the WitnessDyadLlm the crate returns (synthesis). Seventeen engines in docs/ENGINES.md order run through, one plate each.",
    body: `<div class="iso-stage wp-stage" tabindex="0" role="slider" aria-label="Witness pipeline: left and right arrows move the plate between stations, up and down change the engine, Home and End go to the ends of the belt" aria-valuemin="0" aria-valuemax="5" aria-valuenow="0">${svg}<span class="wp-sr" role="status" aria-live="polite" data-status></span></div>`,
  });

const page = k.pageHtml({ title: "Witness pipeline", theme: THEME, body, script: kitScript() + latheScript + partsScript + LIVE, width: 650 });
const name = THEME === "light" ? "witness-pipeline-light.html" : "witness-pipeline.html";
writeFileSync(join(HERE, name), page);
if (THEME === "dark") writeFileSync(join(HERE, "preview.html"), `<!-- @dsCard group="Figures" height=520 -->\n${page}`);
console.log("wrote", name, `${(page.length / 1024).toFixed(0)} KB`, `${count} parts`, `scale ${P.scale.toFixed(3)}`);
