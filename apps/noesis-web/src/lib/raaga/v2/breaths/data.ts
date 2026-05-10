// 12 breath patterns extracted from RAAGA_ENGINE.md, one per chakra.
// Each maps a chakra (1..12) to a pranayama-style breath rhythm + the
// raga-time window in which it's most potent.
//
// Source: raagaegnin/RAAGA_ENGINE.md § "12 Melakarta Chakras → Body Zones"

export type BreathName =
  | 'box-4'              // 4-4-4-4 (Earth rhythm) — Indu
  | 'calming-4-7-8'      // Calming descent — Netra
  | 'bhastrika'          // Bellows — Agni
  | 'coherence-6-0-6-0'  // 6-0-6-0 coherence — Veda
  | 'kapalabhati'        // Skull-shining — Bana
  | 'nadi-shodhana'      // Alternate nostril — Rutu
  | 'ujjayi'             // Victorious — Rishi
  | 'heart-coherence-5-5'// 5-5 — Vasu
  | 'dirgha'             // Full yogic — Brahma
  | 'shoulder-roll'      // Shoulder-rolling — Disi
  | 'brahmari'           // Humming bee — Rudra
  | 'shitali';           // Cooling — Aditya

export interface Breath {
  name: BreathName;
  display: string;
  /** Inhale-hold-exhale-hold pattern in seconds (0 = no hold). */
  pattern: readonly [number, number, number, number];
  /** Approximate cycle period in seconds (sum of pattern). */
  cycleSeconds: number;
  /** Suggested chakra (1..12) — what RAAGA_ENGINE.md pairs with this breath. */
  chakra: number;
  /** Time window in IST 24h. */
  timeWindow: string;
  /** One-line therapeutic intent. */
  intent: string;
}

const breath = (
  name: BreathName,
  display: string,
  pattern: readonly [number, number, number, number],
  chakra: number,
  timeWindow: string,
  intent: string,
): Breath => ({
  name, display, pattern,
  cycleSeconds: pattern[0] + pattern[1] + pattern[2] + pattern[3],
  chakra, timeWindow, intent,
});

export const BREATHS: { readonly [K in BreathName]: Breath } = {
  'box-4':              breath('box-4',              '4-4-4-4 Box',         [4, 4, 4, 4],   1,  '3:00–5:00 AM',     'Grounding, foundational'),
  'calming-4-7-8':      breath('calming-4-7-8',      '4-7-8 Calm',          [4, 7, 8, 0],   2,  '5:00–7:00 AM',     'Anxiety release, sleep'),
  'bhastrika':          breath('bhastrika',          'Bhastrika (Bellows)', [0.5, 0, 0.5, 0], 3, '7:00–9:00 AM',     'Activate fire, energize'),
  'coherence-6-0-6-0':  breath('coherence-6-0-6-0',  '6-0-6-0 Coherence',   [6, 0, 6, 0],   4,  '9:00–11:00 AM',    'HRV coherence, focus'),
  'kapalabhati':        breath('kapalabhati',        'Kapālabhāti',         [0.3, 0, 0.3, 0], 5, '11:00 AM–1:00 PM', 'Skull-shining, purify'),
  'nadi-shodhana':      breath('nadi-shodhana',      'Nāḍī Śodhana',        [4, 4, 4, 4],   6,  '1:00–3:00 PM',     'Nostril alternation, balance'),
  'ujjayi':             breath('ujjayi',             'Ujjāyī',              [4, 0, 6, 0],   7,  '3:00–5:00 PM',     'Victorious, ocean breath'),
  'heart-coherence-5-5':breath('heart-coherence-5-5','5-5 Heart Coherence', [5, 0, 5, 0],   8,  '5:00–7:00 PM',     'Heart-rate coherence, love'),
  'dirgha':             breath('dirgha',             'Dīrgha (Full Yogic)', [6, 2, 8, 2],   9,  '7:00–9:00 PM',     'Three-part full breath'),
  'shoulder-roll':      breath('shoulder-roll',      'Shoulder-roll',       [4, 0, 4, 0],   10, '9:00–11:00 PM',    'Release tension, expand'),
  'brahmari':           breath('brahmari',           'Bhrāmarī (Bee)',      [4, 0, 12, 0],  11, '11:00 PM–1:00 AM', 'Humming bee, vagal calm'),
  'shitali':            breath('shitali',            'Śītalī (Cooling)',    [4, 2, 6, 0],   12, '1:00–3:00 AM',     'Cooling, introspection'),
};

export const ALL_BREATHS: readonly Breath[] = Object.values(BREATHS);

/** Look up the breath for a given chakra (1..12). Always returns one. */
export const breathForChakra = (chakraNum: number): Breath => {
  const b = ALL_BREATHS.find((x) => x.chakra === chakraNum);
  if (!b) throw new RangeError(`No breath for chakra ${chakraNum}`);
  return b;
};
