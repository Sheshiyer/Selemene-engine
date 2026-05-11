// Composition layer — the v2 integration point. Takes a melakarta + v2
// PlayOptions and emits the full Strudel code that activates gamakas,
// timbres, tala accents, breath-paced articulation, and (optionally) a
// tanpura drone underneath.
//
// This is the function `playRaagaFull` calls. Pure — no Strudel imports.

import { ratioOf } from '../shrutis';
import type { Melakarta } from '../melakartas';
import { applyGamaka, type GamakaAnnotation } from './gamakas/apply';
import type { Gamaka } from './gamakas/types';
import { TALAS } from './talas/data';
import type { TalaName } from './talas/types';
import { emitTala } from './talas/emitter';
import { BREATHS, type BreathName } from './breaths/data';
import { breathToArticulation } from './breaths/strudel';
import { TIMBRES, timbreStrudelSuffix, type Timbre } from './samples/timbres';
import { buildTanpuraCode } from './tanpura';
import { PALTAS } from './arpeggios/patterns';
import type { PaltaName } from './arpeggios/types';
import { renderAlaap } from './alaap/render';
import type { AlaapPhase } from './alaap/types';

export interface ComposeInput {
  melakarta: Melakarta;
  rootHz?: number;
  cps?: number;
  direction?: 'arohana' | 'avarohana' | 'both';
  timbre?: Timbre;
  gamakas?: readonly GamakaAnnotation[];
  defaultGamaka?: Gamaka;
  tala?: TalaName;
  breath?: BreathName;
  tanpura?: boolean;

  // ── V2.5 mode selectors ─────────────────────────────────────────────
  /** Override the swara sequence with a palta (Carnatic permutation pattern).
   *  Mutually exclusive with `alaap`. */
  palta?: PaltaName;
  /** Replace the entire pattern with an alaap composition (rhythm-free
   *  multi-phase exploration). Forces tala=undefined, tanpura=true. */
  alaap?: boolean;
  /** Which alaap phases to include. Defaults to all three (vilambit→madhya→drut). */
  alaapPhases?: readonly AlaapPhase[];
}

export interface ComposeOutput {
  /** Main raga code. Pass to evaluate(). */
  ragaCode: string;
  /** Tanpura code if tanpura=true; null otherwise. Send via evaluate() on its
   *  own pattern slot so it loops independently. */
  tanpuraCode: string | null;
  /** Total duration in seconds. */
  durationSeconds: number;
  /** Sanity metadata. */
  meta: {
    swaraCount: number;
    appliedGamakas: readonly string[];
    timbre: Timbre;
    talaName?: TalaName;
    breathName?: BreathName;
    paltaName?: PaltaName;
    alaap?: boolean;
    alaapPhases?: readonly AlaapPhase[];
  };
}

const buildSwaras = (m: Melakarta, dir: ComposeInput['direction']): readonly number[] => {
  const aroha = m.arohana.map(ratioOf);
  const avaro = m.avarohana.map(ratioOf);
  switch (dir) {
    case 'arohana':   return aroha;
    case 'avarohana': return avaro;
    case 'both':
    default:          return [...aroha, ...avaro.slice(1)];
  }
};

export const compose = (input: ComposeInput): ComposeOutput => {
  const {
    melakarta,
    rootHz = 220,
    direction = 'both',
    timbre = 'sitar',
    gamakas = [],
    defaultGamaka = { kind: 'none' },
    tala,
    breath,
    tanpura = false,
    palta,
    alaap = false,
    alaapPhases = ['vilambit', 'madhya', 'drut'],
  } = input;

  // 1. Resolve cps: alaap > explicit > breath > tala > default
  const breathArt = breath ? breathToArticulation(BREATHS[breath]) : null;

  // 2. Build the 8-element swara Hz array (always Sa..Sa' regardless of mode)
  const fullSwaras = [
    ...melakarta.arohana.map(ratioOf),
  ];
  const fullHzs = fullSwaras.map((r) => +(r * rootHz).toFixed(4));

  // ── ALAAP mode — completely overrides the normal swara sequence ──────
  if (alaap) {
    const alaapOut = renderAlaap({
      hzs: fullHzs,
      config: { phases: alaapPhases, tanpura: true },
    });
    const t = TIMBRES[timbre];
    // Alaap uses generous ADSR for sustained, contemplative attack
    let chain = `freq("${alaapOut.miniNotation}")`;
    chain += `.s("${t.sound}")`;
    chain += `.attack(${Math.max(t.attack, 0.2)})`;
    chain += `.decay(${Math.max(t.decay, 0.3)})`;
    chain += `.sustain(${Math.max(t.sustain, 0.85)})`;
    chain += `.release(${Math.max(t.release, 1.5)})`;
    chain += `.gain(${t.gain * 0.85})`;
    if (t.lpf != null) chain += `.lpf(${t.lpf})`;
    if (t.lpq != null) chain += `.lpq(${t.lpq})`;
    if (t.detune != null && t.detune !== 0) chain += `.detune(${t.detune})`;
    return {
      ragaCode: `setcps(${alaapOut.cps}); ${chain}`,
      tanpuraCode: buildTanpuraCode({ rootHz, gain: 0.4 }).code,
      durationSeconds: alaapOut.durationSeconds,
      meta: {
        swaraCount: fullHzs.length,
        appliedGamakas: alaapPhases,
        timbre,
        alaap: true,
        alaapPhases,
      },
    };
  }

  // ── Normal / palta mode ──────────────────────────────────────────────
  let hzs: readonly number[];
  if (palta) {
    // Palta replaces the swara sequence with a permutation pattern
    const swaraIndices = [0, 1, 2, 3, 4, 5, 6, 7];  // Sa..Sa'
    const expanded = PALTAS[palta].generate(swaraIndices);
    hzs = expanded.map((i) => fullHzs[i] ?? fullHzs[fullHzs.length - 1]);
  } else {
    // Standard arohana / avarohana / both
    const ratios = buildSwaras(melakarta, direction);
    hzs = ratios.map((r) => +(r * rootHz).toFixed(4));
  }

  const cps = input.cps ?? breathArt?.cps ?? 0.5;

  // 3. Apply gamakas → freq() mini-notation
  // Map raaga-direction terms to gamaka-annotation terms (ascend/descend).
  const gamakaDirection: 'up' | 'down' | 'both' =
    direction === 'arohana' ? 'up' :
    direction === 'avarohana' ? 'down' :
    'both';
  const { miniNotation, durationSeconds, appliedKinds } = applyGamaka({
    hzs, annotations: gamakas, defaultGamaka, cps, direction: gamakaDirection, holdCycles: 1,
  });

  // 4. Build chain: freq(notation) + timbre + breath ADSR override + tala gain
  const t = TIMBRES[timbre];
  let chain = `freq("${miniNotation}")`;

  // Timbre supplies sound/ADSR; if a breath is set, breath wins on attack/release
  if (breathArt) {
    chain += `.s("${t.sound}")`;
    chain += `.attack(${breathArt.attack})`;
    chain += `.decay(${breathArt.decay})`;
    chain += `.sustain(${breathArt.sustain})`;
    chain += `.release(${breathArt.release})`;
    chain += `.gain(${t.gain})`;
    if (t.lpf != null) chain += `.lpf(${t.lpf})`;
    if (t.lpq != null) chain += `.lpq(${t.lpq})`;
    if (t.hpf != null) chain += `.hpf(${t.hpf})`;
  } else {
    chain += timbreStrudelSuffix(t);
  }

  // 5. Tala accent layer
  if (tala) {
    const talaSpec = TALAS[tala];
    const { gainChain } = emitTala(talaSpec, { swaraCount: hzs.length });
    chain += gainChain;
  }

  // 6. slow() factor — stretch pattern over the right number of cps cycles
  const slowFactor = hzs.length / 2;
  chain += `.slow(${slowFactor})`;

  const ragaCode = `setcps(${cps}); ${chain}`;

  // 7. Optional tanpura drone (separate code, played in parallel)
  const tanpuraCode = tanpura
    ? buildTanpuraCode({ rootHz, gain: 0.35 }).code
    : null;

  return {
    ragaCode,
    tanpuraCode,
    durationSeconds,
    meta: {
      swaraCount: hzs.length,
      appliedGamakas: appliedKinds,
      timbre,
      talaName: tala,
      breathName: breath,
      paltaName: palta,
    },
  };
};
