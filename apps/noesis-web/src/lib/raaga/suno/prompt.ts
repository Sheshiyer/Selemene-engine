// Suno prompt builder for ragas. Maps melakarta metadata → natural-language
// prompt that Suno will accept (it cannot consume structured swara data).
//
// Design rules:
//   - Always `make_instrumental: true`; explicitly say "no vocals" in prompt
//   - Always mention "Indian classical raga" + the specific name
//   - Style adverbs vary by SunoStyle
//   - Mood adjectives derived from raga's chakra position (foundational at low chakras,
//     transcendent at high chakras)
//   - Duration target embedded in prompt + tags
//   - Tags string keeps to <120 chars (Suno truncates long tags)

import { MELAKARTAS } from '../melakartas';
import type { SunoStyle, SunoCustomGenerateRequest } from './types';

const STYLE_DESCRIPTORS: Record<SunoStyle, {
  bpm: string;
  instruments: string;
  mood: string;
  tags: string;
}> = {
  ambient: {
    bpm: '60-70 BPM',
    instruments: 'sustained tanpura drone, sitar lead',
    mood: 'spacious, contemplative, slow-evolving',
    tags: 'indian classical, ambient, drone, instrumental, raga',
  },
  meditative: {
    bpm: '50-60 BPM',
    instruments: 'tanpura drone, bansuri flute, soft tabla',
    mood: 'devotional, peaceful, breath-paced',
    tags: 'indian classical, meditation, raga, instrumental, devotional',
  },
  cinematic: {
    bpm: '80-90 BPM',
    instruments: 'orchestral strings, sarangi, sitar, soft percussion',
    mood: 'epic, grand, narrative-arc',
    tags: 'indian classical, cinematic, orchestral, raga, instrumental',
  },
  acid: {
    bpm: '124-132 BPM',
    instruments: 'TR-808 drums, TB-303 squelch bassline, Jupiter-8 lead, sustained drone',
    mood: 'hypnotic, driving, charanjit-singh-1982-style',
    tags: 'indian classical, acid house, electronic, raga, instrumental, charanjit singh',
  },
};

const CHAKRA_MOOD: Record<number, string> = {
  1:  'earth-foundation, grounding, root-chakra',
  2:  'flow, eyes-open-perception, sacral',
  3:  'fire-activation, transformative, solar-plexus',
  4:  'wisdom-seat, hip-grounded, structural',
  5:  'directed-intention, lower-belly, focused',
  6:  'rhythmic-center, navel, seasonal',
  7:  'sage-discernment, solar-plexus, clarity',
  8:  'heart-opening, devotional-warmth',
  9:  'creative-expansion, chest, breath-of-creation',
  10: 'directional-power, shoulders, expansive',
  11: 'fierce-expression, throat, voice-unblocked',
  12: 'solar-consciousness, crown, luminous',
};

/**
 * Build a complete Suno custom_generate request body for a given melakarta.
 * Returns the JSON object ready to POST to /api/custom_generate.
 */
export const buildSunoPrompt = (
  melakartaNum: number,
  style: SunoStyle = 'ambient',
  durationSec: number = 45,
): SunoCustomGenerateRequest => {
  if (melakartaNum < 1 || melakartaNum > 72) {
    throw new RangeError(`melakarta_num must be 1..72 (got ${melakartaNum})`);
  }
  const m = MELAKARTAS[melakartaNum - 1];
  const desc = STYLE_DESCRIPTORS[style];
  const chakraMood = CHAKRA_MOOD[m.chakra] ?? 'meditative';

  const prompt = [
    `Indian classical raga ${m.name} (Carnatic melakarta #${melakartaNum}),`,
    `${desc.mood} ${style} instrumental,`,
    `no vocals, played at ${desc.bpm}`,
    `with ${desc.instruments}.`,
    `Mood: ${chakraMood}.`,
    `Approximately ${durationSec} seconds.`,
  ].join(' ');

  // Trim to ≤300 chars (Suno guidance) — truncate gracefully if over
  const trimmed = prompt.length > 300 ? prompt.slice(0, 297) + '...' : prompt;

  return {
    prompt: trimmed,
    tags: desc.tags,
    title: `${m.name} (#${melakartaNum}) — ${style}`,
    make_instrumental: true,
    wait_audio: false,
  };
};

/** Helper: get prompts for all 72 melakartas in a given style. */
export const buildAllPrompts = (
  style: SunoStyle = 'ambient',
  durationSec: number = 45,
): SunoCustomGenerateRequest[] => {
  return MELAKARTAS.map((m) => buildSunoPrompt(m.num, style, durationSec));
};
