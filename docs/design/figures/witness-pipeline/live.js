const DATA = __DATA__;
const { P, D, X, LOAD, OUT, STAGES, ENGINES, ROUTE_Y, WRAP_R } = DATA;
const stage = document.querySelector(".wp-stage");
const svgEl = stage.querySelector("svg");
const q = (name) => svgEl.querySelector(`[data-part="${name}"]`);
const el = {
  plate: q("plate"), crank: q("crank"), drumSpokes: q("drum-spokes"), wheel: q("wheel"), die: q("die"),
  carriage: q("carriage"), lever: q("lever"), penA: q("pen-a"), penP: q("pen-p"),
  rodsLower: q("rods-lower"), platen: q("platen"), rodsUpper: q("rods-upper"),
};
const readout = document.querySelector("[data-readout]");
const status = stage.querySelector("[data-status]");
const still = matchMedia("(prefers-reduced-motion: reduce)");
const kit = { iso, solidSvg, lineSvg, dotsSvg, fadedSvg, haloOf, slabOf, cylinder, extrude, sideSeam, lineOnTop, planOutline, insetPlan, faceTextSvg, topMatrix, ring, knurl, pathOf };
const PARTS = makeParts(kit, G, P, D);

const STOPS = [LOAD, ...X, OUT];
const ANGLE = { A: (40 * Math.PI) / 180, AP: 0, P: (-40 * Math.PI) / 180 };
const TOUR = [[LOAD, 0.7], [X[0], 1.7], [X[1], 1.7], [X[2], 1.7], [X[3], 1.7], [OUT, 0.9]];
const TRAVEL = 1.35;
const clamp = (v, lo, hi) => Math.min(hi, Math.max(lo, v));
const sm = (a, b, v) => { const t = clamp((v - a) / (b - a), 0, 1); return t * t * (3 - 2 * t); };
const bump = (dx) => 1 - sm(1.2, 4.4, Math.abs(dx));
const follow = (value, to, dt, tau, calm) => (calm ? to : Math.abs(to - value) < 0.002 ? to : value + (to - value) * (1 - Math.exp(-dt / tau)));

let engine = DATA.firstEngine;
let x = LOAD, target = LOAD;
let lever = 0;
let alpha = 1, alphaTo = 1, resetting = false;
let touring = !still.matches, clock = 0;
let frame = 0, last = 0, visible = false, idleTimer = 0;
let shownKey = "";
let lastStop = -1;

function tourAt(t) {
  let acc = 0;
  for (let i = 0; i < TOUR.length; i++) {
    acc += (i ? TRAVEL : 0) + TOUR[i][1];
    if (t < acc) return { target: TOUR[i][0], done: false };
  }
  return { target: OUT, done: true };
}
function gotAt(px) {
  return { value: px >= X[0] - 0.6, route: px >= X[1] - 0.6, lines: px >= X[2] - 0.6, page: px >= X[3] - 0.6 };
}
function stageIndex(px) {
  if (px < (X[0] + X[1]) / 2) return 0;
  if (px < (X[1] + X[2]) / 2) return 1;
  if (px < (X[2] + X[3]) / 2) return 2;
  return 3;
}
function tokensOf(got, eng) {
  const out = [];
  if (got.value) out.push("#" + String(eng.index).padStart(2, "0"));
  if (got.route) out.push(eng.route === "AP" ? "A+P" : eng.route);
  if (got.lines) out.push("lines");
  if (got.page) out.push("page");
  return out;
}
function nearestStop(px) {
  return STOPS.reduce((best, stop, i) => (Math.abs(stop - px) < Math.abs(STOPS[best] - px) ? i : best), 0);
}
function describe() {
  const eng = ENGINES[engine];
  const got = gotAt(target);
  const tokens = tokensOf(got, eng);
  const where = ["at the magazine", "under the die press", "at the switch gate", "under the reading lens", "under the folio press", "at the outfeed tray"][nearestStop(target)];
  const carries = tokens.length ? "carries " + tokens.join(", ") : "blank";
  const text = `${eng.id} plate ${where}, stage ${STAGES[stageIndex(target)]}, ${carries}`;
  stage.setAttribute("aria-valuenow", String(nearestStop(target)));
  stage.setAttribute("aria-valuetext", text);
  return text;
}

function draw() {
  const eng = ENGINES[engine];
  const got = gotAt(x);
  const key = `${engine}:${got.value}${got.route}${got.lines}${got.page}`;
  if (key !== shownKey) {
    shownKey = key;
    el.plate.innerHTML = PARTS.plate(eng, got);
  }
  el.plate.setAttribute("transform", translateAlong(P, [x, 0, 0]));
  el.plate.style.opacity = alpha.toFixed(3);
  const theta = -(x - LOAD) / WRAP_R;
  el.crank.innerHTML = PARTS.crank(theta);
  el.drumSpokes.innerHTML = PARTS.drumSpokes(theta);
  const d1 = bump(x - X[0]);
  el.die.setAttribute("transform", translateAlong(P, [0, 0, -5.2 * d1]));
  el.wheel.innerHTML = PARTS.wheelSpokes(d1 * 2.3);
  const d2 = bump(x - X[1]);
  const cy = (5 * Math.sin(lever)) / Math.sin(ANGLE.A);
  el.carriage.setAttribute("transform", translateAlong(P, [0, cy, -1.2 * d2]));
  el.lever.innerHTML = PARTS.lever(lever);
  const d3 = bump(x - X[2]);
  const pens = translateAlong(P, [0, 0, -5.2 * d3]);
  el.penA.setAttribute("transform", pens);
  el.penP.setAttribute("transform", pens);
  const d4 = bump(x - X[3]);
  const platenZ = 38 - 5.6 * d4;
  el.platen.setAttribute("transform", translateAlong(P, [0, 0, -5.6 * d4]));
  const rods = PARTS.rods(platenZ);
  el.rodsLower.innerHTML = rods.lower;
  el.rodsUpper.innerHTML = rods.upper;
  const tokens = tokensOf(got, eng);
  const text = `${STAGES[stageIndex(x)]} · ${eng.id} · ${tokens.length ? tokens.join(" ") : "blank"}`;
  if (readout.textContent !== text) readout.textContent = text;
}

function tick(now) {
  const dt = last ? Math.min((now - last) / 1000, 1 / 30) : 1 / 60;
  last = now;
  const calm = still.matches;
  if (touring) {
    clock += dt;
    const tour = tourAt(clock);
    if (tour.done && !resetting) { resetting = true; alphaTo = 0; }
    else if (!resetting) target = tour.target;
  }
  if (resetting) {
    alphaTo = 0;
    if (alpha <= 0.04 || calm) {
      engine = (engine + 1) % ENGINES.length;
      x = target = LOAD;
      lever = 0;
      clock = 0;
      resetting = false;
      alphaTo = 1;
      if (calm) alpha = 1;
    }
  }
  x = follow(x, target, dt, 0.42, calm);
  const leverTo = x >= X[1] - 12 ? ANGLE[ENGINES[engine].route] : 0;
  lever = follow(lever, leverTo, dt, 0.3, calm);
  alpha = follow(alpha, alphaTo, dt, 0.16, calm);
  draw();
  const settled = !touring && !resetting && x === target && lever === leverTo && alpha === alphaTo;
  frame = visible && !settled ? requestAnimationFrame(tick) : 0;
}
const run = () => { if (!frame && visible) { last = 0; frame = requestAnimationFrame(tick); } };
new IntersectionObserver((entries) => { visible = entries[entries.length - 1].isIntersecting; if (visible) run(); }, { rootMargin: "120px 0px" }).observe(stage);
if (still.matches) touring = false;

function takeOver() {
  touring = false;
  resetting = false;
  alphaTo = 1;
  clearTimeout(idleTimer);
  idleTimer = setTimeout(() => { if (!still.matches) { touring = true; resetting = true; run(); } }, 3600);
}
function steer(next, announce = false) {
  takeOver();
  target = clamp(next, LOAD, OUT);
  if (announce) {
    const text = describe();
    status.textContent = text;
  }
  run();
}
stage.addEventListener("pointermove", (event) => {
  if (event.pointerType === "touch") return;
  const box = svgEl.getBoundingClientRect();
  const share = clamp(((event.clientX - box.left) / box.width - 0.1) / 0.8, 0, 1);
  steer(LOAD + share * (OUT - LOAD));
});
stage.addEventListener("pointerdown", (event) => {
  if (event.pointerType !== "touch") return;
  const box = svgEl.getBoundingClientRect();
  const share = clamp(((event.clientX - box.left) / box.width - 0.1) / 0.8, 0, 1);
  steer(STOPS[nearestStop(LOAD + share * (OUT - LOAD))], true);
});
stage.addEventListener("focus", () => { status.textContent = describe(); });
stage.addEventListener("keydown", (event) => {
  const here = nearestStop(touring ? x : target);
  let to = null;
  if (event.key === "ArrowRight") to = Math.min(STOPS.length - 1, here + (touring && x < STOPS[here] - 0.5 ? 0 : 1));
  if (event.key === "ArrowLeft") to = Math.max(0, here - (touring && x > STOPS[here] + 0.5 ? 0 : 1));
  if (event.key === "Home") to = 0;
  if (event.key === "End") to = STOPS.length - 1;
  if (event.key === "ArrowUp" || event.key === "ArrowDown") {
    event.preventDefault();
    engine = (engine + (event.key === "ArrowUp" ? 1 : ENGINES.length - 1)) % ENGINES.length;
    steer(touring ? STOPS[here] : target, true);
    return;
  }
  if (event.key === "Escape") { takeOver(); target = x; run(); return; }
  if (to === null) return;
  event.preventDefault();
  steer(STOPS[to], true);
});
describe();
draw();
