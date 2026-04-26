/**
 * Tarot Wisdom Data - Complete 78-card deck
 * Contains Major Arcana (22) and Minor Arcana (56) with meanings
 */

export type Suit = 'wands' | 'cups' | 'swords' | 'pentacles'
export type Element = 'fire' | 'water' | 'air' | 'earth' | 'spirit'

export interface TarotCard {
  id: string
  name: string
  number: number
  arcana: 'major' | 'minor'
  suit?: Suit
  uprightMeaning: string
  reversedMeaning: string
  keywords: string[]
  element: Element
}

export interface MajorArcana extends TarotCard {
  arcana: 'major'
}

export interface MinorArcana extends TarotCard {
  arcana: 'minor'
  suit: Suit
}

const MAJOR_ARCANA: MajorArcana[] = [
  {
    id: 'major-0',
    name: 'The Fool',
    number: 0,
    arcana: 'major',
    uprightMeaning: 'New beginnings, innocence, spontaneity, free spirit, potential',
    reversedMeaning: 'Recklessness, risk-taking, holding back, naivety',
    keywords: ['beginnings', 'innocence', 'leap of faith', 'spontaneity'],
    element: 'air',
  },
  {
    id: 'major-1',
    name: 'The Magician',
    number: 1,
    arcana: 'major',
    uprightMeaning: 'Manifestation, resourcefulness, power, inspired action',
    reversedMeaning: 'Manipulation, poor planning, untapped talents',
    keywords: ['manifestation', 'willpower', 'creation', 'skill'],
    element: 'air',
  },
  {
    id: 'major-2',
    name: 'The High Priestess',
    number: 2,
    arcana: 'major',
    uprightMeaning: 'Intuition, sacred knowledge, divine feminine, the subconscious mind',
    reversedMeaning: 'Secrets, disconnected from intuition, withdrawal',
    keywords: ['intuition', 'mystery', 'inner voice', 'wisdom'],
    element: 'water',
  },
  {
    id: 'major-3',
    name: 'The Empress',
    number: 3,
    arcana: 'major',
    uprightMeaning: 'Femininity, beauty, nature, nurturing, abundance',
    reversedMeaning: 'Creative block, dependence on others, emptiness',
    keywords: ['abundance', 'nurturing', 'fertility', 'nature'],
    element: 'earth',
  },
  {
    id: 'major-4',
    name: 'The Emperor',
    number: 4,
    arcana: 'major',
    uprightMeaning: 'Authority, structure, control, fatherhood, stability',
    reversedMeaning: 'Domination, excessive control, rigidity, lack of discipline',
    keywords: ['authority', 'structure', 'leadership', 'stability'],
    element: 'fire',
  },
  {
    id: 'major-5',
    name: 'The Hierophant',
    number: 5,
    arcana: 'major',
    uprightMeaning: 'Spiritual wisdom, tradition, conformity, morality, ethics',
    reversedMeaning: 'Personal beliefs, freedom, challenging the status quo',
    keywords: ['tradition', 'spirituality', 'guidance', 'education'],
    element: 'earth',
  },
  {
    id: 'major-6',
    name: 'The Lovers',
    number: 6,
    arcana: 'major',
    uprightMeaning: 'Love, harmony, relationships, values alignment, choices',
    reversedMeaning: 'Self-love, disharmony, imbalance, misalignment of values',
    keywords: ['love', 'union', 'choice', 'harmony'],
    element: 'air',
  },
  {
    id: 'major-7',
    name: 'The Chariot',
    number: 7,
    arcana: 'major',
    uprightMeaning: 'Control, willpower, success, action, determination',
    reversedMeaning: 'Self-discipline, opposition, lack of direction',
    keywords: ['determination', 'victory', 'willpower', 'control'],
    element: 'water',
  },
  {
    id: 'major-8',
    name: 'Strength',
    number: 8,
    arcana: 'major',
    uprightMeaning: 'Inner strength, courage, compassion, influence, persuasion',
    reversedMeaning: 'Self-doubt, weakness, insecurity, raw emotion',
    keywords: ['courage', 'patience', 'compassion', 'inner strength'],
    element: 'fire',
  },
  {
    id: 'major-9',
    name: 'The Hermit',
    number: 9,
    arcana: 'major',
    uprightMeaning: 'Soul-searching, introspection, being alone, inner guidance',
    reversedMeaning: 'Isolation, loneliness, withdrawal, lost your way',
    keywords: ['introspection', 'solitude', 'guidance', 'wisdom'],
    element: 'earth',
  },
  {
    id: 'major-10',
    name: 'Wheel of Fortune',
    number: 10,
    arcana: 'major',
    uprightMeaning: 'Good luck, karma, life cycles, destiny, a turning point',
    reversedMeaning: 'Bad luck, resistance to change, breaking cycles',
    keywords: ['change', 'cycles', 'fate', 'fortune'],
    element: 'fire',
  },
  {
    id: 'major-11',
    name: 'Justice',
    number: 11,
    arcana: 'major',
    uprightMeaning: 'Justice, fairness, truth, cause and effect, law',
    reversedMeaning: 'Unfairness, lack of accountability, dishonesty',
    keywords: ['truth', 'fairness', 'balance', 'consequence'],
    element: 'air',
  },
  {
    id: 'major-12',
    name: 'The Hanged Man',
    number: 12,
    arcana: 'major',
    uprightMeaning: 'Pause, surrender, letting go, new perspectives',
    reversedMeaning: 'Delays, resistance, stalling, indecision',
    keywords: ['surrender', 'perspective', 'pause', 'sacrifice'],
    element: 'water',
  },
  {
    id: 'major-13',
    name: 'Death',
    number: 13,
    arcana: 'major',
    uprightMeaning: 'Endings, change, transformation, transition',
    reversedMeaning: 'Resistance to change, personal transformation, inner purging',
    keywords: ['transformation', 'endings', 'change', 'transition'],
    element: 'water',
  },
  {
    id: 'major-14',
    name: 'Temperance',
    number: 14,
    arcana: 'major',
    uprightMeaning: 'Balance, moderation, patience, purpose, meaning',
    reversedMeaning: 'Imbalance, excess, self-healing, re-alignment',
    keywords: ['balance', 'patience', 'moderation', 'harmony'],
    element: 'fire',
  },
  {
    id: 'major-15',
    name: 'The Devil',
    number: 15,
    arcana: 'major',
    uprightMeaning: 'Shadow self, attachment, addiction, restriction, sexuality',
    reversedMeaning: 'Releasing limiting beliefs, exploring dark thoughts, detachment',
    keywords: ['bondage', 'materialism', 'shadow', 'attachment'],
    element: 'earth',
  },
  {
    id: 'major-16',
    name: 'The Tower',
    number: 16,
    arcana: 'major',
    uprightMeaning: 'Sudden change, upheaval, chaos, revelation, awakening',
    reversedMeaning: 'Personal transformation, fear of change, averting disaster',
    keywords: ['upheaval', 'revelation', 'awakening', 'destruction'],
    element: 'fire',
  },
  {
    id: 'major-17',
    name: 'The Star',
    number: 17,
    arcana: 'major',
    uprightMeaning: 'Hope, faith, purpose, renewal, spirituality',
    reversedMeaning: 'Lack of faith, despair, self-trust, disconnection',
    keywords: ['hope', 'inspiration', 'renewal', 'serenity'],
    element: 'air',
  },
  {
    id: 'major-18',
    name: 'The Moon',
    number: 18,
    arcana: 'major',
    uprightMeaning: 'Illusion, fear, anxiety, subconscious, intuition',
    reversedMeaning: 'Release of fear, repressed emotion, inner confusion',
    keywords: ['illusion', 'intuition', 'uncertainty', 'subconscious'],
    element: 'water',
  },
  {
    id: 'major-19',
    name: 'The Sun',
    number: 19,
    arcana: 'major',
    uprightMeaning: 'Positivity, fun, warmth, success, vitality',
    reversedMeaning: 'Inner child, feeling down, overly optimistic',
    keywords: ['joy', 'success', 'vitality', 'clarity'],
    element: 'fire',
  },
  {
    id: 'major-20',
    name: 'Judgement',
    number: 20,
    arcana: 'major',
    uprightMeaning: 'Reflection, reckoning, awakening, renewal, purpose',
    reversedMeaning: 'Self-doubt, inner critic, ignoring the call',
    keywords: ['rebirth', 'calling', 'absolution', 'reflection'],
    element: 'fire',
  },
  {
    id: 'major-21',
    name: 'The World',
    number: 21,
    arcana: 'major',
    uprightMeaning: 'Completion, integration, accomplishment, travel',
    reversedMeaning: 'Seeking personal closure, short-cuts, delays',
    keywords: ['completion', 'wholeness', 'achievement', 'fulfillment'],
    element: 'earth',
  },
]

const SUIT_ELEMENTS: Record<Suit, Element> = {
  wands: 'fire',
  cups: 'water',
  swords: 'air',
  pentacles: 'earth',
}

const COURT_NAMES = ['Page', 'Knight', 'Queen', 'King']
const COURT_NUMBERS = [11, 12, 13, 14]

interface SuitMeanings {
  ace: { upright: string; reversed: string; keywords: string[] }
  two: { upright: string; reversed: string; keywords: string[] }
  three: { upright: string; reversed: string; keywords: string[] }
  four: { upright: string; reversed: string; keywords: string[] }
  five: { upright: string; reversed: string; keywords: string[] }
  six: { upright: string; reversed: string; keywords: string[] }
  seven: { upright: string; reversed: string; keywords: string[] }
  eight: { upright: string; reversed: string; keywords: string[] }
  nine: { upright: string; reversed: string; keywords: string[] }
  ten: { upright: string; reversed: string; keywords: string[] }
  page: { upright: string; reversed: string; keywords: string[] }
  knight: { upright: string; reversed: string; keywords: string[] }
  queen: { upright: string; reversed: string; keywords: string[] }
  king: { upright: string; reversed: string; keywords: string[] }
}

const WANDS_MEANINGS: SuitMeanings = {
  ace: {
    upright: 'Inspiration, new opportunities, growth, potential',
    reversed: 'Emerging ideas, lack of direction, distractions, delays',
    keywords: ['inspiration', 'potential', 'creation', 'beginnings'],
  },
  two: {
    upright: 'Planning, making decisions, leaving home, personal goals',
    reversed: 'Fear of change, playing it safe, bad planning',
    keywords: ['planning', 'decisions', 'discovery', 'progress'],
  },
  three: {
    upright: 'Looking ahead, expansion, rapid growth, foresight',
    reversed: 'Obstacles, delays, frustration, lack of foresight',
    keywords: ['expansion', 'foresight', 'overseas', 'growth'],
  },
  four: {
    upright: 'Celebration, harmony, marriage, home, community',
    reversed: 'Lack of support, transience, home conflicts',
    keywords: ['celebration', 'harmony', 'homecoming', 'community'],
  },
  five: {
    upright: 'Conflict, disagreements, competition, tension, diversity',
    reversed: 'Inner conflict, conflict avoidance, releasing tension',
    keywords: ['conflict', 'competition', 'disagreement', 'strife'],
  },
  six: {
    upright: 'Success, public recognition, progress, self-confidence',
    reversed: 'Egotism, disrepute, lack of confidence, fall from grace',
    keywords: ['victory', 'success', 'recognition', 'achievement'],
  },
  seven: {
    upright: 'Perseverance, defensive, maintaining control, standing your ground',
    reversed: 'Giving up, overwhelmed, overly protective',
    keywords: ['challenge', 'perseverance', 'defense', 'conviction'],
  },
  eight: {
    upright: 'Movement, fast-paced change, action, alignment, air travel',
    reversed: 'Delays, frustration, resisting change, internal alignment',
    keywords: ['speed', 'action', 'movement', 'quick decisions'],
  },
  nine: {
    upright: 'Resilience, grit, last stand, boundaries, close to the finish',
    reversed: 'Exhaustion, fatigue, questioning your path',
    keywords: ['resilience', 'persistence', 'courage', 'boundaries'],
  },
  ten: {
    upright: 'Burden, extra responsibility, hard work, completion',
    reversed: 'Doing it all, carrying the burden, delegation, release',
    keywords: ['burden', 'responsibility', 'hard work', 'stress'],
  },
  page: {
    upright: 'Exploration, excitement, freedom, discovery, new ideas',
    reversed: 'Lack of direction, procrastination, creating conflict',
    keywords: ['exploration', 'enthusiasm', 'discovery', 'potential'],
  },
  knight: {
    upright: 'Energy, passion, adventure, impulsiveness, action',
    reversed: 'Passion project, haste, scattered energy, delays',
    keywords: ['adventure', 'passion', 'impulsiveness', 'energy'],
  },
  queen: {
    upright: 'Courage, confidence, independence, social butterfly, determination',
    reversed: 'Self-respect, self-confidence, introverted, re-establish sense of self',
    keywords: ['confidence', 'independence', 'warmth', 'vibrancy'],
  },
  king: {
    upright: 'Natural leader, vision, entrepreneur, honor',
    reversed: 'Impulsiveness, haste, ruthless, high expectations',
    keywords: ['leadership', 'vision', 'entrepreneur', 'honor'],
  },
}

const CUPS_MEANINGS: SuitMeanings = {
  ace: {
    upright: 'Love, new relationships, compassion, creativity, overwhelming emotion',
    reversed: 'Self-love, intuition, repressed emotions, emptiness',
    keywords: ['love', 'emotion', 'intuition', 'new relationships'],
  },
  two: {
    upright: 'Unified love, partnership, mutual attraction, relationships',
    reversed: 'Self-love, break-ups, disharmony, distrust',
    keywords: ['partnership', 'attraction', 'unity', 'connection'],
  },
  three: {
    upright: 'Celebration, friendship, creativity, collaborations, community',
    reversed: 'Independence, alone time, hardcore partying, three is a crowd',
    keywords: ['celebration', 'friendship', 'community', 'joy'],
  },
  four: {
    upright: 'Meditation, contemplation, apathy, reevaluation, discontent',
    reversed: 'Retreat, withdrawal, checking in for alignment',
    keywords: ['contemplation', 'apathy', 'reevaluation', 'meditation'],
  },
  five: {
    upright: 'Regret, failure, disappointment, pessimism, loss',
    reversed: 'Personal setbacks, self-forgiveness, moving on',
    keywords: ['loss', 'grief', 'disappointment', 'regret'],
  },
  six: {
    upright: 'Revisiting the past, childhood memories, innocence, joy',
    reversed: 'Living in the past, forgiveness, lacking playfulness',
    keywords: ['nostalgia', 'memories', 'innocence', 'reunion'],
  },
  seven: {
    upright: 'Opportunities, choices, wishful thinking, illusion, fantasy',
    reversed: 'Alignment, personal values, overwhelmed by choices',
    keywords: ['choices', 'fantasy', 'illusion', 'temptation'],
  },
  eight: {
    upright: 'Disappointment, abandonment, withdrawal, escapism',
    reversed: 'Trying one more time, indecision, aimless drifting',
    keywords: ['abandonment', 'withdrawal', 'moving on', 'letting go'],
  },
  nine: {
    upright: 'Contentment, satisfaction, gratitude, wish come true',
    reversed: 'Inner happiness, materialism, dissatisfaction',
    keywords: ['contentment', 'satisfaction', 'wishes fulfilled', 'gratitude'],
  },
  ten: {
    upright: 'Divine love, blissful relationships, harmony, alignment',
    reversed: 'Disconnection, misaligned values, struggling relationships',
    keywords: ['happiness', 'fulfillment', 'family', 'harmony'],
  },
  page: {
    upright: 'Creative opportunities, intuitive messages, curiosity, possibility',
    reversed: 'Emotional immaturity, creative block, self-doubt',
    keywords: ['creativity', 'intuition', 'sensitivity', 'dreams'],
  },
  knight: {
    upright: 'Creativity, romance, charm, imagination, beauty',
    reversed: 'Overactive imagination, unrealistic, jealousy, moodiness',
    keywords: ['romance', 'charm', 'imagination', 'idealism'],
  },
  queen: {
    upright: 'Compassionate, caring, emotionally stable, intuitive',
    reversed: 'Inner feelings, self-care, self-love, co-dependency',
    keywords: ['compassion', 'nurturing', 'intuition', 'emotional security'],
  },
  king: {
    upright: 'Emotionally balanced, compassionate, diplomatic',
    reversed: 'Self-compassion, inner feelings, moodiness, emotionally manipulative',
    keywords: ['emotional balance', 'diplomacy', 'wisdom', 'calm'],
  },
}

const SWORDS_MEANINGS: SuitMeanings = {
  ace: {
    upright: 'Breakthrough, clarity, sharp mind, truth, new ideas',
    reversed: 'Inner clarity, re-thinking an idea, clouded judgement',
    keywords: ['clarity', 'truth', 'breakthrough', 'mental force'],
  },
  two: {
    upright: 'Difficult decisions, weighing options, stalemate, denial',
    reversed: 'Indecision, confusion, information overload, lesser evil',
    keywords: ['indecision', 'stalemate', 'difficult choices', 'denial'],
  },
  three: {
    upright: 'Heartbreak, emotional pain, sorrow, grief, hurt',
    reversed: 'Recovery, forgiveness, moving on, releasing pain',
    keywords: ['heartbreak', 'sorrow', 'grief', 'pain'],
  },
  four: {
    upright: 'Rest, relaxation, meditation, contemplation, recuperation',
    reversed: 'Exhaustion, burn-out, deep contemplation, stagnation',
    keywords: ['rest', 'restoration', 'contemplation', 'recovery'],
  },
  five: {
    upright: 'Conflict, disagreements, competition, defeat, winning at all costs',
    reversed: 'Reconciliation, making amends, past resentment',
    keywords: ['conflict', 'defeat', 'tension', 'hostility'],
  },
  six: {
    upright: 'Transition, change, rite of passage, releasing baggage',
    reversed: 'Personal transition, resistance to change, unfinished business',
    keywords: ['transition', 'moving on', 'change', 'travel'],
  },
  seven: {
    upright: 'Betrayal, deception, getting away with something, strategy',
    reversed: 'Imposter syndrome, self-deception, keeping secrets',
    keywords: ['deception', 'strategy', 'cunning', 'theft'],
  },
  eight: {
    upright: 'Negative thoughts, self-imposed restriction, imprisonment, victim mentality',
    reversed: 'Self-limiting beliefs, inner critic, releasing negative thoughts',
    keywords: ['restriction', 'imprisonment', 'helplessness', 'self-limiting'],
  },
  nine: {
    upright: 'Anxiety, worry, fear, depression, nightmares',
    reversed: 'Inner turmoil, deep-seated fears, secrets, releasing worry',
    keywords: ['anxiety', 'fear', 'worry', 'despair'],
  },
  ten: {
    upright: 'Painful endings, deep wounds, betrayal, loss, crisis',
    reversed: 'Recovery, regeneration, resisting an inevitable end',
    keywords: ['endings', 'betrayal', 'loss', 'rock bottom'],
  },
  page: {
    upright: 'New ideas, curiosity, thirst for knowledge, new communication',
    reversed: 'Self-expression, all talk no action, hurtful words',
    keywords: ['curiosity', 'new ideas', 'communication', 'mental energy'],
  },
  knight: {
    upright: 'Ambitious, action-oriented, driven, fast-thinking',
    reversed: 'Restless, unfocused, impulsive, burn-out',
    keywords: ['ambition', 'action', 'drive', 'determination'],
  },
  queen: {
    upright: 'Independent, unbiased judgement, clear boundaries, direct communication',
    reversed: 'Overly emotional, easily influenced, cold-hearted',
    keywords: ['independence', 'clarity', 'direct communication', 'perception'],
  },
  king: {
    upright: 'Mental clarity, intellectual power, authority, truth',
    reversed: 'Quiet power, inner truth, misuse of power, manipulation',
    keywords: ['authority', 'truth', 'intellect', 'ethical'],
  },
}

const PENTACLES_MEANINGS: SuitMeanings = {
  ace: {
    upright: 'New financial opportunity, manifestation, abundance, prosperity',
    reversed: 'Lost opportunity, lack of planning, scarcity mindset',
    keywords: ['opportunity', 'prosperity', 'manifestation', 'abundance'],
  },
  two: {
    upright: 'Multiple priorities, time management, prioritization, adaptability',
    reversed: 'Over-committed, disorganization, reprioritization',
    keywords: ['balance', 'adaptability', 'priorities', 'time management'],
  },
  three: {
    upright: 'Teamwork, collaboration, learning, implementation, craftsmanship',
    reversed: 'Lack of teamwork, disregard for skills, self-development',
    keywords: ['teamwork', 'learning', 'craftsmanship', 'collaboration'],
  },
  four: {
    upright: 'Saving money, security, conservatism, scarcity, control',
    reversed: 'Over-spending, greed, self-protection, materialism',
    keywords: ['security', 'conservation', 'control', 'stability'],
  },
  five: {
    upright: 'Financial loss, poverty, lack mindset, isolation, worry',
    reversed: 'Recovery from financial loss, spiritual poverty, self-care',
    keywords: ['hardship', 'loss', 'poverty', 'worry'],
  },
  six: {
    upright: 'Giving, receiving, sharing wealth, generosity, charity',
    reversed: 'Self-care, unpaid debts, one-sided charity, strings attached',
    keywords: ['generosity', 'charity', 'giving', 'sharing'],
  },
  seven: {
    upright: 'Long-term view, sustainable results, perseverance, investment',
    reversed: 'Lack of long-term vision, limited success, frustration',
    keywords: ['patience', 'investment', 'perseverance', 'rewards'],
  },
  eight: {
    upright: 'Apprenticeship, repetitive tasks, mastery, skill development',
    reversed: 'Self-development, perfectionism, misdirected activity',
    keywords: ['skill', 'craftsmanship', 'dedication', 'mastery'],
  },
  nine: {
    upright: 'Abundance, luxury, self-sufficiency, financial independence',
    reversed: 'Self-worth, over-investment in work, hustling, materialism',
    keywords: ['abundance', 'luxury', 'independence', 'accomplishment'],
  },
  ten: {
    upright: 'Wealth, financial security, family, long-term success, contribution',
    reversed: 'Family disputes, bankruptcy, fleeting success, loneliness',
    keywords: ['wealth', 'legacy', 'family', 'security'],
  },
  page: {
    upright: 'Ambition, desire, diligence, faithfulness, new career',
    reversed: 'Lack of commitment, greed, procrastination, laziness',
    keywords: ['ambition', 'diligence', 'opportunity', 'manifestation'],
  },
  knight: {
    upright: 'Hard work, productivity, routine, responsibility, persistence',
    reversed: 'Self-discipline, boredom, feeling stuck, perfectionism',
    keywords: ['hard work', 'productivity', 'routine', 'persistence'],
  },
  queen: {
    upright: 'Nurturing, practical, providing financially, working parent',
    reversed: 'Financial independence, self-care, work-home conflict',
    keywords: ['nurturing', 'practical', 'abundance', 'security'],
  },
  king: {
    upright: 'Wealth, business, leadership, security, discipline, abundance',
    reversed: 'Financially inept, obsessed with wealth, stubborn, controlling',
    keywords: ['wealth', 'business', 'discipline', 'leadership'],
  },
}

const SUIT_MEANINGS: Record<Suit, SuitMeanings> = {
  wands: WANDS_MEANINGS,
  cups: CUPS_MEANINGS,
  swords: SWORDS_MEANINGS,
  pentacles: PENTACLES_MEANINGS,
}

function createMinorArcana(): MinorArcana[] {
  const cards: MinorArcana[] = []
  const suits: Suit[] = ['wands', 'cups', 'swords', 'pentacles']
  const numberNames = [
    'ace',
    'two',
    'three',
    'four',
    'five',
    'six',
    'seven',
    'eight',
    'nine',
    'ten',
  ] as const

  for (const suit of suits) {
    const meanings = SUIT_MEANINGS[suit]

    // Number cards (Ace through Ten)
    for (let i = 0; i < 10; i++) {
      const name = numberNames[i]
      const cardName =
        i === 0 ? `Ace of ${capitalize(suit)}` : `${capitalize(name)} of ${capitalize(suit)}`

      cards.push({
        id: `${suit}-${i + 1}`,
        name: cardName,
        number: i + 1,
        arcana: 'minor',
        suit,
        uprightMeaning: meanings[name].upright,
        reversedMeaning: meanings[name].reversed,
        keywords: meanings[name].keywords,
        element: SUIT_ELEMENTS[suit],
      })
    }

    // Court cards (Page, Knight, Queen, King)
    const courtNames = ['page', 'knight', 'queen', 'king'] as const
    for (let i = 0; i < 4; i++) {
      const courtName = courtNames[i]
      cards.push({
        id: `${suit}-${COURT_NUMBERS[i]}`,
        name: `${COURT_NAMES[i]} of ${capitalize(suit)}`,
        number: COURT_NUMBERS[i],
        arcana: 'minor',
        suit,
        uprightMeaning: meanings[courtName].upright,
        reversedMeaning: meanings[courtName].reversed,
        keywords: meanings[courtName].keywords,
        element: SUIT_ELEMENTS[suit],
      })
    }
  }

  return cards
}

function capitalize(str: string): string {
  return str.charAt(0).toUpperCase() + str.slice(1)
}

export interface TarotDeck {
  majorArcana: MajorArcana[]
  minorArcana: MinorArcana[]
  allCards: TarotCard[]
}

export function loadTarotDeck(): TarotDeck {
  const minorArcana = createMinorArcana()
  return {
    majorArcana: MAJOR_ARCANA,
    minorArcana,
    allCards: [...MAJOR_ARCANA, ...minorArcana],
  }
}
