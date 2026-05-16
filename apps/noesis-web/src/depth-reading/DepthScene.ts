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
  private planeGap = 6; // distance in Z between consecutive planes

  // Parallax / breath
  private pointerTarget = new THREE.Vector2(0, 0);
  private pointerCurrent = new THREE.Vector2(0, 0);
  private parallaxAmount = 0.18;
  private parallaxSmoothing = 0.08;
  private breathAmplitude = 0.04;

  // Background tint morph
  private backgroundCurrent = new THREE.Color("#070B1D");
  private backgroundTarget = new THREE.Color("#070B1D");

  // Active plane tracking
  private activeIndex = 0;

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
    // Ambient + directional light so the trail's MeshStandardMaterial picks up rim shading
    this.scene.add(new THREE.AmbientLight(0xffffff, 0.45));
    const keyLight = new THREE.DirectionalLight(0xfff5e0, 0.55);
    keyLight.position.set(2, 3, 5);
    this.scene.add(keyLight);

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
    const geo = new THREE.PlaneGeometry(2.6, 3.4, 1, 1);
    this.sections.forEach((section, i) => {
      const mat = new THREE.MeshBasicMaterial({
        color: new THREE.Color(section.accentColor),
        transparent: true,
        opacity: 0.92,
        side: THREE.DoubleSide,
      });
      // Add a soft inner gradient via vertex colors? For MVP keep flat.
      const mesh = new THREE.Mesh(geo, mat);
      // Position: x from data, y centered, z derived from index
      mesh.position.set(
        section.position.x * 2.5,
        section.position.y,
        -i * this.planeGap,
      );
      mesh.userData.sectionId = section.id;
      mesh.userData.sectionIndex = i;
      mesh.userData.baseColor = new THREE.Color(section.accentColor);
      mesh.userData.backgroundColor = new THREE.Color(section.backgroundColor);

      // A subtle rim plane behind each — slightly larger, lower opacity,
      // tinted with blob1Color. Gives depth without textures.
      const rimMat = new THREE.MeshBasicMaterial({
        color: new THREE.Color(section.blob1Color),
        transparent: true,
        opacity: 0.18,
        side: THREE.DoubleSide,
      });
      const rim = new THREE.Mesh(
        new THREE.PlaneGeometry(3.4, 4.2, 1, 1),
        rimMat,
      );
      rim.position.set(0, 0, -0.05);
      mesh.add(rim);

      this.planeGroup.add(mesh);
      this.planes.push(mesh);

      // ─── Lazy-load Meshy GLB if section has one. The colored plane is
      //     the SSR-safe placeholder; when the GLB lands we add it as a
      //     child of the plane (inheriting its position + section userData).
      //     The plane stays mounted as a fallback in case the GLB fails. ──
      if (section.meshPath) {
        this.loadMeshFor(section, mesh);
      }
    });
  }

  /** Async load a section's GLB and attach it to its plane. Apply per-
   *  section meshTransform overrides (scale, rotation, position). */
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
      // Make sure the GLB casts no shadow + accepts the scene's lights
      root.traverse((node) => {
        if ((node as THREE.Mesh).isMesh) {
          const m = node as THREE.Mesh;
          m.castShadow = false;
          m.receiveShadow = false;
          m.userData.sectionId = section.id;
        }
      });
      // Attach to the plane: GLB rides the plane's transform + breath
      planeMesh.add(root);
      // Once the GLB is in, fade the colored plane to lower opacity so
      // it acts as a back-shadow / atmosphere rather than competing
      const planeMat = planeMesh.material as THREE.MeshBasicMaterial;
      planeMat.opacity = 0.18;
      planeMat.needsUpdate = true;
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
    this.tick();
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

    // Pointer parallax — group tilts slightly toward pointer
    this.pointerCurrent.lerp(this.pointerTarget, this.parallaxSmoothing);
    this.planeGroup.rotation.y = this.pointerCurrent.x * 0.08;
    this.planeGroup.rotation.x = -this.pointerCurrent.y * 0.05;
    // Subtle group x-shift for parallax
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

    // Active plane = whichever plane is closest to camera in Z
    let closestIdx = 0;
    let closestDist = Infinity;
    this.planes.forEach((p, i) => {
      const dist = Math.abs(p.position.z - this.cameraZCurrent + 1);
      if (dist < closestDist) {
        closestDist = dist;
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
