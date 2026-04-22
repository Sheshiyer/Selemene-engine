/**
 * PIPRenderer — WebGL2 Polycontrast Interference Photography shader.
 *
 * Ported from bv-pip-analysis prototype. Renders a live camera feed through
 * a Simplex-noise fbm shader that produces the PIP biofield visualization.
 * No ML dependencies — pure WebGL2 + camera.
 *
 * The shader compiles inline (no .glsl imports) so it works in Next.js
 * without additional webpack config. WebGL2 context is created lazily in
 * init() to avoid SSR issues — always call after mount.
 */

import type { PIPSettings } from "./types";
import { DEFAULT_PIP_SETTINGS } from "./types";

// ─── Vertex Shader ────────────────────────────────────────────────────────────
const VERTEX_SHADER = `#version 300 es
in vec2 aPosition;
out vec2 vUV;
void main(){
  vUV = aPosition * 0.5 + 0.5;
  vUV.y = 1.0 - vUV.y;
  gl_Position = vec4(aPosition, 0., 1.);
}`;

// ─── Fragment Shader ───────────────────────────────────────────────────────────
const FRAGMENT_SHADER = `#version 300 es
precision highp float;
in vec2 vUV;
out vec4 fragColor;

uniform sampler2D uVideo;
uniform sampler2D uMask;
uniform float uTime;
uniform float uSeed;
uniform float uPeriod;
uniform int uHarmonics;
uniform float uSpread;
uniform float uGain;
uniform float uRoughness;
uniform float uExponent;
uniform float uAmplitude;
uniform float uOffset;
uniform int uMonochrome;
uniform float uIntensity;
uniform int uUseMask;

vec3 mod289(vec3 x){ return x - floor(x*(1.0/289.0))*289.0; }
vec4 mod289(vec4 x){ return x - floor(x*(1.0/289.0))*289.0; }
vec4 permute(vec4 x){ return mod289(((x*34.0)+1.0)*x); }
vec4 taylorInvSqrt(vec4 r){ return 1.79284291400159 - 0.85373472095314*r; }

float snoise(vec3 v){
  const vec2 C = vec2(1.0/6.0, 1.0/3.0);
  const vec4 D = vec4(0.0,0.5,1.0,2.0);
  vec3 i = floor(v + dot(v,C.yyy));
  vec3 x0= v - i + dot(i,C.xxx);
  vec3 g = step(x0.yzx, x0.xyz);
  vec3 l = 1.0 - g;
  vec3 i1= min(g.xyz,l.zxy), i2= max(g.xyz,l.zxy);
  vec3 x1= x0 - i1 + C.xxx;
  vec3 x2= x0 - i2 + C.yyy;
  vec3 x3= x0 - D.yyy;
  i = mod289(i);
  vec4 p = permute( permute( permute(
    i.z + vec4(0.0,i1.z,i2.z,1.0))
    + i.y + vec4(0.0,i1.y,i2.y,1.0))
    + i.x + vec4(0.0,i1.x,i2.x,1.0));
  vec4 j = p - 49.0*floor(p*(1.0/49.0));
  vec4 x_ = floor(j*(1.0/7.0));
  vec4 y_ = floor(j - 7.0*x_);
  vec4 x = x_*(1.0/7.0) + (1.0/14.0);
  vec4 y = y_*(1.0/7.0) + (1.0/14.0);
  vec4 h = 1.0 - abs(x)-abs(y);
  vec4 b0 = vec4(x.xy,y.xy), b1 = vec4(x.zw,y.zw);
  vec4 s0 = floor(b0)*2.0+1.0, s1 = floor(b1)*2.0+1.0;
  vec4 sh = -step(h, vec4(0.0));
  vec4 a0 = b0.xzyw + s0.xzyw*sh.xxyy;
  vec4 a1 = b1.xzyw + s1.xzyw*sh.zzww;
  vec3 p0=vec3(a0.xy,h.x), p1=vec3(a0.zw,h.y),
       p2=vec3(a1.xy,h.z), p3=vec3(a1.zw,h.w);
  vec4 norm = taylorInvSqrt(vec4(
    dot(p0,p0), dot(p1,p1), dot(p2,p2), dot(p3,p3)));
  p0*=norm.x; p1*=norm.y; p2*=norm.z; p3*=norm.w;
  vec4 m = max(0.6 - vec4(
    dot(x0,x0), dot(x1,x1), dot(x2,x2), dot(x3,x3)), 0.0);
  m = m*m;
  return 42.0 * dot(m*m, vec4(
    dot(p0,x0), dot(p1,x1), dot(p2,x2), dot(p3,x3)));
}

float fbm(vec3 p){
  p += vec3(uSeed * 0.001);
  float f = 0.0, amp = 1.0, maxValue = 0.0;
  for(int i = 0; i < 8; i++){
    if(i >= uHarmonics) break;
    float n = snoise(p);
    float roughAmp = amp * (1.0 - uRoughness * float(i) / 8.0);
    f += roughAmp * n;
    maxValue += roughAmp;
    p *= uSpread;
    amp *= uGain;
  }
  if(maxValue > 0.0) f /= maxValue;
  f = sign(f) * pow(abs(f), uExponent);
  return f;
}

vec3 mod289_3(vec3 c){ return c; }
float hue2rgb(float p, float q, float t){
  if(t<0.0) t+=1.0; if(t>1.0) t-=1.0;
  if(t<1.0/6.0) return p+(q-p)*6.0*t;
  if(t<1.0/2.0) return q;
  if(t<2.0/3.0) return p+(q-p)*(2.0/3.0-t)*6.0;
  return p;
}

vec3 rgb2hsl(vec3 c){
  float M=max(c.r,max(c.g,c.b)), m=min(c.r,min(c.g,c.b)),
        d=M-m, l=(M+m)*0.5,
        s=d==0.0?0.0:d/(1.0-abs(2.0*l-1.0));
  float h=0.0;
  if(d>0.0){
    if(M==c.r) h=mod((c.g-c.b)/d+(c.g<c.b?6.0:0.0),6.0);
    else if(M==c.g) h=(c.b-c.r)/d+2.0;
    else h=(c.r-c.g)/d+4.0;
    h/=6.0;
  }
  return vec3(h,s,l);
}
vec3 hsl2rgb(vec3 c){
  float h=c.x,s=c.y,l=c.z;
  if(s==0.0) return vec3(l);
  float q=l<0.5?l*(1.0+s):l+s-l*s, p=2.0*l-q;
  return vec3(hue2rgb(p,q,h+1.0/3.0),hue2rgb(p,q,h),hue2rgb(p,q,h-1.0/3.0));
}

void main(){
  vec3 videoCol = texture(uVideo, vUV).rgb;
  float maskValue = 1.0;
  if (uUseMask == 1) {
    maskValue = texture(uMask, vUV).r;
  }
  float frequency = 1.0 / max(uPeriod, 0.001);
  float n = fbm(vec3(vUV * frequency, uTime));
  n = clamp(n * uAmplitude + uOffset, 0.0, 1.0);
  vec3 final;
  if(uMonochrome == 1){
    float m = 1.0 + (n - 0.5) * uIntensity * 2.0;
    final = videoCol * m;
  } else {
    vec2 uvFreq = vUV * frequency;
    float rN = clamp(fbm(vec3(uvFreq+vec2(100.0),uTime))*uAmplitude+uOffset,0.0,1.0);
    float gN = clamp(fbm(vec3(uvFreq+vec2(200.0),uTime))*uAmplitude+uOffset,0.0,1.0);
    float bN = clamp(fbm(vec3(uvFreq+vec2(300.0),uTime))*uAmplitude+uOffset,0.0,1.0);
    vec3 noiseColor = vec3(rN,gN,bN);
    vec3 blended = hsl2rgb(vec3(rgb2hsl(noiseColor).x, rgb2hsl(noiseColor).y, rgb2hsl(videoCol).z));
    final = mix(videoCol, blended, uIntensity);
  }
  vec3 masked = mix(videoCol, final, maskValue);
  fragColor = vec4(masked, 1.0);
}`;

// ─── Renderer Class ───────────────────────────────────────────────────────────

export class PIPRenderer {
  private canvas: HTMLCanvasElement;
  private gl: WebGL2RenderingContext | null = null;
  private program: WebGLProgram | null = null;
  private videoTexture: WebGLTexture | null = null;
  private maskTexture: WebGLTexture | null = null;
  private vao: WebGLVertexArrayObject | null = null;
  private uniforms: Record<string, WebGLUniformLocation | null> = {};
  private settings: PIPSettings;
  private startTime = 0;
  private animationId: number | null = null;
  private video: HTMLVideoElement | null = null;
  private mask: { data: Uint8Array; width: number; height: number } | null = null;

  constructor(canvas: HTMLCanvasElement) {
    this.canvas = canvas;
    this.settings = { ...DEFAULT_PIP_SETTINGS };
  }

  /**
   * Must be called after the canvas is mounted in the DOM.
   * Getting the context in the constructor races with React's render cycle
   * and is the most common cause of "shader not loading" in local dev.
   */
  init(): void {
    const gl = this.canvas.getContext("webgl2", { premultipliedAlpha: false });
    if (!gl) throw new Error("WebGL2 not supported in this browser/context");
    this.gl = gl;

    this.program = this.createProgram(VERTEX_SHADER, FRAGMENT_SHADER);
    gl.useProgram(this.program);

    // Full-screen triangle (1 draw call covers entire viewport)
    this.vao = gl.createVertexArray();
    gl.bindVertexArray(this.vao);
    const buf = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buf);
    gl.bufferData(
      gl.ARRAY_BUFFER,
      new Float32Array([-1, -1, 3, -1, -1, 3]),
      gl.STATIC_DRAW
    );
    const aPos = gl.getAttribLocation(this.program, "aPosition");
    gl.enableVertexAttribArray(aPos);
    gl.vertexAttribPointer(aPos, 2, gl.FLOAT, false, 0, 0);

    // Video texture
    this.videoTexture = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, this.videoTexture);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);

    // Mask texture
    this.maskTexture = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, this.maskTexture);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);

    // Uniform locations
    const names = [
      "uVideo","uMask","uTime","uSeed","uPeriod","uHarmonics",
      "uSpread","uGain","uRoughness","uExponent",
      "uAmplitude","uOffset","uMonochrome","uIntensity","uUseMask",
    ];
    for (const name of names) {
      this.uniforms[name] = gl.getUniformLocation(this.program, name);
    }
    gl.uniform1i(this.uniforms.uVideo, 0);
    gl.uniform1i(this.uniforms.uMask, 1);

    this.startTime = performance.now();
  }

  setVideoSource(video: HTMLVideoElement): void {
    this.video = video;
  }

  setMask(mask: Uint8Array | null, width: number, height: number): void {
    if (!mask) {
      this.mask = null;
      return;
    }
    this.mask = { data: mask, width, height };
  }

  start(): void {
    if (this.animationId !== null) return;
    this.loop();
  }

  stop(): void {
    if (this.animationId !== null) {
      cancelAnimationFrame(this.animationId);
      this.animationId = null;
    }
  }

  pause(): void { this.stop(); }
  resume(): void { this.start(); }

  setParameter<K extends keyof PIPSettings>(name: K, value: PIPSettings[K]): void {
    this.settings[name] = value;
  }

  loadPreset(preset: Partial<PIPSettings>): void {
    this.settings = { ...this.settings, ...preset };
  }

  captureFrameAsDataURL(type = "image/png", quality = 0.95): string {
    return this.canvas.toDataURL(type, quality);
  }

  destroy(): void {
    this.stop();
    if (this.gl && this.program) {
      this.gl.deleteProgram(this.program);
    }
    this.gl = null;
    this.program = null;
  }

  private loop = (): void => {
    this.render();
    this.animationId = requestAnimationFrame(this.loop);
  };

  private render(): void {
    const { gl, video, settings } = this;
    if (!gl || !this.program) return;

    // Match canvas resolution to CSS size
    const w = this.canvas.clientWidth || 640;
    const h = this.canvas.clientHeight || 480;
    if (this.canvas.width !== w || this.canvas.height !== h) {
      this.canvas.width = w;
      this.canvas.height = h;
      gl.viewport(0, 0, w, h);
    }

    gl.clear(gl.COLOR_BUFFER_BIT);

    if (video && video.readyState >= video.HAVE_CURRENT_DATA) {
      gl.activeTexture(gl.TEXTURE0);
      gl.bindTexture(gl.TEXTURE_2D, this.videoTexture);
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGB, gl.RGB, gl.UNSIGNED_BYTE, video);
    }

    if (this.mask && this.maskTexture) {
      gl.activeTexture(gl.TEXTURE1);
      gl.bindTexture(gl.TEXTURE_2D, this.maskTexture);
      gl.texImage2D(
        gl.TEXTURE_2D,
        0,
        gl.R8,
        this.mask.width,
        this.mask.height,
        0,
        gl.RED,
        gl.UNSIGNED_BYTE,
        this.mask.data
      );
      gl.uniform1i(this.uniforms.uUseMask, 1);
    } else {
      gl.uniform1i(this.uniforms.uUseMask, 0);
    }

    const t = (performance.now() - this.startTime) * 0.001 * settings.speed;
    gl.uniform1f(this.uniforms.uTime, t);
    gl.uniform1f(this.uniforms.uSeed, settings.seed);
    gl.uniform1f(this.uniforms.uPeriod, settings.period);
    gl.uniform1i(this.uniforms.uHarmonics, settings.harmonics);
    gl.uniform1f(this.uniforms.uSpread, settings.spread);
    gl.uniform1f(this.uniforms.uGain, settings.gain);
    gl.uniform1f(this.uniforms.uRoughness, settings.roughness);
    gl.uniform1f(this.uniforms.uExponent, settings.exponent);
    gl.uniform1f(this.uniforms.uAmplitude, settings.amplitude);
    gl.uniform1f(this.uniforms.uOffset, settings.offset);
    gl.uniform1i(this.uniforms.uMonochrome, settings.monochrome ? 1 : 0);
    gl.uniform1f(this.uniforms.uIntensity, settings.intensity);

    gl.bindVertexArray(this.vao);
    gl.drawArrays(gl.TRIANGLES, 0, 3);
  }

  private createProgram(vertSrc: string, fragSrc: string): WebGLProgram {
    const gl = this.gl!;
    const vert = this.compileShader(gl.VERTEX_SHADER, vertSrc);
    const frag = this.compileShader(gl.FRAGMENT_SHADER, fragSrc);
    const prog = gl.createProgram()!;
    gl.attachShader(prog, vert);
    gl.attachShader(prog, frag);
    gl.linkProgram(prog);
    if (!gl.getProgramParameter(prog, gl.LINK_STATUS)) {
      const info = gl.getProgramInfoLog(prog);
      gl.deleteProgram(prog);
      throw new Error(`Shader program link failed: ${info}`);
    }
    gl.deleteShader(vert);
    gl.deleteShader(frag);
    return prog;
  }

  private compileShader(type: number, src: string): WebGLShader {
    const gl = this.gl!;
    const shader = gl.createShader(type)!;
    gl.shaderSource(shader, src);
    gl.compileShader(shader);
    if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
      const info = gl.getShaderInfoLog(shader);
      gl.deleteShader(shader);
      throw new Error(`Shader compile failed: ${info}`);
    }
    return shader;
  }
}
