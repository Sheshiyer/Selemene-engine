// ─── Iridescence — view-angle-based color shift ──────────────────────────
// HSV → RGB conversion + a Fresnel-driven hue rotation. Used on
// WaveRibbon and as an optional accent on SigilCore edge.

export const IRIDESCENCE_GLSL = /* glsl */ `
vec3 hsv2rgb(vec3 c) {
  vec4 K = vec4(1.0, 2.0/3.0, 1.0/3.0, 3.0);
  vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
  return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

// fresnel-like factor — 0 at perpendicular view, 1 at grazing angles
float fresnelF(vec3 viewDir, vec3 normal, float power) {
  return pow(1.0 - clamp(dot(normalize(viewDir), normalize(normal)), 0.0, 1.0), power);
}

// returns an iridescent color biased toward baseHue, modulated by view angle
vec3 iridescent(vec3 viewDir, vec3 normal, float baseHue, float saturation, float value) {
  float fres = fresnelF(viewDir, normal, 2.5);
  float hue  = fract(baseHue + fres * 0.55);
  return hsv2rgb(vec3(hue, saturation, value));
}
`;
