<template>
  <div class="orb-card" :class="{ 'orb-card--material': isMaterial }">
    <div ref="wrap" class="orb-stage">
      <canvas
        ref="pane"
        class="orb-canvas"
        :class="{ 'orb-canvas--held': held }"
        @pointerdown="grab"
      />
      <canvas v-if="!isMaterial" ref="spray" class="orb-dots" />

      <button
        v-for="c in buckets"
        :key="c.code"
        :ref="el => holdPin(c.code, el)"
        type="button"
        class="pin"
        :class="{ 'pin--active': picked === c.code }"
        :title="c.name"
        @click="picked = c.code"
      >
        <span class="pin-dot" :class="pingTier(c.best)" />
        <span class="pin-label" v-text="c.code" />
      </button>

      <div class="zoom-stack">
        <button type="button" class="zoom-btn" title="Zoom in" @click="nudgeZoom(1.22)">
          <Plus :size="13" />
        </button>
        <button type="button" class="zoom-btn" title="Zoom out" @click="nudgeZoom(1 / 1.22)">
          <Minus :size="13" />
        </button>
      </div>
    </div>

    <div class="orb-panel">
      <template v-if="current">
        <div class="orb-head">
          <CountryFlag :code="current.code" :size="26" />
          <div class="orb-head-text">
            <span class="orb-head-name" v-text="current.name" />
            <span class="orb-head-count mono" v-text="current.servers.length" />
          </div>
        </div>
        <ul class="orb-list">
          <li
            v-for="srv in current.servers"
            :key="srv.id"
            class="orb-srv"
            :class="{
              'orb-srv--selected': vpnStore.selectedServerId === srv.id,
              'orb-srv--disabled': switching || vpnStore.loading,
              'orb-srv--expired': vpnStore.isServerExpired(srv.id),
            }"
            @click="hop(srv.id)"
          >
            <div class="orb-srv-text">
              <span class="orb-srv-name" v-text="srv.name" />
              <span class="orb-srv-meta mono" v-text="srvMeta(srv)" />
            </div>
            <button
              v-if="vpnStore.settings.multihop_enabled"
              type="button"
              class="entry-btn"
              :class="{ 'entry-btn--active': vpnStore.selectedEntryServerId === srv.id }"
              :title="t('servers.entryTitle')"
              @click.stop="vpnStore.selectEntryServer(
                vpnStore.selectedEntryServerId === srv.id ? null : srv.id
              )"
            >
              <Shuffle :size="11" />
            </button>
            <span
              class="ping-badge"
              :class="pingTier(srv.latencyMs)"
              v-text="pingText(srv)"
            />
            <Check v-if="vpnStore.selectedServerId === srv.id" :size="13" class="orb-check" />
          </li>
        </ul>
      </template>
      <p v-else-if="buckets.length === 0" class="orb-hint" v-text="noMatchText" />
      <p v-else class="orb-hint" v-text="t('servers.globeHint')" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watchEffect, onMounted, onUnmounted } from 'vue';
import { Shuffle, Check, Plus, Minus } from '../lib/appIcons';
import { useVpnStore } from '../stores/vpn';
import { t } from '../i18n';
import { groupServers, pingTier, worldMarkers } from '../lib/geo';
import type { ServerEntry } from '../lib/geo';
import CountryFlag from './CountryFlag.vue';

const props = defineProps<{ query: string }>();

const vpnStore = useVpnStore();

const isMaterial = computed(() => vpnStore.settings.ui_style === 'material');

const buckets = computed(() =>
  groupServers(
    vpnStore.subscriptions,
    props.query,
    vpnStore.settings.language,
    vpnStore.hiddenSubIds,
  ),
);

const picked = ref('');

let ownedCodes = new Set<string>();

watchEffect(() => {
  if (!buckets.value.length) { picked.value = ''; return; }
  if (!buckets.value.some(b => b.code === picked.value))
    picked.value = buckets.value[0].code;
});

watchEffect(() => {
  ownedCodes = new Set(buckets.value.map(b => b.code));
});

const current = computed(() => buckets.value.find(b => b.code === picked.value) ?? null);
const noMatchText = computed(() => t('servers.noServersMatch', { query: props.query }));
const switching = ref(false);

function srvMeta(srv: ServerEntry) { return `${srv.protocol} · ${srv.server}`; }
function pingText(srv: ServerEntry) {
  return srv.latencyMs != null ? `${srv.latencyMs}ms` : '—';
}

async function hop(id: string) {
  if (switching.value || vpnStore.loading) return;
  if (vpnStore.isServerExpired(id)) return;
  switching.value = true;
  try { await vpnStore.switchServer(id); }
  finally { switching.value = false; }
}

const wrap  = ref<HTMLDivElement | null>(null);
const pane  = ref<HTMLCanvasElement | null>(null);
const spray = ref<HTMLCanvasElement | null>(null);
const held  = ref(false);

const pinEls = new Map<string, HTMLElement>();
function holdPin(code: string, el: unknown) {
  if (el instanceof HTMLElement) pinEls.set(code, el);
  else pinEls.delete(code);
}

const atlas = worldMarkers();

const RENDER_SCALE = 0.7;
const DPR_CAP      = 1.25;
const MIN_FRAME_MS = 1000 / 61;
const BODY_FRAC    = 0.44;
const ZOOM_MIN     = 1.1;
const ZOOM_MAX     = 8.0;
const SPREAD_CAP   = 34;

let gl:   WebGLRenderingContext | null = null;
let prog: WebGLProgram | null = null;
let ink:  CanvasRenderingContext2D | null = null;
let slots: Record<string, WebGLUniformLocation | null> = {};
let raf = 0;
let born = 0;
let last = 0;
let calm = false;
let hasLod = false;

let turn   =  Math.PI * 0.45;
let lean   =  0.41;
let zoomNow  = 1.45;
let zoomGoal = 1.45;
let glide  = 0;
let lastX  = 0;
let lastY  = 0;
let idleAt = 0;

const vertSrc = `
attribute vec2 spot;
void main() { gl_Position = vec4(spot, 0.0, 1.0); }
`;

const fragMinimal = `
precision highp float;
uniform vec2  res;
uniform float spin;
uniform float lean;
uniform float zoom;
uniform sampler2D dayMap;

vec3 rotY(vec3 v,float a){float c=cos(a),s=sin(a);return vec3(v.x*c+v.z*s,v.y,-v.x*s+v.z*c);}
vec3 rotX(vec3 v,float a){float c=cos(a),s=sin(a);return vec3(v.x,v.y*c-v.z*s,v.y*s+v.z*c);}

void main() {
  vec2 uv = (gl_FragCoord.xy - 0.5*res) / min(res.x,res.y);
  float R = 0.44 * zoom;
  vec2  p = uv / R;
  float rr = length(p);
  vec3  bg = vec3(0.106, 0.106, 0.114);
  if (rr > 1.0) { gl_FragColor = vec4(bg, 1.0); return; }

  float z  = sqrt(max(1.0 - rr*rr, 0.0));
  vec3  n  = vec3(p.x, p.y, z);
  vec3  s  = rotY(rotX(n, -lean), -spin);

  float lon = atan(s.z, s.x);
  float lat = asin(clamp(s.y, -1.0, 1.0));
  vec2  tuv = vec2(fract(0.5 + lon/6.28318530718), 0.5 + lat/3.14159265359);

  vec3 base = pow(texture2D(dayMap, tuv).rgb, vec3(2.2));
  float g   = dot(base, vec3(0.299, 0.587, 0.114));
  vec3 mono = mix(vec3(g), base, 0.28) * vec3(0.78, 0.82, 0.90);

  float sea = smoothstep(0.005, 0.09, base.b - base.r);
  vec3 lit  = mix(mono, mono*0.40, sea);

  vec3 sun  = normalize(vec3(-0.45, 0.35, 0.85));
  float ndl = max(dot(n, sun), 0.0);
  lit *= (0.44 + 0.70*ndl);

  float rim = pow(1.0 - z, 3.2);
  lit += vec3(0.62, 0.70, 0.82) * rim * 0.10;

  vec3 col = mix(bg, lit*1.02, 1.0 - smoothstep(0.988, 1.0, rr));
  gl_FragColor = vec4(col, 1.0);
}
`;

const fragSrc = `
precision highp float;
uniform vec2  res;
uniform float time;
uniform float spin;
uniform float lean;
uniform float zoom;
uniform sampler2D dayMap;
uniform sampler2D nightMap;

float hash1(vec2 v) {
  return fract(sin(dot(v, vec2(127.1, 311.7))) * 43758.5453);
}
float hash3(vec3 v) {
  return fract(sin(dot(v, vec3(127.1, 311.7, 74.7))) * 43758.5453);
}
float noise3(vec3 x) {
  vec3 i = floor(x); vec3 f = fract(x);
  f = f*f*(3.0-2.0*f);
  float a=hash3(i),             b=hash3(i+vec3(1,0,0)),
        c=hash3(i+vec3(0,1,0)), d=hash3(i+vec3(1,1,0)),
        e=hash3(i+vec3(0,0,1)), g=hash3(i+vec3(1,0,1)),
        h=hash3(i+vec3(0,1,1)), k=hash3(i+vec3(1,1,1));
  return mix(mix(mix(a,b,f.x),mix(c,d,f.x),f.y),
             mix(mix(e,g,f.x),mix(h,k,f.x),f.y),f.z);
}
float fbm(vec3 p) {
  float s=0.0,a=0.5;
  for(int i=0;i<4;i++){s+=a*noise3(p);p=p*2.02+vec3(11.3,7.1,3.9);a*=0.5;}
  return s;
}
vec3 rotY(vec3 v,float a){float c=cos(a),s=sin(a);return vec3(v.x*c+v.z*s,v.y,-v.x*s+v.z*c);}
vec3 rotX(vec3 v,float a){float c=cos(a),s=sin(a);return vec3(v.x,v.y*c-v.z*s,v.y*s+v.z*c);}
float star(vec2 uv,float thr,float tw){
  vec2 g=floor(uv),f=fract(uv);
  float h=hash1(g);
  vec2  c=vec2(fract(h*13.73),fract(h*57.31))*0.8+0.1;
  float core=smoothstep(0.09,0.0,length(f-c));
  float blink=0.65+0.35*sin(time*(0.7+h*2.4)+h*43.0);
  return core*step(thr,h)*mix(1.0,blink,tw);
}
vec3 aces(vec3 x){return clamp(x*(2.51*x+0.03)/(x*(2.43*x+0.59)+0.14),0.0,1.0);}

void main() {
  vec2 uv = (gl_FragCoord.xy - 0.5*res) / min(res.x,res.y);
  float R  = 0.44 * zoom;
  vec2  p  = uv / R;
  float rr = length(p);
  vec3  sun = normalize(vec3(-0.55,0.22,0.62));

  vec2 sky = uv + vec2(spin*0.04,lean*0.025);
  float st = star(sky*26.0,0.78,1.0)
           + star(sky*55.0+17.0,0.86,1.0)*0.6
           + star(sky*110.0+41.0,0.93,0.0)*0.35;
  vec3 col = vec3(0.012,0.014,0.03);
  col += vec3(0.28,0.22,0.48)*fbm(vec3(sky*2.4,5.0))*0.05;
  col += vec3(0.10,0.24,0.42)*fbm(vec3(sky*1.5+9.0,2.0))*0.045;
  col += vec3(0.9,0.95,1.0)*st*0.85;
  col += vec3(0.2,0.45,0.9)*exp(-max(rr-1.0,0.0)*6.5)*0.22*step(1.0,rr);

  if (rr < 1.0) {
    float z = sqrt(max(1.0-rr*rr,0.0));
    vec3  n = vec3(p.x,p.y,z);
    vec3  s = rotY(rotX(n,-lean),-spin);

    float lon = atan(s.z,s.x);
    float lat = asin(clamp(s.y,-1.0,1.0));
    vec2  tuv = vec2(fract(0.5+lon/6.28318530718),
                     0.5+lat/3.14159265359);

    vec3 base = pow(texture2D(dayMap,   tuv).rgb, vec3(2.2));
    vec3 glow = pow(texture2D(nightMap, tuv).rgb, vec3(2.2));

    float ndl = dot(n,sun);
    float day  = smoothstep(-0.26,0.18,ndl);
    float sea  = smoothstep(0.01,0.1,base.b-base.r);
    float spec = pow(max(dot(n,normalize(sun+vec3(0,0,1))),0.0),90.0)*sea*day;

    vec3  cs   = rotY(s,time*0.01);
    float veil = smoothstep(0.6,0.82,fbm(cs*3.4+vec3(0.0,time*0.005,0.0)))*0.5;

    vec3 lit = base*(day*(1.1+0.45*max(ndl,0.0))+0.025*day);
    lit += vec3(1.0,0.9,0.7)*spec*0.5;
    lit  = mix(lit, mix(vec3(0.08,0.1,0.16),vec3(1.0,1.0,1.02),day), veil);
    lit += glow*vec3(1.0,0.82,0.55)*(1.0-day)*(1.0-veil*0.7)*5.5;

    float rim  = pow(1.0-z,2.4);
    lit += vec3(0.25,0.5,1.0)*rim*(0.35+0.55*day);
    lit += vec3(1.0,0.45,0.15)*exp(-pow(ndl*4.5,2.0))*rim*0.6;

    col = mix(col, lit, 1.0-smoothstep(0.995,1.0,rr));
  }

  col = aces(col*1.5);
  col += (hash1(gl_FragCoord.xy+fract(time)*61.0)-0.5)*0.01;
  gl_FragColor = vec4(col,1.0);
}
`;

function bake(kind: number, src: string): WebGLShader | null {
  if (!gl) return null;
  const sh = gl.createShader(kind)!;
  gl.shaderSource(sh, src);
  gl.compileShader(sh);
  if (!gl.getShaderParameter(sh, gl.COMPILE_STATUS)) {
    console.error('shader:', gl.getShaderInfoLog(sh));
    gl.deleteShader(sh); return null;
  }
  return sh;
}

function skin(unit: number, url: string) {
  if (!gl) return;
  const tex = gl.createTexture()!;
  gl.activeTexture(gl.TEXTURE0 + unit);
  gl.bindTexture(gl.TEXTURE_2D, tex);
  gl.texImage2D(gl.TEXTURE_2D,0,gl.RGB,1,1,0,gl.RGB,gl.UNSIGNED_BYTE,
    new Uint8Array([5,12,30]));
  const push = (pic: TexImageSource, w: number, h: number) => {
    if (!gl) return;
    gl.activeTexture(gl.TEXTURE0 + unit);
    gl.bindTexture(gl.TEXTURE_2D, tex);
    gl.texImage2D(gl.TEXTURE_2D,0,gl.RGB,gl.RGB,gl.UNSIGNED_BYTE,pic);
    const pot = (w&(w-1))===0 && (h&(h-1))===0;
    if (pot && hasLod) {
      gl.generateMipmap(gl.TEXTURE_2D);
      gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_MIN_FILTER,gl.LINEAR_MIPMAP_LINEAR);
    } else {
      gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_MIN_FILTER,gl.LINEAR);
    }
    gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_MAG_FILTER,gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_WRAP_S,gl.REPEAT);
    gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_WRAP_T,gl.CLAMP_TO_EDGE);
  };
  const fallback = () => {
    const img = new Image();
    img.onload = () => {
      if (!gl) return;
      gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, 1);
      push(img, img.width, img.height);
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
      push(pic, pic.width, pic.height);
      pic.close();
    })
    .catch(fallback);
}

function wire(): boolean {
  if (!pane.value) return false;
  gl = pane.value.getContext('webgl',
    { antialias:false, alpha:false, powerPreference:'low-power' });
  if (!gl) return false;
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.REPEAT);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
  const vs = bake(gl.VERTEX_SHADER, vertSrc);
  const fs = bake(gl.FRAGMENT_SHADER, isMaterial.value ? fragMinimal : fragSrc);
  if (!vs || !fs) return false;
  prog = gl.createProgram()!;
  gl.attachShader(prog,vs); gl.attachShader(prog,fs);
  gl.linkProgram(prog);
  if (!gl.getProgramParameter(prog,gl.LINK_STATUS)) {
    console.error(gl.getProgramInfoLog(prog)); return false;
  }
  gl.useProgram(prog);
  const buf = gl.createBuffer()!;
  gl.bindBuffer(gl.ARRAY_BUFFER,buf);
  gl.bufferData(gl.ARRAY_BUFFER,new Float32Array([-1,-1,3,-1,-1,3]),gl.STATIC_DRAW);
  const loc = gl.getAttribLocation(prog,'spot');
  gl.enableVertexAttribArray(loc);
  gl.vertexAttribPointer(loc,2,gl.FLOAT,false,0,0);
  slots = {
    res:      gl.getUniformLocation(prog,'res'),
    time:     gl.getUniformLocation(prog,'time'),
    spin:     gl.getUniformLocation(prog,'spin'),
    lean:     gl.getUniformLocation(prog,'lean'),
    zoom:     gl.getUniformLocation(prog,'zoom'),
    dayMap:   gl.getUniformLocation(prog,'dayMap'),
    nightMap: gl.getUniformLocation(prog,'nightMap'),
    uHasLod:  gl.getUniformLocation(prog,'uHasLod'),
  };
  skin(0, '/earth/day.jpg');
  if (!isMaterial.value) skin(1, '/earth/night.jpg');
  return true;
}

let stageW = 0;
let stageH = 0;

function measure() {
  if (!wrap.value) return;
  stageW = wrap.value.clientWidth;
  stageH = wrap.value.clientHeight;
}

function fit() {
  if (!pane.value || !gl || stageW === 0 || stageH === 0) return;
  const dpr = Math.min(devicePixelRatio||1, DPR_CAP) * RENDER_SCALE;
  const w = Math.max(1, Math.round(stageW * dpr));
  const h = Math.max(1, Math.round(stageH * dpr));
  if (pane.value.width!==w || pane.value.height!==h) {
    pane.value.width=w; pane.value.height=h;
    gl.viewport(0,0,w,h);
  }
  if (spray.value) {
    const sd = Math.min(devicePixelRatio||1, 1.5);
    const sw = Math.round(stageW * sd);
    const sh = Math.round(stageH * sd);
    if (spray.value.width!==sw || spray.value.height!==sh) {
      spray.value.width=sw; spray.value.height=sh;
      ink = spray.value.getContext('2d');
      if (ink) { ink.scale(sd,sd); }
    }
  }
}

function paint(now: number) {
  if (!gl||!prog||!pane.value) return;
  gl.uniform2f(slots.res, pane.value.width, pane.value.height);
  gl.uniform1f(slots.time, calm ? 0 : (now-born)/1000);
  gl.uniform1f(slots.spin, turn);
  gl.uniform1f(slots.lean, lean);
  gl.uniform1f(slots.zoom, zoomNow);
  gl.drawArrays(gl.TRIANGLES,0,3);
}

function project(lat: number, lon: number, w: number, h: number) {
  const R  = Math.min(w,h) * BODY_FRAC * zoomNow;
  const la = lat * Math.PI/180;
  const lo = lon * Math.PI/180;
  const sx = Math.cos(la)*Math.cos(lo);
  const sy = Math.sin(la);
  const sz = Math.cos(la)*Math.sin(lo);
  const ca = Math.cos(turn), sa = Math.sin(turn);
  const cb = Math.cos(lean), sb = Math.sin(lean);
  const x1 =  sx*ca + sz*sa;
  const z1 = -sx*sa + sz*ca;
  const y2 = sy*cb - z1*sb;
  const z2 = sy*sb + z1*cb;
  return { x: w/2+x1*R, y: h/2-y2*R, depth: z2 };
}
const spread = new Map<string, { x: number; y: number }>();

function splitAngle(seed: string) {
  let acc = 0;
  for (let i = 0; i < seed.length; i++) acc = (acc * 31 + seed.charCodeAt(i)) % 997;
  return (acc / 997) * Math.PI * 2;
}

function untangle(pins: Array<{ code: string; x: number; y: number; size: number }>) {
  for (let pass = 0; pass < 14; pass++) {
    let touched = false;
    for (let i = 0; i < pins.length; i++) {
      for (let j = i + 1; j < pins.length; j++) {
        const a = pins[i];
        const b = pins[j];
        const gap = (a.size + b.size) * 11 + 3;
        let dx = b.x - a.x;
        let dy = (b.y - b.size * 9) - (a.y - a.size * 9);
        let dist = Math.hypot(dx, dy);
        if (dist >= gap) continue;
        if (dist < 0.001) {
          const ang = splitAngle(a.code + b.code);
          dx = Math.cos(ang);
          dy = Math.sin(ang);
          dist = 1;
        }
        const push = ((gap - dist) / dist) * 0.5;
        a.x -= dx * push; a.y -= dy * push;
        b.x += dx * push; b.y += dy * push;
        touched = true;
      }
    }
    if (!touched) break;
  }
}
type Pin = {
  code: string; el: HTMLElement;
  ax: number; ay: number;
  x: number; y: number;
  depth: number; size: number;
};

const shownPins: Pin[] = [];
const pinPaint = new Map<string, { transform: string; opacity: string; z: string }>();

function applyPinStyle(code: string, el: HTMLElement, transform: string, opacity: string, z: string, hit: string) {
  let prev = pinPaint.get(code);
  if (!prev) {
    prev = { transform: '', opacity: '', z: '' };
    pinPaint.set(code, prev);
  }
  if (prev.transform !== transform) { el.style.transform = transform; prev.transform = transform; }
  if (prev.opacity !== opacity) {
    el.style.opacity = opacity;
    el.style.pointerEvents = hit;
    prev.opacity = opacity;
  }
  if (prev.z !== z) { el.style.zIndex = z; prev.z = z; }
}

function place() {
  const w = stageW;
  const h = stageH;
  if (w === 0 || h === 0) return;
  shownPins.length = 0;

  for (const b of buckets.value) {
    const el = pinEls.get(b.code);
    if (!el) continue;
    const sp = project(b.lat, b.lon, w, h);
    if (sp.depth <= 0.10) {
      spread.delete(b.code);
      applyPinStyle(
        b.code, el,
        `translate(${sp.x.toFixed(1)}px,${sp.y.toFixed(1)}px) translate(-50%,-100%) scale(0)`,
        '0', '0', 'none',
      );
      continue;
    }
    const size = 0.72 + sp.depth * 0.28;
    shownPins.push({ code: b.code, el, ax: sp.x, ay: sp.y, x: sp.x, y: sp.y, depth: sp.depth, size });
  }

  untangle(shownPins);

  for (const s of shownPins) {
    let ox = s.x - s.ax;
    let oy = s.y - s.ay;
    const len = Math.hypot(ox, oy);
    if (len > SPREAD_CAP) {
      ox *= SPREAD_CAP / len;
      oy *= SPREAD_CAP / len;
    }
    let cur = spread.get(s.code);
    if (!cur) { cur = { x: 0, y: 0 }; spread.set(s.code, cur); }
    cur.x += (ox - cur.x) * 0.22;
    cur.y += (oy - cur.y) * 0.22;
    applyPinStyle(
      s.code, s.el,
      `translate(${(s.ax + cur.x).toFixed(1)}px,${(s.ay + cur.y).toFixed(1)}px) translate(-50%,-100%) scale(${s.size.toFixed(3)})`,
      (0.4 + s.depth * 0.6).toFixed(2),
      String(10 + Math.round(s.depth * 40)),
      'auto',
    );
  }
}

const ALPHA_BANDS = 6;
const ALPHA_FILLS: string[] = Array.from({ length: ALPHA_BANDS }, (_, band) => {
  const a = 0.12 + ((band + 0.5) / ALPHA_BANDS) * 0.28;
  return `rgba(160,190,255,${a.toFixed(3)})`;
});
const bandXYR: Float32Array[] = Array.from({ length: ALPHA_BANDS }, () => new Float32Array(768));
const bandCount = new Int32Array(ALPHA_BANDS);

function sprinkle() {
  const w = stageW;
  const h = stageH;
  if (!ink || w === 0 || h === 0) return;
  ink.clearRect(0, 0, w, h);
  bandCount.fill(0);
  const zoomScale = Math.min(zoomNow, 1.5);

  for (const m of atlas) {
    if (ownedCodes.has(m.code)) continue;
    const sp = project(m.lat, m.lon, w, h);
    if (sp.depth <= 0.05) continue;
    let band = (sp.depth * ALPHA_BANDS) | 0;
    if (band >= ALPHA_BANDS) band = ALPHA_BANDS - 1;
    const buf = bandXYR[band];
    const n = bandCount[band];
    if (n * 3 + 2 >= buf.length) continue;
    buf[n * 3] = sp.x;
    buf[n * 3 + 1] = sp.y;
    buf[n * 3 + 2] = (0.85 + sp.depth * 0.75) * zoomScale;
    bandCount[band] = n + 1;
  }

  for (let band = 0; band < ALPHA_BANDS; band++) {
    const n = bandCount[band];
    if (n === 0) continue;
    const buf = bandXYR[band];
    ink.beginPath();
    for (let i = 0; i < n; i++) {
      const x = buf[i * 3];
      const y = buf[i * 3 + 1];
      const r = buf[i * 3 + 2];
      ink.moveTo(x + r, y);
      ink.arc(x, y, r, 0, Math.PI * 2);
    }
    ink.fillStyle = ALPHA_FILLS[band];
    ink.fill();
  }
}

function loop(now: number) {
  raf = requestAnimationFrame(loop);
  if (now - last < MIN_FRAME_MS) return;
  const dt = Math.min((now-last)/1000, 0.1);
  last = now;
  if (!held.value) {
    turn  += glide;
    glide *= 0.93;
    if (!isMaterial.value && !calm && now-idleAt>2600) turn += dt*0.05;
  }
  zoomNow += (zoomGoal-zoomNow)*0.13;
  fit();
  paint(now);
  place();
  if (!isMaterial.value) sprinkle();
}

function nap() {
  if (document.hidden) {
    if (raf) { cancelAnimationFrame(raf); raf = 0; }
    return;
  }
  if (!raf) {
    last = performance.now();
    idleAt = last;
    raf = requestAnimationFrame(loop);
  }
}

function grab(e: PointerEvent) {
  held.value=true; glide=0;
  lastX=e.clientX; lastY=e.clientY;
  idleAt=performance.now();
}

function drag(e: PointerEvent) {
  if (!held.value) return;
  const dx=e.clientX-lastX;
  const dy=e.clientY-lastY;
  lastX=e.clientX; lastY=e.clientY;
  turn += (dx*0.0045)/zoomNow;
  lean  = Math.min(1.15,Math.max(-0.85,lean+(dy*0.004)/zoomNow));
  glide = (dx*0.0045)/zoomNow;
  idleAt=performance.now();
}

function release() { held.value=false; idleAt=performance.now(); }

function roll(e: WheelEvent) {
  e.preventDefault();
  zoomGoal = Math.min(ZOOM_MAX, Math.max(ZOOM_MIN, zoomGoal * Math.exp(-e.deltaY * 0.0012)));
  idleAt = performance.now();
}

function nudgeZoom(f: number) {
  zoomGoal = Math.min(ZOOM_MAX, Math.max(ZOOM_MIN, zoomGoal * f));
  idleAt = performance.now();
}

let watcher: ResizeObserver|null=null;

onMounted(() => {
  if (!wire()) return;
  calm  = matchMedia('(prefers-reduced-motion:reduce)').matches;
  born  = performance.now();
  last  = born;
  idleAt = born;
  measure();
  watcher = new ResizeObserver(()=>{ measure(); fit(); paint(performance.now()); place(); sprinkle(); });
  if (wrap.value) watcher.observe(wrap.value);
  window.addEventListener('pointermove', drag, { passive: true });
  window.addEventListener('pointerup', release, { passive: true });
  wrap.value?.addEventListener('wheel', roll, { passive:false });
  document.addEventListener('visibilitychange', nap);
  raf = requestAnimationFrame(loop);
});

onUnmounted(() => {
  if (raf) cancelAnimationFrame(raf);
  raf = 0;
  window.removeEventListener('pointermove', drag);
  window.removeEventListener('pointerup', release);
  wrap.value?.removeEventListener('wheel', roll);
  document.removeEventListener('visibilitychange', nap);
  watcher?.disconnect();
  watcher = null;
  pinEls.clear();
  pinPaint.clear();
  spread.clear();
  if (gl) { gl.getExtension('WEBGL_lose_context')?.loseContext(); }
  gl = null;
  prog = null;
  ink = null;
});
</script>

<style scoped>
.orb-card {
  position: relative;
  height: 480px;
  border-radius: 20px;
  border: 1px solid rgba(255,255,255,0.09);
  background: rgba(255,255,255,0.03);
  box-shadow: inset 0 1px 0 rgba(255,255,255,0.07), 0 18px 44px rgba(0,0,0,0.4);
  overflow: hidden;
  display: flex;
}

.orb-stage {
  position: relative;
  flex: 1;
  min-width: 0;
}

.orb-canvas {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  cursor: grab;
  touch-action: none;
}
.orb-canvas--held { cursor: grabbing; }

.orb-dots {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
}
.orb-srv--expired {
  opacity: 0.35;
  filter: grayscale(0.7);
  pointer-events: none;
}
.pin {
  position: absolute;
  top: 0; left: 0;
  display: flex;
  align-items: center;
  gap: 0;
  padding: 4px;
  border-radius: 999px;
  border: 1px solid rgba(255,255,255,0.18);
  background: rgba(10,12,22,0.58);
  backdrop-filter: blur(8px);
  color: rgba(235,238,250,0.9);
  font-size: 9px;
  font-weight: 700;
  letter-spacing: 0.05em;
  cursor: pointer;
  will-change: transform, opacity;
  overflow: hidden;
  max-width: 20px;
  transition:
    max-width 280ms cubic-bezier(0.34,1.2,0.64,1),
    padding   280ms cubic-bezier(0.34,1.2,0.64,1),
    gap       280ms cubic-bezier(0.34,1.2,0.64,1),
    border-color 180ms ease,
    background   180ms ease,
    box-shadow   180ms ease;
}

.pin:hover,
.pin--active {
  max-width: 72px;
  padding: 3px 8px 3px 4px;
  gap: 5px;
}

.pin-label {
  overflow: hidden;
  white-space: nowrap;
  max-width: 0;
  opacity: 0;
  transition:
    max-width 280ms cubic-bezier(0.34, 1.2, 0.64, 1),
    opacity 200ms ease;
}

.pin:hover .pin-label,
.pin--active .pin-label {
  max-width: 56px;
  opacity: 1;
}

.pin-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
  background: rgba(235,238,250,0.35);
  transition: width 220ms, height 220ms;
}
.pin:hover .pin-dot,
.pin--active .pin-dot { width: 6px; height: 6px; }

.pin-dot.tier-good { background:#5ee69a; box-shadow:0 0 5px rgba(94,230,154,0.8); }
.pin-dot.tier-ok   { background:#f0d36a; box-shadow:0 0 5px rgba(240,211,106,0.8); }
.pin-dot.tier-slow { background:#ff9f6b; box-shadow:0 0 5px rgba(255,159,107,0.8); }
.pin-dot.tier-bad  { background:#ff8a92; box-shadow:0 0 5px rgba(255,138,146,0.8); }

.pin-label {
  overflow: hidden;
  white-space: nowrap;
  max-width: 0;
  opacity: 0;
  transition:
    max-width 280ms cubic-bezier(0.34,1.2,0.64,1),
    opacity   200ms ease;
}
.pin:hover .pin-label,
.pin--active .pin-label { max-width: 36px; opacity: 1; }

.zoom-stack {
  position: absolute;
  left: 12px;
  bottom: 12px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  z-index: 60;
}

.zoom-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: 9px;
  border: 1px solid rgba(255,255,255,0.14);
  background: rgba(10,12,22,0.55);
  backdrop-filter: blur(10px);
  color: rgba(235,238,250,0.7);
  cursor: pointer;
  transition: background 150ms, color 150ms, transform 150ms;
}
.zoom-btn:hover { background:rgba(255,255,255,0.12); color:#fff; }
.zoom-btn:active { transform:scale(0.9); }

.orb-panel {
  width: 258px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  border-left: 1px solid rgba(255,255,255,0.08);
  background: rgba(10,12,20,0.5);
  backdrop-filter: blur(12px);
  z-index: 70;
}

.orb-head {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 14px 14px 10px;
  border-bottom: 1px solid rgba(255,255,255,0.07);
}
.orb-head-text { display:flex; flex-direction:column; gap:1px; min-width:0; }
.orb-head-name { font-size:13px; font-weight:600; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; }
.orb-head-count { font-size:10.5px; color:rgba(235,238,250,0.45); }

.orb-list {
  list-style: none;
  flex: 1;
  overflow-y: auto;
  padding: 6px;
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.orb-srv {
  display: flex;
  align-items: center;
  gap: 7px;
  padding: 8px 9px;
  border-radius: 11px;
  cursor: pointer;
  transition: background 140ms;
  content-visibility: auto;
  contain-intrinsic-size: auto 42px;
}
.orb-srv:hover { background:rgba(255,255,255,0.06); }
.orb-srv--selected { background:rgba(167,139,250,0.13); }
.orb-srv--selected:hover { background:rgba(167,139,250,0.17); }
.orb-srv--disabled { opacity:0.55; pointer-events:none; }

.orb-srv-text { display:flex; flex-direction:column; gap:1px; min-width:0; flex:1; }
.orb-srv-name { font-size:11.5px; font-weight:500; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; }
.orb-srv-meta { font-size:9.5px; color:rgba(235,238,250,0.35); overflow:hidden; text-overflow:ellipsis; white-space:nowrap; }

.entry-btn {
  display:flex; align-items:center; justify-content:center;
  width:22px; height:22px;
  border-radius:7px;
  border:1px solid rgba(255,255,255,0.12);
  background:transparent;
  color:rgba(235,238,250,0.45);
  cursor:pointer; flex-shrink:0;
  transition:all 150ms;
}
.entry-btn:hover { color:#fff; background:rgba(255,255,255,0.08); }
.entry-btn--active {
  border-color:rgba(143,182,255,0.6);
  background:rgba(143,182,255,0.14);
  color:#8fb6ff;
}

.ping-badge {
  font-size:9.5px; font-weight:600;
  padding:2px 6px;
  border-radius:999px;
  flex-shrink:0;
  background:rgba(255,255,255,0.06);
  color:rgba(235,238,250,0.4);
}
.ping-badge.tier-good { background:rgba(94,230,154,0.12);  color:#5ee69a; }
.ping-badge.tier-ok   { background:rgba(240,211,106,0.12); color:#f0d36a; }
.ping-badge.tier-slow { background:rgba(255,159,107,0.14); color:#ff9f6b; }
.ping-badge.tier-bad  { background:rgba(255,138,146,0.14); color:#ff8a92; }

.orb-check { color:#a78bfa; flex-shrink:0; }

.orb-hint {
  margin:auto; padding:20px;
  font-size:12px; text-align:center;
  color:rgba(235,238,250,0.4);
}

.orb-card--material {
  border-radius: 16px;
  border: 1px solid var(--border);
  background: #1b1b1d;
  box-shadow: none;
}

.orb-card--material .pin {
  background: #2b2b30;
  border-color: rgba(255,255,255,0.14);
  backdrop-filter: none;
  -webkit-backdrop-filter: none;
  color: var(--foreground, #e3e3e6);
}

.orb-card--material .pin--active,
.orb-card--material .pin:hover {
  border-color: #a8c7fa;
  color: #a8c7fa;
  background: #303237;
}

.orb-card--material .pin-dot { background: rgba(227,227,230,0.4); }

.orb-card--material .pin-dot.tier-good { background:#6dd58c; box-shadow:none; }
.orb-card--material .pin-dot.tier-ok   { background:#f0d36a; box-shadow:none; }
.orb-card--material .pin-dot.tier-slow { background:#ffb59b; box-shadow:none; }
.orb-card--material .pin-dot.tier-bad  { background:#f2b8b5; box-shadow:none; }

.orb-card--material .zoom-btn {
  background: #26262b;
  border-color: rgba(255,255,255,0.12);
  backdrop-filter: none;
  -webkit-backdrop-filter: none;
  color: #c9c9ce;
}

.orb-card--material .zoom-btn:hover { background:#333338; color:#fff; }

.orb-card--material .orb-panel {
  background: transparent;
  backdrop-filter: none;
  -webkit-backdrop-filter: none;
  border-left-color: var(--border);
}

.orb-card--material .orb-srv:hover { background: rgba(255,255,255,0.06); }

.orb-card--material .orb-srv--selected,
.orb-card--material .orb-srv--selected:hover {
  background: rgba(168, 199, 250, 0.14);
}

.orb-card--material .orb-check { color:#a8c7fa; }

.orb-card--material .entry-btn--active {
  border-color: rgba(168,199,250,0.55);
  background: rgba(168,199,250,0.14);
  color:#a8c7fa;
}
</style>
