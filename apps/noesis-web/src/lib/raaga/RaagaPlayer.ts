// Strudel-backed raaga player.
//
// `@strudel/web` does not expose `xen()` at the user-API level (it lives in
// `@strudel/tonal` but isn't bundled into the web build). Rather than fight
// the bundle, we bake just-intonation 22-shruti ratios into absolute Hz on
// our side and feed Strudel's `freq()` directly. Audio is bit-identical to
// the xen path — every ratio is exact, no 12-TET quantization anywhere.

import { ratioOf } from './shrutis';
import { getMelakarta, type Melakarta } from './melakartas';

interface PlayOptions {
  /** Hz for Sa. Defaults to 220 (A3, comfortable male tonic). */
  rootHz?: number;
  /** Cycles per second. Defaults to 0.5. */
  cps?: number;
  /** Override timbre. Defaults to "sine". */
  sound?: string;
  /** "arohana" | "avarohana" | "both" (ascend then descend). Default "both". */
  direction?: 'arohana' | 'avarohana' | 'both';
}

type StrudelModule = typeof import('@strudel/web');

export class RaagaPlayer {
  private ready: Promise<StrudelModule> | null = null;

  /** Lazy-init Strudel — must be called from inside a user gesture. */
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

  async play(melakartaNum: number, opts: PlayOptions = {}): Promise<void> {
    const m = getMelakarta(melakartaNum);
    if (!m) throw new Error(`Unknown melakarta: ${melakartaNum}`);

    const { rootHz = 220, cps = 0.5, sound = 'sine', direction = 'both' } = opts;
    const ratios = this.buildRatios(m, direction);
    const hzs = ratios.map((r) => +(r * rootHz).toFixed(4));
    const N = hzs.length;

    const s = await this.getStrudel();
    // setcps must live inside the evaluated string (Strudel's transpiler
    // injects it at parse time; it's not a top-level export).
    const code = `setcps(${cps}); freq("${hzs.join(' ')}").s("${sound}").slow(${N / 2})`;
    await s.evaluate(code);
  }

  async stop(): Promise<void> {
    if (!this.ready) return;
    const s = await this.ready;
    s.hush();
  }
}

let _instance: RaagaPlayer | null = null;
export const getRaagaPlayer = (): RaagaPlayer => (_instance ??= new RaagaPlayer());

export type { Melakarta } from './melakartas';
export { ratioOf };
