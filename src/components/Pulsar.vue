<template>
  <canvas ref="pane" class="pulsar-layer" :class="{ 'pulsar-layer--live': ready }"></canvas>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';

const pane = ref<HTMLCanvasElement | null>(null);
const ready = ref(false);

const RENDER_SCALE = 0.66;
const DPR_CAP = 1.25;

let gl: WebGLRenderingContext | null = null;
let prog: WebGLProgram | null = null;
let frame = 0;
let born = 0;
let dozing = false;
let uRes: WebGLUniformLocation | null = null;
let uTime: WebGLUniformLocation | null = null;

const vertSrc = `
attribute vec2 spot;
void main() {
  gl_Position = vec4(spot, 0.0, 1.0);
}
`;

const sceneSrc = `
precision highp float;
uniform vec2 res;
uniform float time;

const float PI = 3.141592653589793;

float hash(vec2 s) {
  return fract(sin(dot(s, vec2(127.1, 311.7))) * 43758.5453123);
}

float noise(vec2 s) {
  vec2 i = floor(s);
  vec2 f = fract(s);
  vec2 u = f * f * (3.0 - 2.0 * f);
  float a = hash(i);
  float b = hash(i + vec2(1.0, 0.0));
  float c = hash(i + vec2(0.0, 1.0));
  float d = hash(i + vec2(1.0, 1.0));
  return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

float fbm(vec2 s) {
  float acc = 0.0;
  float amp = 0.5;
  for (int i = 0; i < 4; i++) {
    acc += amp * noise(s);
    s = s * 2.03 + vec2(19.7, 7.3);
    amp *= 0.5;
  }
  return acc;
}

float adiff(float a, float b) {
  return atan(sin(a - b), cos(a - b));
}

mat2 spinner(float a) {
  float c = cos(a);
  float s = sin(a);
  return mat2(c, -s, s, c);
}

float stars(vec2 q, float cells, float cut) {
  vec2 cell = floor(q * cells);
  float seed = hash(cell);
  if (seed < cut) {
    return 0.0;
  }
  vec2 pos = fract(q * cells) - vec2(hash(cell + 7.3), hash(cell + 3.1));
  float tw = 0.72 + 0.28 * sin(time * (1.2 + seed * 3.5) + seed * 41.0);
  return exp(-dot(pos, pos) * 240.0) * tw;
}

vec3 aces(vec3 x) {
  return clamp((x * (2.51 * x + 0.03)) / (x * (2.43 * x + 0.59) + 0.14), 0.0, 1.0);
}

void main() {
  vec2 uv = (gl_FragCoord.xy - res * 0.5) / min(res.x, res.y);
  uv = spinner(-0.14) * uv;
  vec2 p = uv - vec2(0.58, 0.24);
  float r = length(p);
  float ang = atan(p.y, p.x);

  float spin = time * 2.1;
  float axis = spin + 0.42 * sin(time * 0.19);
  float bend = ang + r * 1.7;

  float lobeA = max(cos(adiff(bend, axis)), 0.0) + 1e-4;
  float lobeB = max(cos(adiff(bend, axis + PI)), 0.0) + 1e-4;
  float sharp = 30.0 / (1.0 + r * 3.2);
  float beams = pow(lobeA, sharp) + pow(lobeB, sharp);
  float streaks = 0.55 + 0.45 * noise(vec2(r * 10.0 - time * 2.6, bend * 3.0));
  float beamGlow = beams * exp(-r * 2.0) * streaks;
  float halo = (pow(lobeA, sharp * 0.22) + pow(lobeB, sharp * 0.22)) * exp(-r * 1.5) * 0.32;

  float flash = exp(-pow(adiff(axis, 0.85), 2.0) * 22.0);

  float core = 0.0085 / (r * r + 0.0006);
  core += exp(-r * 34.0) * 2.0;
  float shell = exp(-r * 6.0) * (0.45 + 0.55 * fbm(p * 7.0 + time * 0.22));

  vec2 q = spinner(-spin * 0.12) * p;
  float rings = 0.0;
  for (int i = 1; i <= 4; i++) {
    float fi = float(i);
    float d = abs(length(q * vec2(1.0, 2.4)) - fi * 0.082);
    rings += exp(-d * d * 5200.0) * (0.5 / fi);
  }
  rings *= exp(-r * 2.1) * (0.55 + 0.45 * noise(vec2(spin * 0.3, r * 22.0)));

  float windA = fbm(p * 2.7 - time * 0.012) * exp(-r * 1.4);
  float windB = fbm(p * 1.4 + time * 0.008) * exp(-r * 0.9);

  vec3 col = vec3(0.0);
  col += vec3(0.84, 0.91, 1.0) * core * (1.0 + 0.6 * flash);
  col += vec3(0.60, 0.78, 1.0) * beamGlow * (1.1 + 1.0 * flash);
  col += vec3(0.52, 0.60, 1.0) * halo;
  col += vec3(0.45, 0.85, 0.95) * rings * 0.5;
  col += vec3(0.34, 0.29, 0.60) * windA * 0.36;
  col += vec3(0.15, 0.23, 0.44) * windB * 0.30;
  col += vec3(0.80, 0.86, 1.0) * shell * 0.5;

  float sky = stars(uv, 46.0, 0.993) + stars(uv + 31.7, 130.0, 0.996) * 0.7;
  col += vec3(0.90, 0.93, 1.0) * sky * 0.75;
  col += vec3(0.50, 0.62, 1.0) * flash * 0.045;

  col *= 1.0 + 0.05 * (noise(vec2(time * 9.0, 3.7)) - 0.5);

  col = 1.0 - exp(-col * 1.35);
  col = aces(col);
  col += (hash(gl_FragCoord.xy + fract(time) * 61.7) - 0.5) * 0.012;

  float lum = dot(col, vec3(0.299, 0.587, 0.114));
  float alpha = clamp(lum * 1.7, 0.0, 1.0);
  gl_FragColor = vec4(col * alpha, alpha);
}
`;

function crank(kind: number, src: string): WebGLShader | null {
  if (!gl) return null;
  const sh = gl.createShader(kind);
  if (!sh) return null;
  gl.shaderSource(sh, src);
  gl.compileShader(sh);
  if (!gl.getShaderParameter(sh, gl.COMPILE_STATUS)) {
    console.error(gl.getShaderInfoLog(sh));
    gl.deleteShader(sh);
    return null;
  }
  return sh;
}

function wire(): boolean {
  if (!gl) return false;
  const vs = crank(gl.VERTEX_SHADER, vertSrc);
  const fs = crank(gl.FRAGMENT_SHADER, sceneSrc);
  if (!vs || !fs) return false;
  prog = gl.createProgram();
  if (!prog) return false;
  gl.attachShader(prog, vs);
  gl.attachShader(prog, fs);
  gl.linkProgram(prog);
  if (!gl.getProgramParameter(prog, gl.LINK_STATUS)) {
    console.error(gl.getProgramInfoLog(prog));
    return false;
  }
  const mesh = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, mesh);
  gl.bufferData(
    gl.ARRAY_BUFFER,
    new Float32Array([-1, -1, 3, -1, -1, 3]),
    gl.STATIC_DRAW,
  );
  const slot = gl.getAttribLocation(prog, 'spot');
  gl.enableVertexAttribArray(slot);
  gl.vertexAttribPointer(slot, 2, gl.FLOAT, false, 0, 0);
  uRes = gl.getUniformLocation(prog, 'res');
  uTime = gl.getUniformLocation(prog, 'time');
  return true;
}

function fit() {
  const canvas = pane.value;
  if (!canvas || !gl) return;
  const dpr = Math.min(window.devicePixelRatio || 1, DPR_CAP);
  const w = Math.max(1, Math.round(canvas.clientWidth * dpr * RENDER_SCALE));
  const h = Math.max(1, Math.round(canvas.clientHeight * dpr * RENDER_SCALE));
  if (canvas.width !== w || canvas.height !== h) {
    canvas.width = w;
    canvas.height = h;
  }
}

function paint(t: number) {
  const canvas = pane.value;
  if (!canvas || !gl || !prog) return;
  fit();
  gl.viewport(0, 0, canvas.width, canvas.height);
  gl.useProgram(prog);
  gl.uniform2f(uRes, canvas.width, canvas.height);
  gl.uniform1f(uTime, t);
  gl.drawArrays(gl.TRIANGLES, 0, 3);
}

function loop(stamp: number) {
  if (dozing) return;
  const canvas = pane.value;
  if (canvas && canvas.clientWidth > 0) {
    if (!born) born = stamp;
    paint((stamp - born) / 1000);
  }
  frame = requestAnimationFrame(loop);
}

function nap() {
  if (document.hidden) {
    dozing = true;
    cancelAnimationFrame(frame);
  } else if (dozing) {
    dozing = false;
    frame = requestAnimationFrame(loop);
  }
}

function sink(e: Event) {
  e.preventDefault();
  cancelAnimationFrame(frame);
}

function revive() {
  if (wire()) {
    born = 0;
    frame = requestAnimationFrame(loop);
  }
}

function refit() {
  fit();
}

function boot() {
  const canvas = pane.value;
  if (!canvas) return;
  gl = canvas.getContext('webgl', {
    alpha: true,
    premultipliedAlpha: true,
    antialias: false,
    depth: false,
    stencil: false,
  });
  if (!gl || !wire()) return;
  paint(0);
  ready.value = true;
  const still = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  if (still) {
    paint(6);
    return;
  }
  canvas.addEventListener('webglcontextlost', sink);
  canvas.addEventListener('webglcontextrestored', revive);
  document.addEventListener('visibilitychange', nap);
  window.addEventListener('resize', refit);
  frame = requestAnimationFrame(loop);
}

onMounted(boot);

onUnmounted(() => {
  cancelAnimationFrame(frame);
  document.removeEventListener('visibilitychange', nap);
  window.removeEventListener('resize', refit);
  const canvas = pane.value;
  if (canvas) {
    canvas.removeEventListener('webglcontextlost', sink);
    canvas.removeEventListener('webglcontextrestored', revive);
  }
  if (gl) {
    const plug = gl.getExtension('WEBGL_lose_context');
    if (plug) plug.loseContext();
  }
});
</script>

<style scoped>
.pulsar-layer {
  position: fixed;
  inset: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
  z-index: 0;
  background: transparent;
  opacity: 0;
  transition: opacity 700ms ease;
}

.pulsar-layer--live {
  opacity: 1;
}
</style>