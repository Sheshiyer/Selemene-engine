/**
 * Enneagram Wisdom Data - Complete data for all 9 types
 * The Enneagram describes PATTERNS of perception and behavior, not fixed identities.
 */

export type EnneagramNumber = 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9

export type CenterOfIntelligence = 'gut' | 'heart' | 'head'
export type HornevianGroup = 'assertive' | 'compliant' | 'withdrawn'
export type HarmonicGroup = 'positive' | 'competency' | 'reactive'

export interface EnneagramType {
  number: EnneagramNumber
  name: string
  coreFear: string
  coreDesire: string
  coreWeakness: string
  keyMotivations: string[]
  wings: [EnneagramNumber, EnneagramNumber]
  integrationPoint: EnneagramNumber
  disintegrationPoint: EnneagramNumber
  centerOfIntelligence: CenterOfIntelligence
  hornevianGroup: HornevianGroup
  harmonicGroup: HarmonicGroup
  description: string
  healthyTraits: string[]
  averageTraits: string[]
  unhealthyTraits: string[]
}

export interface EnneagramConnection {
  from: EnneagramNumber
  to: EnneagramNumber
  type: 'integration' | 'disintegration' | 'wing'
  description: string
}

const ENNEAGRAM_TYPES: Record<EnneagramNumber, EnneagramType> = {
  1: {
    number: 1,
    name: 'The Reformer',
    coreFear: 'Being corrupt, evil, or defective',
    coreDesire: 'To be good, virtuous, and have integrity',
    coreWeakness: 'Resentment—repressed anger from feeling that things are never good enough',
    keyMotivations: [
      'Striving to be right and to improve everything',
      'Being consistent and self-disciplined',
      'Justifying position and maintaining high standards',
      'Avoiding criticism and blame',
    ],
    wings: [9, 2],
    integrationPoint: 7,
    disintegrationPoint: 4,
    centerOfIntelligence: 'gut',
    hornevianGroup: 'compliant',
    harmonicGroup: 'competency',
    description:
      'Principled, purposeful, self-controlled, and perfectionistic. Ones have a strong sense of right and wrong and advocate for change. They fear being morally flawed and strive for integrity in all they do.',
    healthyTraits: [
      'Wise and discerning',
      'Realistic and accepting',
      'Noble and ethically heroic',
      'Inspiring others to improve',
      'Orderly without being rigid',
    ],
    averageTraits: [
      'Highly critical of self and others',
      'Perfectionistic and detail-oriented',
      'Time-conscious and organized',
      'Suppressing emotions, especially anger',
      'Moralizing and teaching others',
    ],
    unhealthyTraits: [
      'Rigid and inflexible',
      'Self-righteous and punishing',
      'Obsessive about imperfection',
      'Severely depressed when ideals unmet',
      'Condemning and intolerant',
    ],
  },
  2: {
    number: 2,
    name: 'The Helper',
    coreFear: 'Being unwanted or unworthy of love',
    coreDesire: 'To be loved and appreciated',
    coreWeakness: 'Pride—denying own needs while feeling indispensable to others',
    keyMotivations: [
      'Wanting to be loved and appreciated',
      'Expressing positive feelings toward others',
      'Being needed and wanted',
      'Avoiding acknowledging own needs',
    ],
    wings: [1, 3],
    integrationPoint: 4,
    disintegrationPoint: 8,
    centerOfIntelligence: 'heart',
    hornevianGroup: 'compliant',
    harmonicGroup: 'positive',
    description:
      'Generous, demonstrative, people-pleasing, and possessive. Twos are empathetic and caring, often anticipating what others need. They fear being unloved and work to be indispensable.',
    healthyTraits: [
      'Unconditionally loving and caring',
      'Humble and altruistic',
      'Nurturing without expectations',
      'Deeply empathetic and supportive',
      "Encouraging others' independence",
    ],
    averageTraits: [
      'People-pleasing and flattering',
      'Possessive in relationships',
      'Intrusive with advice and help',
      'Self-important about generosity',
      'Expecting appreciation and return',
    ],
    unhealthyTraits: [
      'Manipulative and coercive',
      'Entitled to get needs met',
      'Self-deceptive about motives',
      'Martyred and victimized',
      'Dominating through emotional manipulation',
    ],
  },
  3: {
    number: 3,
    name: 'The Achiever',
    coreFear: 'Being worthless or without inherent value',
    coreDesire: 'To feel valuable and worthwhile',
    coreWeakness: 'Deceit—believing they are only their image and achievements',
    keyMotivations: [
      'Wanting to be affirmed and distinguished',
      'Impressing others and getting attention',
      'Being admired and successful',
      'Avoiding failure at all costs',
    ],
    wings: [2, 4],
    integrationPoint: 6,
    disintegrationPoint: 9,
    centerOfIntelligence: 'heart',
    hornevianGroup: 'assertive',
    harmonicGroup: 'competency',
    description:
      'Adaptive, excelling, driven, and image-conscious. Threes are success-oriented and highly concerned with performance. They fear being worthless and become accomplished to prove their value.',
    healthyTraits: [
      'Authentic and inner-directed',
      'Self-accepting regardless of achievement',
      'Inspiring and motivating others',
      'Genuinely admirable and charming',
      'Effective and goal-oriented',
    ],
    averageTraits: [
      'Competitive and comparing',
      'Image-conscious and concerned with status',
      'Driven and workaholic',
      'Calculating and pragmatic',
      'Self-promoting and packaging oneself',
    ],
    unhealthyTraits: [
      'Deceptive and duplicitous',
      'Exploitative and opportunistic',
      'Vindictive when threatened',
      'Sabotaging others to succeed',
      'Empty and disconnected from self',
    ],
  },
  4: {
    number: 4,
    name: 'The Individualist',
    coreFear: 'Having no identity or personal significance',
    coreDesire: 'To find themselves and their significance',
    coreWeakness: 'Envy—feeling deficient and longing for what others have',
    keyMotivations: [
      'Expressing individuality and uniqueness',
      'Creating an identity from inner experiences',
      'Protecting self through withdrawal',
      'Taking care of emotional needs before attending to others',
    ],
    wings: [3, 5],
    integrationPoint: 1,
    disintegrationPoint: 2,
    centerOfIntelligence: 'heart',
    hornevianGroup: 'withdrawn',
    harmonicGroup: 'reactive',
    description:
      'Expressive, dramatic, self-absorbed, and temperamental. Fours are sensitive, creative, and emotionally honest. They fear being without identity and create a unique self-image.',
    healthyTraits: [
      'Inspired and highly creative',
      'Self-aware and introspective',
      'Emotionally honest and authentic',
      'Compassionate toward suffering',
      'Transforming pain into beauty',
    ],
    averageTraits: [
      'Melancholic and withdrawn',
      'Self-conscious and envious',
      'Moody and hypersensitive',
      'Self-indulgent with feelings',
      'Feeling misunderstood and different',
    ],
    unhealthyTraits: [
      'Depressed and hopeless',
      'Self-destructive and alienated',
      'Tormented by self-contempt',
      'Emotionally blocked',
      'Despising self and life',
    ],
  },
  5: {
    number: 5,
    name: 'The Investigator',
    coreFear: 'Being useless, incapable, or incompetent',
    coreDesire: 'To be capable and competent',
    coreWeakness: 'Avarice—withholding oneself from engagement with the world',
    keyMotivations: [
      'Understanding the environment',
      'Possessing knowledge and being competent',
      'Having everything figured out',
      'Defending self from intrusions',
    ],
    wings: [4, 6],
    integrationPoint: 8,
    disintegrationPoint: 7,
    centerOfIntelligence: 'head',
    hornevianGroup: 'withdrawn',
    harmonicGroup: 'competency',
    description:
      'Perceptive, innovative, secretive, and isolated. Fives are alert, insightful, and curious. They fear being overwhelmed by needs and retreat into their minds.',
    healthyTraits: [
      'Visionary and pioneering',
      'Observing reality objectively',
      'Open-minded and curious',
      'Perceptive and insightful',
      'Independent and innovative',
    ],
    averageTraits: [
      'Detached and preoccupied',
      'Conceptualizing and specializing',
      'Withdrawn and isolated',
      'Provocative and antagonistic',
      'Reducing needs and minimizing involvement',
    ],
    unhealthyTraits: [
      'Nihilistic and eccentric',
      'Isolated and terrified',
      'Distorted perceptions',
      'Rejecting all social attachments',
      'Schizoid and delusional',
    ],
  },
  6: {
    number: 6,
    name: 'The Loyalist',
    coreFear: 'Being without support or guidance',
    coreDesire: 'To have security and support',
    coreWeakness: 'Anxiety—scanning for threats and dangers',
    keyMotivations: [
      'Having security and feeling supported',
      'Testing attitudes of others toward them',
      'Fighting against anxiety and insecurity',
      'Finding trusted authority or belief system',
    ],
    wings: [5, 7],
    integrationPoint: 9,
    disintegrationPoint: 3,
    centerOfIntelligence: 'head',
    hornevianGroup: 'compliant',
    harmonicGroup: 'reactive',
    description:
      'Engaging, responsible, anxious, and suspicious. Sixes are reliable and hardworking, yet can be defensive. They fear being without support and seek security in systems and relationships.',
    healthyTraits: [
      'Courageous and leadership-capable',
      'Self-affirming and trusting',
      'Committed and reliable',
      'Cooperative and community-building',
      'Engaging and responsible',
    ],
    averageTraits: [
      'Anxious and vigilant',
      'Ambivalent and evasive',
      'Suspicious and reactive',
      'Defensive and blaming',
      'Seeking reassurance and security',
    ],
    unhealthyTraits: [
      'Paranoid and fearful',
      'Self-defeating and masochistic',
      'Panicky and volatile',
      'Hysterical and irrational',
      'Striking out at perceived enemies',
    ],
  },
  7: {
    number: 7,
    name: 'The Enthusiast',
    coreFear: 'Being deprived, in pain, or trapped',
    coreDesire: 'To be satisfied and content',
    coreWeakness: 'Gluttony—wanting more and more experiences',
    keyMotivations: [
      'Maintaining freedom and happiness',
      'Avoiding missing out on worthwhile experiences',
      'Keeping self excited and occupied',
      'Avoiding and discharging pain',
    ],
    wings: [6, 8],
    integrationPoint: 5,
    disintegrationPoint: 1,
    centerOfIntelligence: 'head',
    hornevianGroup: 'assertive',
    harmonicGroup: 'positive',
    description:
      'Spontaneous, versatile, acquisitive, and scattered. Sevens are enthusiastic, optimistic, and adventurous. They fear being trapped in pain and pursue endless positive experiences.',
    healthyTraits: [
      'Joyful and grateful',
      'Accomplished and focused',
      'Assimilating experiences deeply',
      'Present and awed by life',
      'Versatile and productive',
    ],
    averageTraits: [
      'Hyperactive and scattered',
      'Uninhibited and excessive',
      'Materialistic and acquisitive',
      'Self-centered and impatient',
      'Avoiding commitment and depth',
    ],
    unhealthyTraits: [
      'Impulsive and infantile',
      'Addictive and debauched',
      'Manic and out of control',
      'Panic-stricken and paralyzed',
      'Claustrophobic and erratic',
    ],
  },
  8: {
    number: 8,
    name: 'The Challenger',
    coreFear: 'Being harmed or controlled by others',
    coreDesire: 'To protect self and determine own direction',
    coreWeakness: 'Lust—wanting to be against the world and prove strength',
    keyMotivations: [
      'Being self-reliant and strong',
      'Resisting weakness and vulnerability',
      'Being important in their world',
      'Dominating the environment and controlling resources',
    ],
    wings: [7, 9],
    integrationPoint: 2,
    disintegrationPoint: 5,
    centerOfIntelligence: 'gut',
    hornevianGroup: 'assertive',
    harmonicGroup: 'reactive',
    description:
      'Self-confident, decisive, confrontational, and dominating. Eights are strong, resourceful, and protective. They fear being controlled and assert themselves powerfully.',
    healthyTraits: [
      'Magnanimous and heroic',
      'Self-mastering and empowering',
      'Courageous and protective',
      'Resourceful and decisive',
      'Authoritative and leading',
    ],
    averageTraits: [
      'Dominating and confrontational',
      'Proud and egocentric',
      'Willful and defiant',
      'Intimidating and threatening',
      'Struggling for control',
    ],
    unhealthyTraits: [
      'Ruthless and dictatorial',
      'Destroying before being destroyed',
      'Terrorizing and violent',
      'Sociopathic and delusional',
      'Megalomania and revenge-seeking',
    ],
  },
  9: {
    number: 9,
    name: 'The Peacemaker',
    coreFear: 'Loss of connection, fragmentation, or conflict',
    coreDesire: 'To have inner peace and stability',
    coreWeakness: 'Sloth—forgetting own priorities and going along to get along',
    keyMotivations: [
      'Creating harmony in environment',
      'Avoiding conflicts and tension',
      'Preserving things as they are',
      'Resisting whatever would upset or disturb them',
    ],
    wings: [8, 1],
    integrationPoint: 3,
    disintegrationPoint: 6,
    centerOfIntelligence: 'gut',
    hornevianGroup: 'withdrawn',
    harmonicGroup: 'positive',
    description:
      'Receptive, reassuring, agreeable, and complacent. Nines are stable, trusting, and supportive. They fear conflict and separation, often merging with others to maintain peace.',
    healthyTraits: [
      'Serene and at peace',
      'Present and self-possessed',
      'Accepting and unself-conscious',
      'Natural mediators and healers',
      'Optimistic and supportive',
    ],
    averageTraits: [
      'Self-effacing and accommodating',
      'Complacent and resigned',
      'Disengaged and unreflective',
      'Passive-aggressive when pushed',
      'Stubbornly resistant to change',
    ],
    unhealthyTraits: [
      'Dissociated and neglectful',
      'Obstinate and fatalistic',
      'Depersonalized and repressed',
      'Helpless and inadequate',
      'Severely disoriented',
    ],
  },
}

const ENNEAGRAM_CONNECTIONS: EnneagramConnection[] = [
  // Integration lines (growth)
  {
    from: 1,
    to: 7,
    type: 'integration',
    description: 'Ones access joy and spontaneity of healthy Sevens',
  },
  {
    from: 2,
    to: 4,
    type: 'integration',
    description: 'Twos access self-awareness and authenticity of healthy Fours',
  },
  {
    from: 3,
    to: 6,
    type: 'integration',
    description: 'Threes access loyalty and commitment of healthy Sixes',
  },
  {
    from: 4,
    to: 1,
    type: 'integration',
    description: 'Fours access objectivity and discipline of healthy Ones',
  },
  {
    from: 5,
    to: 8,
    type: 'integration',
    description: 'Fives access confidence and decisiveness of healthy Eights',
  },
  {
    from: 6,
    to: 9,
    type: 'integration',
    description: 'Sixes access peace and acceptance of healthy Nines',
  },
  {
    from: 7,
    to: 5,
    type: 'integration',
    description: 'Sevens access depth and focus of healthy Fives',
  },
  {
    from: 8,
    to: 2,
    type: 'integration',
    description: 'Eights access openheartedness and care of healthy Twos',
  },
  {
    from: 9,
    to: 3,
    type: 'integration',
    description: 'Nines access energy and self-development of healthy Threes',
  },

  // Disintegration lines (stress)
  {
    from: 1,
    to: 4,
    type: 'disintegration',
    description: 'Ones become moody and irrational like unhealthy Fours',
  },
  {
    from: 2,
    to: 8,
    type: 'disintegration',
    description: 'Twos become aggressive and dominating like unhealthy Eights',
  },
  {
    from: 3,
    to: 9,
    type: 'disintegration',
    description: 'Threes become apathetic and disengaged like unhealthy Nines',
  },
  {
    from: 4,
    to: 2,
    type: 'disintegration',
    description: 'Fours become clingy and manipulative like unhealthy Twos',
  },
  {
    from: 5,
    to: 7,
    type: 'disintegration',
    description: 'Fives become scattered and impulsive like unhealthy Sevens',
  },
  {
    from: 6,
    to: 3,
    type: 'disintegration',
    description: 'Sixes become competitive and image-conscious like unhealthy Threes',
  },
  {
    from: 7,
    to: 1,
    type: 'disintegration',
    description: 'Sevens become critical and perfectionistic like unhealthy Ones',
  },
  {
    from: 8,
    to: 5,
    type: 'disintegration',
    description: 'Eights become withdrawn and fearful like unhealthy Fives',
  },
  {
    from: 9,
    to: 6,
    type: 'disintegration',
    description: 'Nines become anxious and reactive like unhealthy Sixes',
  },
]

export function getEnneagramType(number: EnneagramNumber): EnneagramType {
  return ENNEAGRAM_TYPES[number]
}

export function getAllEnneagramTypes(): EnneagramType[] {
  return Object.values(ENNEAGRAM_TYPES)
}

export function getEnneagramConnections(number: EnneagramNumber): EnneagramConnection[] {
  return ENNEAGRAM_CONNECTIONS.filter((c) => c.from === number || c.to === number)
}

export function getIntegrationConnection(number: EnneagramNumber): EnneagramConnection | undefined {
  return ENNEAGRAM_CONNECTIONS.find((c) => c.from === number && c.type === 'integration')
}

export function getDisintegrationConnection(
  number: EnneagramNumber,
): EnneagramConnection | undefined {
  return ENNEAGRAM_CONNECTIONS.find((c) => c.from === number && c.type === 'disintegration')
}

export function getWingTypes(number: EnneagramNumber): [EnneagramType, EnneagramType] {
  const type = ENNEAGRAM_TYPES[number]
  return [ENNEAGRAM_TYPES[type.wings[0]], ENNEAGRAM_TYPES[type.wings[1]]]
}

export function getTypesByCenter(center: CenterOfIntelligence): EnneagramType[] {
  return Object.values(ENNEAGRAM_TYPES).filter((t) => t.centerOfIntelligence === center)
}

export function getTypesByHornevianGroup(group: HornevianGroup): EnneagramType[] {
  return Object.values(ENNEAGRAM_TYPES).filter((t) => t.hornevianGroup === group)
}

export function getTypesByHarmonicGroup(group: HarmonicGroup): EnneagramType[] {
  return Object.values(ENNEAGRAM_TYPES).filter((t) => t.harmonicGroup === group)
}

export { ENNEAGRAM_TYPES, ENNEAGRAM_CONNECTIONS }
