// Live behaviour of the compass balance. Inlined after the kit, the lathe module (G) and beam.mjs, so
// iso, depthOf, translateAlong, axisVector, beamParts, panShift, stepTilt and friends are in scope.
const DATA = __DATA__;
const { P, DIRS, NAMES, PLACES, PAN_Z, SEAT, TOUR } = DATA;
const FLIGHT = 0.9; // seconds the hand takes to carry the weight
const LIFT = 16; // how high it lifts on the way
const STEP = 4.6; // seconds per stop of the idle tour
const stage = document.querySelector(".iso-stage");
const svg = stage.querySelector("svg");
const readout = document.querySelector("[data-readout]");
const still = matchMedia("(prefers-reduced-motion: reduce)");
const [, , VIEW_W, VIEW_H] = svg.getAttribute("viewBox").split(" ").map(Number);
const beamNodes = Object.fromEntries([...svg.querySelectorAll("[data-beam]")].map((el) => [el.dataset.beam, el]));
const panNodes = DIRS.map((_, index) => [...svg.querySelectorAll(`[data-pan="${index}"]`)]);
const weightNode = svg.querySelector("[data-weight]");
const weightInner = weightNode.firstElementChild;
const seatNode = svg.querySelector('[data-name="stand.seat"]');
const sorted = [...svg.querySelectorAll(".it")].filter((el) => el !== weightNode).map((el) => ({ el, k: Number(el.dataset.k) }));
const EQ = degreesOf(tiltEq(DIRS[0]));

const tilt = { t: [0, 0], v: [0, 0] };
let tiltSettled = true;
let choice = null; // which pan the weight is asked into; null is the seat
const weight = { pos: [SEAT.at[0], SEAT.at[1], SEAT.z], target: null, inPan: null, flying: false, from: null, elapsed: 0 };
let frame = 0;
let last = 0;
let clock = 0;
let touring = !still.matches;
let visible = false;
let idleTimer = 0;
let drawnTilt = null;
let slot = null;

const polarOf = ([x, y, z]) => [Math.hypot(x, y), Math.atan2(y, x), z];
const fromPolar = ([rho, phi, z]) => [rho * Math.cos(phi), rho * Math.sin(phi), z];
const smooth = (u) => u * u * (3 - 2 * u);
const panCentre = (index) => {
  const d = DIRS[index];
  const shift = panShift(tilt.t, d);
  return [MODEL.L * d[0] + shift[0], MODEL.L * d[1] + shift[1], PAN_Z + shift[2]];
};
const targetOf = () => (weight.target === null ? [SEAT.at[0], SEAT.at[1], SEAT.z] : panCentre(weight.target));

function choose(next) {
  if (next === choice) return;
  choice = next;
  weight.target = next;
  weight.inPan = null;
  weight.flying = true;
  weight.from = polarOf(weight.pos);
  weight.elapsed = 0;
  stage.setAttribute("aria-valuenow", String(next === null ? 0 : next + 1));
  stage.setAttribute("aria-valuetext", next === null ? "Level: the weight is on its seat and no pan is loaded" : `Weight in the ${NAMES[next]} pan, ${PLACES[next]}: the beam settles ${EQ.toFixed(1)} degrees toward it`);
  run();
}

function moveWeight(dt, calm) {
  const target = targetOf();
  if (!weight.flying) {
    weight.pos = target;
    return;
  }
  weight.elapsed += dt;
  const u = calm ? 1 : Math.min(1, weight.elapsed / FLIGHT);
  if (u >= 1) {
    weight.flying = false;
    weight.pos = target;
    weight.inPan = weight.target;
    return;
  }
  const to = polarOf(target);
  const from = weight.from;
  const turn = ((to[1] - from[1] + Math.PI * 3) % (Math.PI * 2)) - Math.PI;
  const e = smooth(u);
  const here = fromPolar([from[0] + (to[0] - from[0]) * e, from[1] + turn * e, from[2] + (to[2] - from[2]) * e]);
  here[2] += LIFT * 4 * u * (1 - u);
  weight.pos = here;
}

function nearestPan(t) {
  let best = 0;
  DIRS.forEach((d, index) => {
    if (d[0] * t[0] + d[1] * t[1] > DIRS[best][0] * t[0] + DIRS[best][1] * t[1]) best = index;
  });
  return best;
}

function words() {
  const deg = degreesOf(tilt.t);
  const name = deg >= 0.5 ? NAMES[nearestPan(tilt.t)] : null;
  const angle = `${Math.round(deg)}°`;
  if (weight.flying) {
    if (weight.target === null) return `lifting${name ? ` · ${name} · ${angle}` : ""}`;
    return `placing · ${NAMES[weight.target]}${name ? ` · ${angle}` : ""}`;
  }
  if (weight.inPan !== null) return tiltSettled ? `settles · ${NAMES[weight.inPan]} · ${angle}` : `swinging · ${name ?? NAMES[weight.inPan]} · ${angle}`;
  return tiltSettled ? "level" : `levelling${deg >= 0.5 ? ` · ${angle}` : ""}`;
}

function placeWeight() {
  weightInner.setAttribute("transform", translateAlong(P, weight.pos));
  let want;
  if (!weight.flying && weight.inPan !== null) want = ["after", panNodes[weight.inPan][panNodes[weight.inPan].length - 1]];
  else if (!weight.flying && weight.target === null) want = ["after", seatNode];
  else {
    const key = depthOf(weight.pos, P) + 0.35;
    const next = sorted.find((item) => item.k > key);
    want = next ? ["before", next.el] : ["end", null];
  }
  if (slot && slot[0] === want[0] && slot[1] === want[1]) return;
  slot = want;
  if (want[0] === "after") want[1].after(weightNode);
  else if (want[0] === "before") want[1].before(weightNode);
  else svg.append(weightNode);
}

function draw() {
  if (!drawnTilt || drawnTilt[0] !== tilt.t[0] || drawnTilt[1] !== tilt.t[1]) {
    const parts = beamParts(P, tilt.t, DIRS);
    for (const name of Object.keys(beamNodes)) beamNodes[name].innerHTML = parts[name];
    DIRS.forEach((d, index) => {
      const shift = panShift(tilt.t, d);
      const transform = translateAlong(P, shift);
      for (const el of panNodes[index]) el.setAttribute("transform", transform);
    });
    drawnTilt = [tilt.t[0], tilt.t[1]];
  }
  placeWeight();
  const text = words();
  if (readout.textContent !== text) readout.textContent = text;
}

function tick(now) {
  const dt = last ? Math.min((now - last) / 1000, 1 / 30) : 1 / 60;
  last = now;
  const calm = still.matches;
  if (touring) {
    clock += dt;
    choose(TOUR[Math.floor(clock / STEP) % TOUR.length]);
  }
  moveWeight(dt, calm);
  tiltSettled = stepTilt(tilt, weight.inPan === null ? null : DIRS[weight.inPan], dt, calm);
  draw();
  const settled = !touring && !weight.flying && tiltSettled;
  frame = visible && !settled ? requestAnimationFrame(tick) : 0;
}

function run() {
  if (!frame && visible) {
    last = 0;
    frame = requestAnimationFrame(tick);
  }
}

function steer(next) {
  touring = false;
  clearTimeout(idleTimer);
  idleTimer = setTimeout(() => {
    touring = !still.matches;
    clock = 0;
    run();
  }, 3600);
  choose(next);
  run();
}

function panUnder(event) {
  const box = svg.getBoundingClientRect();
  const vx = ((event.clientX - box.left) / box.width) * VIEW_W;
  const vy = ((event.clientY - box.top) / box.height) * VIEW_H;
  const o = iso([0, 0, PAN_Z], P);
  const ax = axisVector(P, [1, 0, 0]);
  const ay = axisVector(P, [0, 1, 0]);
  const det = ax[0] * ay[1] - ax[1] * ay[0];
  const rx = vx - o[0];
  const ry = vy - o[1];
  const wx = (rx * ay[1] - ry * ay[0]) / det;
  const wy = (ax[0] * ry - ax[1] * rx) / det;
  if (Math.hypot(wx, wy) < 18) return null;
  return nearestPan([wx, wy]);
}

new IntersectionObserver(
  (entries) => {
    visible = entries[entries.length - 1].isIntersecting;
    if (visible) run();
  },
  { rootMargin: "120px 0px" },
).observe(stage);

stage.addEventListener("pointermove", (event) => steer(panUnder(event)));
stage.addEventListener("pointerdown", (event) => steer(panUnder(event)));
stage.addEventListener("keydown", (event) => {
  const keys = { ArrowLeft: 0, ArrowDown: 1, ArrowRight: 2, ArrowUp: 3, Home: null, Escape: null, Digit0: null };
  const code = event.key === "0" ? "Digit0" : event.key;
  if (!(code in keys)) return;
  event.preventDefault();
  steer(keys[code]);
});

draw();
