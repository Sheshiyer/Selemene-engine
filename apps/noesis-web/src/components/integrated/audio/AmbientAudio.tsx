"use client";

// ─── AmbientAudio ──────────────────────────────────────────────────────
// Web-Audio-API ambient drone synthesizer, driven by the shared
// AudioStateProvider (mute, volume, current chapter).
//
// Architecture per chapter:
//   - Root oscillator (sine, chapter-specific Hz)
//   - Fifth oscillator (sine, root * (3/2 or 4/3 depending on mode))
//   - Color oscillator (triangle or square, octave + minor/major third)
//   - All three routed through a low-pass filter (gentle, ~1200 Hz)
//   - Filter cutoff modulated by an LFO sine (~0.05 Hz) → "breathing" timbre
//   - Master gain envelope: chapter-fade ramps over 3s on chapter change.
//
// AudioContext lifecycle:
//   - Created lazily on first user gesture (browsers require gesture).
//   - Auto-suspends on tab blur, resumes on focus.
//   - Mute → master gain 0; not destroyed so unmuting is instant.
//
// Per design § 5.11 — volume default 8%, four chapter modes that
// cross-fade as reader scrolls between Parts. Strudel is deliberately
// not used here because the RaagaPlayer singleton already owns the
// Strudel evaluator and a second eval-stream would conflict.

import { useEffect, useRef } from "react";
import { useAudioState, type ChapterIndex } from "./AudioState";

interface ChapterPreset {
  root: number; // Hz
  /** Fifth offset ratio (3/2 for perfect fifth, 4/3 for fourth). */
  fifthRatio: number;
  /** Color oscillator: ratio relative to root + waveform. */
  color: { ratio: number; wave: OscillatorType };
  /** Filter cutoff base in Hz. */
  cutoff: number;
  /** LFO modulation depth on cutoff (Hz). */
  cutoffDepth: number;
  /** Resonance Q. */
  q: number;
}

// C2 = 65.41, G2 = 98.0, F2 = 87.31, E2 = 82.41. Choices match brief.
const PRESETS: Record<ChapterIndex, ChapterPreset> = {
  0: {
    // STABILIZE — minor pentatonic, slow, dark.
    root: 65.41,
    fifthRatio: 3 / 2,
    color: { ratio: 2 * (6 / 5), wave: "triangle" }, // octave + minor third (Eb)
    cutoff: 900,
    cutoffDepth: 250,
    q: 1.5,
  },
  1: {
    // HEAL — up a fifth, breathy/warmer.
    root: 98.0,
    fifthRatio: 3 / 2,
    color: { ratio: 2 * (5 / 4), wave: "triangle" }, // octave + major third (B)
    cutoff: 1200,
    cutoffDepth: 300,
    q: 1.2,
  },
  2: {
    // CREATE — mixolydian on F, brighter timbres.
    root: 87.31,
    fifthRatio: 3 / 2,
    color: { ratio: 2 * (9 / 8), wave: "sawtooth" }, // octave + major 2nd, brighter wave but dialed back via gain
    cutoff: 1600,
    cutoffDepth: 450,
    q: 1.0,
  },
  3: {
    // MUTATE — chromatic descent, dissolving.
    root: 82.41,
    fifthRatio: 4 / 3, // descending tendency (perfect fourth instead)
    color: { ratio: 2 * (11 / 8), wave: "triangle" }, // tritone-ish, unstable
    cutoff: 750,
    cutoffDepth: 600, // wider modulation = "dissolving"
    q: 2.2,
  },
};

const CROSSFADE_S = 3.0;
const COLOR_GAIN_REL = 0.18; // color osc quieter than root/fifth
const FIFTH_GAIN_REL = 0.55;

interface DroneNodes {
  ctx: AudioContext;
  master: GainNode;
  filter: BiquadFilterNode;
  lfo: OscillatorNode;
  lfoGain: GainNode;
  rootOsc: OscillatorNode;
  fifthOsc: OscillatorNode;
  colorOsc: OscillatorNode;
  rootGain: GainNode;
  fifthGain: GainNode;
  colorGain: GainNode;
}

function buildDrone(preset: ChapterPreset): DroneNodes {
  // Lazy AudioContext — must be invoked from a user gesture for autoplay
  // policies, but we'll attempt anyway; if creation succeeds in suspended
  // state we resume() on first interaction.
  const Ctor =
    window.AudioContext ||
    (window as unknown as { webkitAudioContext: typeof AudioContext })
      .webkitAudioContext;
  const ctx = new Ctor();

  const master = ctx.createGain();
  master.gain.value = 0; // start silent — fade in after chapter is set
  master.connect(ctx.destination);

  const filter = ctx.createBiquadFilter();
  filter.type = "lowpass";
  filter.frequency.value = preset.cutoff;
  filter.Q.value = preset.q;
  filter.connect(master);

  // LFO modulates cutoff (very slow — ~0.05 Hz, period ~20 s).
  const lfo = ctx.createOscillator();
  lfo.type = "sine";
  lfo.frequency.value = 0.05;
  const lfoGain = ctx.createGain();
  lfoGain.gain.value = preset.cutoffDepth;
  lfo.connect(lfoGain);
  lfoGain.connect(filter.frequency);
  lfo.start();

  // Root oscillator.
  const rootOsc = ctx.createOscillator();
  rootOsc.type = "sine";
  rootOsc.frequency.value = preset.root;
  const rootGain = ctx.createGain();
  rootGain.gain.value = 1.0;
  rootOsc.connect(rootGain);
  rootGain.connect(filter);
  rootOsc.start();

  // Fifth oscillator.
  const fifthOsc = ctx.createOscillator();
  fifthOsc.type = "sine";
  fifthOsc.frequency.value = preset.root * preset.fifthRatio;
  const fifthGain = ctx.createGain();
  fifthGain.gain.value = FIFTH_GAIN_REL;
  fifthOsc.connect(fifthGain);
  fifthGain.connect(filter);
  fifthOsc.start();

  // Color oscillator.
  const colorOsc = ctx.createOscillator();
  colorOsc.type = preset.color.wave;
  colorOsc.frequency.value = preset.root * preset.color.ratio;
  const colorGain = ctx.createGain();
  colorGain.gain.value = COLOR_GAIN_REL;
  colorOsc.connect(colorGain);
  colorGain.connect(filter);
  colorOsc.start();

  return {
    ctx,
    master,
    filter,
    lfo,
    lfoGain,
    rootOsc,
    fifthOsc,
    colorOsc,
    rootGain,
    fifthGain,
    colorGain,
  };
}

function applyPreset(nodes: DroneNodes, preset: ChapterPreset, when: number) {
  const t = when;
  nodes.rootOsc.frequency.cancelScheduledValues(t);
  nodes.fifthOsc.frequency.cancelScheduledValues(t);
  nodes.colorOsc.frequency.cancelScheduledValues(t);
  nodes.filter.frequency.cancelScheduledValues(t);
  nodes.filter.Q.cancelScheduledValues(t);
  nodes.lfoGain.gain.cancelScheduledValues(t);

  nodes.rootOsc.frequency.linearRampToValueAtTime(preset.root, t + CROSSFADE_S);
  nodes.fifthOsc.frequency.linearRampToValueAtTime(preset.root * preset.fifthRatio, t + CROSSFADE_S);
  nodes.colorOsc.frequency.linearRampToValueAtTime(preset.root * preset.color.ratio, t + CROSSFADE_S);
  nodes.filter.frequency.linearRampToValueAtTime(preset.cutoff, t + CROSSFADE_S);
  nodes.filter.Q.linearRampToValueAtTime(preset.q, t + CROSSFADE_S);
  nodes.lfoGain.gain.linearRampToValueAtTime(preset.cutoffDepth, t + CROSSFADE_S);
  // Color waveform cannot be ramped — replace type instantly (audible click
  // is masked by the slow gain envelope of cross-chapter transitions).
  nodes.colorOsc.type = preset.color.wave;
}

interface AmbientAudioProps {
  /** Placeholder; W6 will compute raga from chart. Currently ignored beyond logging. */
  nakshatra?: string;
}

export function AmbientAudio({ nakshatra }: AmbientAudioProps) {
  const { chapter, muted, volume, setRunning } = useAudioState();
  const nodesRef = useRef<DroneNodes | null>(null);
  const gestureBoundRef = useRef<boolean>(false);

  // Lazy-init AudioContext on first user gesture.
  useEffect(() => {
    if (typeof window === "undefined") return;
    if (gestureBoundRef.current) return;

    const init = async () => {
      if (nodesRef.current) return;
      try {
        const nodes = buildDrone(PRESETS[chapter]);
        nodesRef.current = nodes;
        // Resume context (may already be running on Chromium when triggered from gesture).
        // Read state via getter to avoid narrowing across the await.
        const stateBefore: AudioContextState = nodes.ctx.state;
        if (stateBefore === "suspended") {
          await nodes.ctx.resume();
        }
        // Fade in master to target volume.
        const target = muted ? 0 : volume;
        const t = nodes.ctx.currentTime;
        nodes.master.gain.cancelScheduledValues(t);
        nodes.master.gain.setValueAtTime(0, t);
        nodes.master.gain.linearRampToValueAtTime(target, t + CROSSFADE_S);
        const stateAfter: AudioContextState = nodes.ctx.state;
        setRunning(stateAfter === "running");
        // Light dev-time hint.
        if (process.env.NODE_ENV !== "production") {
          // eslint-disable-next-line no-console
          console.info(
            `[AmbientAudio] init — chapter=${chapter} nakshatra=${nakshatra ?? "—"} state=${nodes.ctx.state}`,
          );
        }
      } catch (err) {
        if (process.env.NODE_ENV !== "production") {
          // eslint-disable-next-line no-console
          console.warn("[AmbientAudio] init failed", err);
        }
      }
    };

    const onGesture = () => {
      void init();
      window.removeEventListener("pointerdown", onGesture);
      window.removeEventListener("keydown", onGesture);
      window.removeEventListener("touchstart", onGesture);
    };

    window.addEventListener("pointerdown", onGesture, { once: true });
    window.addEventListener("keydown", onGesture, { once: true });
    window.addEventListener("touchstart", onGesture, { once: true });
    gestureBoundRef.current = true;

    return () => {
      window.removeEventListener("pointerdown", onGesture);
      window.removeEventListener("keydown", onGesture);
      window.removeEventListener("touchstart", onGesture);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // React to chapter changes.
  useEffect(() => {
    const nodes = nodesRef.current;
    if (!nodes) return;
    applyPreset(nodes, PRESETS[chapter], nodes.ctx.currentTime);
  }, [chapter]);

  // React to mute / volume changes.
  useEffect(() => {
    const nodes = nodesRef.current;
    if (!nodes) return;
    const target = muted ? 0 : volume;
    const t = nodes.ctx.currentTime;
    nodes.master.gain.cancelScheduledValues(t);
    nodes.master.gain.linearRampToValueAtTime(target, t + 0.4);
  }, [muted, volume]);

  // Auto-pause on tab blur, resume on focus.
  useEffect(() => {
    if (typeof document === "undefined") return;
    const onVis = async () => {
      const nodes = nodesRef.current;
      if (!nodes) return;
      try {
        const s: AudioContextState = nodes.ctx.state;
        if (document.hidden) {
          if (s === "running") {
            await nodes.ctx.suspend();
            setRunning(false);
          }
        } else {
          if (s === "suspended") {
            await nodes.ctx.resume();
            const after: AudioContextState = nodes.ctx.state;
            setRunning(after === "running");
          }
        }
      } catch {
        /* ignore */
      }
    };
    document.addEventListener("visibilitychange", onVis);
    return () => document.removeEventListener("visibilitychange", onVis);
  }, [setRunning]);

  // Cleanup on unmount.
  useEffect(() => {
    return () => {
      const nodes = nodesRef.current;
      if (!nodes) return;
      try {
        nodes.rootOsc.stop();
        nodes.fifthOsc.stop();
        nodes.colorOsc.stop();
        nodes.lfo.stop();
        void nodes.ctx.close();
      } catch {
        /* ignore */
      }
      nodesRef.current = null;
    };
  }, []);

  return null;
}
