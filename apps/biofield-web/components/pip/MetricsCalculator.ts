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

export class MetricsCalculator {
  private previousFrame: ImageData | null = null;

  calculateFromImageData(imageData: ImageData, mask?: Uint8Array): FrameMetrics {
    const { data, width, height } = imageData;
    const totalPixels = width * height;

    const intensities: number[] = [];
    const hues: number[] = [];
    const saturations: number[] = [];

    for (let i = 0; i < totalPixels; i++) {
      if (mask && mask[i] === 0) continue;
      const idx = i * 4;
      const r = data[idx];
      const g = data[idx + 1];
      const b = data[idx + 2];

      const intensity = 0.299 * r + 0.587 * g + 0.114 * b;
      intensities.push(intensity);

      const { h, s } = this.rgbToHsv(r, g, b);
      hues.push(h);
      saturations.push(s);
    }

    if (intensities.length === 0) {
      return {
        timestamp: Date.now(),
        avgIntensity: 0,
        intensityStdDev: 0,
        maxIntensity: 0,
        minIntensity: 0,
        lightQuantaDensity: 0,
        normalizedArea: 0,
        innerNoise: 0,
        innerNoisePercent: 0,
        horizontalSymmetry: 0,
        verticalSymmetry: 0,
        dominantHue: 0,
        saturationMean: 0,
        colorEntropy: 0,
        frameToFrameChange: 0,
      };
    }

    const avgIntensity = this.mean(intensities);
    const intensityStdDev = this.stdDev(intensities, avgIntensity);
    const lightQuantaDensity = avgIntensity / 255;
    const normalizedArea = intensities.length / totalPixels;
    const innerNoise = intensityStdDev / 255;
    const innerNoisePercent = innerNoise * 100;
    const dominantHue = this.circularMean(hues);
    const saturationMean = this.mean(saturations);

    return {
      timestamp: Date.now(),
      avgIntensity,
      intensityStdDev,
      maxIntensity: Math.max(...intensities),
      minIntensity: Math.min(...intensities),
      lightQuantaDensity,
      normalizedArea,
      innerNoise,
      innerNoisePercent,
      horizontalSymmetry: this.computeHorizontalSymmetry(data, width, height),
      verticalSymmetry: this.computeVerticalSymmetry(data, width, height),
      dominantHue,
      saturationMean,
      colorEntropy: this.computeEntropy(intensities),
      frameToFrameChange: this.computeFrameChange(imageData),
    };
  }

  calculateScores(metrics: FrameMetrics): CompositeScores {
    const energy = this.clamp(
      metrics.lightQuantaDensity * 55 + metrics.saturationMean * 25 + (1 - metrics.innerNoise) * 20,
      0,
      100
    );
    const symmetry = this.clamp(
      ((metrics.horizontalSymmetry + metrics.verticalSymmetry) / 2) * 100,
      0,
      100
    );
    const coherence = this.clamp(
      (100 - metrics.innerNoisePercent) * 0.6 + symmetry * 0.4,
      0,
      100
    );
    const complexity = this.clamp(
      metrics.colorEntropy * 12 + metrics.frameToFrameChange * 40,
      0,
      100
    );

    return {
      energy,
      symmetry,
      coherence,
      complexity,
      overall: this.clamp((energy + symmetry + coherence + complexity) / 4, 0, 100),
    };
  }

  private computeFrameChange(current: ImageData): number {
    if (!this.previousFrame) {
      this.previousFrame = current;
      return 0;
    }

    let diff = 0;
    const a = this.previousFrame.data;
    const b = current.data;
    const len = Math.min(a.length, b.length);

    for (let i = 0; i < len; i += 4) {
      diff += Math.abs(a[i] - b[i]);
      diff += Math.abs(a[i + 1] - b[i + 1]);
      diff += Math.abs(a[i + 2] - b[i + 2]);
    }

    this.previousFrame = current;
    return (diff / (len / 4)) / 255;
  }

  private computeHorizontalSymmetry(data: Uint8ClampedArray, width: number, height: number): number {
    let total = 0;
    let count = 0;

    for (let y = 0; y < height; y++) {
      for (let x = 0; x < Math.floor(width / 2); x++) {
        const left = (y * width + x) * 4;
        const right = (y * width + (width - 1 - x)) * 4;
        const li = 0.299 * data[left] + 0.587 * data[left + 1] + 0.114 * data[left + 2];
        const ri = 0.299 * data[right] + 0.587 * data[right + 1] + 0.114 * data[right + 2];
        total += 1 - Math.abs(li - ri) / 255;
        count++;
      }
    }

    return count > 0 ? total / count : 0;
  }

  private computeVerticalSymmetry(data: Uint8ClampedArray, width: number, height: number): number {
    let total = 0;
    let count = 0;

    for (let y = 0; y < Math.floor(height / 2); y++) {
      for (let x = 0; x < width; x++) {
        const top = (y * width + x) * 4;
        const bottom = ((height - 1 - y) * width + x) * 4;
        const ti = 0.299 * data[top] + 0.587 * data[top + 1] + 0.114 * data[top + 2];
        const bi = 0.299 * data[bottom] + 0.587 * data[bottom + 1] + 0.114 * data[bottom + 2];
        total += 1 - Math.abs(ti - bi) / 255;
        count++;
      }
    }

    return count > 0 ? total / count : 0;
  }

  private computeEntropy(intensities: number[]): number {
    const bins = new Array(16).fill(0);
    for (const i of intensities) bins[Math.min(15, Math.floor(i / 16))]++;
    const total = intensities.length;
    let entropy = 0;
    for (const count of bins) {
      if (count === 0) continue;
      const p = count / total;
      entropy -= p * Math.log2(p);
    }
    return entropy;
  }

  private rgbToHsv(r: number, g: number, b: number): { h: number; s: number } {
    const rn = r / 255;
    const gn = g / 255;
    const bn = b / 255;
    const max = Math.max(rn, gn, bn);
    const min = Math.min(rn, gn, bn);
    const d = max - min;

    let h = 0;
    if (d !== 0) {
      if (max === rn) h = ((gn - bn) / d) % 6;
      else if (max === gn) h = (bn - rn) / d + 2;
      else h = (rn - gn) / d + 4;
      h *= 60;
      if (h < 0) h += 360;
    }

    const s = max === 0 ? 0 : d / max;
    return { h, s };
  }

  private circularMean(values: number[]): number {
    if (values.length === 0) return 0;
    const sumSin = values.reduce((acc, h) => acc + Math.sin((h * Math.PI) / 180), 0);
    const sumCos = values.reduce((acc, h) => acc + Math.cos((h * Math.PI) / 180), 0);
    const angle = Math.atan2(sumSin / values.length, sumCos / values.length);
    return (angle * 180) / Math.PI;
  }

  private mean(values: number[]): number {
    return values.reduce((a, b) => a + b, 0) / values.length;
  }

  private stdDev(values: number[], mean: number): number {
    const variance = values.reduce((a, b) => a + Math.pow(b - mean, 2), 0) / values.length;
    return Math.sqrt(variance);
  }

  private clamp(v: number, min: number, max: number): number {
    return Math.min(max, Math.max(min, v));
  }
}
