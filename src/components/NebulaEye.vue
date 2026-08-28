<template>
  <canvas ref="pane" class="eye-layer" :class="{ 'eye-layer--live': ready }"></canvas>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';

const pane = ref<HTMLCanvasElement | null>(null);
const ready = ref(false);

const RENDER_SCALE = 0.75;
const DPR_CAP = 1.25;
const BOOT_DELAY = 50;

let gl: WebGLRenderingContext | null = null;
let sceneProg: WebGLProgram | null = null;
let postProg: WebGLProgram | null = null;
let film: WebGLTexture | null = null;
let fbo: WebGLFramebuffer | null = null;
let parallelExt: { COMPLETION_STATUS_KHR: number } | null = null;
let frame = 0;
let warm = 0;
let bootTimer: ReturnType<typeof setTimeout> | null = null;
let dead = false;
let born = 0;
let dozing = false;
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
const float RE = 3.2;
const float SQUISH = 0.902;

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
    s = s * 2.07 + vec2(19.7, 7.3);
    amp *= 0.5;
  }
  return acc;
}

mat2 whirl(float a) {
  float c = cos(a);
  float s = sin(a);
  return mat2(c, -s, s, c);
}

vec3 tilt(vec3 v) {
  v.yz = whirl(0.46) * v.yz;
  v.xy = whirl(-0.12) * v.xy;
  return v;
}

float slab(float r, float e0, float e1, float e2, float e3) {
  return smoothstep(e0, e1, r) * (1.0 - smoothstep(e2, e3, r));
}

float ringTau(float r, float sharp) {
  float grain = mix(0.5, noise(vec2(r * 34.0, 11.3)), sharp);
  float lanes = mix(0.5, noise(vec2(r * 12.0, 1.7)), sharp);
  float weave = mix(0.5, noise(vec2(r * 6.2, 5.9)), sharp * 0.7 + 0.3);
  float c = slab(r, 3.92, 4.08, 4.80, 4.88) * (0.16 + 0.14 * weave);
  float b = slab(r, 4.88, 4.97, 6.14, 6.22) * (1.05 + 0.90 * weave) * (0.68 + 0.50 * lanes);
  float hg = 1.0 - 0.50 * exp(-(r - 5.72) * (r - 5.72) * 220.0);
  float cas = slab(r, 6.22, 6.29, 6.43, 6.50) * (0.06 + 0.06 * weave);
  float a = slab(r, 6.48, 6.57, 7.18, 7.27) * (0.46 + 0.26 * lanes);
  float enc = 1.0 - 0.93 * exp(-(r - 7.09) * (r - 7.09) * 950.0);
  float kee = 1.0 - 0.70 * exp(-(r - 7.235) * (r - 7.235) * 3400.0);
  return (c + b * hg + cas + a * enc * kee) * (0.74 + 0.52 * grain);
}

float strand(float r, float ang) {
  float wave = 0.012 * sin(ang * 7.0 + time * 0.10);
  float off = r - 7.46 - wave;
  float core = exp(-off * off * 2400.0);
  float clump = 0.50 + 0.50 * noise(vec2(ang * 2.5 + time * 0.05, 3.7));
  return core * clump * 0.55;
}

vec3 ringHue(float r) {
  vec3 h = mix(vec3(0.46, 0.44, 0.42), vec3(0.94, 0.88, 0.75), smoothstep(4.55, 5.10, r));
  h = mix(h, vec3(0.82, 0.76, 0.65), smoothstep(6.30, 6.60, r));
  h = mix(h, vec3(0.88, 0.84, 0.76), smoothstep(7.30, 7.46, r));
  return h;
}

vec3 face(float lat, float lon) {
  float churn = fbm(vec2(lon * 2.2, lat * 6.5)) - 0.5;
  float bandY = lat + churn * 0.10;
  float belts = sin(bandY * 16.0 + 2.0 * fbm(vec2(bandY * 2.4, 3.1))) + 0.35 * sin(bandY * 33.0 + 1.7);
  float eq = exp(-bandY * bandY * 10.0);
  vec3 tone = mix(vec3(0.68, 0.51, 0.31), vec3(0.93, 0.81, 0.58), clamp(0.5 + 0.5 * belts, 0.0, 1.0));
  tone = mix(tone, vec3(0.98, 0.91, 0.70), eq * 0.7);
  tone = mix(tone, vec3(0.56, 0.61, 0.63), smoothstep(1.05, 1.35, abs(bandY)));
  float u = PI * 0.5 - lat;
  float hexEdge = 0.30 + 0.021 * cos(lon * 6.0 - time * 0.015);
  tone = mix(tone, vec3(0.43, 0.53, 0.56), smoothstep(hexEdge + 0.04, hexEdge - 0.04, u) * 0.55);
  float rough = fbm(vec2(lon * 6.0 + churn * 3.0, lat * 20.0));
  tone *= 0.90 + 0.20 * rough;
  float belted = smoothstep(0.12, 0.40, abs(bandY)) * (1.0 - smoothstep(0.75, 1.00, abs(bandY)));
  float oval = smoothstep(0.70, 0.82, fbm(vec2(lon * 3.5 + 9.0 - time * 0.012, lat * 11.0)));
  tone = mix(tone, vec3(0.99, 0.95, 0.83), oval * belted * 0.35);
  return tone;
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

void main() {
  vec2 sc = (gl_FragCoord.xy - res * vec2(0.72, 0.58)) / min(res.x, res.y);
  sc = whirl(-0.22) * sc;
  float aim = length(sc);

  float sway = clamp(drift.x, -1.0, 1.0);
  float nod = clamp(drift.y, -1.0, 1.0);
  float yaw = 0.14 * sin(time * 0.019) + sway * 0.07;
  float pitch = 0.20 + 0.04 * sin(time * 0.011 + 1.7) + nod * 0.05;

  vec3 ro = vec3(sin(yaw) * cos(pitch), sin(pitch), cos(yaw) * cos(pitch)) * 16.0;
  vec3 fwd = normalize(-ro);
  vec3 right = normalize(cross(vec3(0.0, 1.0, 0.0), fwd));
  vec3 up = cross(fwd, right);
  vec3 rd = normalize(fwd * 1.45 + right * sc.x + up * sc.y);

  vec3 roT = tilt(ro);
  vec3 rdT = tilt(rd);
  vec3 sunT = tilt(normalize(vec3(0.86 + 0.08 * sin(time * 0.005), 0.36, 0.10 + 0.10 * cos(time * 0.004))));

  vec3 squish = vec3(1.0, 1.0 / SQUISH, 1.0);
  vec3 roS = roT * squish;
  vec3 rdS = rdT * squish;
  float qa = dot(rdS, rdS);
  float qb = dot(roS, rdS);
  float dsc = qb * qb - qa * (dot(roS, roS) - RE * RE);
  float tp = -1.0;
  float aP = 0.0;
  if (dsc > 0.0) {
    tp = (-qb - sqrt(dsc)) / qa;
    aP = smoothstep(0.0, 0.30, dsc);
  }

  float tr = -1.0;
  if (abs(rdT.y) > 0.0001) {
    float tt = -roT.y / rdT.y;
    if (tt > 0.0) {
      float rrr = length((roT + rdT * tt).xz);
      if (rrr > 3.85 && rrr < 7.65) tr = tt;
    }
  }

  vec3 colP = vec3(0.0);
  if (tp > 0.0) {
    vec3 pp = roT + rdT * tp;
    vec3 n = normalize(vec3(pp.x, pp.y / (SQUISH * SQUISH), pp.z));
    float lat = asin(clamp(n.y, -1.0, 1.0));
    float spin = 0.024 + 0.012 * exp(-lat * lat * 9.0);
    float lon = atan(pp.z, pp.x) + time * spin;
    vec3 tone = face(lat, lon);

    float mu = dot(n, sunT);
    float diff = smoothstep(-0.05, 0.22, mu) * (0.35 + 0.65 * clamp(mu, 0.0, 1.0));
    float ndv = max(dot(n, -rdT), 0.0);
    float limb = 0.40 + 0.60 * pow(ndv, 0.5);

    float shad = 1.0;
    if (abs(sunT.y) > 0.001) {
      float ts = -pp.y / sunT.y;
      if (ts > 0.0) {
        vec2 sxz = (pp + sunT * ts).xz;
        float rs = length(sxz);
        if (rs > 3.85 && rs < 7.65) {
          float tsh = ringTau(rs, 0.6) + strand(rs, atan(sxz.y, sxz.x));
          shad = mix(0.22, 1.0, exp(-tsh * 1.6));
        }
      }
    }

    vec3 dusk = mix(vec3(1.0), vec3(1.06, 0.74, 0.52), smoothstep(0.30, 0.02, abs(mu)) * 0.45);
    colP = tone * dusk * (0.030 + 1.25 * diff * shad) * limb;
    colP += vec3(0.60, 0.72, 0.94) * pow(1.0 - ndv, 3.5) * (0.08 + 0.42 * diff);
    colP += tone * 0.045 * (1.0 - diff);
  }

  vec3 colR = vec3(0.0);
  float aR = 0.0;
  if (tr > 0.0) {
    vec3 rp = roT + rdT * tr;
    float rr = length(rp.xz);
    float ang = atan(rp.z, rp.x);
    float graze = clamp(abs(rdT.y) * 5.0, 0.30, 1.0);
    float sharp = graze * (1.0 - smoothstep(14.0, 26.0, tr));
    float tau = ringTau(rr, sharp) + strand(rr, ang);

    float sh = 1.0;
    float tb = -dot(rp, sunT);
    if (tb > 0.0) {
      vec3 sq = (rp + sunT * tb) * squish;
      sh = smoothstep(RE - 0.04, RE + 0.30, length(sq));
    }

    float mu0 = max(abs(sunT.y), 0.06);
    float muv = max(abs(rdT.y), 0.06);
    float refl = 1.0 - exp(-tau / mu0);
    float thru = tau * exp(-tau * 2.4) * 2.6;
    float sameSide = step(0.0, roT.y * sunT.y);
    float bright = mix(thru, refl, sameSide);
    float phase = 1.0 + 0.45 * pow(max(dot(-rdT, sunT), 0.0), 4.0);
    vec3 hue = ringHue(rr);
    hue = mix(hue * vec3(0.72, 0.64, 0.58), hue, sameSide);
    float albVar = 0.74 + 0.30 * noise(vec2(rr * 21.0, 4.4));
    albVar *= 0.86 + 0.18 * noise(vec2(rr * 47.0, 9.2));
    albVar = mix(0.88, albVar, sharp);
    colR = hue * bright * albVar * (0.05 + 0.95 * sh * phase);
    colR *= mix(1.0, 0.82, smoothstep(13.0, 24.0, tr));
    aR = clamp(1.0 - exp(-tau / muv), 0.0, 0.97);
  }

  vec3 col = vec3(0.0);
  float alpha = 0.0;
  bool ringFront = tr > 0.0 && (tp < 0.0 || tr < tp);
  if (ringFront) {
    col = colR * aR + colP * aP * (1.0 - aR);
    alpha = aR + aP * (1.0 - aR);
  } else {
    col = colP * aP + colR * aR * (1.0 - aP);
    alpha = aP + aR * (1.0 - aP);
  }

  float cc = dot(roT, rdT);
  vec3 nearP = (roT + rdT * max(-cc, 0.0)) * squish;
  float halo = exp(-abs(length(nearP) - RE) * 5.0) * 0.16;
  col += vec3(0.72, 0.79, 0.95) * halo;
  alpha = clamp(alpha + halo, 0.0, 1.0);

  float skyReach = 1.0 - smoothstep(1.1, 2.0, aim);
  vec2 sky = vec2(atan(rd.x, rd.z), rd.y);
  float glint = stars(sky, 42.0, 0.994) + stars(sky + 31.7, 120.0, 0.9965) * 0.6;
  vec3 dust = vec3(0.30, 0.24, 0.44) * fbm(sky * 2.2 + time * 0.006) * 0.22;
  vec3 back = (dust + vec3(0.90, 0.93, 1.0) * glint * 0.85) * skyReach * (1.0 - alpha);
  col += back;
  alpha = clamp(alpha + dot(back, vec3(0.299, 0.587, 0.114)) * 1.6, 0.0, 1.0);

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
    boost += max(texture2D(tex, vUv + dir * 2.5 / res).rgb - 0.35, 0.0);
    boost += max(texture2D(tex, vUv + dir * 7.5 / res).rgb - 0.35, 0.0);
  }
  boost = boost / 16.0 * 1.15;

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
  return prog;
}

function compiledYet(): boolean {
  if (!gl || !sceneProg || !postProg) return false;
  if (!parallelExt) return true;
  return (
    gl.getProgramParameter(sceneProg, parallelExt.COMPLETION_STATUS_KHR) &&
    gl.getProgramParameter(postProg, parallelExt.COMPLETION_STATUS_KHR)
  );
}

function finishWire(): boolean {
  if (!gl || !sceneProg || !postProg) return false;
  if (!gl.getProgramParameter(sceneProg, gl.LINK_STATUS)) {
    console.error(gl.getProgramInfoLog(sceneProg));
    return false;
  }
  if (!gl.getProgramParameter(postProg, gl.LINK_STATUS)) {
    console.error(gl.getProgramInfoLog(postProg));
    return false;
  }
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
  const cw = canvas.clientWidth || window.innerWidth;
  const ch = canvas.clientHeight || window.innerHeight;
  const w = Math.max(1, Math.round(cw * dpr * RENDER_SCALE));
  const h = Math.max(1, Math.round(ch * dpr * RENDER_SCALE));
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

  gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);
  gl.viewport(0, 0, canvas.width, canvas.height);
  gl.useProgram(sceneProg);
  gl.uniform2f(gl.getUniformLocation(sceneProg, 'res'), canvas.width, canvas.height);
  gl.uniform1f(gl.getUniformLocation(sceneProg, 'time'), t);
  gl.uniform2f(gl.getUniformLocation(sceneProg, 'drift'), driftX, driftY);
  gl.drawArrays(gl.TRIANGLES, 0, 3);

  gl.bindFramebuffer(gl.FRAMEBUFFER, null);
  gl.viewport(0, 0, canvas.width, canvas.height);
  gl.useProgram(postProg);
  gl.activeTexture(gl.TEXTURE0);
  gl.bindTexture(gl.TEXTURE_2D, film);
  gl.uniform1i(gl.getUniformLocation(postProg, 'tex'), 0);
  gl.uniform2f(gl.getUniformLocation(postProg, 'res'), canvas.width, canvas.height);
  gl.uniform1f(gl.getUniformLocation(postProg, 'time'), t);
  gl.drawArrays(gl.TRIANGLES, 0, 3);
}

function loop(stamp: number) {
  if (dozing || dead) return;
  const canvas = pane.value;
  if (canvas && canvas.clientWidth > 0) {
    if (!born) born = stamp;
    paint((stamp - born) / 1000);
  }
  frame = requestAnimationFrame(loop);
}

function chase(e: PointerEvent) {
  aimX = Math.max(-1, Math.min(1, (e.clientX / window.innerWidth - 0.5) * 2));
  aimY = Math.max(-1, Math.min(1, (e.clientY / window.innerHeight - 0.5) * 2));
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
  cancelAnimationFrame(warm);
}

function revive() {
  if (dead || !gl) return;
  sceneProg = weld(sceneSrc);
  postProg = weld(postSrc);
  if (!sceneProg || !postProg) return;
  born = 0;
  ready.value = false;
  warm = requestAnimationFrame(warmup);
}

function warmup() {
  if (dead || !gl) return;
  if (!compiledYet()) {
    warm = requestAnimationFrame(warmup);
    return;
  }
  if (!finishWire()) return;
  paint(0);
  ready.value = true;
  const still = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  if (still) {
    paint(12);
    return;
  }
  frame = requestAnimationFrame(loop);
}

function boot() {
  if (dead) return;
  const canvas = pane.value;
  if (!canvas) return;
  gl = canvas.getContext('webgl', {
    alpha: true,
    premultipliedAlpha: true,
    antialias: false,
    depth: false,
    stencil: false,
  });
  if (!gl) return;
  parallelExt = gl.getExtension('KHR_parallel_shader_compile');
  sceneProg = weld(sceneSrc);
  postProg = weld(postSrc);
  if (!sceneProg || !postProg) return;
  canvas.addEventListener('webglcontextlost', sink);
  canvas.addEventListener('webglcontextrestored', revive);
  document.addEventListener('visibilitychange', nap);
  window.addEventListener('pointermove', chase);
  warm = requestAnimationFrame(warmup);
}

onMounted(() => {
  bootTimer = setTimeout(boot, BOOT_DELAY);
});

onUnmounted(() => {
  dead = true;
  if (bootTimer) clearTimeout(bootTimer);
  cancelAnimationFrame(warm);
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
  gl = null;
  sceneProg = null;
  postProg = null;
  film = null;
  fbo = null;
});
</script>

<style scoped>
.eye-layer {
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

.eye-layer--live {
  opacity: 1;
}
</style>
