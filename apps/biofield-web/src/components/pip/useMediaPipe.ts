"use client";

import { useCallback, useEffect, useRef, useState } from "react";

// ─── Inline MediaPipe type definitions ────────────────────────────────────────
// These mirror the @mediapipe/tasks-vision API surface. The actual module is
// loaded at runtime from the MediaPipe CDN ESM bundle to avoid requiring npm.

interface FilesetResolverStatic {
  forVisionTasks(basePath: string): Promise<WasmFileset>;
}

interface WasmFileset {
  wasmLoaderPath: string;
  wasmBinaryPath: string;
}

interface ImageSegmenterOptions {
  baseOptions: { modelAssetPath: string; delegate?: "GPU" | "CPU" };
  outputCategoryMask?: boolean;
  outputConfidenceMasks?: boolean;
  runningMode?: "IMAGE" | "VIDEO";
}

interface SegmentationMask {
  /** Float32 confidence values per pixel (0=background, 1=person). */
  getAsFloat32Array(): Float32Array;
  width: number;
  height: number;
  close(): void;
}

interface ImageSegmenterResult {
  confidenceMasks?: SegmentationMask[];
  categoryMask?: SegmentationMask;
  close(): void;
}

interface ImageSegmenterStatic {
  createFromOptions(fileset: WasmFileset, options: ImageSegmenterOptions): Promise<ImageSegmenterInstance>;
}

interface ImageSegmenterInstance {
  segmentForVideo(source: HTMLVideoElement, timestampMs: number): ImageSegmenterResult;
  close(): void;
}

interface FaceLandmarkerOptions {
  baseOptions: { modelAssetPath: string; delegate?: "GPU" | "CPU" };
  runningMode?: "IMAGE" | "VIDEO";
  numFaces?: number;
  minFaceDetectionConfidence?: number;
  minTrackingConfidence?: number;
}

interface NormalizedLandmark {
  x: number;
  y: number;
  z: number;
}

interface FaceLandmarkerResult {
  faceLandmarks: NormalizedLandmark[][];
  // Note: FaceLandmarkerResult does NOT have close() — only segmentation results do.
}

interface FaceLandmarkerStatic {
  createFromOptions(fileset: WasmFileset, options: FaceLandmarkerOptions): Promise<FaceLandmarkerInstance>;
}

interface FaceLandmarkerInstance {
  detectForVideo(source: HTMLVideoElement, timestampMs: number): FaceLandmarkerResult;
  close(): void;
}

interface MediaPipeVisionModule {
  FilesetResolver: FilesetResolverStatic;
  ImageSegmenter: ImageSegmenterStatic;
  FaceLandmarker: FaceLandmarkerStatic;
}

// ─── Model URLs ───────────────────────────────────────────────────────────────
const MP_CDN = "https://cdn.jsdelivr.net/npm/@mediapipe/tasks-vision@0.10.14";
const SELFIE_SEGMENTATION_MODEL =
  "https://storage.googleapis.com/mediapipe-models/image_segmenter/selfie_segmenter/float16/latest/selfie_segmenter.tflite";
const FACE_LANDMARK_MODEL =
  "https://storage.googleapis.com/mediapipe-models/face_landmarker/face_landmarker/float16/1/face_landmarker.task";

export interface MediaPipeMask {
  /** Confidence float32 array (0=background, 1=person), row-major. */
  data: Float32Array;
  width: number;
  height: number;
}

export interface MediaPipeFaceResult {
  landmarks: NormalizedLandmark[][];
}

export interface UseMediaPipeResult {
  ready: boolean;
  error: string | null;
  /** Run segmentation on current video frame. Returns null until ready. */
  segmentFrame(video: HTMLVideoElement): MediaPipeMask | null;
  /** Run face landmark detection on current video frame. Returns null until ready. */
  detectFace(video: HTMLVideoElement): MediaPipeFaceResult | null;
}

/**
 * Loads MediaPipe Tasks Vision from CDN and initialises the Selfie Segmenter
 * and Face Landmarker. The CDN bundle loads WASM lazily on first call.
 *
 * Requires a network connection to cdn.jsdelivr.net and storage.googleapis.com.
 * No npm dependency needed — the module is imported via indirect dynamic import.
 */
export function useMediaPipe(): UseMediaPipeResult {
  const segmenterRef = useRef<ImageSegmenterInstance | null>(null);
  const faceRef = useRef<FaceLandmarkerInstance | null>(null);
  const [ready, setReady] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    async function init() {
      // MediaPipe WASM emits TFLite/XNNPACK init messages through console.error.
      // Suppress them during init so they don't appear in the Next.js issues overlay.
      const origError = console.error;
      console.error = (...args: unknown[]) => {
        const msg = String(args[0] ?? "");
        if (msg.startsWith("INFO:") || msg.includes("XNNPACK") || msg.includes("gl_context") || msg.includes("face_landmarker")) return;
        origError.apply(console, args);
      };

      try {
        // Indirect dynamic import bypasses TypeScript's URL import check while
        // preserving full type safety via the inline interface above.
        // eslint-disable-next-line @typescript-eslint/no-implied-eval
        const mp = await new Function("u", "return import(u)")(
          `${MP_CDN}/vision_bundle.mjs`,
        ) as MediaPipeVisionModule;

        const fileset = await mp.FilesetResolver.forVisionTasks(
          `${MP_CDN}/wasm`,
        );

        const [segmenter, face] = await Promise.all([
          mp.ImageSegmenter.createFromOptions(fileset, {
            baseOptions: {
              modelAssetPath: SELFIE_SEGMENTATION_MODEL,
              delegate: "GPU",
            },
            outputConfidenceMasks: true,
            runningMode: "VIDEO",
          }),
          mp.FaceLandmarker.createFromOptions(fileset, {
            baseOptions: {
              modelAssetPath: FACE_LANDMARK_MODEL,
              delegate: "GPU",
            },
            runningMode: "VIDEO",
            numFaces: 1,
            minFaceDetectionConfidence: 0.5,
            minTrackingConfidence: 0.5,
          }),
        ]);

        if (cancelled) {
          segmenter.close();
          face.close();
          return;
        }

        segmenterRef.current = segmenter;
        faceRef.current = face;
        setReady(true);
      } catch (err) {
        if (!cancelled) {
          const message = err instanceof Error ? err.message : "MediaPipe init failed";
          console.warn("[useMediaPipe]", message);
          setError(message);
        }
      } finally {
        console.error = origError;
      }
    }

    void init();

    return () => {
      cancelled = true;
      segmenterRef.current?.close();
      faceRef.current?.close();
      segmenterRef.current = null;
      faceRef.current = null;
      setReady(false);
      setError(null);
    };
  }, []);

  const segmentFrame = useCallback((video: HTMLVideoElement): MediaPipeMask | null => {
    const seg = segmenterRef.current;
    if (!seg || video.readyState < video.HAVE_CURRENT_DATA) return null;

    const result = seg.segmentForVideo(video, performance.now());
    const mask = result.confidenceMasks?.[0];
    if (!mask) {
      result.close();
      return null;
    }

    const out: MediaPipeMask = {
      data: mask.getAsFloat32Array().slice(), // copy before mask.close()
      width: mask.width,
      height: mask.height,
    };
    result.close();
    return out;
  }, []);

  const detectFace = useCallback((video: HTMLVideoElement): MediaPipeFaceResult | null => {
    const face = faceRef.current;
    if (!face || video.readyState < video.HAVE_CURRENT_DATA) return null;

    const result = face.detectForVideo(video, performance.now());
    const out: MediaPipeFaceResult = { landmarks: result.faceLandmarks };
    // FaceLandmarkerResult has no close() — only ImageSegmenterResult does.
    return out;
  }, []);

  return { ready, error, segmentFrame, detectFace };
}
