"use client";

import { useCallback, useEffect, useRef, useState } from "react";

export interface CameraDevice {
  deviceId: string;
  label: string;
}

export interface UseCameraResult {
  videoRef: React.RefObject<HTMLVideoElement | null>;
  isStreaming: boolean;
  devices: CameraDevice[];
  error: string | null;
  startCamera: (deviceId?: string) => Promise<void>;
  stopCamera: () => void;
}

export function useCamera(): UseCameraResult {
  const videoRef = useRef<HTMLVideoElement | null>(null);
  const streamRef = useRef<MediaStream | null>(null);
  const [isStreaming, setIsStreaming] = useState(false);
  const [devices, setDevices] = useState<CameraDevice[]>([]);
  const [error, setError] = useState<string | null>(null);

  // Enumerate cameras on mount (labels may be empty until permission granted).
  useEffect(() => {
    async function enumerate() {
      try {
        const all = await navigator.mediaDevices.enumerateDevices();
        setDevices(toVideoDevices(all));
      } catch {
        // Permissions not yet granted — list populated after startCamera.
      }
    }
    void enumerate();
  }, []);

  // Stop all tracks on unmount.
  useEffect(() => {
    return () => {
      stopStream(streamRef.current);
    };
  }, []);

  const startCamera = useCallback(async (deviceId?: string) => {
    setError(null);
    try {
      const constraints: MediaStreamConstraints = {
        video: deviceId ? { deviceId: { exact: deviceId } } : { facingMode: "user" },
        audio: false,
      };
      const stream = await navigator.mediaDevices.getUserMedia(constraints);
      streamRef.current = stream;

      if (videoRef.current) {
        videoRef.current.srcObject = stream;
        await videoRef.current.play();
      }

      setIsStreaming(true);

      // Re-enumerate after permission granted — labels are now populated.
      const all = await navigator.mediaDevices.enumerateDevices();
      setDevices(toVideoDevices(all));
    } catch (err) {
      setError(err instanceof Error ? err.message : "Camera access denied.");
      setIsStreaming(false);
    }
  }, []);

  const stopCamera = useCallback(() => {
    stopStream(streamRef.current);
    streamRef.current = null;
    if (videoRef.current) videoRef.current.srcObject = null;
    setIsStreaming(false);
  }, []);

  return { videoRef, isStreaming, devices, error, startCamera, stopCamera };
}

function stopStream(stream: MediaStream | null) {
  if (stream) {
    for (const track of stream.getTracks()) track.stop();
  }
}

function toVideoDevices(devices: MediaDeviceInfo[]): CameraDevice[] {
  return devices
    .filter((d) => d.kind === "videoinput")
    .map((d, i) => ({
      deviceId: d.deviceId,
      label: d.label || `Camera ${i + 1}`,
    }));
}
