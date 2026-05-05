import type { PIPSettings } from "./types";
import type { MediaPipeMask } from "./useMediaPipe";

// ─── Vertex shader ────────────────────────────────────────────────────────────
const VERT_SRC = `#version 300 es
in vec2 a_position;
out vec2 v_texCoord;
void main() {
  gl_Position = vec4(a_position, 0.0, 1.0);
  v_texCoord = a_position * 0.5 + 0.5;
}`;

// ─── Fragment shader — exact TouchDesigner noise algorithm ────────────────────
// Ported 1:1 from touchdesigner-embed.html with our segmentation mask on top.
const FRAG_SRC = `#version 300 es
precision highp float;

uniform sampler2D u_video;
uniform sampler2D u_mask;     // MediaPipe selfie segmentation confidence mask
uniform float u_maskStrength; // 0 = mask off, 1 = person-gate active
uniform float u_time;
uniform float u_intensity;    // overall composite strength (default 0.96)

in vec2 v_texCoord;
out vec4 fragColor;

// ── Baked TouchDesigner parameters (from touchdesigner-embed.html) ────────────
const float SEED      = 8057.0;
const float PERIOD    = 0.06;   // spatial frequency → scale = 1/PERIOD ≈ 16.7
const float SPREAD    = 2.0;    // lacunarity
const float GAIN      = 0.29;   // persistence
const float ROUGHNESS = 0.33;
const float EXPONENT  = 1.04;
const float AMPLITUDE = 0.96;
const float OFFSET    = 0.47;
const float VIDEO_INF = 0.3;    // how much video RGB bends the noise coord
const float HUE_SHIFT = 0.82;
const float COLOR_SAT = 0.83;
const float BLUR_AMT  = 2.0;    // soft blur blend — keep low to preserve vivid false-color

// ── 3D Simplex noise (Ashima Arts) ───────────────────────────────────────────
vec3 mod289(vec3 x) { return x - floor(x*(1.0/289.0))*289.0; }
vec4 mod289(vec4 x) { return x - floor(x*(1.0/289.0))*289.0; }
vec4 permute(vec4 x) { return mod289(((x*34.0)+1.0)*x); }
vec4 taylorInvSqrt(vec4 r) { return 1.79284291400159 - 0.85373472095314*r; }

float snoise(vec3 v) {
  const vec2 C = vec2(1.0/6.0, 1.0/3.0);
  const vec4 D = vec4(0.0, 0.5, 1.0, 2.0);
  vec3 i  = floor(v + dot(v, C.yyy));
  vec3 x0 = v - i + dot(i, C.xxx);
  vec3 g  = step(x0.yzx, x0.xyz);
  vec3 l  = 1.0 - g;
  vec3 i1 = min(g.xyz, l.zxy);
  vec3 i2 = max(g.xyz, l.zxy);
  vec3 x1 = x0 - i1 + C.xxx;
  vec3 x2 = x0 - i2 + C.yyy;
  vec3 x3 = x0 - D.yyy;
  i = mod289(i);
  vec4 p = permute(permute(permute(
      i.z + vec4(0.0, i1.z, i2.z, 1.0))
    + i.y + vec4(0.0, i1.y, i2.y, 1.0))
    + i.x + vec4(0.0, i1.x, i2.x, 1.0));
  vec4 j  = p - 49.0*floor(p*(1.0/49.0));
  vec4 x_ = floor(j*(1.0/7.0));
  vec4 y_ = floor(j - 7.0*x_);
  vec4 x  = x_*(1.0/7.0) + (1.0/14.0);
  vec4 y  = y_*(1.0/7.0) + (1.0/14.0);
  vec4 h  = 1.0 - abs(x) - abs(y);
  vec4 b0 = vec4(x.xy, y.xy);
  vec4 b1 = vec4(x.zw, y.zw);
  vec4 s0 = floor(b0)*2.0 + 1.0;
  vec4 s1 = floor(b1)*2.0 + 1.0;
  vec4 sh = -step(h, vec4(0.0));
  vec4 a0 = b0.xzyw + s0.xzyw*sh.xxyy;
  vec4 a1 = b1.xzyw + s1.xzyw*sh.zzww;
  vec3 p0 = vec3(a0.xy, h.x);
  vec3 p1 = vec3(a0.zw, h.y);
  vec3 p2 = vec3(a1.xy, h.z);
  vec3 p3 = vec3(a1.zw, h.w);
  vec4 norm = taylorInvSqrt(vec4(dot(p0,p0),dot(p1,p1),dot(p2,p2),dot(p3,p3)));
  p0 *= norm.x; p1 *= norm.y; p2 *= norm.z; p3 *= norm.w;
  vec4 m = max(0.6 - vec4(dot(x0,x0),dot(x1,x1),dot(x2,x2),dot(x3,x3)), 0.0);
  m = m*m;
  return 42.0 * dot(m*m, vec4(dot(p0,x0),dot(p1,x1),dot(p2,x2),dot(p3,x3)));
}

// ── fBm (4 octaves, exact TD constants) ──────────────────────────────────────
float fbm(vec3 p) {
  p += vec3(SEED * 0.001);
  float f = 0.0, amp = 1.0, maxV = 0.0;
  for (int i = 0; i < 4; i++) {
    float roughAmp = amp * (1.0 - ROUGHNESS * float(i) / 8.0);
    f    += roughAmp * snoise(p);
    maxV += roughAmp;
    p    *= SPREAD;
    amp  *= GAIN;
  }
  return maxV > 0.0 ? f / maxV : f;
}

// ── HSL helpers ───────────────────────────────────────────────────────────────
vec3 rgb2hsl(vec3 c) {
  float M = max(c.r, max(c.g, c.b));
  float m = min(c.r, min(c.g, c.b));
  float d = M - m;
  float l = (M + m) * 0.5;
  float s = d == 0.0 ? 0.0 : d / (1.0 - abs(2.0*l - 1.0));
  float h = 0.0;
  if (d > 0.0) {
    if      (M == c.r) h = mod((c.g-c.b)/d + (c.g < c.b ? 6.0 : 0.0), 6.0);
    else if (M == c.g) h = (c.b-c.r)/d + 2.0;
    else               h = (c.r-c.g)/d + 4.0;
    h /= 6.0;
  }
  return vec3(h, s, l);
}
float hue2rgb(float p, float q, float t) {
  if (t < 0.0) t += 1.0;
  if (t > 1.0) t -= 1.0;
  if (t < 1.0/6.0) return p + (q-p)*6.0*t;
  if (t < 0.5)     return q;
  if (t < 2.0/3.0) return p + (q-p)*(2.0/3.0-t)*6.0;
  return p;
}
vec3 hsl2rgb(vec3 c) {
  if (c.y == 0.0) return vec3(c.z);
  float q = c.z < 0.5 ? c.z*(1.0+c.y) : c.z+c.y-c.z*c.y;
  float p = 2.0*c.z - q;
  return vec3(hue2rgb(p,q,c.x+1.0/3.0), hue2rgb(p,q,c.x), hue2rgb(p,q,c.x-1.0/3.0));
}

// Color blend mode: overlay hue+sat onto base luminosity — preserves face detail
vec3 tdColorComposite(vec3 base, vec3 overlay) {
  vec3 hb = rgb2hsl(base);
  vec3 ho = rgb2hsl(overlay);
  return hsl2rgb(vec3(ho.x, mix(hb.y, ho.y, 0.8), hb.z));
}

// 3×3 Gaussian blur for soft energy-field look
vec3 gaussianBlur(sampler2D tex, vec2 uv, float amount) {
  if (amount <= 0.0) return texture(tex, uv).rgb;
  vec2 ts = vec2(1.0) / vec2(textureSize(tex, 0));
  float w[9];
  w[0]=0.0625; w[1]=0.125; w[2]=0.0625;
  w[3]=0.125;  w[4]=0.25;  w[5]=0.125;
  w[6]=0.0625; w[7]=0.125; w[8]=0.0625;
  vec2 o[9];
  o[0]=vec2(-1,-1); o[1]=vec2(0,-1); o[2]=vec2(1,-1);
  o[3]=vec2(-1, 0); o[4]=vec2(0, 0); o[5]=vec2(1, 0);
  o[6]=vec2(-1, 1); o[7]=vec2(0, 1); o[8]=vec2(1, 1);
  vec3 r = vec3(0.0);
  for (int i = 0; i < 9; i++) r += texture(tex, uv + o[i]*ts*amount).rgb * w[i];
  return r;
}

// Noise → HSL colour (video hue influences output when noise is low)
vec3 enhancedNoiseColor(float n, vec3 videoCol) {
  vec3 vHSL = rgb2hsl(videoCol);
  float hue = n * HUE_SHIFT + vHSL.x * (1.0 - HUE_SHIFT);
  float sat = clamp(mix(0.85, 1.0, fract(n + 0.33)) * 0.97, 0.0, 1.0);
  float lit = mix(0.35, 0.95, fract(n + 0.66));
  return hsl2rgb(vec3(hue, sat, lit));
}

void main() {
  // WebGL UV origin is bottom-left; video texture expects top-left origin.
  vec2 uv = vec2(v_texCoord.x, 1.0 - v_texCoord.y);
  vec3 videoCol = texture(u_video, uv).rgb;

  // ── Spiral coordinate: radial bands + angular phase = concentric spiral ──
  // Radial distance from centre gives concentric rings; subtracting the angle
  // twists them into a right-hand spiral.  Time rotates the whole field.
  vec2 center = uv - vec2(0.5);
  float r      = length(center) * 5.5;
  float theta  = atan(center.y, center.x);
  float spiral = r - theta * 0.45;
  // Video RGB bends the spiral locally (VIDEO_INF = 30%) — person's body warps the field
  vec3 noiseCoord = vec3(uv / max(PERIOD, 0.0001), spiral + u_time * 0.12)
                  + videoCol * 2.0 * VIDEO_INF;  // videoCol is already vec3

  float n = fbm(noiseCoord);
  n = sign(n) * pow(abs(n), EXPONENT);
  n = clamp(n * AMPLITUDE + OFFSET, 0.0, 1.0);

  // Composite: TD Color blend mode (hue+sat from noise, luminosity from video).
  vec3 noiseCol = enhancedNoiseColor(n, videoCol);
  vec3 composite = tdColorComposite(videoCol, noiseCol);
  composite = mix(videoCol, composite, u_intensity);

  // Soft blur blend — mixes in 59 % blurred video for energy-field softness.
  vec3 blurred = gaussianBlur(u_video, uv, BLUR_AMT * 0.01);
  composite = mix(composite, blurred, clamp(BLUR_AMT * 0.1, 0.0, 0.8));

  // Segmentation mask: person pixels = full composite, background = 80%.
  // Before mask loads (maskStrength=0) the full composite shows everywhere.
  // Use uv to match video orientation (top-down).
  float personConf   = texture(u_mask, uv).r;
  float bgAlpha      = mix(1.0, 0.8, u_maskStrength);   // background keeps 80% composite
  float pixelAlpha   = mix(bgAlpha, 1.0, personConf * u_maskStrength);
  vec3 outCol = mix(videoCol, composite, pixelAlpha);

  fragColor = vec4(clamp(outCol, 0.0, 1.0), 1.0);
}`;

const UNIFORM_NAMES = [
  "u_video", "u_mask", "u_maskStrength", "u_time", "u_intensity",
] as const;

type UniformName = (typeof UNIFORM_NAMES)[number];

export class PIPRenderer {
  private gl: WebGL2RenderingContext | null = null;
  private program: WebGLProgram | null = null;
  private videoTexture: WebGLTexture | null = null;
  private maskTexture: WebGLTexture | null = null;
  private vao: WebGLVertexArrayObject | null = null;
  private uniforms: Partial<Record<UniformName, WebGLUniformLocation | null>> = {};

  constructor(private readonly canvas: HTMLCanvasElement) {}

  /** Must be called from useEffect (after canvas is mounted in the DOM). */
  init(): boolean {
    const gl = this.canvas.getContext("webgl2");
    if (!gl) {
      console.warn("[PIPRenderer] WebGL2 not available.");
      return false;
    }
    this.gl = gl;

    const prog = this.buildProgram(gl, VERT_SRC, FRAG_SRC);
    if (!prog) return false;
    this.program = prog;

    // Fullscreen quad: two triangles covering clip space [-1, 1]².
    const vao = gl.createVertexArray();
    if (!vao) return false;
    gl.bindVertexArray(vao);

    const buf = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buf);
    gl.bufferData(
      gl.ARRAY_BUFFER,
      new Float32Array([-1, -1,  1, -1, -1,  1,  -1,  1,  1, -1,  1,  1]),
      gl.STATIC_DRAW,
    );

    const posLoc = gl.getAttribLocation(prog, "a_position");
    gl.enableVertexAttribArray(posLoc);
    gl.vertexAttribPointer(posLoc, 2, gl.FLOAT, false, 0, 0);
    gl.bindVertexArray(null);
    this.vao = vao;

    // Video texture — updated each frame via texImage2D.
    const tex = gl.createTexture();
    if (!tex) return false;
    gl.bindTexture(gl.TEXTURE_2D, tex);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
    this.videoTexture = tex;

    // Mask texture — updated each frame from MediaPipe confidence mask.
    const maskTex = gl.createTexture();
    if (!maskTex) return false;
    gl.bindTexture(gl.TEXTURE_2D, maskTex);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
    // Seed a 1×1 white pixel so the shader has a valid sampler before any mask arrives.
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.R32F, 1, 1, 0, gl.RED, gl.FLOAT, new Float32Array([1]));
    this.maskTexture = maskTex;

    for (const name of UNIFORM_NAMES) {
      this.uniforms[name] = gl.getUniformLocation(prog, name);
    }

    return true;
  }

  render(video: HTMLVideoElement, timeMs: number, s: PIPSettings, mask?: MediaPipeMask | null): void {
    const gl = this.gl;
    if (!gl || !this.program || !this.vao) return;

    // Keep canvas resolution in sync with its CSS display size.
    const w = this.canvas.clientWidth  || 640;
    const h = this.canvas.clientHeight || 480;
    if (this.canvas.width !== w || this.canvas.height !== h) {
      this.canvas.width  = w;
      this.canvas.height = h;
    }
    gl.viewport(0, 0, gl.drawingBufferWidth, gl.drawingBufferHeight);

    // Upload current video frame as texture.
    if (video.readyState >= video.HAVE_CURRENT_DATA) {
      gl.bindTexture(gl.TEXTURE_2D, this.videoTexture);
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, video);
    }

    // Upload segmentation mask when available (single-channel float R32F).
    const maskStrength = mask ? 1.0 : 0.0;
    if (mask) {
      gl.bindTexture(gl.TEXTURE_2D, this.maskTexture);
      gl.texImage2D(
        gl.TEXTURE_2D, 0, gl.R32F,
        mask.width, mask.height, 0,
        gl.RED, gl.FLOAT, mask.data,
      );
    }

    gl.useProgram(this.program);
    gl.bindVertexArray(this.vao);

    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_2D, this.videoTexture);

    gl.activeTexture(gl.TEXTURE1);
    gl.bindTexture(gl.TEXTURE_2D, this.maskTexture);

    gl.uniform1i(this.uniforms.u_video ?? null, 0);
    gl.uniform1i(this.uniforms.u_mask ?? null, 1);
    gl.uniform1f(this.uniforms.u_maskStrength ?? null, maskStrength);
    gl.uniform1f(this.uniforms.u_time ?? null, timeMs / 1000);
    gl.uniform1f(this.uniforms.u_intensity ?? null, s.intensity);

    gl.drawArrays(gl.TRIANGLES, 0, 6);
    gl.bindVertexArray(null);
  }

  dispose(): void {
    const gl = this.gl;
    if (!gl) return;
    if (this.videoTexture) gl.deleteTexture(this.videoTexture);
    if (this.maskTexture)  gl.deleteTexture(this.maskTexture);
    if (this.program)      gl.deleteProgram(this.program);
    if (this.vao)          gl.deleteVertexArray(this.vao);
    this.gl = null;
    this.program = null;
  }

  private buildProgram(
    gl: WebGL2RenderingContext,
    vertSrc: string,
    fragSrc: string,
  ): WebGLProgram | null {
    const vert = this.compileShader(gl, gl.VERTEX_SHADER, vertSrc);
    const frag = this.compileShader(gl, gl.FRAGMENT_SHADER, fragSrc);
    if (!vert || !frag) return null;

    const prog = gl.createProgram();
    if (!prog) return null;
    gl.attachShader(prog, vert);
    gl.attachShader(prog, frag);
    gl.linkProgram(prog);
    gl.deleteShader(vert);
    gl.deleteShader(frag);

    if (!gl.getProgramParameter(prog, gl.LINK_STATUS)) {
      console.error("[PIPRenderer] link error:", gl.getProgramInfoLog(prog));
      gl.deleteProgram(prog);
      return null;
    }
    return prog;
  }

  private compileShader(
    gl: WebGL2RenderingContext,
    type: number,
    src: string,
  ): WebGLShader | null {
    const shader = gl.createShader(type);
    if (!shader) return null;
    gl.shaderSource(shader, src);
    gl.compileShader(shader);
    if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
      console.error("[PIPRenderer] shader compile error:", gl.getShaderInfoLog(shader));
      gl.deleteShader(shader);
      return null;
    }
    return shader;
  }
}
