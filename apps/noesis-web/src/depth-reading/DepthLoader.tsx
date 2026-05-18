// ─── DepthLoader — fullscreen GLSL loading shader ─────────────────────
// A tiny raymarched volumetric nebula adapted from a #つぶやきGLSL
// (one-tweet GLSL by the demoscene community). Runs in its own WebGL2
// context on an overlay canvas, fades out via CSS opacity, then
// unmounts. While it's visible the DepthScene runs its intro animation
// (planes float in from far back).
//
// Lifecycle:
//   1. mount      → canvas appears at opacity 1, shader animates
//   2. parent sets `fadingOut={true}` (typically after some hold time)
//   3. opacity transitions to 0 over `fadeDurationMs`
//   4. onFadeComplete() fires; parent unmounts the loader
//
// Why raw WebGL2 (not Three.js): the loader needs to render BEFORE the
// main scene's heavy Three.js init, so we keep this dependency-free.

"use client";

import { useEffect, useRef, useState } from "react";

interface DepthLoaderProps {
  /** When true, start the CSS opacity fade-out. */
  fadingOut: boolean;
  /** Fade duration in ms (default 900). */
  fadeDurationMs?: number;
  /** Called once the fade-out transition completes — parent should unmount. */
  onFadeComplete: () => void;
  /** Optional caption shown center-bottom while loading. */
  caption?: string;
}

// ─── Shader sources (GLSL ES 3.0) ──────────────────────────────────────
// Vertex: a single oversized triangle that covers the viewport.
const VERT_SRC = /* glsl */ `#version 300 es
precision highp float;
out vec2 v_uv;
void main() {
  // Triangle covering [-1,1]^2 in clip space via gl_VertexID
  vec2 p = vec2(
    float((gl_VertexID & 1) << 2),  // 0, 4, 0
    float((gl_VertexID & 2) << 1)   // 0, 0, 4
  );
  v_uv = p * 0.5;
  gl_Position = vec4(p - 1.0, 0.0, 1.0);
}`;

// Fragment: the #つぶやきGLSL volumetric raymarcher, translated to
// proper GLSL ES 3.0. Hue locked to 0.6 (cyan-violet) so it lives in
// the Witness Violet / Flow Indigo palette of the rest of the scene.
const FRAG_SRC = /* glsl */ `#version 300 es
precision highp float;
uniform vec2 r;     // resolution
uniform float t;    // time (seconds)
out vec4 fragColor;

// Standard HSV → RGB
vec3 hsv(float h, float s, float v) {
  vec4 K = vec4(1.0, 2.0/3.0, 1.0/3.0, 3.0);
  vec3 p = abs(fract(vec3(h) + K.xyz) * 6.0 - K.www);
  return v * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), s);
}

void main() {
  vec2 FC = gl_FragCoord.xy;
  vec4 o = vec4(0.0);
  float i = 0.0, e = 0.0, R = 0.0, s = 0.0, g = 0.0;
  // q starts at (0, -1, -1) — original code does q.zy-- on a zero-init vec3
  vec3 q = vec3(0.0, -1.0, -1.0);
  vec3 p = vec3(0.0);
  vec3 d = vec3(FC / r - 0.6, 1.0);

  for (; i < 99.0; i++) {
    e += i / 8e5;
    o.rgb += hsv(0.6, R + g * 0.3, e * i / 40.0);
    s = 4.0;
    q += d * e * R * 0.2;
    p = q;
    g += p.y / s;
    R = length(p);
    // vec3(scalar, scalar, vec3) takes .x of the third arg → p.x preserved
    p = vec3(
      R - 0.5 + sin(t) * 0.02,
      exp2(mod(-p.z, s) / R) - 0.2,
      p.x
    );
    // --p.y : decrement then assign to e
    p.y -= 1.0;
    e = p.y;
    for (; s < 1e3; s += s) {
      e += 0.03 - abs(dot(sin(p.yzx * s), cos(p.xzz * s)) / s * 0.6);
    }
  }
  // Vignette so the edges fall off into pure void instead of clipping
  vec2 uv = FC / r * 2.0 - 1.0;
  float vig = smoothstep(1.35, 0.4, length(uv));
  fragColor = vec4(o.rgb * vig, 1.0);
}`;

function compileShader(gl: WebGL2RenderingContext, type: number, src: string): WebGLShader {
  const sh = gl.createShader(type)!;
  gl.shaderSource(sh, src);
  gl.compileShader(sh);
  if (!gl.getShaderParameter(sh, gl.COMPILE_STATUS)) {
    const log = gl.getShaderInfoLog(sh);
    gl.deleteShader(sh);
    throw new Error(`[DepthLoader] shader compile failed: ${log}`);
  }
  return sh;
}

function linkProgram(gl: WebGL2RenderingContext, vert: WebGLShader, frag: WebGLShader): WebGLProgram {
  const prog = gl.createProgram()!;
  gl.attachShader(prog, vert);
  gl.attachShader(prog, frag);
  gl.linkProgram(prog);
  if (!gl.getProgramParameter(prog, gl.LINK_STATUS)) {
    const log = gl.getProgramInfoLog(prog);
    gl.deleteProgram(prog);
    throw new Error(`[DepthLoader] program link failed: ${log}`);
  }
  return prog;
}

export function DepthLoader({
  fadingOut,
  fadeDurationMs = 900,
  onFadeComplete,
  caption = "AWAITING THE FIELD",
}: DepthLoaderProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const rafRef = useRef<number>(0);
  const startTimeRef = useRef<number>(0);
  const [removed, setRemoved] = useState(false);

  // ─── WebGL2 setup + animation loop ─────────────────────────────────
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const gl = canvas.getContext("webgl2", { antialias: false, alpha: false });
    if (!gl) {
      // Hard fallback: skip shader, immediately call onFadeComplete so the
      // scene still loads. The bg stays a flat void color.
      console.warn("[DepthLoader] WebGL2 unavailable; skipping shader");
      const timer = window.setTimeout(onFadeComplete, fadeDurationMs);
      return () => window.clearTimeout(timer);
    }

    // Resize canvas to device pixels
    const resize = () => {
      const dpr = Math.min(window.devicePixelRatio || 1, 1.5);
      const w = Math.floor(window.innerWidth * dpr);
      const h = Math.floor(window.innerHeight * dpr);
      if (canvas.width !== w) canvas.width = w;
      if (canvas.height !== h) canvas.height = h;
      gl.viewport(0, 0, w, h);
    };
    resize();
    window.addEventListener("resize", resize);

    // Compile / link
    let prog: WebGLProgram;
    try {
      const v = compileShader(gl, gl.VERTEX_SHADER, VERT_SRC);
      const f = compileShader(gl, gl.FRAGMENT_SHADER, FRAG_SRC);
      prog = linkProgram(gl, v, f);
      gl.deleteShader(v);
      gl.deleteShader(f);
    } catch (e) {
      console.error(e);
      const timer = window.setTimeout(onFadeComplete, fadeDurationMs);
      return () => window.clearTimeout(timer);
    }

    gl.useProgram(prog);
    const uR = gl.getUniformLocation(prog, "r");
    const uT = gl.getUniformLocation(prog, "t");

    // Empty VAO — vertex positions come from gl_VertexID
    const vao = gl.createVertexArray();
    gl.bindVertexArray(vao);

    startTimeRef.current = performance.now();
    const draw = () => {
      gl.uniform2f(uR, canvas.width, canvas.height);
      gl.uniform1f(uT, (performance.now() - startTimeRef.current) / 1000);
      gl.clearColor(0.027, 0.043, 0.114, 1.0); // void
      gl.clear(gl.COLOR_BUFFER_BIT);
      gl.drawArrays(gl.TRIANGLES, 0, 3);
      rafRef.current = requestAnimationFrame(draw);
    };
    draw();

    return () => {
      cancelAnimationFrame(rafRef.current);
      window.removeEventListener("resize", resize);
      gl.deleteVertexArray(vao);
      gl.deleteProgram(prog);
    };
  }, [fadeDurationMs, onFadeComplete]);

  // ─── Fade-out handling ─────────────────────────────────────────────
  useEffect(() => {
    if (!fadingOut) return;
    const timer = window.setTimeout(() => {
      setRemoved(true);
      onFadeComplete();
    }, fadeDurationMs);
    return () => window.clearTimeout(timer);
  }, [fadingOut, fadeDurationMs, onFadeComplete]);

  if (removed) return null;

  return (
    <div
      style={{
        position: "fixed",
        inset: 0,
        zIndex: 100,
        pointerEvents: fadingOut ? "none" : "auto",
        opacity: fadingOut ? 0 : 1,
        transition: `opacity ${fadeDurationMs}ms cubic-bezier(0.4, 0.0, 0.2, 1)`,
        background: "#070B1D",
      }}
    >
      <canvas
        ref={canvasRef}
        style={{
          position: "absolute",
          inset: 0,
          width: "100%",
          height: "100%",
          display: "block",
        }}
      />
      {/* Caption + spinner-style pulse */}
      <div
        style={{
          position: "absolute",
          left: "50%",
          bottom: "12vh",
          transform: "translateX(-50%)",
          fontFamily: "var(--font-mono, 'SF Mono', monospace)",
          fontSize: "clamp(0.625rem, 0.8vw, 0.78rem)",
          letterSpacing: "0.45em",
          color: "var(--c-gold, #C5A017)",
          textTransform: "uppercase",
          opacity: 0.82,
          textAlign: "center",
          animation: "depthLoaderPulse 2.4s ease-in-out infinite",
        }}
      >
        {caption}
      </div>
      <style>{`
        @keyframes depthLoaderPulse {
          0%, 100% { opacity: 0.45; }
          50%      { opacity: 0.92; }
        }
      `}</style>
    </div>
  );
}
