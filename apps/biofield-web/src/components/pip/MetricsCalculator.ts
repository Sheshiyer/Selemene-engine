import type { MediaPipeMask, MediaPipeFaceResult } from "./useMediaPipe";
import type { FrameMetrics, CompositeScores } from "./types";

// ─── Sampling ─────────────────────────────────────────────────────────────────
// We read raw pixel data from the WebGL canvas every SAMPLE_INTERVAL_MS ms.
// All maths runs on the CPU via a reused off-screen canvas context.
const SAMPLE_INTERVAL_MS = 100; // 10 fps metric refresh, < 1 ms per frame budget

// Histogram bin count for Shannon entropy calculation.
const HIST_BINS = 64;

export class MetricsCalculator {
  private offscreen: OffscreenCanvas | null = null;
  private ctx: OffscreenCanvasRenderingContext2D | null = null;
  private lastSampleMs = 0;
  private lastMetrics: FrameMetrics | null = null;
  private lastComposite: CompositeScores | null = null;

  /**
   * Call once per RAF frame. Returns the most-recently computed values;
   * actual computation only runs every SAMPLE_INTERVAL_MS.
   */
  compute(
    canvas: HTMLCanvasElement,
    mask: MediaPipeMask | null,
    face: MediaPipeFaceResult | null,
  ): { frame: FrameMetrics; composite: CompositeScores } | null {
    const now = performance.now();
    if (now - this.lastSampleMs < SAMPLE_INTERVAL_MS) {
      // Return cached result between sample intervals.
      if (this.lastMetrics && this.lastComposite) {
        return { frame: this.lastMetrics, composite: this.lastComposite };
      }
      return null;
    }

    const w = canvas.width;
    const h = canvas.height;
    if (w === 0 || h === 0) return null;

    // Lazily create / resize off-screen canvas used to read pixels.
    if (!this.offscreen || this.offscreen.width !== w || this.offscreen.height !== h) {
      this.offscreen = new OffscreenCanvas(w, h);
      this.ctx = this.offscreen.getContext("2d", { willReadFrequently: true }) as OffscreenCanvasRenderingContext2D | null;
    }

    const ctx = this.ctx;
    if (!ctx) return null;

    ctx.drawImage(canvas, 0, 0, w, h);
    const imageData = ctx.getImageData(0, 0, w, h);
    const pixels = imageData.data; // Uint8ClampedArray, RGBA

    // ── Luminance & variance ──────────────────────────────────────────────────
    let sumL = 0;
    let sumL2 = 0;
    const n = w * h;

    // Weighted by mask confidence when available.
    let maskWeight = 0;

    for (let i = 0; i < n; i++) {
      const r = pixels[i * 4];
      const g = pixels[i * 4 + 1];
      const b = pixels[i * 4 + 2];
      // BT.709 luminance
      const lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
      const w_i = mask ? mask.data[i] ?? 1 : 1;
      sumL  += lum * w_i;
      sumL2 += lum * lum * w_i;
      maskWeight += w_i;
    }

    const safeWeight = maskWeight > 0 ? maskWeight : n;
    const meanL = sumL / safeWeight;
    const variance = sumL2 / safeWeight - meanL * meanL;
    const averageLuminance = meanL / 255; // normalised [0,1]
    const pixelVariance = Math.sqrt(Math.max(0, variance)) / 255;

    // ── Shannon entropy ───────────────────────────────────────────────────────
    const hist = new Float32Array(HIST_BINS);
    for (let i = 0; i < n; i++) {
      const r = pixels[i * 4];
      const g = pixels[i * 4 + 1];
      const b = pixels[i * 4 + 2];
      const lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
      const bin = Math.min(HIST_BINS - 1, Math.floor((lum / 255) * HIST_BINS));
      hist[bin] += 1;
    }
    let entropy = 0;
    const logN = Math.log(n);
    for (let k = 0; k < HIST_BINS; k++) {
      if (hist[k] > 0) {
        const p = hist[k] / n;
        entropy -= p * Math.log(p);
      }
    }
    const entropyScore = entropy / logN; // normalised to [0, 1]

    // ── Bilateral symmetry ────────────────────────────────────────────────────
    // Compares left-half luminance to mirror of right-half.
    let symDiff = 0;
    let symTotal = 0;
    const halfW = Math.floor(w / 2);
    for (let y = 0; y < h; y++) {
      for (let x = 0; x < halfW; x++) {
        const lIdx = (y * w + x) * 4;
        const rIdx = (y * w + (w - 1 - x)) * 4;
        const lLum = 0.2126 * pixels[lIdx] + 0.7152 * pixels[lIdx + 1] + 0.0722 * pixels[lIdx + 2];
        const rLum = 0.2126 * pixels[rIdx] + 0.7152 * pixels[rIdx + 1] + 0.0722 * pixels[rIdx + 2];
        symDiff += Math.abs(lLum - rLum);
        symTotal += (lLum + rLum) / 2 + 1; // +1 to avoid div-by-zero
      }
    }
    const pixelSymmetry = 1 - symDiff / (symTotal || 1);

    // ── Face-landmark symmetry boost ─────────────────────────────────────────
    // When face landmarks are available, compute the horizontal deviation of
    // key midline landmarks to produce a geometry-based symmetry score.
    let geometrySymmetry = pixelSymmetry;
    if (face && face.landmarks.length > 0) {
      const lms = face.landmarks[0];
      // MediaPipe Face Landmarker: landmark 0 = nose tip (approx midline).
      // We compare left-eye outer (130) vs right-eye outer (359) horizontal positions.
      // All coordinates normalised [0,1].
      const leftEye  = lms[130];
      const rightEye = lms[359];
      const noseTip  = lms[1];

      if (leftEye && rightEye && noseTip) {
        // Ideal: nose is equidistant from both eyes on x-axis.
        const midX = (leftEye.x + rightEye.x) / 2;
        const deviation = Math.abs(noseTip.x - midX) / (Math.abs(rightEye.x - leftEye.x) + 1e-6);
        // deviation ≈ 0 → perfect symmetry, > 0.1 → asymmetric pose
        geometrySymmetry = 1 - Math.min(1, deviation * 5);
      }
    }

    const symmetryScore = (pixelSymmetry + geometrySymmetry) / 2;

    // ── Frame metrics ─────────────────────────────────────────────────────────
    const frame: FrameMetrics = {
      averageLuminance,
      pixelVariance,
      symmetryScore,
      entropyScore,
      timestamp: now,
    };

    // ── Composite scores ──────────────────────────────────────────────────────
    // Map raw metrics to the biofield-domain composite score schema.
    const composite: CompositeScores = {
      // Light quanta density: how bright + how complex the field is.
      lightQuantaDensity: clamp(averageLuminance * 0.6 + pixelVariance * 0.4),

      // Normalised area: proxy for how much of the frame is person vs background.
      normalizedArea: mask ? clamp(maskWeight / n) : 0.5,

      // Body symmetry: our combined pixel + geometry score.
      bodySymmetry: clamp(symmetryScore),

      // Pattern regularity: inverted entropy — low entropy = high regularity.
      patternRegularity: clamp(1 - entropyScore),

      // Overall coherence: weighted average of all four.
      overallCoherence: clamp(
        averageLuminance * 0.2 +
        symmetryScore * 0.3 +
        (1 - entropyScore) * 0.25 +
        pixelVariance * 0.25,
      ),
    };

    this.lastSampleMs = now;
    this.lastMetrics = frame;
    this.lastComposite = composite;
    return { frame, composite };
  }

  dispose(): void {
    this.offscreen = null;
    this.ctx = null;
    this.lastMetrics = null;
    this.lastComposite = null;
  }
}

function clamp(v: number, lo = 0, hi = 1): number {
  return Math.max(lo, Math.min(hi, v));
}
