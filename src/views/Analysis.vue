<script lang="ts">
export default { name: 'AnalysisView' };
</script>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, onActivated, onDeactivated } from 'vue';
import { ArrowDown, ArrowUp, ShieldCheck, ShieldOff, Signal, Timer, HardDriveDownload, HardDriveUpload } from 'lucide-vue-next';
import { useVpnStore } from '../stores/vpn';
import { t } from '../i18n';

const vpnStore = useVpnStore();

const SPAN = 90;
const PING_SPAN = 60;
const SAMPLE_MS = 1000;
const CHART_W = 600;
const CHART_H = 170;
const SPARK_W = 160;
const SPARK_H = 34;

const rx = new Float64Array(SPAN);
const tx = new Float64Array(SPAN);
const ping = new Float64Array(PING_SPAN);

let head = 0;
let pingHead = 0;
let pingFilled = 0;

const tick = ref(0);

let timer: number | null = null;

function sample() {
  if (document.hidden) return;

  rx[head] = (vpnStore.status.speed_rx / 1_000_000) * 8;
  tx[head] = (vpnStore.status.speed_tx / 1_000_000) * 8;
  head = head + 1 === SPAN ? 0 : head + 1;

  const ms = vpnStore.currentPingMs;
  if (typeof ms === 'number' && Number.isFinite(ms)) {
    ping[pingHead] = ms;
    pingHead = pingHead + 1 === PING_SPAN ? 0 : pingHead + 1;
    if (pingFilled < PING_SPAN) pingFilled++;
  }

  tick.value++;
}

function ordered(buf: Float64Array, at: number, count: number, out: Float64Array): Float64Array {
  const len = buf.length;
  const from = (at - count + len) % len;
  for (let i = 0; i < count; i++) out[i] = buf[(from + i) % len];
  return out;
}

const rxView = new Float64Array(SPAN);
const txView = new Float64Array(SPAN);
const pingView = new Float64Array(PING_SPAN);

function niceMax(v: number): number {
  const target = Math.max(v * 1.15, 1);
  const steps = [1, 2, 5, 10, 20, 50, 100, 200, 500, 1000];
  for (const s of steps) {
    if (target <= s) return s;
  }
  return Math.ceil(target / 1000) * 1000;
}

const throughput = computed(() => {
  void tick.value;
  ordered(rx, head, SPAN, rxView);
  ordered(tx, head, SPAN, txView);

  let peak = 0;
  let sum = 0;
  let used = 0;
  for (let i = 0; i < SPAN; i++) {
    const d = rxView[i];
    const u = txView[i];
    if (d > peak) peak = d;
    if (u > peak) peak = u;
    if (d > 0) {
      sum += d;
      used++;
    }
  }

  const max = niceMax(peak);
  const dl = curve(rxView, SPAN, CHART_W, CHART_H, max);
  const ul = curve(txView, SPAN, CHART_W, CHART_H, max);

  return {
    max,
    dlLine: dl,
    ulLine: ul,
    dlArea: dl ? dl + ' L ' + CHART_W + ' ' + CHART_H + ' L 0 ' + CHART_H + ' Z' : '',
    ulArea: ul ? ul + ' L ' + CHART_W + ' ' + CHART_H + ' L 0 ' + CHART_H + ' Z' : '',
    peak: peak.toFixed(1) + ' Mbit/s',
    avg: (used ? sum / used : 0).toFixed(1) + ' Mbit/s',
  };
});

const pingLine = computed(() => {
  void tick.value;
  if (pingFilled < 2) return '';
  ordered(ping, pingHead, pingFilled, pingView);
  let max = 50;
  for (let i = 0; i < pingFilled; i++) {
    if (pingView[i] > max) max = pingView[i];
  }
  return curve(pingView, pingFilled, SPARK_W, SPARK_H, max);
});

function curve(data: Float64Array, count: number, w: number, h: number, max: number): string {
  if (count < 2) return '';
  const span = h - 6;
  const step = w / (count - 1);
  const parts: string[] = new Array(count);

  let px = 0;
  let py = h - Math.min(data[0] / max, 1) * span - 3;
  parts[0] = 'M ' + px.toFixed(1) + ' ' + py.toFixed(1);

  for (let i = 1; i < count; i++) {
    const x = i * step;
    const y = h - Math.min(data[i] / max, 1) * span - 3;
    parts[i] = ' Q ' + px.toFixed(1) + ' ' + py.toFixed(1) + ' ' + ((px + x) / 2).toFixed(1) + ' ' + ((py + y) / 2).toFixed(1);
    px = x;
    py = y;
  }

  return parts.join('') + ' L ' + px.toFixed(1) + ' ' + py.toFixed(1);
}

const speedMaxLabel = computed(() => throughput.value.max + ' Mbit/s');
const speedMidLabel = computed(() => (throughput.value.max / 2).toFixed(throughput.value.max < 4 ? 1 : 0));

const pingTone = computed(() => {
  const ms = vpnStore.currentPingMs;
  if (ms === null || ms === undefined) return '';
  if (ms < 80) return 'ping-good';
  if (ms < 200) return 'ping-ok';
  if (ms < 500) return 'ping-slow';
  return 'ping-bad';
});

const pills = computed(() => [
  {
    key: 'ping',
    icon: Signal,
    tone: pingTone.value,
    label: t('analysis.currentPing'),
    value: vpnStore.currentPingDisplay,
  },
  {
    key: 'shield',
    icon: vpnStore.status.kill_switch ? ShieldCheck : ShieldOff,
    tone: vpnStore.status.kill_switch ? 'ping-good' : '',
    label: t('analysis.killSwitch'),
    value: vpnStore.status.kill_switch ? t('analysis.active') : t('analysis.off'),
  },
  {
    key: 'session',
    icon: Timer,
    tone: '',
    label: t('analysis.session'),
    value: vpnStore.sessionDuration,
  },
  {
    key: 'rx',
    icon: HardDriveDownload,
    tone: '',
    label: t('analysis.downloaded'),
    value: vpnStore.totalRxFormatted,
  },
  {
    key: 'tx',
    icon: HardDriveUpload,
    tone: '',
    label: t('analysis.uploaded'),
    value: vpnStore.totalTxFormatted,
  },
]);

function start() {
  if (timer !== null) return;
  timer = window.setInterval(sample, SAMPLE_MS);
}

function stop() {
  if (timer === null) return;
  window.clearInterval(timer);
  timer = null;
}

function onVisibility() {
  if (document.hidden) stop();
  else start();
}

onMounted(() => {
  start();
  document.addEventListener('visibilitychange', onVisibility);
});

onActivated(() => {
  if (!document.hidden) start();
});

onDeactivated(stop);

onUnmounted(() => {
  stop();
  document.removeEventListener('visibilitychange', onVisibility);
});
</script>

<template>
  <div class="page">
    <div class="page-content">
      <div class="page-header rise" style="animation-delay: 0ms">
        <div>
          <h1 class="page-title" v-text="t('analysis.title')"></h1>
          <p class="page-sub" v-text="t('analysis.subtitle')"></p>
        </div>
        <div class="pill pill--live" :class="{ 'pill--on': vpnStore.status.connected }">
          <span class="live-dot"></span>
          <span v-text="vpnStore.status.connected ? t('analysis.connected') : t('analysis.disconnected')"></span>
        </div>
      </div>

      <div class="pill-strip rise" style="animation-delay: 70ms">
        <div v-for="p in pills" :key="p.key" class="pill">
          <component :is="p.icon" :size="13" class="pill-icon" :class="p.tone" aria-hidden="true" />
          <span class="pill-label" v-text="p.label"></span>
          <span class="pill-value mono" :class="p.tone" v-text="p.value"></span>
        </div>
      </div>

      <div class="hero rise" style="animation-delay: 150ms">
        <div class="hero-block">
          <div class="hero-head">
            <ArrowDown :size="15" class="hero-icon hero-icon--dl" aria-hidden="true" />
            <span class="hero-label" v-text="t('analysis.currentDown')"></span>
          </div>
          <p class="hero-value hero-value--dl mono" v-text="vpnStore.speedRxFormatted"></p>
        </div>
        <div class="hero-sep" aria-hidden="true"></div>
        <div class="hero-block">
          <div class="hero-head">
            <ArrowUp :size="15" class="hero-icon hero-icon--ul" aria-hidden="true" />
            <span class="hero-label" v-text="t('analysis.currentUp')"></span>
          </div>
          <p class="hero-value hero-value--ul mono" v-text="vpnStore.speedTxFormatted"></p>
        </div>
      </div>

      <div class="chart-card rise" style="animation-delay: 230ms">
        <div class="chart-header">
          <h2 class="chart-title" v-text="t('analysis.throughput')"></h2>
          <div class="chart-legend">
            <span class="legend-item">
              <span class="legend-dot legend-dot--dl" aria-hidden="true"></span>
              <span v-text="t('analysis.download')"></span>
            </span>
            <span class="legend-item">
              <span class="legend-dot legend-dot--ul" aria-hidden="true"></span>
              <span v-text="t('analysis.upload')"></span>
            </span>
          </div>
        </div>
        <div class="chart-wrap">
          <svg class="chart-svg" viewBox="0 0 600 170" preserveAspectRatio="none">
            <defs>
              <linearGradient id="gradDl" x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" stop-color="rgba(56, 224, 178, 0.35)" />
                <stop offset="100%" stop-color="rgba(56, 224, 178, 0)" />
              </linearGradient>
              <linearGradient id="gradUl" x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" stop-color="rgba(255, 158, 100, 0.28)" />
                <stop offset="100%" stop-color="rgba(255, 158, 100, 0)" />
              </linearGradient>
            </defs>
            <line v-for="g in [0.25, 0.5, 0.75]" :key="g" x1="0" :y1="170 * g" x2="600" :y2="170 * g" class="grid-line" />
            <path :d="throughput.dlArea" fill="url(#gradDl)" />
            <path :d="throughput.ulArea" fill="url(#gradUl)" />
            <path :d="throughput.dlLine" class="line line--dl" />
            <path :d="throughput.ulLine" class="line line--ul" />
          </svg>
          <div class="axis">
            <span class="axis-label mono" v-text="speedMaxLabel"></span>
            <span class="axis-label mono" v-text="speedMidLabel"></span>
            <span class="axis-label mono">0</span>
          </div>
        </div>
        <div class="chart-foot">
          <span class="foot-pill">
            <span v-text="t('analysis.peak')"></span>
            <span class="mono foot-val" v-text="throughput.peak"></span>
          </span>
          <span class="foot-pill">
            <span v-text="t('analysis.avg')"></span>
            <span class="mono foot-val" v-text="throughput.avg"></span>
          </span>
        </div>
      </div>

      <div class="conn-card rise" style="animation-delay: 310ms">
        <h2 class="card-title" v-text="t('analysis.connection')"></h2>
        <div class="conn-rows">
          <div class="conn-row">
            <span class="conn-label" v-text="t('analysis.server')"></span>
            <span class="conn-value mono" v-text="vpnStore.status.server_name ?? '—'"></span>
          </div>
          <div v-if="vpnStore.status.multihop" class="conn-row">
            <span class="conn-label" v-text="t('analysis.entry')"></span>
            <span class="conn-value mono" v-text="vpnStore.status.entry_server_name ?? '—'"></span>
          </div>
          <div class="conn-row">
            <span class="conn-label" v-text="t('analysis.interface')"></span>
            <span class="conn-value mono" v-text="vpnStore.status.interface ?? '—'"></span>
          </div>
          <div class="conn-row">
            <span class="conn-label" v-text="t('analysis.pid')"></span>
            <span class="conn-value mono" v-text="vpnStore.status.pid ?? '—'"></span>
          </div>
          <div class="conn-row">
            <span class="conn-label" v-text="t('analysis.pingHistory')"></span>
            <svg class="ping-spark" viewBox="0 0 160 34" preserveAspectRatio="none">
              <path :d="pingLine" class="line line--ping" />
            </svg>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.page {
  position: relative;
  min-height: 100%;
}

.page-content {
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
  gap: 16px;
  contain: layout style;
}

.rise {
  opacity: 0;
  animation: riseIn 520ms cubic-bezier(0.22, 0.9, 0.3, 1) forwards;
  will-change: transform, opacity;
}

@keyframes riseIn {
  from { opacity: 0; transform: translateY(14px); }
  to { opacity: 1; transform: translateY(0); }
}

.page-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.page-title { font-size: 21px; font-weight: 600; letter-spacing: -0.02em; }
.page-sub { font-size: 12.5px; color: var(--muted-foreground); margin-top: 4px; }

.pill-strip {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.pill {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  padding: 7px 13px;
  border-radius: 999px;
  border: 1px solid var(--border);
  background: rgba(12, 16, 26, 0.82);
  font-size: 11.5px;
  color: var(--muted-foreground);
  transition: transform 200ms cubic-bezier(0.22, 0.9, 0.3, 1), border-color 200ms;
}

.pill:hover {
  transform: translateY(-2px);
  border-color: rgba(120, 200, 210, 0.25);
}

.pill-icon { color: var(--muted-foreground); flex-shrink: 0; }
.pill-label { white-space: nowrap; }
.pill-value { color: var(--foreground); white-space: nowrap; }

.pill--live { color: var(--muted-foreground); }
.pill--on { color: #5ee69a; border-color: rgba(94, 230, 154, 0.25); }

.live-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: currentColor;
  animation: pulseDot 2s ease-in-out infinite;
}

@keyframes pulseDot {
  0%, 100% { opacity: 1; transform: scale(1); }
  50% { opacity: 0.45; transform: scale(0.8); }
}

.hero {
  display: flex;
  align-items: stretch;
  gap: 26px;
  padding: 6px 4px;
  max-width: 560px;
}

.hero-block { display: flex; flex-direction: column; gap: 6px; }

.hero-head { display: flex; align-items: center; gap: 7px; }

.hero-icon--dl { color: #38e0b2; }
.hero-icon--ul { color: #ff9e64; }

.hero-label {
  font-size: 11.5px;
  color: var(--muted-foreground);
  text-transform: uppercase;
  letter-spacing: 0.06em;
}

.hero-value {
  font-size: 34px;
  font-weight: 550;
  letter-spacing: -0.02em;
  line-height: 1.1;
  font-variant-numeric: tabular-nums;
}

.hero-value--dl { color: #cffff0; text-shadow: 0 0 22px rgba(56, 224, 178, 0.35); }
.hero-value--ul { color: #ffe4d1; text-shadow: 0 0 22px rgba(255, 158, 100, 0.3); }

.hero-sep {
  width: 1px;
  background: linear-gradient(180deg, transparent, var(--border), transparent);
}

.chart-card {
  padding: 16px 18px;
  border-radius: 18px;
  border: 1px solid var(--border);
  background: rgba(12, 16, 26, 0.82);
  contain: layout paint style;
}

.chart-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
  flex-wrap: wrap;
  gap: 8px;
}

.chart-title { font-size: 13px; font-weight: 500; }

.chart-legend {
  display: flex;
  align-items: center;
  gap: 14px;
  font-size: 11.5px;
  color: var(--muted-foreground);
}

.legend-item { display: flex; align-items: center; gap: 6px; }

.legend-dot { width: 7px; height: 7px; border-radius: 50%; flex-shrink: 0; }
.legend-dot--dl { background: #38e0b2; box-shadow: 0 0 8px rgba(56, 224, 178, 0.7); }
.legend-dot--ul { background: #ff9e64; box-shadow: 0 0 8px rgba(255, 158, 100, 0.6); }

.chart-wrap { position: relative; }

.chart-svg { width: 100%; height: 170px; display: block; }

.grid-line { stroke: rgba(255, 255, 255, 0.05); stroke-width: 1; }

.line { fill: none; stroke-width: 1.8; stroke-linejoin: round; stroke-linecap: round; }
.line--dl { stroke: #38e0b2; }
.line--ul { stroke: #ff9e64; }
.line--ping { stroke: #7ab8ff; stroke-width: 1.4; }

.axis {
  position: absolute;
  top: 0;
  right: 6px;
  height: 100%;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  pointer-events: none;
}

.axis-label { font-size: 10px; color: rgba(255, 255, 255, 0.35); font-variant-numeric: tabular-nums; }

.chart-foot {
  display: flex;
  gap: 8px;
  margin-top: 12px;
}

.foot-pill {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  padding: 5px 11px;
  border-radius: 999px;
  border: 1px solid var(--border);
  background: rgba(255, 255, 255, 0.03);
  font-size: 11px;
  color: var(--muted-foreground);
}

.foot-val { color: var(--foreground); font-variant-numeric: tabular-nums; }

.conn-card {
  padding: 16px 18px;
  border-radius: 18px;
  border: 1px solid var(--border);
  background: rgba(12, 16, 26, 0.82);
  contain: layout paint style;
}

.card-title { font-size: 13px; font-weight: 500; margin-bottom: 12px; }

.conn-rows { display: flex; flex-direction: column; }

.conn-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 0;
  border-bottom: 1px solid var(--border);
  gap: 12px;
}

.conn-row:last-child { border-bottom: none; }

.conn-label { font-size: 12.5px; color: var(--muted-foreground); }
.conn-value { font-size: 12.5px; color: var(--foreground); }

.ping-spark { width: 160px; height: 34px; flex-shrink: 0; }
</style>
