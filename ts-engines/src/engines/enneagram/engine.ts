/**
 * EnneagramEngine - Consciousness engine implementation for Enneagram
 * The Enneagram describes PATTERNS, not fixed identities.
 */

import type {
  ConsciousnessEngine,
  EngineInput,
  EngineMetadata,
  EngineOutput,
  WitnessPrompt,
} from '../../types'
import {
  type AssessmentResult,
  calculateFromAnswerArray,
  getAssessmentQuestions,
} from './assessment'
import type { EnneagramNumber, EnneagramType } from './wisdom'
import {
  getDisintegrationConnection,
  getEnneagramType,
  getIntegrationConnection,
  getWingTypes,
} from './wisdom'
import {
  generateAssessmentWitnessPrompts,
  generateMovementPrompts,
  generateTypeWitnessPrompts,
} from './witness'

interface EnneagramResult {
  mode: 'assessment' | 'lookup' | 'questions'
  assessment?: AssessmentResultOutput
  typeAnalysis?: TypeAnalysisOutput
  questions?: QuestionOutput[]
}

interface AssessmentResultOutput {
  scores: Array<{ type: number; normalizedScore: number }>
  primaryType: TypeSummary
  wing: TypeSummary
  confidence: number
  tritype?: number[]
  note: string
}

interface TypeSummary {
  number: number
  name: string
  description: string
}

interface TypeAnalysisOutput {
  type: TypeDetailOutput
  wings: Array<{ number: number; name: string }>
  integration: { type: number; name: string; description: string }
  disintegration: { type: number; name: string; description: string }
  center: string
  hornevianGroup: string
  harmonicGroup: string
}

interface TypeDetailOutput {
  number: number
  name: string
  coreFear: string
  coreDesire: string
  coreWeakness: string
  description: string
  keyMotivations: string[]
  healthyTraits: string[]
  averageTraits: string[]
  unhealthyTraits: string[]
}

interface QuestionOutput {
  id: number
  text: string
  scale: string
}

export class EnneagramEngine implements ConsciousnessEngine {
  metadata(): EngineMetadata {
    return {
      id: 'enneagram',
      name: 'Enneagram Consciousness Engine',
      description:
        'Enneagram assessment and type analysis with witness prompts for self-inquiry. The Enneagram describes patterns of perception and behavior, not fixed identities. Use for pattern recognition and self-observation.',
      version: '1.0.0',
      required_phase: 1, // Requires some self-awareness
      input_schema: {
        answers: {
          type: 'array',
          required: false,
          description:
            'Array of 45 numbers (1-5 scale) corresponding to assessment question responses. If provided, runs full assessment.',
        },
        type: {
          type: 'number',
          required: false,
          description:
            'Enneagram type number (1-9) for direct lookup. Returns type description and prompts without assessment.',
          enum: [1, 2, 3, 4, 5, 6, 7, 8, 9],
        },
        wing: {
          type: 'number',
          required: false,
          description: 'Wing type (adjacent to primary type). Used with type parameter.',
          enum: [1, 2, 3, 4, 5, 6, 7, 8, 9],
        },
        includeAssessment: {
          type: 'boolean',
          required: false,
          description: 'If true with no answers, returns assessment questions.',
          default: false,
        },
        includeMovementPrompts: {
          type: 'boolean',
          required: false,
          description: 'If true, includes prompts about integration and stress patterns.',
          default: false,
        },
      },
    }
  }

  async calculate(input: EngineInput): Promise<EngineOutput> {
    const startTime = performance.now()

    const answers = input.parameters.answers as number[] | undefined
    const typeParam = input.parameters.type as number | undefined
    const wingParam = input.parameters.wing as number | undefined
    const includeAssessment = input.parameters.includeAssessment as boolean | undefined
    const includeMovementPrompts = input.parameters.includeMovementPrompts as boolean | undefined
    const seed = input.seed

    let result: EnneagramResult
    let witnessPrompts: WitnessPrompt[]

    // Mode 1: Assessment with answers provided
    if (answers && answers.length > 0) {
      const assessmentResult = calculateFromAnswerArray(answers)
      result = this.buildAssessmentResult(assessmentResult)
      witnessPrompts = generateAssessmentWitnessPrompts(
        assessmentResult.primaryType,
        assessmentResult.wing,
        assessmentResult.confidence,
        seed,
      )

      // Add movement prompts if requested
      if (includeMovementPrompts) {
        const movementPrompts = generateMovementPrompts(assessmentResult.primaryType, seed)
        witnessPrompts = [...witnessPrompts, ...movementPrompts].slice(0, 5)
      }
    }
    // Mode 2: Direct type lookup
    else if (typeParam !== undefined) {
      const validType = this.validateType(typeParam)
      const validWing = wingParam ? this.validateWing(validType, wingParam) : undefined
      result = this.buildTypeLookupResult(validType, validWing)
      witnessPrompts = generateTypeWitnessPrompts(validType, validWing, seed)

      // Add movement prompts if requested
      if (includeMovementPrompts) {
        const movementPrompts = generateMovementPrompts(validType, seed)
        witnessPrompts = [...witnessPrompts, ...movementPrompts].slice(0, 5)
      }
    }
    // Mode 3: Return assessment questions
    else if (includeAssessment) {
      result = this.buildQuestionsResult()
      witnessPrompts = [
        {
          prompt:
            'As you answer these questions, notice which ones bring the strongest reaction. What does that reveal?',
          context: 'Pre-assessment reflection',
          themes: ['self-observation', 'curiosity', 'openness'],
        },
        {
          prompt:
            'Remember: no type is better or worse. The Enneagram illuminates patterns, not worth.',
          context: 'Assessment guidance',
          themes: ['non-judgment', 'compassion', 'awareness'],
        },
      ]
    }
    // Mode 4: No parameters - return instructions
    else {
      result = {
        mode: 'questions',
        questions: [],
      }
      witnessPrompts = [
        {
          prompt:
            'What draws you to explore the Enneagram? What are you hoping to understand about yourself?',
          context: 'Starting inquiry',
          themes: ['intention', 'curiosity', 'self-knowledge'],
        },
      ]
    }

    const endTime = performance.now()

    return {
      engine_id: 'enneagram',
      result: result as unknown as Record<string, unknown>,
      witness_prompts: witnessPrompts,
      calculated_at: new Date().toISOString(),
      processing_time_ms: Math.round(endTime - startTime),
    }
  }

  private validateType(type: number): EnneagramNumber {
    if (type < 1 || type > 9 || !Number.isInteger(type)) {
      throw new Error(`Invalid Enneagram type: ${type}. Must be integer 1-9.`)
    }
    return type as EnneagramNumber
  }

  private validateWing(primaryType: EnneagramNumber, wing: number): EnneagramNumber | undefined {
    if (wing < 1 || wing > 9 || !Number.isInteger(wing)) {
      return undefined
    }
    // Wing must be adjacent
    const prev = primaryType === 1 ? 9 : primaryType - 1
    const next = primaryType === 9 ? 1 : primaryType + 1
    if (wing !== prev && wing !== next) {
      return undefined
    }
    return wing as EnneagramNumber
  }

  private buildAssessmentResult(assessment: AssessmentResult): EnneagramResult {
    const primaryType = getEnneagramType(assessment.primaryType)
    const wingType = getEnneagramType(assessment.wing)

    return {
      mode: 'assessment',
      assessment: {
        scores: assessment.scores.map((s) => ({
          type: s.type,
          normalizedScore: s.normalizedScore,
        })),
        primaryType: {
          number: primaryType.number,
          name: primaryType.name,
          description: primaryType.description,
        },
        wing: {
          number: wingType.number,
          name: wingType.name,
          description: wingType.description,
        },
        confidence: Math.round(assessment.confidence * 100) / 100,
        tritype: assessment.tritype,
        note: 'These results indicate pattern tendencies, not fixed identity. The Enneagram is a tool for self-observation, not labeling.',
      },
      typeAnalysis: this.buildTypeAnalysis(assessment.primaryType),
    }
  }

  private buildTypeLookupResult(type: EnneagramNumber, wing?: EnneagramNumber): EnneagramResult {
    return {
      mode: 'lookup',
      typeAnalysis: this.buildTypeAnalysis(type, wing),
    }
  }

  private buildTypeAnalysis(type: EnneagramNumber, wing?: EnneagramNumber): TypeAnalysisOutput {
    const typeData = getEnneagramType(type)
    const [wing1, wing2] = getWingTypes(type)
    const integrationConn = getIntegrationConnection(type)
    const disintegrationConn = getDisintegrationConnection(type)

    const integrationData = integrationConn ? getEnneagramType(integrationConn.to) : null
    const disintegrationData = disintegrationConn ? getEnneagramType(disintegrationConn.to) : null

    return {
      type: {
        number: typeData.number,
        name: typeData.name,
        coreFear: typeData.coreFear,
        coreDesire: typeData.coreDesire,
        coreWeakness: typeData.coreWeakness,
        description: typeData.description,
        keyMotivations: typeData.keyMotivations,
        healthyTraits: typeData.healthyTraits,
        averageTraits: typeData.averageTraits,
        unhealthyTraits: typeData.unhealthyTraits,
      },
      wings: [
        { number: wing1.number, name: wing1.name },
        { number: wing2.number, name: wing2.name },
      ],
      integration: integrationData
        ? {
            type: integrationData.number,
            name: integrationData.name,
            description: integrationConn?.description ?? '',
          }
        : { type: 0, name: 'Unknown', description: '' },
      disintegration: disintegrationData
        ? {
            type: disintegrationData.number,
            name: disintegrationData.name,
            description: disintegrationConn?.description ?? '',
          }
        : { type: 0, name: 'Unknown', description: '' },
      center: typeData.centerOfIntelligence,
      hornevianGroup: typeData.hornevianGroup,
      harmonicGroup: typeData.harmonicGroup,
    }
  }

  private buildQuestionsResult(): EnneagramResult {
    const questions = getAssessmentQuestions()

    return {
      mode: 'questions',
      questions: questions.map((q) => ({
        id: q.id,
        text: q.text,
        scale: '1 (rarely) to 5 (almost always)',
      })),
    }
  }
}
