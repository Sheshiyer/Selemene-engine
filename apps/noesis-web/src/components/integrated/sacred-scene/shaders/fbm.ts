// ─── Fractal Brownian Motion built on simplex noise ──────────────────────
// Layers octaves of snoise at decreasing amplitude / increasing frequency
// to produce cloud-like, organic patterns. Used by SigilCore for plasma
// surface and SceneFog for atmospheric drift.
//
// Depends on snoise(vec3) being declared earlier in the shader (use
// NOISE_3D_GLSL before this string) and snoise2(vec2) for the 2D variant.

export const FBM_3D_GLSL = /* glsl */ `
float fbm3(vec3 p, int octaves, float lacunarity, float gain) {
  float amp = 0.5;
  float freq = 1.0;
  float sum = 0.0;
  for (int i = 0; i < 8; i++) {
    if (i >= octaves) break;
    sum += amp * snoise(p * freq);
    freq *= lacunarity;
    amp  *= gain;
  }
  return sum;
}
`;

export const FBM_2D_GLSL = /* glsl */ `
float fbm2(vec2 p, int octaves, float lacunarity, float gain) {
  float amp = 0.5;
  float freq = 1.0;
  float sum = 0.0;
  for (int i = 0; i < 8; i++) {
    if (i >= octaves) break;
    sum += amp * snoise2(p * freq);
    freq *= lacunarity;
    amp  *= gain;
  }
  return sum;
}
`;
