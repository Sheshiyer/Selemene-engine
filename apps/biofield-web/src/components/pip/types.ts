export interface PIPSettings {
  noiseScale: number;
  noiseSpeed: number;
  layerCount: number;
  intensity: number;
  colorShift: number;
  threshold: number;
}

export const DEFAULT_PIP_SETTINGS: PIPSettings = {
  noiseScale: 3.0,
  noiseSpeed: 0.12,
  layerCount: 4,
  intensity: 0.55,
  colorShift: 0.0,
  threshold: 0.35,
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
