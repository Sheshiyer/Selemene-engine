/**
 * Tarot Witness Prompts - Non-prescriptive inquiry generation
 * Prompts INVITE reflection without telling the user what to do
 */

import { SeededRandom, getDefaultSeed } from '../../utils/random'
import type { WitnessPrompt } from '../../types'
import type { TarotReading, PositionReading } from './reading'
import { getKeyCards } from './reading'

/**
 * Templates for generating non-prescriptive witness prompts
 * These INVITE reflection rather than prescribe action
 */
const CARD_REFLECTION_TEMPLATES = [
  'What does {card} mirror in your current situation?',
  'Where in your life might the energy of {card} be present?',
  'What arises when you sit with the image of {card}?',
  'How does {card} speak to what you are experiencing?',
  'What feelings emerge as you contemplate {card}?',
  'In what ways might {card} reflect an inner truth?',
  'What does your response to {card} reveal about this moment?',
  'Where do you notice the themes of {card} appearing in your life?',
]

const POSITION_REFLECTION_TEMPLATES = [
  'As you consider {card} in the position of {position}, what stands out?',
  'What does it mean to you that {card} appears here, representing {position_meaning}?',
  'How might {card} illuminate your understanding of {position}?',
]

const REVERSED_TEMPLATES = [
  'With {card} appearing reversed, what inner landscape might it be reflecting?',
  'The reversed {card} invites you to look within—what do you find there?',
  'What shadow aspect of {card} resonates with your current experience?',
]

const RELATIONSHIP_TEMPLATES = [
  'How do {card1} and {card2} seem to speak to each other in this reading?',
  'What story emerges between {card1} and {card2}?',
  'Notice the interplay between {card1} and {card2}—what patterns arise?',
]

const OVERALL_TEMPLATES = [
  'What overall feeling or intuition arises from this reading?',
  'What question does this reading seem to be asking you?',
  'As you take in the full spread, what truth wants to be acknowledged?',
  'What does this reading illuminate that you may not have seen before?',
]

function fillTemplate(template: string, replacements: Record<string, string>): string {
  let result = template
  for (const [key, value] of Object.entries(replacements)) {
    result = result.replace(new RegExp(`\\{${key}\\}`, 'g'), value)
  }
  return result
}

/**
 * Generate witness prompts for a single card
 */
function generateCardPrompt(positionReading: PositionReading, rng: SeededRandom): WitnessPrompt {
  const { position, drawnCard } = positionReading
  const cardName = drawnCard.card.name

  let template: string
  let themes: string[]

  if (drawnCard.isReversed) {
    template = rng.pick(REVERSED_TEMPLATES)
    themes = ['shadow', 'inversion', 'inner work', ...drawnCard.card.keywords]
  } else {
    // Mix of card-focused and position-focused templates
    const usePositionTemplate = rng.nextBool(0.3)
    if (usePositionTemplate) {
      template = rng.pick(POSITION_REFLECTION_TEMPLATES)
    } else {
      template = rng.pick(CARD_REFLECTION_TEMPLATES)
    }
    themes = drawnCard.card.keywords
  }

  const prompt = fillTemplate(template, {
    card: cardName,
    position: position.name,
    position_meaning: position.meaning,
  })

  return {
    prompt,
    context: `${cardName} in ${position.name} position${drawnCard.isReversed ? ' (reversed)' : ''}`,
    themes,
  }
}

/**
 * Generate a relationship prompt between two cards
 */
function generateRelationshipPrompt(
  card1: PositionReading,
  card2: PositionReading,
  rng: SeededRandom,
): WitnessPrompt {
  const template = rng.pick(RELATIONSHIP_TEMPLATES)
  const prompt = fillTemplate(template, {
    card1: card1.drawnCard.card.name,
    card2: card2.drawnCard.card.name,
  })

  return {
    prompt,
    context: `Connection between ${card1.position.name} and ${card2.position.name}`,
    themes: [...card1.drawnCard.card.keywords, ...card2.drawnCard.card.keywords],
  }
}

/**
 * Generate an overall reading prompt
 */
function generateOverallPrompt(rng: SeededRandom): WitnessPrompt {
  return {
    prompt: rng.pick(OVERALL_TEMPLATES),
    context: 'Overall reading reflection',
    themes: ['wholeness', 'integration', 'meaning'],
  }
}

/**
 * Generate witness prompts for a tarot reading
 * Returns 2-3 non-prescriptive prompts based on key cards
 */
export function generateWitnessPrompts(reading: TarotReading, seed?: number): WitnessPrompt[] {
  const rng = new SeededRandom(seed ?? getDefaultSeed())
  const prompts: WitnessPrompt[] = []
  const keyCards = getKeyCards(reading)

  // Always generate at least one card-specific prompt
  if (keyCards.length > 0) {
    const primaryCard = rng.pick(keyCards)
    prompts.push(generateCardPrompt(primaryCard, rng))
  }

  // For multi-card spreads, add a relationship prompt
  if (reading.positions.length > 1) {
    const positions = [...reading.positions]
    rng.shuffle(positions)
    if (positions.length >= 2) {
      prompts.push(generateRelationshipPrompt(positions[0], positions[1], rng))
    }
  }

  // Add an overall reflection prompt for larger spreads
  if (reading.positions.length >= 3) {
    prompts.push(generateOverallPrompt(rng))
  }

  // Ensure we have 2-3 prompts
  while (prompts.length < 2 && keyCards.length > 0) {
    const card = rng.pick(keyCards)
    prompts.push(generateCardPrompt(card, rng))
  }

  // Cap at 3 prompts
  return prompts.slice(0, 3)
}

/**
 * Generate prompts specifically for a question-based reading
 */
export function generateQuestionBasedPrompts(
  reading: TarotReading,
  seed?: number,
): WitnessPrompt[] {
  const basePrompts = generateWitnessPrompts(reading, seed)

  // Add question-specific prompt if a question was provided
  if (reading.question) {
    const questionPrompt: WitnessPrompt = {
      prompt: `How do these cards speak to your question: "${reading.question}"?`,
      context: 'Question reflection',
      themes: ['inquiry', 'meaning', 'resonance'],
    }
    return [questionPrompt, ...basePrompts.slice(0, 2)]
  }

  return basePrompts
}
