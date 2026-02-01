/**
 * Enneagram Witness Prompts - Non-prescriptive inquiry generation
 * Prompts INVITE observation of patterns, not identification AS a type.
 * The Enneagram describes patterns, not fixed identities.
 */

import type { WitnessPrompt } from '../../types'
import { SeededRandom, getDefaultSeed } from '../../utils/random'
import type { EnneagramNumber, EnneagramType } from './wisdom'
import { getDisintegrationConnection, getEnneagramType, getIntegrationConnection } from './wisdom'

/**
 * Core fear observation templates
 * Invite noticing where the pattern arises, not identifying with it
 */
const CORE_FEAR_TEMPLATES = [
  'Where do you notice the pattern of {coreFear} arising in your life?',
  'When does the fear of {coreFear} seem most present?',
  'What situations tend to activate the concern about {coreFear}?',
  'How does your body respond when the fear of {coreFear} surfaces?',
]

/**
 * Core desire observation templates
 */
const CORE_DESIRE_TEMPLATES = [
  'What arises when you consider the longing for {coreDesire}?',
  'Where do you notice the pull toward {coreDesire} most strongly?',
  'How does striving for {coreDesire} shape your daily choices?',
  'When does the desire for {coreDesire} feel most urgent?',
]

/**
 * Pattern observation templates (not type identification)
 */
const PATTERN_TEMPLATES = [
  'Can you observe the {pattern} pattern without trying to change it?',
  'What happens when you simply witness the {pattern} tendency?',
  'Where does the {pattern} pattern serve you? Where does it limit you?',
  'How old is this {pattern} pattern? When did it first appear?',
]

/**
 * Body-centered inquiry templates
 */
const BODY_TEMPLATES = [
  'What happens in your body when {trigger} occurs?',
  'Where do you feel {emotion} most strongly in your body?',
  'What physical sensations accompany the {pattern} pattern?',
  'How does your breath change when {trigger} happens?',
]

/**
 * Integration point templates (growth direction)
 */
const INTEGRATION_TEMPLATES = [
  'What would it feel like to access the {quality} of Type {integrationPoint}?',
  'When have you experienced moments of {quality}?',
  'What supports you in moving toward {quality}?',
  'How might {quality} already be available to you?',
]

/**
 * Wing observation templates
 */
const WING_TEMPLATES = [
  'How do you notice the influence of {wingType} patterns in your experience?',
  'Where do the {wingName} qualities show up alongside your core patterns?',
  'What does the {wingName} bring to how you navigate the world?',
]

/**
 * General self-inquiry templates (type-agnostic)
 */
const GENERAL_TEMPLATES = [
  'What does this pattern protect you from?',
  'When did this way of being first become necessary?',
  'What becomes possible when you can simply observe this pattern?',
  'How might this pattern be an attempt to get something you need?',
  'What would you have to feel if this pattern dissolved?',
]

/**
 * Type-specific pattern descriptions for templates
 */
const TYPE_PATTERNS: Record<
  EnneagramNumber,
  {
    patterns: string[]
    triggers: string[]
    emotions: string[]
    qualities: string[] // Integration qualities
  }
> = {
  1: {
    patterns: [
      'correcting',
      'improving',
      'criticizing self',
      'noticing imperfection',
      'holding back anger',
    ],
    triggers: ['things are done incorrectly', 'standards are not met', 'someone cuts corners'],
    emotions: ['frustration', 'resentment', 'righteous anger', 'disappointment'],
    qualities: ['spontaneity', 'joy', 'playfulness', 'relaxation'],
  },
  2: {
    patterns: [
      'helping',
      'giving',
      'anticipating needs',
      'seeking appreciation',
      'denying own needs',
    ],
    triggers: ['your help is not acknowledged', 'someone rejects your care', 'you feel unneeded'],
    emotions: ['pride', 'resentment', 'longing for appreciation', 'feeling indispensable'],
    qualities: ['self-awareness', 'emotional honesty', 'attending to own needs', 'depth'],
  },
  3: {
    patterns: ['achieving', 'performing', 'adapting image', 'comparing', 'staying busy'],
    triggers: [
      'you experience failure',
      'you are not seen as successful',
      'you have nothing to do',
    ],
    emotions: ['anxiety about worth', 'emptiness', 'competitiveness', 'drive'],
    qualities: ['loyalty', 'authenticity', 'commitment to others', 'slowing down'],
  },
  4: {
    patterns: [
      'longing',
      'comparing self to others',
      'intensifying emotions',
      'feeling misunderstood',
      'seeking uniqueness',
    ],
    triggers: [
      'others seem to have what you lack',
      'you feel ordinary',
      'your feelings are dismissed',
    ],
    emotions: ['melancholy', 'envy', 'longing', 'feeling deficient'],
    qualities: ['objectivity', 'discipline', 'emotional equanimity', 'practical action'],
  },
  5: {
    patterns: ['observing', 'withdrawing', 'analyzing', 'minimizing needs', 'hoarding energy'],
    triggers: ['too many demands are made', 'you feel intruded upon', 'you lack competence'],
    emotions: ['overwhelm', 'detachment', 'fear of inadequacy', 'mental exhaustion'],
    qualities: ['engagement', 'confidence', 'decisiveness', 'taking action'],
  },
  6: {
    patterns: [
      'doubting',
      'scanning for threats',
      'seeking reassurance',
      'testing loyalty',
      'preparing for worst',
    ],
    triggers: ['you feel unsupported', 'authority is unreliable', 'danger seems imminent'],
    emotions: ['anxiety', 'suspicion', 'vigilance', 'ambivalence'],
    qualities: ['inner peace', 'trust', 'acceptance', 'calm presence'],
  },
  7: {
    patterns: ['planning', 'reframing', 'seeking options', 'avoiding pain', 'staying positive'],
    triggers: ['you feel trapped', 'options are limited', 'pain must be faced'],
    emotions: ['restlessness', 'anxiety', 'frustration with limits', 'fear of deprivation'],
    qualities: ['depth', 'focus', 'staying with difficulty', 'inner stillness'],
  },
  8: {
    patterns: ['taking charge', 'confronting', 'protecting', 'asserting control', 'testing others'],
    triggers: ['you feel vulnerable', 'someone tries to control you', 'injustice occurs'],
    emotions: ['anger', 'intensity', 'protectiveness', 'desire for control'],
    qualities: ['tenderness', 'vulnerability', 'care for others', 'openheartedness'],
  },
  9: {
    patterns: ['merging', 'accommodating', 'numbing out', 'avoiding conflict', 'going along'],
    triggers: [
      'conflict arises',
      'your position must be stated',
      'others push for your preference',
    ],
    emotions: ['inertia', 'resistance', 'numbness', 'hidden anger'],
    qualities: ['energy', 'self-development', 'asserting presence', 'taking initiative'],
  },
}

function fillTemplate(template: string, replacements: Record<string, string>): string {
  let result = template
  for (const [key, value] of Object.entries(replacements)) {
    result = result.replace(new RegExp(`\\{${key}\\}`, 'g'), value)
  }
  return result
}

/**
 * Generate witness prompts for a specific Enneagram type
 */
export function generateTypeWitnessPrompts(
  type: EnneagramNumber,
  wing?: EnneagramNumber,
  seed?: number,
): WitnessPrompt[] {
  const rng = new SeededRandom(seed ?? getDefaultSeed())
  const prompts: WitnessPrompt[] = []
  const typeData = getEnneagramType(type)
  const typePatterns = TYPE_PATTERNS[type]
  const integrationConn = getIntegrationConnection(type)

  // Core fear prompt
  const fearTemplate = rng.pick(CORE_FEAR_TEMPLATES)
  prompts.push({
    prompt: fillTemplate(fearTemplate, { coreFear: typeData.coreFear.toLowerCase() }),
    context: `Type ${type} - ${typeData.name} core fear`,
    themes: ['fear', 'pattern recognition', typeData.name.toLowerCase()],
  })

  // Pattern observation prompt
  const patternTemplate = rng.pick(PATTERN_TEMPLATES)
  const pattern = rng.pick(typePatterns.patterns)
  prompts.push({
    prompt: fillTemplate(patternTemplate, { pattern }),
    context: `Type ${type} - Behavioral pattern`,
    themes: ['observation', 'non-judgment', 'awareness'],
  })

  // Body-centered inquiry
  const bodyTemplate = rng.pick(BODY_TEMPLATES)
  const trigger = rng.pick(typePatterns.triggers)
  const emotion = rng.pick(typePatterns.emotions)
  prompts.push({
    prompt: fillTemplate(bodyTemplate, { trigger, emotion, pattern }),
    context: `Type ${type} - Somatic awareness`,
    themes: ['body', 'sensation', 'embodiment'],
  })

  // Integration prompt (growth direction)
  if (integrationConn) {
    const integrationTemplate = rng.pick(INTEGRATION_TEMPLATES)
    const quality = rng.pick(typePatterns.qualities)
    prompts.push({
      prompt: fillTemplate(integrationTemplate, {
        quality,
        integrationPoint: integrationConn.to.toString(),
      }),
      context: `Integration to Type ${integrationConn.to}`,
      themes: ['growth', 'integration', 'potential'],
    })
  }

  // Wing-influenced prompt if wing provided
  if (wing) {
    const wingType = getEnneagramType(wing)
    const wingTemplate = rng.pick(WING_TEMPLATES)
    prompts.push({
      prompt: fillTemplate(wingTemplate, {
        wingType: wing.toString(),
        wingName: wingType.name,
      }),
      context: `Wing influence - Type ${wing}`,
      themes: ['wing', wingType.name.toLowerCase(), 'nuance'],
    })
  }

  // General self-inquiry
  prompts.push({
    prompt: rng.pick(GENERAL_TEMPLATES),
    context: 'Open inquiry',
    themes: ['self-inquiry', 'curiosity', 'compassion'],
  })

  // Return 3-4 prompts
  rng.shuffle(prompts)
  return prompts.slice(0, wing ? 4 : 3)
}

/**
 * Generate prompts based on assessment results
 */
export function generateAssessmentWitnessPrompts(
  primaryType: EnneagramNumber,
  wing: EnneagramNumber,
  confidence: number,
  seed?: number,
): WitnessPrompt[] {
  const rng = new SeededRandom(seed ?? getDefaultSeed())
  const prompts = generateTypeWitnessPrompts(primaryType, wing, seed)

  // Add confidence-adjusted prompt
  if (confidence < 0.3) {
    prompts.unshift({
      prompt:
        'Rather than seeking a single type, what if you explored how multiple patterns live in you?',
      context: 'Multiple patterns recognized',
      themes: ['complexity', 'nuance', 'self-compassion'],
    })
  } else if (confidence > 0.7) {
    const typeData = getEnneagramType(primaryType)
    prompts.unshift({
      prompt: `Notice: recognizing ${typeData.name} patterns does not mean you ARE this type. What opens when you hold this lightly?`,
      context: 'Pattern vs. identity',
      themes: ['non-identification', 'freedom', 'awareness'],
    })
  }

  return prompts.slice(0, 4)
}

/**
 * Generate prompts for stress and growth directions
 */
export function generateMovementPrompts(type: EnneagramNumber, seed?: number): WitnessPrompt[] {
  const rng = new SeededRandom(seed ?? getDefaultSeed())
  const prompts: WitnessPrompt[] = []

  const integrationConn = getIntegrationConnection(type)
  const disintegrationConn = getDisintegrationConnection(type)

  if (integrationConn) {
    const integrationType = getEnneagramType(integrationConn.to)
    prompts.push({
      prompt: `When have you noticed yourself accessing ${integrationType.name.toLowerCase()} qualities? What supported that?`,
      context: `Growth toward Type ${integrationConn.to}`,
      themes: ['growth', 'integration', integrationType.name.toLowerCase()],
    })
  }

  if (disintegrationConn) {
    const disintegrationType = getEnneagramType(disintegrationConn.to)
    prompts.push({
      prompt: `Under stress, do you notice yourself moving toward ${disintegrationType.name.toLowerCase()} patterns? What triggers this?`,
      context: `Stress movement toward Type ${disintegrationConn.to}`,
      themes: ['stress', 'awareness', 'self-compassion'],
    })
  }

  prompts.push({
    prompt: 'What supports you in staying present rather than moving into automatic patterns?',
    context: 'Presence practice',
    themes: ['presence', 'awareness', 'choice'],
  })

  rng.shuffle(prompts)
  return prompts
}
