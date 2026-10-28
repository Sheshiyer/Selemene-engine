// The beam of the compass balance and the lever model that tilts it.
// build.mjs imports this to draw the rest pose and record the audit shapes; the page inlines the same
// source (imports stripped, exports stripped) so every frame is rebuilt by the same functions.
import { solidSvg, lineSvg, segment } from "../kit/iso-kit.mjs";
import * as G from "../kit/lathe.mjs";

// Lever model. Lengths in world units (1 unit is about 2 mm), masses in grams, g in units/s².
export const MODEL = {
  L: 50, // arm, pivot to hook
  e: 6.3, // the hooks hang this far below the pivot: what makes the balance stable
  pan: 10, // each pan with its chains
  weight: 1, // the one weight the reader places
  beam: 12, // the cross beam with its hub
  hb: -4.5, // the beam's centre of mass sits 4.5 above the pivot (the hub cup rests on the ball)
  g: 4905, // 9.81 m/s² in units of 2 mm
  zeta: 0.75, // damping ratio of the swing: one small overshoot, then rest
};

export const PIVOT = [0, 0, 62];
export const HUB = { r: 8, s0: 3, s1: 6, cap: 2.4, capH: 1.2 };
export const ARM = { half: [1.5, 1.1], s0: 7.9 };
export const END_R = 2.6;
export const JOIN_R = 1.3;
export const CHAIN = 28;
export const EYE_R = 0.6;
export const EYE_RING = 9.9;
export const PAN_R = 11;
export const PAN_H = 2;
export const POINTER = { angle: Math.PI / 4, hang: 7.6, len: 29, bob: 1.3 };

export const RECT = [
  [-ARM.half[0], -ARM.half[1]],
  [ARM.half[0], -ARM.half[1]],
  [ARM.half[0], ARM.half[1]],
  [-ARM.half[0], ARM.half[1]],
];
export const ARM_S1 = MODEL.L - Math.sqrt(END_R * END_R - (ARM.half[0] ** 2 + ARM.half[1] ** 2));
export const ARM_S = (HUB.s0 + HUB.s1) / 2;

// Restoring stiffness and inertia about the pivot, with or without the weight in a pan.
export function stiffnessOf(loaded) {
  const M = MODEL;
  return 4 * M.pan * M.e + (loaded ? M.weight * M.e : 0) + M.beam * M.hb;
}
export function inertiaOf(loaded) {
  const M = MODEL;
  return 4 * M.pan * M.L * M.L + (loaded ? M.weight * M.L * M.L : 0) + (M.beam * M.L * M.L) / 3 + M.beam * M.hb * M.hb;
}
// Where the beam settles with the weight in the pan that hangs in direction d (a floor unit vector).
export const tiltEq = (d) => (d ? [(MODEL.weight * MODEL.L * d[0]) / stiffnessOf(true), (MODEL.weight * MODEL.L * d[1]) / stiffnessOf(true)] : [0, 0]);
export const degreesOf = (t) => (Math.hypot(t[0], t[1]) * 180) / Math.PI;

// One time step of the two-axis swing: state.t is the tilt vector (radians, toward the dipping side),
// state.v its rate. Semi-implicit Euler in sub-steps. Returns true once settled (and snaps there).
export function stepTilt(state, d, dt, calm = false) {
  const loaded = Boolean(d);
  const eq = tiltEq(d);
  if (calm) {
    state.t = eq;
    state.v = [0, 0];
    return true;
  }
  const w2 = (MODEL.g * stiffnessOf(loaded)) / inertiaOf(loaded);
  const c = 2 * MODEL.zeta * Math.sqrt(w2);
  const drive = loaded ? [(MODEL.g * MODEL.weight * MODEL.L * d[0]) / inertiaOf(true), (MODEL.g * MODEL.weight * MODEL.L * d[1]) / inertiaOf(true)] : [0, 0];
  const n = 8;
  const h = dt / n;
  for (let i = 0; i < n; i++) {
    for (let a = 0; a < 2; a++) {
      state.v[a] += (drive[a] - w2 * state.t[a] - c * state.v[a]) * h;
      state.t[a] += state.v[a] * h;
    }
  }
  const error = Math.hypot(state.t[0] - eq[0], state.t[1] - eq[1]);
  const speed = Math.hypot(state.v[0], state.v[1]);
  if (error < 1e-3 && speed < 1e-3) {
    state.t = eq;
    state.v = [0, 0];
    return true;
  }
  return false;
}

// The rigid rotation of the beam for a tilt vector t: the end in direction t̂ drops by sin|t|.
export function rotationOf(t) {
  const theta = Math.hypot(t[0], t[1]);
  if (theta < 1e-9) return (v) => [v[0], v[1], v[2]];
  const d = [t[0] / theta, t[1] / theta];
  const p = [-d[1], d[0]];
  const c = Math.cos(theta);
  const s = Math.sin(theta);
  return (v) => {
    const vp = v[0] * p[0] + v[1] * p[1];
    const vd = v[0] * d[0] + v[1] * d[1];
    const vz = v[2];
    const along = vd * c + vz * s;
    return [vp * p[0] + along * d[0], vp * p[1] + along * d[1], -vd * s + vz * c];
  };
}

const at = (R, v) => G.add3(PIVOT, R(v));

// Beam-frame points that the pans and chains hang from.
export const hookOf = (R, d) => at(R, [MODEL.L * d[0], MODEL.L * d[1], -MODEL.e]);
export const endOf = (R, d) => at(R, [MODEL.L * d[0], MODEL.L * d[1], ARM_S]);
export const stemOf = (R, d) => [at(R, [MODEL.L * d[0], MODEL.L * d[1], ARM_S - END_R]), at(R, [MODEL.L * d[0], MODEL.L * d[1], -MODEL.e + JOIN_R])];
export function hubFrame(R) {
  return G.frameOf(PIVOT, R([0, 0, 1]), R([1, 0, 0]), R([0, 1, 0]));
}
export function armFrame(R, d) {
  const n = R([0, 0, 1]);
  return G.frameOf(G.add3(PIVOT, G.mul3(n, ARM_S)), R([d[0], d[1], 0]), R([-d[1], d[0], 0]), n);
}
export function pointerOf(R) {
  const F = hubFrame(R);
  const hang = G.pointOf(F, HUB.s0, POINTER.hang, POINTER.angle);
  const bob = G.add3(hang, G.mul3(F.a, -POINTER.len));
  return { hang, bob, top: G.add3(bob, G.mul3(F.a, POINTER.bob)) };
}

// Every piece of the beam at tilt t, as svg strings keyed by name, in a painter's order that holds
// for any tilt the model reaches (far arm first, the hub in the middle, the pointer last).
export function beamParts(P, t, DIRS) {
  const R = rotationOf(t);
  const parts = {};
  const S = (paths, style) => solidSvg(paths, style);
  const F = hubFrame(R);
  parts.hub =
    S(G.disc(HUB.s0, HUB.s1, HUB.r, F, P, { bevel: 0.5 }), { tone: "hi", crease: "lo", inner: lineSvg(G.arcOf(F, HUB.s0 + 1.1, HUB.r, P), { tone: "faint" }) }) +
    lineSvg(G.circleOf(F, HUB.s1, HUB.r - 1.8, P), { tone: "faint" }) +
    S(G.disc(HUB.s1, HUB.s1 + HUB.capH, HUB.cap, F, P, { bevel: 0.3 }), { tone: "hi", crease: "lo" });
  DIRS.forEach((d, index) => {
    const FA = armFrame(R, d);
    parts[`arm${index}`] = S(G.prismOf(FA, RECT, ARM.s0, ARM_S1, P), { tone: "hi", crease: "lo" });
    parts[`end${index}`] = S(G.sphereOf(endOf(R, d), END_R, P, { steps: 28 }), { tone: "hi" });
    const [top, bottom] = stemOf(R, d);
    parts[`stem${index}`] = lineSvg(segment(top, bottom, P), { tone: "mid" });
  });
  const { hang, bob } = pointerOf(R);
  parts.pointer = lineSvg(segment(hang, bob, P), { tone: "lit" }).replace("<path ", '<path style="stroke-width:1.1" ');
  parts.bob = S(G.sphereOf(bob, POINTER.bob, P, { steps: 24 }), { tone: "hi", lit: true });
  return parts;
}

// Where the pan in direction d has moved from its rest position at tilt t.
export function panShift(t, d) {
  const R = rotationOf(t);
  return G.sub3(hookOf(R, d), hookOf(rotationOf([0, 0]), d));
}

export const BEAM_NAMES = (count) => ["hub", ...Array.from({ length: count }, (_, i) => [`arm${i}`, `end${i}`, `stem${i}`]).flat(), "pointer", "bob"];
