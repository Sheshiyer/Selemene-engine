// Strudel-backed raaga player.
//
// V1 path (default): pure sine waves at exact 22-shruti just-intonation Hz.
// V2 path (opt-in): adds gamakas, sample timbres, tala-aware structure,
//   breath-paced cps, optional tanpura drone, optional WAV download.
// V2 is additive — every v1 callsite keeps working unchanged.

import { ratioOf } from './shrutis';
import { getMelakarta, type Melakarta } from './melakartas';
import type { Gamaka } from './v2/gamakas/types';
import type { GamakaAnnotation } from './v2/gamakas/apply';
import type { TalaName } from './v2/talas/types';
import type { BreathName } from './v2/breaths/data';
import type { Timbre } from './v2/samples/timbres';
import { v2Enabled } from './v2/feature-flag';
import { compose } from './v2/compose';

export interface PlayOptions {
  // ── V1 (always honored) ────────────────────────────────────────────────
  rootHz?: number;
  cps?: number;
  sound?: string;
  direction?: 'arohana' | 'avarohana' | 'both';

  // ── V2 (additive) ──────────────────────────────────────────────────────
  v2?: boolean;
  timbre?: Timbre;
  gamakas?: readonly GamakaAnnotation[];
  defaultGamaka?: Gamaka;
  tala?: TalaName;
  breath?: BreathName;
  tanpura?: boolean;
}

type StrudelModule = typeof import('@strudel/web');

export class RaagaPlayer {
  private ready: Promise<StrudelModule> | null = null;

  private async getStrudel(): Promise<StrudelModule> {
    if (!this.ready) {
      this.ready = (async () => {
        const mod = await import('@strudel/web');
        await mod.initStrudel({ prebake: () => {} });
        return mod;
      })();
    }
    return this.ready;
  }

  private buildRatios(m: Melakarta, direction: PlayOptions['direction']): number[] {
    const aroha = m.arohana.map(ratioOf);
    const avaro = m.avarohana.map(ratioOf);
    switch (direction) {
      case 'arohana':   return aroha;
      case 'avarohana': return avaro;
      case 'both':
      default:          return [...aroha, ...avaro.slice(1)];
    }
  }

  private shouldUseV2(opts: PlayOptions): boolean {
    if (opts.v2 != null) return opts.v2;
    return v2Enabled();
  }

  async play(melakartaNum: number, opts: PlayOptions = {}): Promise<void> {
    const m = getMelakarta(melakartaNum);
    if (!m) throw new Error(`Unknown melakarta: ${melakartaNum}`);

    if (this.shouldUseV2(opts)) {
      return this.playV2(m, opts);
    }
    return this.playV1(m, opts);
  }

  /** V1 path — bit-identical to the shipped v1 (sine waves, exact shrutis). */
  private async playV1(m: Melakarta, opts: PlayOptions): Promise<void> {
    const { rootHz = 220, cps = 0.5, sound = 'sine', direction = 'both' } = opts;
    const ratios = this.buildRatios(m, direction);
    const hzs = ratios.map((r) => +(r * rootHz).toFixed(4));
    const N = hzs.length;
    const s = await this.getStrudel();
    const code = `setcps(${cps}); freq("${hzs.join(' ')}").s("${sound}").slow(${N / 2})`;
    await s.evaluate(code);
  }

  /** V2 path — gamakas, timbres, tala, breath, optional tanpura. */
  private async playV2(m: Melakarta, opts: PlayOptions): Promise<void> {
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
    const s = await this.getStrudel();
    await s.evaluate(composed.ragaCode);
    if (composed.tanpuraCode) {
      // Tanpura plays in parallel — Strudel maintains a single global
      // pattern; we can run it as a concurrent "stack" by combining via cat.
      // For the simple case (most users won't notice), evaluate it as a
      // separate call after a short delay so it doesn't replace the main pattern.
      // TODO(P3): switch to `stack(rg, tan).play()` once the tanpura scheduler
      // is implemented as a polyphonic layer.
      // For now: log it; UI exposes a separate "drone" button.
      // eslint-disable-next-line no-console
      console.info('[Nadashakti V2] tanpura code prepared:', composed.tanpuraCode);
    }
  }

  async stop(): Promise<void> {
    if (!this.ready) return;
    const s = await this.ready;
    s.hush();
  }

  /** V2 helper — render a raaga to an OfflineAudioContext WAV blob URL.
   *  Phase 2.3 (T-043..T-046). Returns null on browsers without OfflineAudioContext. */
  async renderWav(melakartaNum: number, opts: PlayOptions = {}): Promise<string | null> {
    const m = getMelakarta(melakartaNum);
    if (!m) throw new Error(`Unknown melakarta: ${melakartaNum}`);
    const { renderToWavBlobUrl } = await import('./v2/render/offline');
    return renderToWavBlobUrl(m, opts);
  }
}

let _instance: RaagaPlayer | null = null;
export const getRaagaPlayer = (): RaagaPlayer => (_instance ??= new RaagaPlayer());

export type { Melakarta } from './melakartas';
export { ratioOf };
