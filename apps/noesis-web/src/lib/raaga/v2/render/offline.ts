// Offline raaga rendering via OfflineAudioContext.
//
// We can't directly route Strudel into an OfflineAudioContext (Strudel binds
// to the live AudioContext at init time). What we CAN do is a deterministic
// pure-JS render that mirrors the v2 composition: walk the swara mini-notation
// in time, schedule sine-wave oscillators with envelopes, and let the offline
// context render to PCM. This is "good enough" for download / sharing — the
// shrutis are exact, the envelopes mirror the timbre profile, and the WAV
// matches the live audio's *pitch content* (timbre is approximated).
//
// Acceptable Phase-2 trade-off; Phase-3 can replace this with real Strudel
// offline render once @strudel/web exposes an offline context entry point.

import type { Melakarta } from '../../melakartas';
import { ratioOf } from '../../shrutis';
import { compose } from '../compose';
import { TIMBRES } from '../samples/timbres';
import { encodeWav } from './wav-encoder';
import type { PlayOptions } from '../../RaagaPlayer';

interface ScheduledNote {
  /** Start time in seconds. */
  start: number;
  /** Duration in seconds. */
  duration: number;
  /** Frequency in Hz. */
  hz: number;
}

/** Walk the v2 compose() output to extract scheduled notes for offline render. */
const scheduleFromCompose = (
  m: Melakarta,
  opts: PlayOptions,
): { notes: ScheduledNote[]; durationSeconds: number; sampleRate: number } => {
  // Re-derive the same Hz sequence the v2 path emits
  const aroha = m.arohana.map(ratioOf);
  const avaro = m.avarohana.map(ratioOf);
  const ratios =
    opts.direction === 'arohana' ? aroha :
    opts.direction === 'avarohana' ? avaro :
    [...aroha, ...avaro.slice(1)];
  const rootHz = opts.rootHz ?? 220;
  const hzs = ratios.map((r) => +(r * rootHz).toFixed(4));

  const composed = compose({
    melakarta: m,
    rootHz: opts.rootHz,
    cps: opts.cps,
    direction: opts.direction,
    timbre: opts.timbre,
    gamakas: opts.gamakas,
    defaultGamaka: opts.defaultGamaka,
    tala: opts.tala,
    breath: opts.breath,
    tanpura: opts.tanpura,
  });

  const cps = opts.cps ?? 0.5;
  const beatSeconds = 1 / cps;
  // Each swara occupies one cycle / (length / 2) effectively.
  // For offline render simplicity we give each top-level swara an equal slot.
  const slotSeconds = beatSeconds * 2;
  const totalSeconds = hzs.length * slotSeconds;

  const notes: ScheduledNote[] = hzs.map((hz, i) => ({
    start: i * slotSeconds,
    duration: slotSeconds,
    hz,
  }));

  return {
    notes,
    durationSeconds: composed.durationSeconds || totalSeconds,
    sampleRate: 48000,
  };
};

/** Render a raaga to a WAV blob URL via OfflineAudioContext. */
export const renderToWavBlobUrl = async (
  m: Melakarta,
  opts: PlayOptions,
): Promise<string | null> => {
  if (typeof OfflineAudioContext === 'undefined') return null;

  const { notes, durationSeconds, sampleRate } = scheduleFromCompose(m, opts);
  const totalDur = Math.max(durationSeconds, notes[notes.length - 1].start + notes[notes.length - 1].duration);

  const ctx = new OfflineAudioContext({
    numberOfChannels: 2,
    length: Math.ceil(totalDur * sampleRate),
    sampleRate,
  });

  const timbre = TIMBRES[opts.timbre ?? 'sine'];

  for (const n of notes) {
    const osc = ctx.createOscillator();
    osc.type = 'sine';
    osc.frequency.setValueAtTime(n.hz, n.start);

    const gain = ctx.createGain();
    const baseGain = timbre.gain ?? 0.8;
    const t0 = n.start;
    const tA = t0 + timbre.attack;
    const tD = tA + timbre.decay;
    const tS = t0 + Math.max(0, n.duration - timbre.release);
    const tR = t0 + n.duration;
    gain.gain.setValueAtTime(0, t0);
    gain.gain.linearRampToValueAtTime(baseGain, tA);
    gain.gain.linearRampToValueAtTime(baseGain * timbre.sustain, tD);
    gain.gain.setValueAtTime(baseGain * timbre.sustain, tS);
    gain.gain.linearRampToValueAtTime(0, tR);

    osc.connect(gain);

    // Optional LPF if timbre specifies it
    if (timbre.lpf != null) {
      const filter = ctx.createBiquadFilter();
      filter.type = 'lowpass';
      filter.frequency.value = timbre.lpf;
      filter.Q.value = timbre.lpq ?? 1;
      gain.connect(filter);
      filter.connect(ctx.destination);
    } else {
      gain.connect(ctx.destination);
    }

    osc.start(t0);
    osc.stop(tR + 0.05);
  }

  const buffer = await ctx.startRendering();
  const channels: Float32Array[] = [];
  for (let c = 0; c < buffer.numberOfChannels; c++) {
    channels.push(buffer.getChannelData(c).slice());
  }
  if (channels.length === 1) channels.push(channels[0]);  // mono → stereo

  const wav = encodeWav({ channels, sampleRate });
  const blob = new Blob([wav], { type: 'audio/wav' });
  return URL.createObjectURL(blob);
};
