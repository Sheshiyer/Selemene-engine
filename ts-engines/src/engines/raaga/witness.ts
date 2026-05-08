import type { WitnessPrompt } from '../../types'
import type { Melakarta } from './wisdom'

export function generateWitnessPrompts(melakarta: Melakarta, dosha?: string): WitnessPrompt[] {
  return [
    {
      prompt: `As you listen to ${melakarta.name}, notice where in your body the sound lands first. What does it illuminate?`,
      context: `Raga ${melakarta.name} (melakarta #${melakarta.num}) uses ${melakarta.ma_type === 'prati' ? 'tivra (sharp) Madhyama' : 'shuddha Madhyama'} — a quality that shapes its emotional texture.`,
      themes: ['listening', 'somatic awareness', 'resonance'],
    },
    {
      prompt: `The ascending and descending forms of this raga differ slightly in character. What quality do you notice in the ascent versus the descent?`,
      context: `The arohana–avarohana arc creates a journey: moving upward carries one quality; returning carries another.`,
      themes: ['duality', 'direction', 'musical intelligence'],
    },
    ...(dosha
      ? [
          {
            prompt: `This raga is recommended for your ${dosha} constitution. How does it address an imbalance you currently feel?`,
            context: `Ayurvedic sound therapy suggests that specific interval relationships within a raga can modulate vata, pitta, or kapha energies.`,
            themes: ['Ayurveda', 'balance', 'self-inquiry'],
          },
        ]
      : []),
  ]
}
