<template>
  <canvas ref="pane" class="star-layer" :class="{ 'star-layer--live': ready }"></canvas>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue';

const props = defineProps<{ active: boolean }>();

const pane = ref<HTMLCanvasElement | null>(null);
const ready = ref(false);

const RENDER_SCALE = 0.75;
const DPR_CAP = 1.25;
const MIN_FRAME_MS = 1000 / 61;

let gl: WebGLRenderingContext | null = null;
let sceneProg: WebGLProgram | null = null;
let postProg: WebGLProgram | null = null;
let film: WebGLTexture | null = null;
let fbo: WebGLFramebuffer | null = null;
let frame = 0;
let born = 0;
let lastPaint = 0;
let dozing = false;
type UniformSlots = {
  sceneRes: WebGLUniformLocation | null;
  sceneTime: WebGLUniformLocation | null;
  sceneDrift: WebGLUniformLocation | null;
  postTex: WebGLUniformLocation | null;
  postRes: WebGLUniformLocation | null;
  postTime: WebGLUniformLocation | null;
};
let uSlots: UniformSlots | null = null;
let aimX = 0;
let aimY = 0;
let driftX = 0;
let driftY = 0;

const vertSrc = `
attribute vec2 spot;
varying vec2 vUv;
void main() {
  vUv = spot * 0.5 + 0.5;
  gl_Position = vec4(spot, 0.0, 1.0);
}
`;

const sceneSrc = `
precision highp float;
uniform vec2 res;
uniform float time;
uniform vec2 drift;

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
  for (int i = 0; i < 5; i++) {
    acc += amp * noise(s);
    s = s * 2.07 + vec2(23.1, 9.7);
    amp *= 0.55;
  }
  return acc;
}

float ridged(vec2 s) {
  float acc = 0.0;
  float amp = 0.55;
  for (int i = 0; i < 4; i++) {
    acc += amp * (1.0 - abs(2.0 * noise(s) - 1.0));
    s = s * 2.13 + vec2(11.3, 17.9);
    amp *= 0.5;
  }
  return acc;
}

mat2 whirl(float a) {
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

vec2 zap(float f) {
  float taper = sin(f * PI);
  float slither = sin(f * 5.0 + time * 1.2) * 0.085
    + sin(f * 11.0 - time * 1.9) * 0.034
    + sin(f * 23.0 + time * 3.2) * 0.013
    + sin(f * 47.0 - time * 4.6) * 0.005;
  float sag = sin(f * 2.6 + time * 0.5) * 0.035;
  return vec2(-0.68 + f * 0.66, 0.42 - f * 0.33 + (slither + sag) * taper);
}

float segdist(vec2 p, vec2 a, vec2 b) {
  vec2 ab = b - a;
  float h = clamp(dot(p - a, ab) / dot(ab, ab), 0.0, 1.0);
  return length(p - a - ab * h);
}

void main() {
  vec2 sc = (gl_FragCoord.xy - res * 0.5) / min(res.x, res.y);
  sc += drift * 0.03;

  vec3 col = vec3(0.0);

  float murk = fbm(sc * 2.1 + vec2(4.7, 1.3));
  col += vec3(0.018, 0.021, 0.036) * murk;

  vec2 q = whirl(0.85) * (sc - vec2(0.16, -0.04));
  vec2 e = vec2(q.x / 0.30, q.y / 0.56);
  float ell = length(e);
  float rag = fbm(q * 2.4 + vec2(5.1, 2.7));
  float env = exp(-ell * ell * 1.5) * (0.45 + 1.1 * rag);
  float halo = exp(-ell * ell * 0.42) * 0.22;

  vec2 warp = vec2(
    fbm(q * 2.8 + vec2(0.0, time * 0.045)),
    fbm(q * 2.8 - vec2(8.2, 3.1) - vec2(time * 0.038, 0.0))
  );
  float bulk = fbm(q * 3.4 + (warp - 0.5) * 1.7 + vec2(0.0, time * 0.02));
  float wisp = ridged(q * 5.4 + (warp - 0.5) * 2.5);
  float shred = ridged(q * 9.5 + (warp - 0.5) * 3.5);
  float dust = fbm(q * 3.9 - (warp - 0.5) * 2.2 + vec2(19.0, 7.0));

  float body = env * (pow(bulk, 1.4) * 1.5 + pow(wisp, 3.0) * 2.1 + pow(shred, 4.0) * 1.1) + halo * bulk;
  float lanes = smoothstep(0.68, 0.30, dust);
  body *= 0.22 + 0.78 * lanes;

  vec2 hot = q - vec2(0.015, -0.14);
  float core = exp(-dot(hot, hot) * 18.0);
  float blaze = exp(-dot(hot, hot) * 70.0);

  float glowmix = clamp(body, 0.0, 1.0);
  vec3 plasma = mix(vec3(0.44, 0.52, 0.76), vec3(1.02, 1.02, 1.05), glowmix);
  col += plasma * pow(body, 1.25) * 1.55;
  col += vec3(0.94, 0.97, 1.08) * core * (bulk * 1.4 + 0.4) * (0.3 + 0.7 * lanes);
  col += vec3(1.0) * blaze * 2.3 * (0.5 + 0.5 * bulk);

  float dmin = 1e4;
  vec2 prev = zap(0.0);
  for (int i = 1; i <= 40; i++) {
    vec2 cur = zap(float(i) / 40.0);
    dmin = min(dmin, segdist(sc, prev, cur));
    prev = cur;
  }
  float charge = 0.78 + 0.22 * sin(time * 2.3) * sin(time * 0.7 + 1.2);
  float strand = exp(-pow(dmin * 190.0, 1.5));
  float aura = exp(-dmin * 26.0) * 0.36 + exp(-dmin * 7.0) * 0.05;
  col += (vec3(1.0, 1.0, 1.0) * strand * 2.1 + vec3(0.72, 0.70, 0.94) * aura) * charge;

  float open = 1.0 - clamp(body * 1.6 + core, 0.0, 0.92);
  float glint = stars(sc, 44.0, 0.992) + stars(sc + 13.7, 120.0, 0.996) * 0.6;
  col += vec3(0.9, 0.93, 1.0) * glint * 0.8 * open;

  col = 1.0 - exp(-col);
  float lum = dot(col, vec3(0.299, 0.587, 0.114));
  float alpha = clamp(lum * 2.2, 0.0, 1.0);
  gl_FragColor = vec4(col, alpha);
}
`;

const postSrc = `
precision highp float;
uniform sampler2D tex;
uniform vec2 res;
uniform float time;
varying vec2 vUv;

const float PI = 3.141592653589793;

float hash(vec2 s) {
  return fract(sin(dot(s, vec2(127.1, 311.7))) * 43758.5453123);
}

vec3 aces(vec3 x) {
  return clamp((x * (2.51 * x + 0.03)) / (x * (2.43 * x + 0.59) + 0.14), 0.0, 1.0);
}

void main() {
  vec4 base = texture2D(tex, vUv);
  vec3 boost = vec3(0.0);
  for (int i = 0; i < 8; i++) {
    float a = float(i) * PI * 0.25;
    vec2 dir = vec2(cos(a), sin(a));
    boost += max(texture2D(tex, vUv + dir * 2.5 / res).rgb - 0.4, 0.0);
    boost += max(texture2D(tex, vUv + dir * 7.5 / res).rgb - 0.4, 0.0);
  }
  boost = boost / 16.0 * 1.1;

  vec3 col = aces(base.rgb + boost);
  col += (hash(gl_FragCoord.xy + fract(time) * 61.7) - 0.5) * 0.012;
  float alpha = clamp(base.a + dot(boost, vec3(0.299, 0.587, 0.114)) * 1.4, 0.0, 1.0);
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

function weld(fragSrc: string): WebGLProgram | null {
  if (!gl) return null;
  const vs = crank(gl.VERTEX_SHADER, vertSrc);
  const fs = crank(gl.FRAGMENT_SHADER, fragSrc);
  if (!vs || !fs) return null;
  const prog = gl.createProgram();
  if (!prog) return null;
  gl.attachShader(prog, vs);
  gl.attachShader(prog, fs);
  gl.linkProgram(prog);
  if (!gl.getProgramParameter(prog, gl.LINK_STATUS)) {
    console.error(gl.getProgramInfoLog(prog));
    return null;
  }
  return prog;
}

function wire(): boolean {
  if (!gl) return false;
  sceneProg = weld(sceneSrc);
  postProg = weld(postSrc);
  uSlots = null;
  if (!sceneProg || !postProg) return false;
  const mesh = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, mesh);
  gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1, -1, 3, -1, -1, 3]), gl.STATIC_DRAW);
  for (const prog of [sceneProg, postProg]) {
    const slot = gl.getAttribLocation(prog, 'spot');
    gl.enableVertexAttribArray(slot);
    gl.vertexAttribPointer(slot, 2, gl.FLOAT, false, 0, 0);
  }
  film = gl.createTexture();
  fbo = gl.createFramebuffer();
  return true;
}

function fit() {
  const canvas = pane.value;
  if (!canvas || !gl || !film || !fbo) return;
  const dpr = Math.min(window.devicePixelRatio || 1, DPR_CAP);
  const w = Math.max(1, Math.round(canvas.clientWidth * dpr * RENDER_SCALE));
  const h = Math.max(1, Math.round(canvas.clientHeight * dpr * RENDER_SCALE));
  if (canvas.width === w && canvas.height === h) return;
  canvas.width = w;
  canvas.height = h;
  gl.bindTexture(gl.TEXTURE_2D, film);
  gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, w, h, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
  gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);
  gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, film, 0);
  gl.bindFramebuffer(gl.FRAMEBUFFER, null);
}

function paint(t: number) {
  const canvas = pane.value;
  if (!canvas || !gl || !sceneProg || !postProg) return;
  fit();
  driftX += (aimX - driftX) * 0.035;
  driftY += (aimY - driftY) * 0.035;

  if (!uSlots) {
    uSlots = {
      sceneRes: gl.getUniformLocation(sceneProg, 'res'),
      sceneTime: gl.getUniformLocation(sceneProg, 'time'),
      sceneDrift: gl.getUniformLocation(sceneProg, 'drift'),
      postTex: gl.getUniformLocation(postProg, 'tex'),
      postRes: gl.getUniformLocation(postProg, 'res'),
      postTime: gl.getUniformLocation(postProg, 'time'),
    };
  }

  gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);
  gl.viewport(0, 0, canvas.width, canvas.height);
  gl.useProgram(sceneProg);
  gl.uniform2f(uSlots.sceneRes, canvas.width, canvas.height);
  gl.uniform1f(uSlots.sceneTime, t);
  gl.uniform2f(uSlots.sceneDrift, driftX, driftY);
  gl.drawArrays(gl.TRIANGLES, 0, 3);

  gl.bindFramebuffer(gl.FRAMEBUFFER, null);
  gl.viewport(0, 0, canvas.width, canvas.height);
  gl.useProgram(postProg);
  gl.activeTexture(gl.TEXTURE0);
  gl.bindTexture(gl.TEXTURE_2D, film);
  gl.uniform1i(uSlots.postTex, 0);
  gl.uniform2f(uSlots.postRes, canvas.width, canvas.height);
  gl.uniform1f(uSlots.postTime, t);
  gl.drawArrays(gl.TRIANGLES, 0, 3);
}

function loop(stamp: number) {
  if (dozing) return;
  frame = requestAnimationFrame(loop);
  if (stamp - lastPaint < MIN_FRAME_MS) return;
  lastPaint = stamp;
  const canvas = pane.value;
  if (canvas && canvas.clientWidth > 0) {
    if (!born) born = stamp;
    paint((stamp - born) / 1000);
  }
}

function chase(e: PointerEvent) {
  if (dozing) return;
  aimX = Math.max(-1, Math.min(1, (e.clientX / window.innerWidth - 0.5) * 2));
  aimY = Math.max(-1, Math.min(1, (e.clientY / window.innerHeight - 0.5) * 2));
}

function nap() {
  if (document.hidden || !props.active) {
    dozing = true;
    cancelAnimationFrame(frame);
  } else if (dozing) {
    dozing = false;
    born = 0;
    frame = requestAnimationFrame(loop);
  }
}

watch(() => props.active, nap);

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
    paint(12);
    return;
  }
  canvas.addEventListener('webglcontextlost', sink);
  canvas.addEventListener('webglcontextrestored', revive);
  document.addEventListener('visibilitychange', nap);
  window.addEventListener('pointermove', chase, { passive: true });
  frame = requestAnimationFrame(loop);
}

onMounted(boot);

onUnmounted(() => {
  cancelAnimationFrame(frame);
  document.removeEventListener('visibilitychange', nap);
  window.removeEventListener('pointermove', chase);
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
.star-layer {
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

.star-layer--live {
  opacity: 1;
}
</style>
