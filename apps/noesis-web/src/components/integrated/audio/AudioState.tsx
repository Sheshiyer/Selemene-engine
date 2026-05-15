"use client";

// ─── AudioState — shared client context for the integrated-reading audio layer.
// Holds: current chapter index (0..3 mapped to Parts I-IV), mute toggle,
// volume (0..1), and a "running" flag (AudioContext.state === 'running').
//
// Persistence: muted + volume saved to localStorage under
// `noesis.integrated.audio.{muted,volume}`. Chapter is ephemeral —
// derived from IntersectionObserver on #part-N elements.

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
  type ReactNode,
} from "react";

export type ChapterIndex = 0 | 1 | 2 | 3;
export const CHAPTER_LABELS = [
  "I · STABILIZE",
  "II · HEAL",
  "III · CREATE",
  "IV · MUTATE",
] as const;

const LS_MUTED = "noesis.integrated.audio.muted";
const LS_VOLUME = "noesis.integrated.audio.volume";
const DEFAULT_VOLUME = 0.08;

interface AudioStateValue {
  chapter: ChapterIndex;
  muted: boolean;
  volume: number;
  running: boolean;
  toggleMute: () => void;
  setVolume: (v: number) => void;
  setRunning: (r: boolean) => void;
  setChapter: (c: ChapterIndex) => void;
}

const Ctx = createContext<AudioStateValue | null>(null);

interface ProviderProps {
  children: ReactNode;
  /** CSS-selector for chapter anchors used to drive IntersectionObserver.
   *  Defaults to `#part-1, #part-2, #part-3, #part-4`. */
  partSelector?: string;
}

export function AudioStateProvider({
  children,
  partSelector = "#part-1, #part-2, #part-3, #part-4",
}: ProviderProps) {
  const [muted, setMuted] = useState<boolean>(false);
  const [volume, setVolumeState] = useState<number>(DEFAULT_VOLUME);
  const [running, setRunning] = useState<boolean>(false);
  const [chapter, setChapter] = useState<ChapterIndex>(0);
  const ioRef = useRef<IntersectionObserver | null>(null);

  // Hydrate from localStorage on mount.
  useEffect(() => {
    try {
      const m = localStorage.getItem(LS_MUTED);
      if (m !== null) setMuted(m === "1");
      const v = localStorage.getItem(LS_VOLUME);
      if (v !== null) {
        const parsed = parseFloat(v);
        if (Number.isFinite(parsed) && parsed >= 0 && parsed <= 1) {
          setVolumeState(parsed);
        }
      }
    } catch {
      /* localStorage unavailable; carry on with defaults */
    }
  }, []);

  // Persist mute + volume.
  useEffect(() => {
    try {
      localStorage.setItem(LS_MUTED, muted ? "1" : "0");
    } catch {
      /* ignore */
    }
  }, [muted]);
  useEffect(() => {
    try {
      localStorage.setItem(LS_VOLUME, String(volume));
    } catch {
      /* ignore */
    }
  }, [volume]);

  // Watch part-N sections to drive chapter changes.
  useEffect(() => {
    if (typeof window === "undefined") return;
    const targets = Array.from(document.querySelectorAll<HTMLElement>(partSelector));
    if (targets.length === 0) return;

    // Sort by their #part-N suffix so we can map element → index reliably.
    const indexed = targets
      .map((el) => {
        const m = el.id.match(/^part-(\d+)$/);
        const n = m ? parseInt(m[1], 10) - 1 : 0;
        return { el, idx: Math.min(3, Math.max(0, n)) as ChapterIndex };
      })
      .sort((a, b) => a.idx - b.idx);

    // Track visibility ratios; pick the part with the largest ratio.
    const ratios = new Map<HTMLElement, number>();
    indexed.forEach(({ el }) => ratios.set(el, 0));

    const io = new IntersectionObserver(
      (entries) => {
        for (const e of entries) {
          ratios.set(e.target as HTMLElement, e.intersectionRatio);
        }
        let best: ChapterIndex = 0;
        let bestRatio = -1;
        for (const { el, idx } of indexed) {
          const r = ratios.get(el) ?? 0;
          if (r > bestRatio) {
            bestRatio = r;
            best = idx;
          }
        }
        if (bestRatio > 0) setChapter(best);
      },
      {
        // Wide tracking — we want to know which part dominates the viewport.
        threshold: [0, 0.1, 0.25, 0.5, 0.75, 1],
        rootMargin: "-20% 0px -20% 0px",
      },
    );
    indexed.forEach(({ el }) => io.observe(el));
    ioRef.current = io;
    return () => {
      io.disconnect();
      ioRef.current = null;
    };
  }, [partSelector]);

  const toggleMute = useCallback(() => setMuted((m) => !m), []);
  const setVolume = useCallback((v: number) => {
    const clamped = Math.max(0, Math.min(1, v));
    setVolumeState(clamped);
  }, []);

  const value = useMemo<AudioStateValue>(
    () => ({
      chapter,
      muted,
      volume,
      running,
      toggleMute,
      setVolume,
      setRunning,
      setChapter,
    }),
    [chapter, muted, volume, running, toggleMute, setVolume],
  );

  return <Ctx.Provider value={value}>{children}</Ctx.Provider>;
}

export function useAudioState(): AudioStateValue {
  const v = useContext(Ctx);
  if (!v) {
    throw new Error("useAudioState must be used inside <AudioStateProvider>");
  }
  return v;
}
