// ─── DepthScene — depth gallery for the integrated reading ────────────
// Port of codrops/atmospheric-depth-gallery (Engine + Gallery + Scroll +
// Background) consolidated into one module that a React Client Component
// can mount into a canvas.
//
// Each "section" of the reading becomes a plane positioned in 3D space.
// The reader scrolls vertically; scroll velocity drives Z-translation of
// the camera through the planes + an atmospheric background tint that
// morphs between adjacent planes' palettes.
//
// What we kept from codrops:
//   • Z-axis depth scrolling (smoothed)
//   • Per-plane parallax tilt on pointer move
//   • Plane breath (subtle scale/tilt)
//   • Background color morph between adjacent planes
//
// What we simplified / removed (can re-add later):
//   • Trail particles + TrailController
//   • Tweakpane debug panel
//   • Texture preloading (Phase 1 = colored planes only; textures next)
//   • Custom GLSL shaders for background (start with vertex-color planes)
//
// Click handlers: hit-test on raycaster; emits onPlaneClick(sectionId).

import * as THREE from "three";
import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader.js";
import { RoomEnvironment } from "three/examples/jsm/environments/RoomEnvironment.js";
import type { SectionData } from "./data/sections";
import { DepthTrail } from "./DepthTrail";
import { DepthBubbles } from "./DepthBubbles";

export interface DepthSceneOptions {
  canvas: HTMLCanvasElement;
  sections: SectionData[];
  onPlaneClick?: (sectionId: string) => void;
  onActivePlaneChange?: (sectionId: string) => void;
}

export class DepthScene {
  private canvas: HTMLCanvasElement;
  private sections: SectionData[];
  private renderer: THREE.WebGLRenderer;
  private scene: THREE.Scene;
  private camera: THREE.PerspectiveCamera;
  private clock: THREE.Clock;
  private rafId: number | null = null;
  private isRunning = false;
  private isDisposed = false;

  // Planes
  private planes: THREE.Mesh[] = [];
  private planeGroup: THREE.Group;
  private raycaster = new THREE.Raycaster();
  private pointerNdc = new THREE.Vector2(0, 0);

  // Scroll state
  private scrollY = 0; // virtual scroll position in arbitrary units
  private scrollVelocity = 0;
  private cameraZTarget = 6;
  private cameraZCurrent = 6;

  // ─── Intro animation state ───────────────────────────────────────────
  // playIntro() is called from React when the loading shader starts to
  // fade out. The plane group eases from a small/distant pose to its
  // natural pose over `introDuration` seconds, so the cards appear to
  // float into existence behind the dissolving shader.
  private introPlaying = false;
  private introStartTime = 0;
  private introDuration = 1.4;
  private planeGap = 6; // distance in Z between consecutive planes

  // Parallax / breath
  private pointerTarget = new THREE.Vector2(0, 0);
  private pointerCurrent = new THREE.Vector2(0, 0);
  // Lowered for less-aggressive zoom + hover response per user feedback —
  // the landmarks should feel like compact intimate objects, not over-
  // scaled "lean-in" hero pieces.
  private parallaxAmount = 0.08;
  private parallaxSmoothing = 0.08;
  private breathAmplitude = 0.025;

  // Background tint morph
  private backgroundCurrent = new THREE.Color("#070B1D");
  private backgroundTarget = new THREE.Color("#070B1D");

  // Active plane tracking
  private activeIndex = 0;

  // ─── Texture loader + 4-view rotation state ────────────────────────────
  // Each plane attempts to load up to 4 view PNGs (front/left/back/right).
  // When the active plane is detected, its material.map is swapped between
  // the 4 textures based on pointer.x (rotating the user "around" the
  // landmark). Missing views fall back to "front" silently.
  private textureLoader = new THREE.TextureLoader();
  // Tracks the last "view" key applied per plane index to avoid setting
  // material.needsUpdate every frame for the same texture.
  private lastViewKey: Array<"front" | "left" | "back" | "right" | null> = [];

  // ─── Trail + bubbles + wrap-around loop ───────────────────────────────
  private trail: DepthTrail;
  private bubbles: DepthBubbles;
  private trailHead = new THREE.Vector3();
  // Trail path parameters (parametric waving through the planes)
  private trailHorizontalCycles = 1.85;
  private trailHorizontalWidth = 2.4;
  private trailVerticalCycles = 2.1;
  private trailVerticalAmplitude = 0.65;
  private trailDepthAhead = 1.0; // how far ahead of camera the trail head sits
  // Wrap-loop state — when reader scrolls past the last plane, scrollY wraps
  private isWrapping = false;
  private wrapStartTime = 0;
  private wrapDuration = 0.55; // seconds for the trail to fade-during-wrap

  // Callbacks
  private onPlaneClick?: (sectionId: string) => void;
  private onActivePlaneChange?: (sectionId: string) => void;

  // Bound listeners (so dispose can detach)
  private boundResize = () => this.resize();
  private boundWheel = (e: WheelEvent) => this.handleWheel(e);
  private boundTouchStart = (e: TouchEvent) => this.handleTouchStart(e);
  private boundTouchMove = (e: TouchEvent) => this.handleTouchMove(e);
  private boundPointerMove = (e: PointerEvent) => this.handlePointerMove(e);
  private boundClick = (e: MouseEvent) => this.handleClick(e);

  private touchStartY = 0;

  constructor(opts: DepthSceneOptions) {
    this.canvas = opts.canvas;
    this.sections = opts.sections;
    this.onPlaneClick = opts.onPlaneClick;
    this.onActivePlaneChange = opts.onActivePlaneChange;

    this.scene = new THREE.Scene();
    this.scene.background = this.backgroundCurrent;
    this.scene.fog = new THREE.Fog(this.backgroundCurrent, 8, 35);

    this.camera = new THREE.PerspectiveCamera(45, 1, 0.1, 100);
    this.camera.position.set(0, 0, this.cameraZTarget);

    this.renderer = new THREE.WebGLRenderer({
      canvas: this.canvas,
      antialias: true,
      alpha: false,
    });
    this.renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 2));
    this.renderer.outputColorSpace = THREE.SRGBColorSpace;
    // ─── Tone mapping — without this, PBR materials in a black void
    //     crush to flat-dark. ACES Filmic + slight overexposure pulls
    //     midtones out so bronze/gold actually reads as metal. ─────────
    this.renderer.toneMapping = THREE.ACESFilmicToneMapping;
    this.renderer.toneMappingExposure = 1.25;

    this.planeGroup = new THREE.Group();
    this.scene.add(this.planeGroup);

    // Trail (the wavy thread) + Bubbles (sparkle pool around its head)
    this.trail = new DepthTrail({
      color: "#F0EDE3",
      glowColor: "#C5A017",
      glowIntensity: 1.4,
      baseOpacity: 0.55,
      maxPoints: 220,
      radiusHead: 0.014,
      radiusTail: 0.003,
    });
    this.bubbles = new DepthBubbles({
      color: "#F0EDE3",
      maxParticles: 18,
      spawnPerSecond: 22,
      spawnRadius: 0.48,
    });
    this.scene.add(this.trail.group);
    this.scene.add(this.bubbles.group);
    // ─── 3-point lighting tuned for PBR meshes in deep void ───────────
    // Without a real key + rim, the GLBs read as muddy unlit blobs
    // because the background and the material are both near-black.
    // Key: warm directional from front-top-right (the "sun").
    // Rim: cool back-light from behind so the silhouette glows against
    //      the void instead of disappearing into it.
    // Fill: weak under-light + ambient bloom to keep shadows readable.
    this.scene.add(new THREE.AmbientLight(0xfff8e7, 0.55));
    const keyLight = new THREE.DirectionalLight(0xffeac4, 1.35);
    keyLight.position.set(3, 4, 5);
    this.scene.add(keyLight);
    const rimLight = new THREE.DirectionalLight(0x8aa6ff, 0.85);
    rimLight.position.set(-3, 1, -4);
    this.scene.add(rimLight);
    const fillLight = new THREE.DirectionalLight(0xd9d2ff, 0.35);
    fillLight.position.set(0, -3, 3);
    this.scene.add(fillLight);

    // ─── Environment map — PBR materials need *something* to reflect,
    //     otherwise metalness > 0 looks like dead matte plastic. The
    //     PMREMGenerator + RoomEnvironment combo from three's examples
    //     gives us a soft procedural studio without any HDR file. ─────
    {
      const pmrem = new THREE.PMREMGenerator(this.renderer);
      const envScene = new RoomEnvironment();
      this.scene.environment = pmrem.fromScene(envScene, 0.04).texture;
      pmrem.dispose();
    }

    this.clock = new THREE.Clock();

    this.buildPlanes();
    this.seedTrail();
    this.resize();
    this.bindEvents();
  }

  /** Pre-populate the trail with a few points so the first frame isn't empty. */
  private seedTrail() {
    for (let i = 10; i >= 0; i--) {
      const seedZ = this.cameraZTarget - i * 0.18;
      this.trail.addPoint(this.computeTrailHead(0, seedZ));
    }
  }

  /** Parametric trail head — waves in X + Y, sits a constant distance
   *  ahead of the camera in Z. As progress grows the wave phase advances
   *  so the trail "draws" forward through the planes. */
  private computeTrailHead(progress: number, cameraZ: number): THREE.Vector3 {
    const phase = progress * Math.PI * 2;
    const x = Math.sin(phase * this.trailHorizontalCycles) * this.trailHorizontalWidth * 0.45;
    const y = Math.sin(phase * this.trailVerticalCycles) * this.trailVerticalAmplitude;
    // Trail head sits slightly ahead of the camera so we see it leading
    const z = cameraZ - this.trailDepthAhead - progress * 1.2;
    return this.trailHead.set(x, y, z);
  }

  /** Build the 15 planes from section data — colored, no textures yet. */
  private buildPlanes() {
    // Square geometry matches the 1024×1024 PNG silhouettes exactly so
    // the void-black surrounding each rendered subject blends pixel-
    // perfect with the scene background. Sized down from 3.4 to 2.2
    // per user feedback — the previous size felt over-zoomed and
    // crowded the canvas; this leaves room for interaction.
    const geo = new THREE.PlaneGeometry(2.2, 2.2, 1, 1);
    this.sections.forEach((section, i) => {
      // Start the plane FULLY TRANSPARENT (alpha 0) — invisible until
      // the PNG texture lands. No colored "box" backdrop is shown at
      // any point. The PNG itself is the visual; we never want to see
      // the underlying plane material color.
      const mat = new THREE.MeshBasicMaterial({
        color: 0xffffff,
        transparent: true,
        opacity: 0, // hidden until PNG texture promotes
        side: THREE.DoubleSide,
        depthWrite: false, // prevents transparent planes from occluding each other
      });
      const mesh = new THREE.Mesh(geo, mat);
      mesh.position.set(
        section.position.x * 2.5,
        section.position.y,
        -i * this.planeGap,
      );
      mesh.userData.sectionId = section.id;
      mesh.userData.sectionIndex = i;
      mesh.userData.baseColor = new THREE.Color(section.accentColor);
      mesh.userData.backgroundColor = new THREE.Color(section.backgroundColor);

      this.planeGroup.add(mesh);
      this.planes.push(mesh);
      this.lastViewKey.push(null);

      // ─── Load the section's PNG view textures. The plane stays
      //     invisible (opacity 0) until -front.png lands, at which
      //     point the material is promoted to textured mode.
      this.loadViewTexturesFor(section.id, mesh);

      // ─── Legacy: Meshy GLB pipeline. The code path is preserved
      //     (`loadMeshFor`) but no longer invoked — we're 100% on PNG
      //     textures now. If you want GLBs back, restore the
      //     `if (section.meshPath) this.loadMeshFor(...)` call here.
    });
  }

  /** Attempt to load up to 4 view textures for a section. Stores the
   *  loaded set on the plane's `userData.viewTextures`. Promotes the
   *  material to textured mode when "front" lands. Silently ignores
   *  views that 404. */
  private loadViewTexturesFor(sectionId: string, mesh: THREE.Mesh) {
    const views: Array<"front" | "left" | "back" | "right"> = [
      "front",
      "left",
      "back",
      "right",
    ];
    mesh.userData.viewTextures = {} as Record<string, THREE.Texture>;
    for (const v of views) {
      const url = `/depth-reading/images/${sectionId}-${v}.png`;
      this.textureLoader.load(
        url,
        (tex) => {
          tex.colorSpace = THREE.SRGBColorSpace;
          tex.anisotropy = 8;
          (mesh.userData.viewTextures as Record<string, THREE.Texture>)[v] = tex;
          // Promote to textured mode when front lands. Keep transparent
          // = true so the per-frame focus-fade in tick() can modulate
          // opacity (only the closest-to-active plane stays fully lit).
          if (v === "front") {
            const mat = mesh.material as THREE.MeshBasicMaterial;
            mat.map = tex;
            mat.color.set(0xffffff); // clear accent tint — show PNG color as-is
            mat.transparent = true; // required for the focus-fade
            mat.needsUpdate = true;
          }
        },
        undefined,
        () => {
          // 404 or load failure — silently skip; pointer rotation will
          // fall back to "front" for any missing views.
        },
      );
    }
  }

  /** Per-frame Y-rotation registry — GLB meshes added here auto-rotate
   *  slowly so the viewer perceives them as 3D, not flat posters. */
  private rotatingMeshes: Array<{ obj: THREE.Object3D; speed: number }> = [];

  /** Async load a section's GLB and attach it to its plane. Apply per-
   *  section meshTransform overrides (scale, rotation, position). On
   *  successful load the colored plane + rim plane are HIDDEN so the
   *  GLB stands alone — no card frame around it. */
  private async loadMeshFor(section: SectionData, planeMesh: THREE.Mesh) {
    if (!section.meshPath) return;
    const loader = new GLTFLoader();
    try {
      const gltf = await loader.loadAsync(section.meshPath);
      const root = gltf.scene;
      // Apply per-section transform overrides
      const t = section.meshTransform;
      if (t?.scale) root.scale.setScalar(t.scale);
      if (t?.rotation) {
        root.rotation.set(
          t.rotation.x ?? 0,
          t.rotation.y ?? 0,
          t.rotation.z ?? 0,
        );
      }
      if (t?.position) {
        root.position.set(
          t.position.x ?? 0,
          t.position.y ?? 0,
          t.position.z ?? 0,
        );
      }
      // ─── Override the GLB's baked Hunyuan textures with palette-driven
      //     polished-metal materials. Hunyuan bakes albedo from the
      //     ChatGPT-generated reference image, which lands as muddy bronze
      //     in the void. We want each section to read in its cardinal
      //     color (gold / violet / indigo / emerald) as if cast in
      //     polished bronze. Keep normal maps so surface detail survives.
      const accentColor = (planeMesh.userData.baseColor as THREE.Color) ?? new THREE.Color("#C5A017");
      root.traverse((node) => {
        if ((node as THREE.Mesh).isMesh) {
          const m = node as THREE.Mesh;
          m.castShadow = false;
          m.receiveShadow = false;
          m.userData.sectionId = section.id;
          // Preserve the original normal map (surface micro-detail) if present
          const old = m.material as THREE.MeshStandardMaterial;
          const normalMap = old?.normalMap ?? null;
          const newMat = new THREE.MeshStandardMaterial({
            color: accentColor,
            metalness: 0.85,
            roughness: 0.28,
            normalMap,
            // envMap inherits from scene.environment automatically when null
            envMapIntensity: 1.15,
          });
          m.material = newMat;
          // Old material's textures can be GC'd
          if (old && old !== newMat) {
            old.dispose?.();
          }
        }
      });
      // Attach to the plane: GLB rides the plane's transform + breath
      planeMesh.add(root);

      // ─── Keep a FAINT colored backdrop behind the mesh so the eye has
      //     something to anchor against in the void. Setting to 0 made
      //     the mesh float in nothing and read as a hovering sticker.
      //     0.08 gives a barely-there color halo. ─────────────────────
      const planeMat = planeMesh.material as THREE.MeshBasicMaterial;
      planeMat.opacity = 0.08;
      planeMat.transparent = true;
      planeMat.needsUpdate = true;
      // The rim plane was added as a child during buildPlanes(). Now
      // that the GLB is here, hide it too.
      planeMesh.children.forEach((child) => {
        if (child === root) return; // don't hide the GLB itself
        if (child instanceof THREE.Mesh) {
          child.visible = false;
        }
      });

      // ─── Register the GLB for slow auto-rotation so the eye reads it
      //     as 3D, not a flat poster. Subtle — 0.15 rad/s on Y axis. ───
      this.rotatingMeshes.push({ obj: root, speed: 0.15 });
    } catch (err) {
      // Silent fallback — colored plane stays visible. Most common cause
      // is GLB not yet uploaded; we'll see this for any meshPath that
      // 404s, and the user just sees the colored plane.
      // eslint-disable-next-line no-console
      console.warn(`[depth-reading] mesh load failed for ${section.id}:`, err);
    }
  }

  start() {
    if (this.isRunning || this.isDisposed) return;
    this.isRunning = true;
    this.clock.start();
    // Pre-position the plane group at the intro "from" pose so the very
    // first paint (before the loader fades) is already coherent: small
    // + pushed back. playIntro() will animate it from here to identity.
    this.planeGroup.scale.setScalar(0.55);
    this.planeGroup.position.z = -6;
    this.tick();
  }

  /** Begin the float-in animation. Typically called by the React parent
   *  the moment the loading shader starts to fade out. */
  playIntro() {
    if (!this.isRunning) return;
    this.introPlaying = true;
    this.introStartTime = this.clock.elapsedTime;
  }

  stop() {
    this.isRunning = false;
    if (this.rafId !== null) {
      cancelAnimationFrame(this.rafId);
      this.rafId = null;
    }
  }

  dispose() {
    this.isDisposed = true;
    this.stop();
    window.removeEventListener("resize", this.boundResize);
    this.canvas.removeEventListener("wheel", this.boundWheel);
    this.canvas.removeEventListener("touchstart", this.boundTouchStart);
    this.canvas.removeEventListener("touchmove", this.boundTouchMove);
    this.canvas.removeEventListener("pointermove", this.boundPointerMove);
    this.canvas.removeEventListener("click", this.boundClick);
    this.planes.forEach((mesh) => {
      mesh.geometry.dispose();
      const mat = mesh.material as THREE.Material;
      mat.dispose();
      mesh.children.forEach((child) => {
        if (child instanceof THREE.Mesh) {
          child.geometry.dispose();
          (child.material as THREE.Material).dispose();
        }
      });
    });
    this.trail.dispose();
    this.bubbles.dispose();
    this.renderer.dispose();
  }

  private bindEvents() {
    window.addEventListener("resize", this.boundResize);
    this.canvas.addEventListener("wheel", this.boundWheel, { passive: false });
    this.canvas.addEventListener("touchstart", this.boundTouchStart, { passive: true });
    this.canvas.addEventListener("touchmove", this.boundTouchMove, { passive: false });
    this.canvas.addEventListener("pointermove", this.boundPointerMove);
    this.canvas.addEventListener("click", this.boundClick);
  }

  private handleWheel(e: WheelEvent) {
    e.preventDefault();
    this.scrollVelocity += e.deltaY * 0.0008;
  }

  private handleTouchStart(e: TouchEvent) {
    this.touchStartY = e.touches[0]?.clientY ?? 0;
  }

  private handleTouchMove(e: TouchEvent) {
    e.preventDefault();
    const y = e.touches[0]?.clientY ?? 0;
    const dy = this.touchStartY - y;
    this.scrollVelocity += dy * 0.001;
    this.touchStartY = y;
  }

  private handlePointerMove(e: PointerEvent) {
    const rect = this.canvas.getBoundingClientRect();
    const x = ((e.clientX - rect.left) / rect.width) * 2 - 1;
    const y = -(((e.clientY - rect.top) / rect.height) * 2 - 1);
    this.pointerNdc.set(x, y);
    this.pointerTarget.set(x, y);
  }

  private handleClick(e: MouseEvent) {
    if (!this.onPlaneClick) return;
    const rect = this.canvas.getBoundingClientRect();
    const x = ((e.clientX - rect.left) / rect.width) * 2 - 1;
    const y = -(((e.clientY - rect.top) / rect.height) * 2 - 1);
    this.raycaster.setFromCamera(new THREE.Vector2(x, y), this.camera);
    const intersects = this.raycaster.intersectObjects(this.planes, false);
    if (intersects.length > 0) {
      const hit = intersects[0].object as THREE.Mesh;
      const sectionId = hit.userData.sectionId as string | undefined;
      if (sectionId) this.onPlaneClick(sectionId);
    }
  }

  private resize() {
    const w = this.canvas.clientWidth || window.innerWidth || 1;
    const h = this.canvas.clientHeight || window.innerHeight || 1;
    this.camera.aspect = w / h;
    this.camera.updateProjectionMatrix();
    this.renderer.setSize(w, h, false);
  }

  private tick() {
    if (!this.isRunning) return;
    this.rafId = requestAnimationFrame(() => this.tick());
    // Capture both delta and elapsed once at the top of each frame
    const delta = this.clock.getDelta();
    const elapsed = this.clock.elapsedTime;

    // Decay scroll velocity (friction)
    this.scrollVelocity *= 0.94;
    this.scrollY += this.scrollVelocity;

    // ─── TRUE CIRCULAR LOOP (infinite carousel) ──────────────────────
    // scrollY is UNBOUNDED forward + backward. The camera moves forever
    // in the direction of scroll. Planes recycle around the camera
    // modulo loopLength so the viewer never sees a teleport — when a
    // plane passes behind the camera it silently relocates to the front
    // of the chain (out of view, behind you), then comes back into
    // view as the camera continues forward.
    const loopLength = this.sections.length * this.planeGap;
    // Camera target = unbounded forward motion (no wrap on scrollY)
    this.cameraZTarget = 6 - this.scrollY;
    this.cameraZCurrent += (this.cameraZTarget - this.cameraZCurrent) * 0.12;
    this.camera.position.z = this.cameraZCurrent;

    // Recycle each plane: keep its apparent Z within ±loopLength/2 of
    // the camera. If a plane is more than half the loop behind, push it
    // forward (= ahead in the loop). If more than half ahead, pull it
    // back. Both checks run independently each frame so reverse-scroll
    // wraps cleanly too.
    const halfLoop = loopLength / 2;
    this.planes.forEach((plane, i) => {
      const baseZ = -i * this.planeGap;
      let z = baseZ;
      // Shift plane by integer multiples of loopLength so it lands in
      // the "active band" (cameraZ - halfLoop, cameraZ + halfLoop)
      const offset = z - this.cameraZCurrent;
      if (offset > halfLoop) {
        z -= loopLength * Math.ceil((offset - halfLoop) / loopLength);
      } else if (offset < -halfLoop) {
        z += loopLength * Math.ceil((-offset - halfLoop) / loopLength);
      }
      plane.position.z = z;
    });

    // ─── Intro float-in ────────────────────────────────────────────────
    // Eases planeGroup.scale and planeGroup.position.z from a small/far
    // pose to identity. Runs once after playIntro() is called by the
    // React parent (i.e. when the loading shader starts to fade out).
    if (this.introPlaying) {
      const tNorm = Math.min(1, (elapsed - this.introStartTime) / this.introDuration);
      // Cubic ease-out — fast settle, soft landing
      const ease = 1 - Math.pow(1 - tNorm, 3);
      const scale = 0.55 + (1 - 0.55) * ease;
      this.planeGroup.scale.setScalar(scale);
      this.planeGroup.position.z = -6 + 6 * ease;
      if (tNorm >= 1) {
        this.introPlaying = false;
        this.planeGroup.scale.setScalar(1);
        this.planeGroup.position.z = 0;
      }
    }

    // Pointer parallax — group tilts slightly toward pointer
    this.pointerCurrent.lerp(this.pointerTarget, this.parallaxSmoothing);
    // Halved rotation multipliers per user feedback — hover tilt was
    // too aggressive. Group rotates by ~0.04 rad max instead of 0.08.
    this.planeGroup.rotation.y = this.pointerCurrent.x * 0.04;
    this.planeGroup.rotation.x = -this.pointerCurrent.y * 0.025;
    // Subtle group x-shift for parallax (additive to intro position.z)
    this.planeGroup.position.x = -this.pointerCurrent.x * this.parallaxAmount * 0.3;

    // Breath — every plane subtly scales 4:7:8 cycle (19s period)
    const BREATH_PERIOD = 19;
    const breathPhase =
      (elapsed % BREATH_PERIOD) / BREATH_PERIOD; // 0..1
    const breathScale =
      1 + Math.sin(breathPhase * Math.PI * 2) * this.breathAmplitude;
    this.planes.forEach((p) => {
      p.scale.setScalar(breathScale);
    });

    // ─── GLB auto-rotation ──────────────────────────────────────────────
    // Loaded meshes registered via loadMeshFor() spin slowly on Y so the
    // viewer reads them as 3D objects rather than flat textured cards.
    for (const r of this.rotatingMeshes) {
      r.obj.rotation.y += r.speed * delta;
    }

    // ─── Active plane detection (race-condition-safe) ──────────────────
    // Camera looks in -Z direction. A plane is "ahead" of the camera
    // when planeZ < cameraZ → ahead = cameraZ - planeZ > 0.
    // The "active marker" sits 1 unit ahead of the camera. We want the
    // plane whose ahead-distance is closest to 1 — BUT we EXCLUDE any
    // plane that's behind the camera (ahead < -0.2 grace zone) because
    // such a plane is invisible and shouldn't carry the active label.
    //
    // ALSO opacity-fade non-active planes: focusDist drives opacity so
    // only the closest-to-active plane is visually dominant. This
    // eliminates the race where the label says one section but the
    // canvas visually shows an adjacent section's plane.
    let closestIdx = this.activeIndex; // start from current to provide hysteresis
    let closestDist = Infinity;
    this.planes.forEach((p, i) => {
      const ahead = this.cameraZCurrent - p.position.z;
      // ── Visual fade: planes far from the active marker fade out. ──
      const focusDist = Math.abs(ahead - 1);
      // Smooth fall-off: full opacity at focusDist=0, zero at focusDist=4
      const baseOpacity = Math.max(0, 1 - focusDist / 4);
      // CRITICAL: only show planes that have actually loaded their PNG.
      // Without this guard, an untextured plane would lerp toward
      // baseOpacity and become a glaring solid-color rectangle.
      const hasTexture = !!(p.userData.viewTextures as
        | Record<string, THREE.Texture | undefined>
        | undefined)?.front;
      const targetOpacity = hasTexture ? baseOpacity : 0;
      const mat = p.material as THREE.MeshBasicMaterial;
      // Lerp toward target to soften per-frame jitter
      mat.opacity = mat.opacity + (targetOpacity - mat.opacity) * 0.18;

      // ── Active selection: skip planes that are behind camera ──
      if (ahead < -0.2) return; // plane is behind, can't be active
      if (focusDist < closestDist) {
        closestDist = focusDist;
        closestIdx = i;
      }
    });
    if (closestIdx !== this.activeIndex) {
      this.activeIndex = closestIdx;
      const sectionId = this.sections[closestIdx]?.id;
      if (sectionId && this.onActivePlaneChange) {
        this.onActivePlaneChange(sectionId);
      }
    }

    // Background morph — lerp from current to active plane's background
    const activePlane = this.planes[this.activeIndex];
    if (activePlane) {
      this.backgroundTarget = activePlane.userData.backgroundColor as THREE.Color;
    }
    this.backgroundCurrent.lerp(this.backgroundTarget, 0.04);
    (this.scene.background as THREE.Color).copy(this.backgroundCurrent);
    if (this.scene.fog) {
      (this.scene.fog as THREE.Fog).color.copy(this.backgroundCurrent);
    }

    // ─── 4-view rotation swap on the active plane ──────────────────────
    // Pointer position dictates which face of the active landmark is
    // shown. We "rotate around" the landmark with the mouse:
    //   pointer.x < -0.5      → "right" view (looking from the left)
    //   pointer.x in [-0.5, 0.5] → "front"
    //   pointer.x > 0.5       → "left" view (looking from the right)
    //   pointer.y < -0.6      → "back" view (looking from above-behind)
    // Non-active planes always show their front. Missing views fall back
    // silently to front. Texture swap only fires on key change to avoid
    // per-frame uniform churn.
    if (activePlane && activePlane.userData.viewTextures) {
      const views = activePlane.userData.viewTextures as Record<
        "front" | "left" | "back" | "right",
        THREE.Texture | undefined
      >;
      let target: "front" | "left" | "back" | "right" = "front";
      if (this.pointerCurrent.y < -0.6) {
        target = "back";
      } else if (this.pointerCurrent.x < -0.5) {
        target = "right";
      } else if (this.pointerCurrent.x > 0.5) {
        target = "left";
      }
      // Fall back to front if the chosen view is missing
      const tex = views[target] ?? views.front;
      if (tex && this.lastViewKey[this.activeIndex] !== target) {
        const mat = activePlane.material as THREE.MeshBasicMaterial;
        mat.map = tex;
        mat.needsUpdate = true;
        this.lastViewKey[this.activeIndex] = target;
      }
    }
    // Any plane that lost active status returns to "front" on its next
    // active turn — reset its lastViewKey lazily here.
    this.planes.forEach((plane, idx) => {
      if (idx === this.activeIndex) return;
      const views = plane.userData.viewTextures as
        | Record<string, THREE.Texture | undefined>
        | undefined;
      if (!views?.front) return;
      const mat = plane.material as THREE.MeshBasicMaterial;
      if (mat.map !== views.front) {
        mat.map = views.front;
        mat.needsUpdate = true;
        this.lastViewKey[idx] = "front";
      }
    });

    // ─── Trail + bubbles (continuous, no wrap fade needed) ─────────────
    // Camera moves forever forward; trail accumulates points behind it
    // and gets trimmed naturally by maxPoints. Wave phase cycles 0→1
    // per lap via scrollY % loopLength.
    const lapPosition = ((this.scrollY % loopLength) + loopLength) % loopLength;
    const progress = lapPosition / loopLength;
    const trailOpacity = 0.55;
    this.trail.material.opacity = trailOpacity;

    const head = this.computeTrailHead(progress, this.cameraZCurrent);
    this.trail.addPoint(head);
    this.bubbles.update(delta, head, trailOpacity, true);

    // Render
    this.renderer.render(this.scene, this.camera);
  }
}
