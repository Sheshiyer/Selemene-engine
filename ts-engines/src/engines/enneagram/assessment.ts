/**
 * Enneagram Assessment - Questionnaire and scoring algorithm
 * Questions focus on behavioral observations, not identity statements.
 */

import type { EnneagramNumber } from './wisdom'

export interface AssessmentQuestion {
  id: number
  text: string
  associatedType: EnneagramNumber
  weight: number // 1.0 = standard, 1.5 = high-signal question
}

export interface AssessmentAnswer {
  questionId: number
  value: number // 1-5 scale: 1=rarely, 2=sometimes, 3=often, 4=usually, 5=almost always
}

export interface AssessmentResult {
  scores: TypeScore[]
  primaryType: EnneagramNumber
  secondaryType: EnneagramNumber
  wing: EnneagramNumber
  confidence: number // 0-1, how distinct the primary type is from others
  tritype?: [EnneagramNumber, EnneagramNumber, EnneagramNumber] // One from each center if confidence high
}

export interface TypeScore {
  type: EnneagramNumber
  score: number
  normalizedScore: number // 0-100 scale
}

/**
 * Assessment questions - 4-5 questions per type (45 total)
 * Uses behavioral observation framing, NOT "you are" statements
 */
const ASSESSMENT_QUESTIONS: AssessmentQuestion[] = [
  // Type 1 - The Reformer (5 questions)
  {
    id: 1,
    text: 'How often do you find yourself noticing errors or mistakes that others overlook?',
    associatedType: 1,
    weight: 1.0,
  },
  {
    id: 2,
    text: 'When working on a task, how frequently do you feel it must be done "the right way"?',
    associatedType: 1,
    weight: 1.5,
  },
  {
    id: 3,
    text: 'How often do you feel a sense of irritation when others cut corners?',
    associatedType: 1,
    weight: 1.0,
  },
  {
    id: 4,
    text: 'How frequently do you mentally critique yourself for not living up to your own standards?',
    associatedType: 1,
    weight: 1.0,
  },
  {
    id: 5,
    text: 'How often do you postpone relaxation until everything is in proper order?',
    associatedType: 1,
    weight: 1.0,
  },

  // Type 2 - The Helper (5 questions)
  {
    id: 6,
    text: 'How often do you anticipate what others need before they ask?',
    associatedType: 2,
    weight: 1.0,
  },
  {
    id: 7,
    text: 'How important is it that people appreciate what you do for them?',
    associatedType: 2,
    weight: 1.5,
  },
  {
    id: 8,
    text: "How frequently do you find yourself putting others' needs before your own?",
    associatedType: 2,
    weight: 1.0,
  },
  {
    id: 9,
    text: 'How often do you feel hurt when your help is not acknowledged?',
    associatedType: 2,
    weight: 1.0,
  },
  {
    id: 10,
    text: 'How often do you know exactly what to say to make someone feel better?',
    associatedType: 2,
    weight: 1.0,
  },

  // Type 3 - The Achiever (5 questions)
  {
    id: 11,
    text: 'How often do you adjust your presentation based on what will impress others?',
    associatedType: 3,
    weight: 1.0,
  },
  {
    id: 12,
    text: 'How much does achieving goals drive your sense of self-worth?',
    associatedType: 3,
    weight: 1.5,
  },
  {
    id: 13,
    text: 'How frequently do you compare your accomplishments to those around you?',
    associatedType: 3,
    weight: 1.0,
  },
  {
    id: 14,
    text: 'How often do you feel restless when not actively working toward something?',
    associatedType: 3,
    weight: 1.0,
  },
  {
    id: 15,
    text: 'How important is it to you to be seen as successful?',
    associatedType: 3,
    weight: 1.0,
  },

  // Type 4 - The Individualist (5 questions)
  {
    id: 16,
    text: 'How often do you feel that something essential is missing from your life?',
    associatedType: 4,
    weight: 1.0,
  },
  {
    id: 17,
    text: 'How frequently do you experience emotions more intensely than those around you?',
    associatedType: 4,
    weight: 1.5,
  },
  {
    id: 18,
    text: "How often do you feel that others don't truly understand your inner world?",
    associatedType: 4,
    weight: 1.0,
  },
  {
    id: 19,
    text: 'How much do you value being authentic even if it means being different?',
    associatedType: 4,
    weight: 1.0,
  },
  {
    id: 20,
    text: 'How frequently do you find deep meaning in melancholy or longing?',
    associatedType: 4,
    weight: 1.0,
  },

  // Type 5 - The Investigator (5 questions)
  {
    id: 21,
    text: 'How often do you need time alone to recharge after social interactions?',
    associatedType: 5,
    weight: 1.0,
  },
  {
    id: 22,
    text: 'How important is it to fully understand something before engaging with it?',
    associatedType: 5,
    weight: 1.5,
  },
  {
    id: 23,
    text: 'How frequently do you observe situations rather than participate?',
    associatedType: 5,
    weight: 1.0,
  },
  {
    id: 24,
    text: 'How often do you feel drained by too many demands on your time or energy?',
    associatedType: 5,
    weight: 1.0,
  },
  {
    id: 25,
    text: 'How much do you value knowledge and competence over social approval?',
    associatedType: 5,
    weight: 1.0,
  },

  // Type 6 - The Loyalist (5 questions)
  {
    id: 26,
    text: 'How often do you find yourself anticipating what could go wrong?',
    associatedType: 6,
    weight: 1.0,
  },
  {
    id: 27,
    text: 'How important is having reliable people or systems you can count on?',
    associatedType: 6,
    weight: 1.5,
  },
  {
    id: 28,
    text: "How frequently do you question whether you can trust someone's intentions?",
    associatedType: 6,
    weight: 1.0,
  },
  {
    id: 29,
    text: 'How often do you seek reassurance or second opinions before making decisions?',
    associatedType: 6,
    weight: 1.0,
  },
  {
    id: 30,
    text: 'How much do you value loyalty and commitment in relationships?',
    associatedType: 6,
    weight: 1.0,
  },

  // Type 7 - The Enthusiast (5 questions)
  {
    id: 31,
    text: 'How often do you find yourself planning multiple exciting activities or projects?',
    associatedType: 7,
    weight: 1.0,
  },
  {
    id: 32,
    text: 'How difficult is it to sit with uncomfortable feelings without distracting yourself?',
    associatedType: 7,
    weight: 1.5,
  },
  {
    id: 33,
    text: 'How frequently do you feel that more options means more freedom?',
    associatedType: 7,
    weight: 1.0,
  },
  {
    id: 34,
    text: 'How often do you reframe negative situations into positive ones?',
    associatedType: 7,
    weight: 1.0,
  },
  {
    id: 35,
    text: 'How much does the fear of missing out influence your decisions?',
    associatedType: 7,
    weight: 1.0,
  },

  // Type 8 - The Challenger (5 questions)
  {
    id: 36,
    text: 'How often do you take charge when no one else is leading?',
    associatedType: 8,
    weight: 1.0,
  },
  {
    id: 37,
    text: 'How important is it to never let anyone have power over you?',
    associatedType: 8,
    weight: 1.5,
  },
  {
    id: 38,
    text: 'How frequently do you confront situations directly rather than avoiding them?',
    associatedType: 8,
    weight: 1.0,
  },
  {
    id: 39,
    text: 'How often do you notice your physical presence and intensity affects others?',
    associatedType: 8,
    weight: 1.0,
  },
  {
    id: 40,
    text: 'How much do you respect people who stand up for themselves?',
    associatedType: 8,
    weight: 1.0,
  },

  // Type 9 - The Peacemaker (5 questions)
  {
    id: 41,
    text: 'How often do you go along with others to maintain harmony?',
    associatedType: 9,
    weight: 1.0,
  },
  {
    id: 42,
    text: 'How difficult is it to know what you truly want separate from what others want?',
    associatedType: 9,
    weight: 1.5,
  },
  {
    id: 43,
    text: 'How frequently do you avoid conflict even when it might be necessary?',
    associatedType: 9,
    weight: 1.0,
  },
  {
    id: 44,
    text: 'How often do you find yourself numbing out or checking out mentally?',
    associatedType: 9,
    weight: 1.0,
  },
  {
    id: 45,
    text: 'How much does maintaining inner peace matter compared to asserting your position?',
    associatedType: 9,
    weight: 1.0,
  },
]

export function getAssessmentQuestions(): AssessmentQuestion[] {
  return [...ASSESSMENT_QUESTIONS]
}

export function getQuestionsByType(type: EnneagramNumber): AssessmentQuestion[] {
  return ASSESSMENT_QUESTIONS.filter((q) => q.associatedType === type)
}

/**
 * Calculate assessment results from answers
 */
export function calculateAssessment(answers: AssessmentAnswer[]): AssessmentResult {
  // Initialize scores for all types
  const rawScores: Record<EnneagramNumber, number> = {
    1: 0,
    2: 0,
    3: 0,
    4: 0,
    5: 0,
    6: 0,
    7: 0,
    8: 0,
    9: 0,
  }
  const maxPossibleScores: Record<EnneagramNumber, number> = {
    1: 0,
    2: 0,
    3: 0,
    4: 0,
    5: 0,
    6: 0,
    7: 0,
    8: 0,
    9: 0,
  }

  // Calculate max possible score for each type
  for (const question of ASSESSMENT_QUESTIONS) {
    maxPossibleScores[question.associatedType] += 5 * question.weight
  }

  // Calculate raw scores from answers
  for (const answer of answers) {
    const question = ASSESSMENT_QUESTIONS.find((q) => q.id === answer.questionId)
    if (question) {
      rawScores[question.associatedType] += answer.value * question.weight
    }
  }

  // Normalize scores to 0-100 scale
  const typeScores: TypeScore[] = ([1, 2, 3, 4, 5, 6, 7, 8, 9] as EnneagramNumber[]).map(
    (type) => ({
      type,
      score: rawScores[type],
      normalizedScore: Math.round((rawScores[type] / maxPossibleScores[type]) * 100),
    }),
  )

  // Sort by score descending
  typeScores.sort((a, b) => b.normalizedScore - a.normalizedScore)

  const primaryType = typeScores[0].type
  const secondaryType = typeScores[1].type

  // Calculate confidence as the gap between primary and secondary
  const confidence = Math.min(
    1,
    (typeScores[0].normalizedScore - typeScores[1].normalizedScore) / 30,
  )

  // Determine wing (must be adjacent type)
  const adjacentTypes = getAdjacentTypes(primaryType)
  const wing = typeScores.find((ts) => adjacentTypes.includes(ts.type))?.type ?? adjacentTypes[0]

  // Calculate tritype if confidence is high enough
  let tritype: [EnneagramNumber, EnneagramNumber, EnneagramNumber] | undefined
  if (confidence >= 0.3) {
    tritype = calculateTritype(typeScores)
  }

  return {
    scores: typeScores,
    primaryType,
    secondaryType,
    wing,
    confidence,
    tritype,
  }
}

/**
 * Calculate scores from simple answer array (1-5 values in question order)
 */
export function calculateFromAnswerArray(answers: number[]): AssessmentResult {
  const assessmentAnswers: AssessmentAnswer[] = answers.map((value, index) => ({
    questionId: ASSESSMENT_QUESTIONS[index]?.id ?? index + 1,
    value: Math.max(1, Math.min(5, value)), // Clamp to 1-5
  }))
  return calculateAssessment(assessmentAnswers)
}

function getAdjacentTypes(type: EnneagramNumber): EnneagramNumber[] {
  const prev = type === 1 ? 9 : ((type - 1) as EnneagramNumber)
  const next = type === 9 ? 1 : ((type + 1) as EnneagramNumber)
  return [prev, next]
}

function calculateTritype(
  scores: TypeScore[],
): [EnneagramNumber, EnneagramNumber, EnneagramNumber] {
  // Group types by center
  const gutTypes: EnneagramNumber[] = [8, 9, 1]
  const heartTypes: EnneagramNumber[] = [2, 3, 4]
  const headTypes: EnneagramNumber[] = [5, 6, 7]

  const getTopFromCenter = (center: EnneagramNumber[]): EnneagramNumber => {
    const centerScores = scores.filter((s) => center.includes(s.type))
    centerScores.sort((a, b) => b.normalizedScore - a.normalizedScore)
    return centerScores[0].type
  }

  const gutType = getTopFromCenter(gutTypes)
  const heartType = getTopFromCenter(heartTypes)
  const headType = getTopFromCenter(headTypes)

  // Return in order of score (not center order)
  const tritypeScores = [
    { type: gutType, score: scores.find((s) => s.type === gutType)?.normalizedScore ?? 0 },
    { type: heartType, score: scores.find((s) => s.type === heartType)?.normalizedScore ?? 0 },
    { type: headType, score: scores.find((s) => s.type === headType)?.normalizedScore ?? 0 },
  ]
  tritypeScores.sort((a, b) => b.score - a.score)

  return [tritypeScores[0].type, tritypeScores[1].type, tritypeScores[2].type]
}

export { ASSESSMENT_QUESTIONS }
