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

// ─── Fragment shader (2D Simplex fBm → biofield palette) ─────────────────────
const FRAG_SRC = `#version 300 es
precision highp float;

uniform sampler2D u_video;
uniform sampler2D u_mask;      // MediaPipe selfie segmentation confidence mask
uniform float u_maskStrength;  // 0 = mask off, 1 = full mask gating
uniform float u_time;
uniform float u_noiseScale;
uniform float u_noiseSpeed;
uniform int u_layerCount;
uniform float u_intensity;
uniform float u_colorShift;
uniform float u_threshold;

in vec2 v_texCoord;
out vec4 fragColor;

// ── 2D Simplex noise (Gustavson) ─────────────────────────────────────────────
vec3 mod289v3(vec3 x) { return x - floor(x * (1.0/289.0)) * 289.0; }
vec2 mod289v2(vec2 x) { return x - floor(x * (1.0/289.0)) * 289.0; }
vec3 permute3(vec3 x) { return mod289v3(((x*34.0)+1.0)*x); }

float snoise(vec2 v) {
  const vec4 C = vec4(0.211324865405187, 0.366025403784439,
                     -0.577350269189626, 0.024390243902439);
  vec2 i  = floor(v + dot(v, C.yy));
  vec2 x0 = v - i + dot(i, C.xx);
  vec2 i1  = (x0.x > x0.y) ? vec2(1.0, 0.0) : vec2(0.0, 1.0);
  vec4 x12 = x0.xyxy + C.xxzz;
  x12.xy  -= i1;
  i = mod289v2(i);
  vec3 p = permute3(permute3(i.y + vec3(0.0, i1.y, 1.0))
                            + i.x + vec3(0.0, i1.x, 1.0));
  vec3 m  = max(0.5 - vec3(dot(x0,x0), dot(x12.xy,x12.xy), dot(x12.zw,x12.zw)), 0.0);
  m = m*m; m = m*m;
  vec3 x  = 2.0 * fract(p * C.www) - 1.0;
  vec3 h  = abs(x) - 0.5;
  vec3 ox = floor(x + 0.5);
  vec3 a0 = x - ox;
  m *= 1.79284291400159 - 0.85373472095314 * (a0*a0 + h*h);
  vec3 g;
  g.x  = a0.x  * x0.x   + h.x  * x0.y;
  g.yz = a0.yz * x12.xz + h.yz * x12.yw;
  return 130.0 * dot(m, g);
}

// ── Fractal Brownian Motion ───────────────────────────────────────────────────
float fbm(vec2 p, int octaves) {
  float v = 0.0, amp = 0.5, freq = 1.0;
  for (int i = 0; i < 8; i++) {
    if (i >= octaves) break;
    v    += snoise(p * freq) * amp;
    freq *= 2.0;
    amp  *= 0.5;
  }
  return v;
}

// ── Biofield colour palette ───────────────────────────────────────────────────
// Maps [0,1] to teal/blue/purple/gold interference fringe colours.
vec3 biofieldPalette(float t) {
  t = fract(t + u_colorShift);
  return vec3(0.5) + vec3(0.5) *
    cos(6.28318 * (vec3(1.0, 0.7, 0.4) * t + vec3(0.0, 0.15, 0.20)));
}

void main() {
  // Video texture is flipped vertically from WebGL's coordinate system.
  vec4 video = texture(u_video, vec2(v_texCoord.x, 1.0 - v_texCoord.y));

  vec2 p = v_texCoord * u_noiseScale;
  float n = fbm(p + u_time * u_noiseSpeed, u_layerCount) * 0.5 + 0.5;

  // Gate: only paint biofield overlay where noise exceeds threshold.
  float gate = smoothstep(u_threshold - 0.1, u_threshold + 0.1, n);
  vec3  pip  = biofieldPalette(n);

  // Segmentation mask: confidence value is in the red channel.
  // u_maskStrength = 0 → ignore mask, 1 → only show biofield on person pixels.
  float personConf = texture(u_mask, v_texCoord).r;
  float maskGate   = mix(1.0, personConf, u_maskStrength);

  fragColor = vec4(mix(video.rgb, pip, u_intensity * gate * maskGate), 1.0);
}`;

const UNIFORM_NAMES = [
  "u_video", "u_mask", "u_maskStrength", "u_time", "u_noiseScale", "u_noiseSpeed",
  "u_layerCount", "u_intensity", "u_colorShift", "u_threshold",
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
    gl.uniform1f(this.uniforms.u_noiseScale ?? null, s.noiseScale);
    gl.uniform1f(this.uniforms.u_noiseSpeed ?? null, s.noiseSpeed);
    gl.uniform1i(this.uniforms.u_layerCount ?? null, s.layerCount);
    gl.uniform1f(this.uniforms.u_intensity ?? null, s.intensity);
    gl.uniform1f(this.uniforms.u_colorShift ?? null, s.colorShift);
    gl.uniform1f(this.uniforms.u_threshold ?? null, s.threshold);

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
