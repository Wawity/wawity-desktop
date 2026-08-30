<template>
  <canvas ref="pane" class="earth-layer" :class="{ 'earth-layer--live': ready }"></canvas>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue';

const props = defineProps<{ active: boolean }>();

const pane = ref<HTMLCanvasElement | null>(null);
const ready = ref(false);

const RENDER_SCALE = 0.66;
const DPR_CAP = 1.25;
const MIN_FRAME_MS = 1000 / 61;
const MAX_STEP_MS = 100;

let gl: WebGLRenderingContext | null = null;
let prog: WebGLProgram | null = null;
let frame = 0;
let last = 0;
let clock = 0;
let dozing = false;
let stale = true;
let boxW = 0;
let boxH = 0;
let sizer: ResizeObserver | null = null;
let mesh: WebGLBuffer | null = null;
const skins: WebGLTexture[] = [];
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
uniform sampler2D dayMap;
uniform sampler2D nightMap;

float hash(vec2 s) {
  return fract(sin(dot(s, vec2(127.1, 311.7))) * 43758.5453123);
}

float hash3(vec3 s) {
  return fract(sin(dot(s, vec3(127.1, 311.7, 74.7))) * 43758.5453123);
}

float noise3(vec3 s) {
  vec3 i = floor(s);
  vec3 f = fract(s);
  vec3 u = f * f * (3.0 - 2.0 * f);
  float a = mix(hash3(i), hash3(i + vec3(1.0, 0.0, 0.0)), u.x);
  float b = mix(hash3(i + vec3(0.0, 1.0, 0.0)), hash3(i + vec3(1.0, 1.0, 0.0)), u.x);
  float c = mix(hash3(i + vec3(0.0, 0.0, 1.0)), hash3(i + vec3(1.0, 0.0, 1.0)), u.x);
  float d = mix(hash3(i + vec3(0.0, 1.0, 1.0)), hash3(i + vec3(1.0, 1.0, 1.0)), u.x);
  return mix(mix(a, b, u.y), mix(c, d, u.y), u.z);
}

float fbm3(vec3 s) {
  float acc = 0.0;
  float amp = 0.5;
  for (int i = 0; i < 4; i++) {
    acc += amp * noise3(s);
    s = s * 2.07 + vec3(13.7, 5.3, 9.1);
    amp *= 0.5;
  }
  return acc;
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
  vec2 p = (uv - vec2(0.66, -0.05)) / 0.36;
  float d2 = dot(p, p);

  vec3 sun = normalize(vec3(-0.55, 0.18, 0.72));

  vec3 col = vec3(0.0);
  float alpha = 0.0;

  float glint = stars(uv, 46.0, 0.9935) + stars(uv + 31.7, 130.0, 0.996) * 0.7;
  col += vec3(0.9, 0.93, 1.0) * glint * 0.7;
  col += (vec3(0.30, 0.24, 0.50) * fbm3(vec3(uv * 2.4, 3.1))
    + vec3(0.13, 0.20, 0.40) * fbm3(vec3(uv * 1.2, 9.4))) * 0.26;

  if (d2 < 1.0) {
    float z = sqrt(1.0 - d2);
    vec3 n = vec3(p.x, p.y, z);

    float tilt = 0.41;
    vec3 nt = vec3(
      n.x * cos(tilt) - n.y * sin(tilt),
      n.x * sin(tilt) + n.y * cos(tilt),
      n.z
    );

    float spin = time * 0.05;
    vec3 s = vec3(
      nt.x * cos(spin) + nt.z * sin(spin),
      nt.y,
      -nt.x * sin(spin) + nt.z * cos(spin)
    );

    float lon = atan(s.z, s.x);
    float lat = asin(clamp(s.y, -1.0, 1.0));
    vec2 tuv = vec2(fract(0.5 + lon / 6.28318530718), 0.5 + lat / 3.14159265359);

    vec3 base = pow(texture2D(dayMap, tuv).rgb, vec3(2.2));
    vec3 nglow = pow(texture2D(nightMap, tuv).rgb, vec3(2.2));

    float ndl = dot(n, sun);
    float day = smoothstep(-0.26, 0.18, ndl);
    float sea = smoothstep(0.01, 0.1, base.b - base.r);
    float spec = pow(max(dot(n, normalize(sun + vec3(0.0, 0.0, 1.0))), 0.0), 90.0) * sea * day;

    float cloudSpin = time * 0.062;
    vec3 cs = vec3(
      nt.x * cos(cloudSpin) + nt.z * sin(cloudSpin),
      nt.y,
      -nt.x * sin(cloudSpin) + nt.z * cos(cloudSpin)
    );
    float veil = smoothstep(0.60, 0.82, fbm3(cs * 3.4 + vec3(27.0, time * 0.005, 27.0))) * 0.5;

    vec3 lit = base * (day * (1.1 + 0.45 * max(ndl, 0.0)) + 0.025 * day);
    lit += vec3(1.0, 0.9, 0.7) * spec * 0.5;
    lit = mix(lit, mix(vec3(0.08, 0.10, 0.16), vec3(1.0, 1.0, 1.02), day), veil);
    lit += nglow * vec3(1.0, 0.82, 0.55) * (1.0 - day) * (1.0 - veil * 0.7) * 5.5;

    float rim = pow(1.0 - z, 2.2);
    float dusk = smoothstep(-0.12, 0.25, ndl);
    lit += vec3(0.25, 0.50, 1.0) * rim * (0.25 + dusk * 0.9);
    float termGlow = exp(-pow(ndl, 2.0) * 18.0) * rim;
    lit += vec3(1.0, 0.45, 0.15) * termGlow * 0.8;

    col = lit;
    alpha = 1.0;
  } else {
    float halo = exp(-(sqrt(d2) - 1.0) * 9.0);
    float side = clamp(dot(normalize(vec3(p, 0.35)), sun) * 0.5 + 0.5, 0.0, 1.0);
    col += vec3(0.30, 0.55, 1.0) * halo * (0.12 + 0.5 * side);
    alpha = halo * (0.25 + 0.5 * side);
  }

  col = 1.0 - exp(-col * 1.5);
  col = aces(col);
  col += (hash(gl_FragCoord.xy + fract(time) * 61.7) - 0.5) * 0.012;
  float lum = dot(col, vec3(0.299, 0.587, 0.114));
  alpha = clamp(max(alpha, lum * 1.6), 0.0, 1.0);
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
  if (!prog) { gl.deleteShader(vs); gl.deleteShader(fs); return false; }
  gl.attachShader(prog, vs);
  gl.attachShader(prog, fs);
  gl.linkProgram(prog);
  gl.deleteShader(vs);
  gl.deleteShader(fs);
  if (!gl.getProgramParameter(prog, gl.LINK_STATUS)) {
    console.error(gl.getProgramInfoLog(prog));
    return false;
  }
  if (!mesh) {
    mesh = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, mesh);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1, -1, 3, -1, -1, 3]), gl.STATIC_DRAW);
  } else {
    gl.bindBuffer(gl.ARRAY_BUFFER, mesh);
  }
  const slot = gl.getAttribLocation(prog, 'spot');
  gl.enableVertexAttribArray(slot);
  gl.vertexAttribPointer(slot, 2, gl.FLOAT, false, 0, 0);
  uRes = gl.getUniformLocation(prog, 'res');
  uTime = gl.getUniformLocation(prog, 'time');
  gl.useProgram(prog);
  gl.uniform1i(gl.getUniformLocation(prog, 'dayMap'), 0);
  gl.uniform1i(gl.getUniformLocation(prog, 'nightMap'), 1);
  stale = true;
  if (skins.length === 0) {
    skin(0, '/earth/day.jpg');
    skin(1, '/earth/night.jpg');
  } else {
    for (let i = 0; i < skins.length; i++) {
      gl.activeTexture(gl.TEXTURE0 + i);
      gl.bindTexture(gl.TEXTURE_2D, skins[i]);
    }
  }
  return true;
}

function skin(unit: number, url: string) {
  if (!gl) return;
  const tex = gl.createTexture();
  if (!tex) return;
  skins[unit] = tex;
  gl.activeTexture(gl.TEXTURE0 + unit);
  gl.bindTexture(gl.TEXTURE_2D, tex);
  gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGB, 1, 1, 0, gl.RGB, gl.UNSIGNED_BYTE, new Uint8Array([5, 12, 30]));
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
  const push = (pic: TexImageSource) => {
    if (!gl) return;
    gl.activeTexture(gl.TEXTURE0 + unit);
    gl.bindTexture(gl.TEXTURE_2D, tex);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGB, gl.RGB, gl.UNSIGNED_BYTE, pic);
  };
  const fallback = () => {
    const img = new Image();
    img.onload = () => {
      if (!gl) return;
      gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, 1);
      push(img);
      gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, 0);
    };
    img.onerror = () => console.error('missing:', url);
    img.src = url;
  };
  fetch(url)
    .then((r) => {
      if (!r.ok) throw new Error(url);
      return r.blob();
    })
    .then((blob) => createImageBitmap(blob, {
      imageOrientation: 'flipY',
      resizeWidth: 2048,
      resizeHeight: 1024,
      resizeQuality: 'high',
    }))
    .then((pic) => {
      push(pic);
      pic.close();
    })
    .catch(fallback);
}

function measure() {
  const canvas = pane.value;
  if (!canvas) return;
  const w = canvas.clientWidth;
  const h = canvas.clientHeight;
  if (w === boxW && h === boxH) return;
  boxW = w;
  boxH = h;
  stale = true;
}

function fit() {
  const canvas = pane.value;
  if (!canvas || !gl) return;
  const dpr = Math.min(window.devicePixelRatio || 1, DPR_CAP);
  const w = Math.max(1, Math.round(boxW * dpr * RENDER_SCALE));
  const h = Math.max(1, Math.round(boxH * dpr * RENDER_SCALE));
  if (canvas.width !== w || canvas.height !== h) {
    canvas.width = w;
    canvas.height = h;
  }
  gl.viewport(0, 0, canvas.width, canvas.height);
  gl.uniform2f(uRes, canvas.width, canvas.height);
  stale = false;
}

function paint(t: number) {
  const canvas = pane.value;
  if (!canvas || !gl || !prog) return;
  gl.useProgram(prog);
  if (stale) fit();
  gl.uniform1f(uTime, t);
  gl.drawArrays(gl.TRIANGLES, 0, 3);
}

function loop(stamp: number) {
  if (dozing) return;
  frame = requestAnimationFrame(loop);
  if (!last) last = stamp;
  const step = stamp - last;
  if (step < MIN_FRAME_MS) return;
  last = stamp;
  clock += Math.min(step, MAX_STEP_MS) / 1000;
  if (boxW > 0 && boxH > 0) paint(clock);
}

function nap() {
  if (document.hidden || !props.active) {
    dozing = true;
    cancelAnimationFrame(frame);
  } else if (dozing) {
    dozing = false;
    last = 0;
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
    last = 0;
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
    powerPreference: 'low-power',
  });
  if (!gl || !wire()) return;
  measure();
  paint(0);
  ready.value = true;
  const still = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  if (still) {
    paint(30);
    return;
  }
  canvas.addEventListener('webglcontextlost', sink);
  canvas.addEventListener('webglcontextrestored', revive);
  document.addEventListener('visibilitychange', nap);
  sizer = new ResizeObserver(measure);
  sizer.observe(canvas);
  if (props.active && !document.hidden) {
    frame = requestAnimationFrame(loop);
  } else {
    dozing = true;
  }
}

onMounted(boot);

onUnmounted(() => {
  dozing = true;
  cancelAnimationFrame(frame);
  document.removeEventListener('visibilitychange', nap);
  if (sizer) {
    sizer.disconnect();
    sizer = null;
  }
  const canvas = pane.value;
  if (canvas) {
    canvas.removeEventListener('webglcontextlost', sink);
    canvas.removeEventListener('webglcontextrestored', revive);
  }
  if (gl) {
    for (const tex of skins) if (tex) gl.deleteTexture(tex);
    if (mesh) gl.deleteBuffer(mesh);
    if (prog) gl.deleteProgram(prog);
    const plug = gl.getExtension('WEBGL_lose_context');
    if (plug) plug.loseContext();
  }
  skins.length = 0;
  mesh = null;
  prog = null;
  gl = null;
});
</script>

<style scoped>
.earth-layer {
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

.earth-layer--live {
  opacity: 1;
}
</style>
