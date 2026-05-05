export interface PIPSettings {
  noiseScale: number;
  noiseSpeed: number;
  layerCount: number;
  intensity: number;
  colorShift: number;
  threshold: number;
}

export const DEFAULT_PIP_SETTINGS: PIPSettings = {
  noiseScale: 3.5,
  noiseSpeed: 0.18,
  layerCount: 5,
  intensity: 1.0,   // full composite — shader's BLUR_AMT controls the final blend
  colorShift: 0.0,
  threshold: 0.0,   // kept for API compat; not used in shader
};

export interface FrameMetrics {
  averageLuminance: number;
  pixelVariance: number;
  symmetryScore: number;
  entropyScore: number;
  timestamp: number;
}

export interface CompositeScores {
  lightQuantaDensity: number;
  normalizedArea: number;
  bodySymmetry: number;
  patternRegularity: number;
  overallCoherence: number;
}
