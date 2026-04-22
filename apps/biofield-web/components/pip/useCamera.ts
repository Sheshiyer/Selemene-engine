"use client";

/**
 * useCamera — WebRTC camera access hook.
 * Ported from bv-pip-analysis prototype.
 */

import { useState, useEffect, useCallback, useRef } from "react";

interface UseCameraOptions {
  width?: number;
  height?: number;
  facingMode?: "user" | "environment";
  deviceId?: string;
}

interface CameraDevice {
  deviceId: string;
  label: string;
}

export interface UseCameraReturn {
  videoRef: React.RefObject<HTMLVideoElement>;
  stream: MediaStream | null;
  isPlaying: boolean;
  isLoading: boolean;
  error: string | null;
  devices: CameraDevice[];
  selectedDevice: string | null;
  start: () => Promise<void>;
  stop: () => void;
  pause: () => void;
  resume: () => void;
  selectDevice: (deviceId: string) => void;
}

export function useCamera(options: UseCameraOptions = {}): UseCameraReturn {
  const { width = 640, height = 480, facingMode = "user", deviceId } = options;

  const [stream, setStream] = useState<MediaStream | null>(null);
  const [isPlaying, setIsPlaying] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [devices, setDevices] = useState<CameraDevice[]>([]);
  const [selectedDevice, setSelectedDevice] = useState<string | null>(
    deviceId ?? null
  );

  const videoRef = useRef<HTMLVideoElement>(null) as React.RefObject<HTMLVideoElement>;
  const streamRef = useRef<MediaStream | null>(null);

  const refreshDevices = useCallback(async () => {
    try {
      const all = await navigator.mediaDevices.enumerateDevices();
      const videoDevices = all
        .filter((d) => d.kind === "videoinput")
        .map((d) => ({
          deviceId: d.deviceId,
          label: d.label || `Camera ${d.deviceId.slice(0, 8)}`,
        }));
      setDevices(videoDevices);
      if (!selectedDevice && videoDevices.length > 0) {
        setSelectedDevice(videoDevices[0].deviceId);
      }
    } catch {
      // enumerateDevices may fail before permission granted — ignore
    }
  }, [selectedDevice]);

  const start = useCallback(async () => {
    setIsLoading(true);
    setError(null);

    // Stop existing stream
    if (streamRef.current) {
      streamRef.current.getTracks().forEach((t) => t.stop());
    }

    try {
      const constraints: MediaStreamConstraints = {
        video: {
          width: { ideal: width },
          height: { ideal: height },
          facingMode,
          ...(selectedDevice ? { deviceId: { exact: selectedDevice } } : {}),
        },
        audio: false,
      };

      const ms = await navigator.mediaDevices.getUserMedia(constraints);
      streamRef.current = ms;
      setStream(ms);

      if (videoRef.current) {
        videoRef.current.srcObject = ms;
        await videoRef.current.play();
        setIsPlaying(true);
      }

      await refreshDevices();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Camera access denied");
    } finally {
      setIsLoading(false);
    }
  }, [width, height, facingMode, selectedDevice, refreshDevices]);

  const stop = useCallback(() => {
    if (streamRef.current) {
      streamRef.current.getTracks().forEach((t) => t.stop());
      streamRef.current = null;
      setStream(null);
    }
    if (videoRef.current) videoRef.current.srcObject = null;
    setIsPlaying(false);
  }, []);

  const pause = useCallback(() => {
    videoRef.current?.pause();
    setIsPlaying(false);
  }, []);

  const resume = useCallback(() => {
    videoRef.current?.play();
    setIsPlaying(true);
  }, []);

  const selectDevice = useCallback((id: string) => {
    setSelectedDevice(id);
  }, []);

  // Restart stream when device selection changes (after initial mount)
  const mountedRef = useRef(false);
  useEffect(() => {
    if (!mountedRef.current) { mountedRef.current = true; return; }
    if (streamRef.current && selectedDevice) start();
  }, [selectedDevice]); // eslint-disable-line react-hooks/exhaustive-deps

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      streamRef.current?.getTracks().forEach((t) => t.stop());
    };
  }, []);

  return {
    videoRef,
    stream,
    isPlaying,
    isLoading,
    error,
    devices,
    selectedDevice,
    start,
    stop,
    pause,
    resume,
    selectDevice,
  };
}
