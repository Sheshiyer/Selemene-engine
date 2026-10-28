// Moving parts of the witness pipeline bench. Built once at rest by build.mjs and
// rebuilt per frame in the page from the same functions (this file is inlined there).
// k is the iso kit, G the lathe module, P the projection, D the shared dimensions.
export function makeParts(k, G, P, D) {
  const S = (paths, style) => k.solidSvg(paths, style);
  const L = (d, style) => k.lineSvg(d, style);
  const at = (p) => k.iso(p, P);
  const TOP = D.beltTop;

  // The plate: a bronze blank riding the belt, built centred on x = 0 at y = 0.
  // `got` says which impressions it carries: value (engine number), route (A | P | AP), lines, page.
  function plate(engine, got) {
    const out = [];
    if (got.page) {
      const plan = { x: -5.5, y: -8, w: 11, d: 16, r: 1 };
      const h = D.plateH * 2;
      out.push(`<path class="iso-halo" d="${k.haloOf(plan, TOP, 0.1, P)}" style="filter:blur(3px)"/>`);
      out.push(S(k.slabOf(plan, TOP, h, P, 3, 0.4), { tone: "hi", crease: "lo", inner: L(k.sideSeam(plan, TOP + D.plateH, P, 3), { tone: "lo" }) }));
      out.push(L(k.lineOnTop([-3.6, 0], [2.4, 0], TOP + h, P), { tone: "mid", free: true }));
      out.push(k.dotsSvg([at([3.8, 0, TOP + h])], { size: 0.5, tone: "hi" }));
      return out.join("");
    }
    const plan = { x: -11, y: -8, w: 22, d: 16, r: 1.2 };
    const h = D.plateH;
    out.push(`<path class="iso-halo" d="${k.haloOf(plan, TOP, 0.1, P)}" style="filter:blur(3px)"/>`);
    out.push(S(k.slabOf(plan, TOP, h, P, 3, 0.4), { tone: "hi", crease: "lo" }));
    out.push(L(k.planOutline(k.insetPlan(plan, 1.4), TOP + h, P, 3), { tone: "faint" }));
    if (got.value) {
      out.push(k.faceTextSvg(k.topMatrix([-5.5, 2.6], TOP + h + 0.05, P, "x"), String(engine.index).padStart(2, "0"), { size: 8, tone: "hi", anchor: "middle" }));
    }
    if (got.route) {
      const y = D.routeY[engine.route];
      out.push(L(k.ring(6.5, y, 1.2, TOP + h, P, 14), { tone: "hi" }));
      out.push(k.dotsSvg([at([6.5, y, TOP + h])], { size: 0.45, tone: "lo" }));
    }
    if (got.lines) {
      for (const y of [D.penY.A, D.penY.P]) out.push(L(k.lineOnTop([-8.6, y], [3.4, y], TOP + h, P), { tone: "mid", free: true }));
    }
    return out.join("");
  }

  // Crank on the right drum's near shaft end: boss, arm and knurled handle, turned by theta.
  function crank(theta) {
    const F = G.frameOf([D.drumX, 0, D.drumZ], [0, 1, 0], [1, 0, 0], [0, 0, 1]);
    const c = Math.cos(theta);
    const s = Math.sin(theta);
    const rot = ([t, w]) => [c * t - s * w, s * t + c * w];
    const arm = [[-2.4, -1.6], [10.4, -1.6], [10.4, 1.6], [-2.4, 1.6]].map(rot);
    const tip = G.pointOf(F, D.crankS + 2, 10.4, theta);
    const HF = G.frameOf(tip, [0, 1, 0], [1, 0, 0], [0, 0, 1]);
    const out = [];
    out.push(S(G.prismOf(F, arm, D.crankS, D.crankS + 2, P), { tone: "mid" }));
    out.push(S(G.disc(0, 7.4, 1.3, HF, P, { steps: 18 }), { tone: "mid", crease: "none" }));
    out.push(k.fadedSvg(G.ribsOf(HF, 0.8, 6.6, 1.3, 14, P, { fade: [0.1, 0.55], seams: true }), { tone: "lo" }));
    out.push(S(G.disc(7.4, 8.4, 1.8, HF, P, { steps: 18 }), { tone: "mid" }));
    out.push(k.dotsSvg([at(G.pointOf(HF, 8.4, 0, 0))], { size: 0.4, tone: "hi" }));
    return out.join("");
  }

  // Four spokes on the left drum's near end face, turning with the belt.
  function drumSpokes(theta) {
    const F = G.frameOf([-D.drumX, 0, D.drumZ], [0, 1, 0], [1, 0, 0], [0, 0, 1]);
    return L(G.spokesOf(F, D.drumS, 2.4, 6.6, 4, P, { phase: theta }), { tone: "lo" });
  }

  // Three spokes on the die press handwheel, turning as the screw drives the ram.
  function wheelSpokes(psi) {
    const [x, y, z] = D.wheel;
    const d = [0, 1, 2].map((i) => {
      const a = psi + (i * Math.PI * 2) / 3;
      return k.lineOnTop([x + Math.cos(a) * 2.2, y + Math.sin(a) * 2.2], [x + Math.cos(a) * 7.6, y + Math.sin(a) * 7.6], z, P);
    });
    return L(d.join(""), { tone: "mid" });
  }

  // The gold lever on the switch gate, standing on its pivot and tilted to phi
  // (0 is upright, +phi leans toward +y, the Aletheios side; -phi toward Pichet).
  function lever(phi) {
    const [x, y, z] = D.lever;
    const F = G.frameOf([0, y, z], [1, 0, 0], [0, 1, 0], [0, 0, 1]);
    const c = Math.cos(phi);
    const s = Math.sin(phi);
    const rot = ([t, w]) => [s * t + c * w, c * t - s * w]; // (along, across) → (u = y, v = z)
    const arm = [[-2.4, -1.2], [14, -1.2], [14, 1.2], [-2.4, 1.2]].map(rot);
    const tip = rot([13.6, 0]);
    const HF = G.frameOf([x - 2.2, y + tip[0], z + tip[1]], [1, 0, 0], [0, 1, 0], [0, 0, 1]);
    const out = [];
    out.push(S(G.prismOf(F, arm, x - 1, x + 1, P), { tone: "hi", lit: true }));
    out.push(S(G.disc(0, 4.4, 1.8, HF, P, { steps: 18 }), { tone: "hi", lit: true }));
    out.push(k.fadedSvg(G.ribsOf(HF, 0.6, 3.8, 1.8, 14, P, { fade: [0.1, 0.55], seams: true }), { tone: "mid" }));
    return out.join("");
  }

  // The folio press guide rods, split at the platen so the platen hides the right part.
  function rods(platenZ) {
    const [x0, z0, z1] = [D.folioX, D.rodZ0, D.rodZ1];
    const lower = [];
    const upper = [];
    for (const dx of [-8, 8]) {
      lower.push(S(k.cylinder(x0 + dx, 0, 1.4, z0, Math.max(0.2, platenZ - z0), P, 16), { tone: "mid", crease: "none" }));
      upper.push(S(k.cylinder(x0 + dx, 0, 1.4, platenZ + D.platenH, Math.max(0.2, z1 - platenZ - D.platenH), P, 16), { tone: "mid", crease: "none" }));
    }
    return { lower: lower.join(""), upper: upper.join("") };
  }

  return { plate, crank, drumSpokes, wheelSpokes, lever, rods };
}
