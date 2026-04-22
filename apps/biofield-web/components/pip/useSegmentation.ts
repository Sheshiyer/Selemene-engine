"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import { FaceLandmarker, FilesetResolver, ImageSegmenter } from "@mediapipe/tasks-vision";

export interface SegmentationState {
  isReady: boolean;
  bodyDetected: boolean;
  faceDetected: boolean;
  mask: Uint8Array | null;
  width: number;
  height: number;
  error: string | null;
}

interface UseSegmentationReturn extends SegmentationState {
  process: (video: HTMLVideoElement) => Promise<void>;
}

export function useSegmentation(): UseSegmentationReturn {
  const bodyRef = useRef<ImageSegmenter | null>(null);
  const faceRef = useRef<FaceLandmarker | null>(null);

  const [state, setState] = useState<SegmentationState>({
    isReady: false,
    bodyDetected: false,
    faceDetected: false,
    mask: null,
    width: 0,
    height: 0,
    error: null,
  });

  useEffect(() => {
    let disposed = false;

    const init = async () => {
      try {
        const vision = await FilesetResolver.forVisionTasks(
          "https://cdn.jsdelivr.net/npm/@mediapipe/tasks-vision@latest/wasm"
        );

        const body = await ImageSegmenter.createFromOptions(vision, {
          baseOptions: {
            modelAssetPath:
              "https://storage.googleapis.com/mediapipe-models/image_segmenter/selfie_segmenter/float16/latest/selfie_segmenter.tflite",
            delegate: "GPU",
          },
          runningMode: "VIDEO",
          outputCategoryMask: true,
          outputConfidenceMasks: true,
        });

        const face = await FaceLandmarker.createFromOptions(vision, {
          baseOptions: {
            modelAssetPath:
              "https://storage.googleapis.com/mediapipe-models/face_landmarker/face_landmarker/float16/1/face_landmarker.task",
            delegate: "GPU",
          },
          runningMode: "VIDEO",
          numFaces: 1,
          minFaceDetectionConfidence: 0.5,
          minTrackingConfidence: 0.5,
        });

        if (!disposed) {
          bodyRef.current = body;
          faceRef.current = face;
          setState((prev) => ({ ...prev, isReady: true, error: null }));
        }
      } catch (err) {
        if (!disposed) {
          setState((prev) => ({
            ...prev,
            error: err instanceof Error ? err.message : "Segmentation init failed",
          }));
        }
      }
    };

    init();

    return () => {
      disposed = true;
      bodyRef.current = null;
      faceRef.current = null;
    };
  }, []);

  const process = useCallback(async (video: HTMLVideoElement) => {
    if (!bodyRef.current || !faceRef.current) return;
    const w = video.videoWidth || 640;
    const h = video.videoHeight || 480;
    if (w === 0 || h === 0) return;

    const ts = performance.now();
    const bodyResult = bodyRef.current.segmentForVideo(video, ts);
    const faceResult = faceRef.current.detectForVideo(video, ts);

    const mask = new Uint8Array(w * h);
    let bodyDetected = false;

    const confidenceMask = bodyResult.confidenceMasks?.[0];
    if (confidenceMask && typeof confidenceMask.getAsFloat32Array === "function") {
      const raw = confidenceMask.getAsFloat32Array();
      const mw = confidenceMask.width;
      const mh = confidenceMask.height;

      for (let y = 0; y < h; y++) {
        for (let x = 0; x < w; x++) {
          const sx = Math.floor((x * mw) / w);
          const sy = Math.floor((y * mh) / h);
          const src = sy * mw + sx;
          const dst = y * w + x;
          const v = raw[src];
          if (v > 0.5) {
            mask[dst] = 255;
            bodyDetected = true;
          }
        }
      }
    }

    const faceDetected = (faceResult.faceLandmarks?.length ?? 0) > 0;

    // If face is detected but body mask is empty, keep full mask as safe fallback
    if (faceDetected && !bodyDetected) {
      mask.fill(255);
      bodyDetected = true;
    }

    setState((prev) => ({
      ...prev,
      bodyDetected,
      faceDetected,
      mask,
      width: w,
      height: h,
    }));
  }, []);

  return {
    ...state,
    process,
  };
}
