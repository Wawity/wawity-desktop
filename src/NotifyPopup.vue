<template>
  <div class="notify-root">
    <TransitionGroup name="ncard" tag="div" class="notify-stack">
      <div v-for="item in items" :key="item.id" class="ncard" :class="`ncard--${item.variant}`">
        <div class="ncard-hole">
          <img v-if="holeImg" class="hole-img" :src="holeImg" alt="" />
          <template v-else>
            <div class="hole-ring"></div>
            <div class="hole-disc"></div>
          </template>
        </div>
        <img class="ncard-logo" src="/rpc.jpg" alt="" />
        <div class="ncard-text">
            <div class="ncard-title" v-text="item.title"></div>
            <div v-if="item.body" class="ncard-body" v-text="item.body"></div>
        </div>
        <div class="ncard-bar" :style="{ animationDuration: item.duration + 'ms' }"></div>
      </div>
    </TransitionGroup>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/tauri';

interface NotifyItem {
  id: number;
  title: string;
  body?: string;
  variant: string;
  duration: number;
}

const items = ref<NotifyItem[]>([]);
const holeImg = ref('');
let counter = 0;

const holeVert = `
attribute vec2 spot;
void main() {
  gl_Position = vec4(spot, 0.0, 1.0);
}
`;

const holeFrag = `
precision highp float;
uniform vec2 res;
uniform float time;

const float DIN = 2.6;
const float DOUT = 9.5;

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

mat2 whirl(float a) {
  float c = cos(a);
  float s = sin(a);
  return mat2(c, -s, s, c);
}

vec3 bbody(float k) {
  vec3 col = mix(vec3(0.45, 0.08, 0.02), vec3(1.0, 0.42, 0.12), smoothstep(0.0, 0.4, k));
  col = mix(col, vec3(1.0, 0.85, 0.62), smoothstep(0.4, 0.75, k));
  col = mix(col, vec3(0.85, 0.90, 1.0), smoothstep(0.75, 1.05, k));
  return col;
}

float diskGlow(vec3 hit, float rr) {
  float ang = atan(hit.z, hit.x);
  float omega = 6.0 / pow(rr, 1.5);
  float shear = ang + (fract(time / 36.0) - 0.5) * 36.0 * omega;
  float lr = log(rr);
  float lanes = noise(vec2(shear * 4.0, lr * 5.0)) * 0.65
    + noise(vec2(shear * 8.0 + 47.3, lr * 11.0 + 9.1)) * 0.35;
  float calm = smoothstep(DOUT * 0.9, DOUT * 0.4, rr);
  lanes = mix(0.5, lanes, 0.3 + 0.7 * calm);
  float fadeIn = smoothstep(DIN, DIN * 1.35, rr);
  float fadeOut = 1.0 - smoothstep(DOUT * 0.45, DOUT * 0.95, rr);
  return (0.35 + 0.9 * lanes) * fadeIn * fadeOut;
}

float stars(vec2 q, float cells, float cut) {
  vec2 cell = floor(q * cells);
  float seed = hash(cell);
  if (seed < cut) {
    return 0.0;
  }
  vec2 pos = fract(q * cells) - vec2(hash(cell + 7.3), hash(cell + 3.1));
  return exp(-dot(pos, pos) * 240.0);
}

void main() {
  vec2 sc = (gl_FragCoord.xy - res * 0.5) / min(res.x, res.y);
  sc = whirl(-0.22) * sc;
  float aim = length(sc);

  float yaw = 0.18 * sin(time * 0.021);
  float pitch = 0.30 + 0.05 * sin(time * 0.013 + 1.7);

  vec3 ro = vec3(sin(yaw) * cos(pitch), sin(pitch), cos(yaw) * cos(pitch)) * 24.0;
  vec3 fwd = normalize(-ro);
  vec3 right = normalize(cross(vec3(0.0, 1.0, 0.0), fwd));
  vec3 up = cross(fwd, right);
  vec3 rd = normalize(fwd * 1.5 + right * sc.x + up * sc.y);

  vec3 p = ro;
  vec3 v = rd;
  vec3 h = cross(p, v);
  float h2 = dot(h, h);

  vec3 acc = vec3(0.0);
  float trans = 1.0;
  float captured = 0.0;
  int hits = 0;

  for (int i = 0; i < 100; i++) {
    float r2 = dot(p, p);
    if (r2 < 1.0) {
      captured = 1.0;
      break;
    }
    if (r2 > 1600.0) {
      break;
    }
    float r = sqrt(r2);
    float dt = clamp(0.045 * r, 0.03, 0.6);
    dt = min(dt, abs(p.y) * 0.9 + 0.05);

    vec3 pull = -1.5 * h2 * p / (r2 * r2 * r);
    vec3 pPrev = p;
    v += pull * dt;
    p += v * dt;

    if (pPrev.y * p.y < 0.0 && hits < 3) {
      float f = pPrev.y / (pPrev.y - p.y);
      vec3 hit = mix(pPrev, p, f);
      float rr = length(hit.xz);
      if (rr > DIN && rr < DOUT) {
        bool secondary = hits > 0;
        if (!secondary || rr < DOUT * 0.75) {
          float glow = diskGlow(hit, rr);
          float temp = pow(DIN / rr, 0.75);
          vec3 tint = bbody(temp);
          vec3 tangent = normalize(vec3(-hit.z, 0.0, hit.x));
          float beta = min(sqrt(0.5 / rr), 0.7);
          float dop = 1.0 / max(1.0 - beta * dot(tangent, normalize(v)), 0.35);
          float beam = clamp(dop * dop * dop, 0.15, 3.2);
          float gred = sqrt(max(1.0 - 1.0 / rr, 0.0));
          float w = secondary ? 0.42 : 1.0;
          acc += tint * glow * beam * gred * 1.6 * trans * w;
          trans *= mix(1.0, 0.55, clamp(glow, 0.0, 1.0));
        }
      }
      hits++;
    }
  }

  vec3 col = acc;

  if (captured < 0.5) {
    vec3 dome = normalize(v);
    vec2 sky = vec2(atan(dome.x, dome.z), dome.y);
    vec3 neb = vec3(0.35, 0.25, 0.55) * fbm(sky * 2.6)
      + vec3(0.15, 0.22, 0.45) * fbm(sky * 1.3);
    float glint = stars(sky, 46.0, 0.993) + stars(sky + 31.7, 130.0, 0.996) * 0.7;
    float skyReach = 1.0 - smoothstep(0.18, 0.46, aim);
    col += (neb * 0.4 + vec3(0.9, 0.93, 1.0) * glint * 0.8) * skyReach * trans;
  }

  col = 1.0 - exp(-col);
  float lum = dot(col, vec3(0.299, 0.587, 0.114));
  float vin = 1.0 - smoothstep(0.34, 0.5, aim);
  float alpha = clamp(lum * 1.7 + captured * 0.9, 0.0, 1.0) * vin;
  gl_FragColor = vec4(col, alpha);
}
`;

function bakeHole() {
  try {
    const size = 256;
    const cv = document.createElement('canvas');
    cv.width = size;
    cv.height = size;
    const gl = cv.getContext('webgl', {
      alpha: true,
      premultipliedAlpha: false,
      antialias: false,
      depth: false,
      stencil: false,
      preserveDrawingBuffer: true,
    });
    if (!gl) return;
    const make = (kind: number, src: string) => {
      const sh = gl.createShader(kind);
      if (!sh) return null;
      gl.shaderSource(sh, src);
      gl.compileShader(sh);
      if (!gl.getShaderParameter(sh, gl.COMPILE_STATUS)) {
        gl.deleteShader(sh);
        return null;
      }
      return sh;
    };
    const vs = make(gl.VERTEX_SHADER, holeVert);
    const fs = make(gl.FRAGMENT_SHADER, holeFrag);
    if (!vs || !fs) return;
    const prog = gl.createProgram();
    if (!prog) return;
    gl.attachShader(prog, vs);
    gl.attachShader(prog, fs);
    gl.linkProgram(prog);
    if (!gl.getProgramParameter(prog, gl.LINK_STATUS)) return;
    gl.useProgram(prog);
    const mesh = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, mesh);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1, -1, 3, -1, -1, 3]), gl.STATIC_DRAW);
    const slot = gl.getAttribLocation(prog, 'spot');
    gl.enableVertexAttribArray(slot);
    gl.vertexAttribPointer(slot, 2, gl.FLOAT, false, 0, 0);
    gl.viewport(0, 0, size, size);
    gl.uniform2f(gl.getUniformLocation(prog, 'res'), size, size);
    gl.uniform1f(gl.getUniformLocation(prog, 'time'), 12.0);
    gl.clearColor(0, 0, 0, 0);
    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.drawArrays(gl.TRIANGLES, 0, 3);
    holeImg.value = cv.toDataURL('image/png');
    const plug = gl.getExtension('WEBGL_lose_context');
    if (plug) plug.loseContext();
  } catch {}
}

function push(title: string, body: string | undefined, variant: string) {
  const id = ++counter;
  const duration = 5000;
  items.value.push({ id, title, body, variant, duration });
  if (items.value.length > 3) {
    items.value.shift();
  }
  setTimeout(() => {
    items.value = items.value.filter(i => i.id !== id);
  }, duration);
}

onMounted(() => {
  bakeHole();
  listen('wawity-notify', (e) => {
    const p = (e.payload ?? {}) as { title?: string; body?: string | null; variant?: string };
    push(String(p.title ?? ''), p.body ? String(p.body) : undefined, String(p.variant ?? 'info'));
  })
    .then(() => invoke('notify_ready'))
    .catch(() => {});
});
</script>

<style>
html,
body,
#app {
  background: transparent !important;
  margin: 0;
  overflow: hidden;
}
</style>

<style scoped>
.notify-root {
  position: fixed;
  inset: 0;
  display: flex;
  flex-direction: column;
  justify-content: flex-end;
  align-items: flex-end;
  padding: 12px;
  overflow: hidden;
  user-select: none;
}

.notify-stack {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 10px;
}

.ncard {
  position: relative;
  width: 340px;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px 16px;
  border-radius: 16px;
  background: rgba(10, 13, 22, 0.94);
  border: 1px solid rgba(255, 255, 255, 0.08);
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.45);
  overflow: hidden;
}

.ncard--success { --accent: #34d399; --accent-dim: rgba(52, 211, 153, 0.32); }
.ncard--error   { --accent: #f87171; --accent-dim: rgba(248, 113, 113, 0.32); }
.ncard--warning { --accent: #fbbf24; --accent-dim: rgba(251, 191, 36, 0.32); }
.ncard--info    { --accent: #7aa5ff; --accent-dim: rgba(122, 165, 255, 0.32); }

.ncard-hole {
  position: absolute;
  right: -48px;
  top: 50%;
  width: 176px;
  height: 176px;
  margin-top: -88px;
  pointer-events: none;
}

.hole-img {
  width: 100%;
  height: 100%;
  display: block;
}

.hole-ring {
  position: absolute;
  inset: 0;
  border-radius: 50%;
  background: radial-gradient(circle, transparent 33%, var(--accent-dim) 41%, transparent 54%);
  filter: blur(2px);
}

.hole-disc {
  position: absolute;
  inset: 27%;
  border-radius: 50%;
  background: radial-gradient(circle, #000 56%, rgba(0, 0, 0, 0.9) 68%, transparent 74%);
  box-shadow: 0 0 20px 2px var(--accent-dim);
}

.ncard-logo {
  width: 38px;
  height: 38px;
  border-radius: 11px;
  object-fit: cover;
  flex-shrink: 0;
  position: relative;
  z-index: 1;
}

.ncard-text {
  position: relative;
  z-index: 1;
  min-width: 0;
  padding-right: 36px;
}

.ncard-title {
  font-size: 13px;
  font-weight: 600;
  color: #f1f4ff;
  line-height: 1.3;
}

.ncard-body {
  font-size: 12px;
  color: rgba(226, 232, 255, 0.65);
  margin-top: 3px;
  line-height: 1.35;
  word-break: break-word;
}

.ncard-bar {
  position: absolute;
  left: 0;
  bottom: 0;
  height: 2px;
  width: 100%;
  background: var(--accent);
  transform-origin: left;
  animation-name: notifyShrink;
  animation-timing-function: linear;
  animation-fill-mode: forwards;
}

@keyframes notifyShrink {
  from { transform: scaleX(1); }
  to { transform: scaleX(0); }
}

.ncard-enter-active {
  transition: all 0.35s cubic-bezier(0.21, 1.02, 0.55, 1);
}

.ncard-leave-active {
  transition: all 0.3s cubic-bezier(0.55, 0, 0.55, 0.2);
  position: absolute;
  right: 0;
}

.ncard-enter-from {
  opacity: 0;
  transform: translateX(120%);
}

.ncard-leave-to {
  opacity: 0;
  transform: translateX(120%);
}

.ncard-move {
  transition: transform 0.3s ease;
}
</style>