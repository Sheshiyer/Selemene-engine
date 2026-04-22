/**
 * PIP (Polycontrast Interference Photography) types
 * Ported from bv-pip-analysis prototype.
 */

export interface PIPSettings {
  seed: number;
  period: number;
  harmonics: number;
  spread: number;
  gain: number;
  roughness: number;
  exponent: number;
  amplitude: number;
  offset: number;
  speed: number;
  intensity: number;
  monochrome: boolean;
}

export const DEFAULT_PIP_SETTINGS: PIPSettings = {
  seed: 1348,
  period: 0.22,
  harmonics: 2,
  spread: 2.6,
  gain: 0.7,
  roughness: 0.5,
  exponent: 0.5,
  amplitude: 0.53,
  offset: 0.5,
  speed: 1.0,
  intensity: 1.0,
  monochrome: false,
};

export interface FrameMetrics {
  timestamp: number;
  avgIntensity: number;
  intensityStdDev: number;
  maxIntensity: number;
  minIntensity: number;
  lightQuantaDensity: number;
  normalizedArea: number;
  innerNoise: number;
  innerNoisePercent: number;
  horizontalSymmetry: number;
  verticalSymmetry: number;
  dominantHue: number;
  saturationMean: number;
  colorEntropy: number;
  frameToFrameChange: number;
}

export interface CompositeScores {
  energy: number;
  symmetry: number;
  coherence: number;
  complexity: number;
  overall: number;
}

export interface CaptureResult {
  dataUrl: string;
  metrics: FrameMetrics | null;
  scores: CompositeScores | null;
  timestamp: number;
}
