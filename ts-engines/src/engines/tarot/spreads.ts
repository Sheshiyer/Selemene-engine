/**
 * Tarot Spread Layouts - Defines various reading patterns
 */

export enum SpreadType {
  SINGLE_CARD = 'single_card',
  THREE_CARD = 'three_card',
  CELTIC_CROSS = 'celtic_cross',
  RELATIONSHIP = 'relationship',
  CAREER = 'career',
}

export interface SpreadPosition {
  position: number
  name: string
  meaning: string
}

export interface SpreadDefinition {
  type: SpreadType
  name: string
  description: string
  positions: SpreadPosition[]
}

const SINGLE_CARD_SPREAD: SpreadDefinition = {
  type: SpreadType.SINGLE_CARD,
  name: 'Single Card',
  description: 'A simple one-card draw for daily guidance or quick insight',
  positions: [
    {
      position: 0,
      name: 'The Card',
      meaning: 'The essence of your inquiry or the energy present now',
    },
  ],
}

const THREE_CARD_SPREAD: SpreadDefinition = {
  type: SpreadType.THREE_CARD,
  name: 'Three Card Spread',
  description: 'Past, Present, and Future - a classic spread for understanding the flow of time',
  positions: [
    {
      position: 0,
      name: 'Past',
      meaning: 'What has led to this moment; influences from the past',
    },
    {
      position: 1,
      name: 'Present',
      meaning: 'The current situation; where you stand now',
    },
    {
      position: 2,
      name: 'Future',
      meaning: 'What may unfold; potential outcomes and energies ahead',
    },
  ],
}

const CELTIC_CROSS_SPREAD: SpreadDefinition = {
  type: SpreadType.CELTIC_CROSS,
  name: 'Celtic Cross',
  description: 'A comprehensive 10-card spread providing deep insight into a situation',
  positions: [
    {
      position: 0,
      name: 'Present',
      meaning: 'The heart of the matter; your current situation',
    },
    {
      position: 1,
      name: 'Challenge',
      meaning: 'What crosses you; the immediate challenge or obstacle',
    },
    {
      position: 2,
      name: 'Foundation',
      meaning: 'The root of the situation; what lies beneath',
    },
    {
      position: 3,
      name: 'Recent Past',
      meaning: 'What is passing away; recent influences',
    },
    {
      position: 4,
      name: 'Crown',
      meaning: 'What crowns you; your aspirations or best possible outcome',
    },
    {
      position: 5,
      name: 'Near Future',
      meaning: 'What lies ahead; influences entering your life',
    },
    {
      position: 6,
      name: 'Self',
      meaning: 'How you see yourself in this situation',
    },
    {
      position: 7,
      name: 'Environment',
      meaning: 'External influences; how others see you or affect the situation',
    },
    {
      position: 8,
      name: 'Hopes and Fears',
      meaning: 'Your inner hopes and fears regarding the outcome',
    },
    {
      position: 9,
      name: 'Outcome',
      meaning: 'The potential resolution; where things may lead',
    },
  ],
}

const RELATIONSHIP_SPREAD: SpreadDefinition = {
  type: SpreadType.RELATIONSHIP,
  name: 'Relationship Spread',
  description: 'A 7-card spread examining the dynamics between two people or entities',
  positions: [
    {
      position: 0,
      name: 'You',
      meaning: 'Your current state and energy in the relationship',
    },
    {
      position: 1,
      name: 'Partner/Other',
      meaning: "The other person's current state and energy",
    },
    {
      position: 2,
      name: 'The Connection',
      meaning: 'The nature of your bond; what brings you together',
    },
    {
      position: 3,
      name: 'Strengths',
      meaning: 'What works well between you; the foundation of connection',
    },
    {
      position: 4,
      name: 'Challenges',
      meaning: 'What needs attention; areas of friction or growth',
    },
    {
      position: 5,
      name: 'External Influences',
      meaning: 'Outside factors affecting the relationship',
    },
    {
      position: 6,
      name: 'Potential',
      meaning: 'Where this connection may lead; underlying potential',
    },
  ],
}

const CAREER_SPREAD: SpreadDefinition = {
  type: SpreadType.CAREER,
  name: 'Career Spread',
  description: 'A 5-card spread for professional and vocational inquiry',
  positions: [
    {
      position: 0,
      name: 'Current Situation',
      meaning: 'Where you stand now in your career or work life',
    },
    {
      position: 1,
      name: 'Obstacles',
      meaning: 'What stands in your way; challenges to navigate',
    },
    {
      position: 2,
      name: 'Hidden Influences',
      meaning: 'Unseen factors affecting your professional path',
    },
    {
      position: 3,
      name: 'Guidance',
      meaning: 'Wisdom to consider; the energy to embody',
    },
    {
      position: 4,
      name: 'Potential Outcome',
      meaning: 'Where the current path may lead',
    },
  ],
}

export const SPREAD_DEFINITIONS: Record<SpreadType, SpreadDefinition> = {
  [SpreadType.SINGLE_CARD]: SINGLE_CARD_SPREAD,
  [SpreadType.THREE_CARD]: THREE_CARD_SPREAD,
  [SpreadType.CELTIC_CROSS]: CELTIC_CROSS_SPREAD,
  [SpreadType.RELATIONSHIP]: RELATIONSHIP_SPREAD,
  [SpreadType.CAREER]: CAREER_SPREAD,
}

export function getSpreadDefinition(type: SpreadType): SpreadDefinition {
  return SPREAD_DEFINITIONS[type]
}

export function getSpreadPositionCount(type: SpreadType): number {
  return SPREAD_DEFINITIONS[type].positions.length
}

export function parseSpreadType(value: string): SpreadType | null {
  const normalized = value.toLowerCase().replace(/[-\s]/g, '_')
  const types = Object.values(SpreadType)
  return types.find((t) => t === normalized) ?? null
}
