<script lang="ts">
export default { name: 'AnalysisView' };
</script>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, onActivated, onDeactivated } from 'vue';
import { ArrowDown, ArrowUp, ShieldCheck, ShieldOff, Signal, Timer, HardDriveDownload, HardDriveUpload } from '../lib/appIcons';
import { invoke } from '@tauri-apps/api/tauri';
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

interface AppTrafficEntry {
  name: string;
  rx: number;
  tx: number;
}

const appStats = ref<AppTrafficEntry[]>([]);
let statsCounter = 0;
let statsBusy = false;

async function pollAppStats() {
  if (statsBusy) return;
  statsBusy = true;
  try {
    appStats.value = await invoke<AppTrafficEntry[]>('get_app_traffic');
  } catch {
    
  } finally {
    statsBusy = false;
  }
}

function barWidth(a: AppTrafficEntry) {
  const max = Math.max(...appStats.value.map((x) => x.rx + x.tx), 1);
  return Math.max(3, ((a.rx + a.tx) / max) * 100);
}

function fmtBytes(b: number): string {
  if (b >= 1_073_741_824) return (b / 1_073_741_824).toFixed(2) + ' GB';
  if (b >= 1_048_576) return (b / 1_048_576).toFixed(1) + ' MB';
  if (b >= 1_024) return (b / 1_024).toFixed(0) + ' KB';
  return b + ' B';
}

function sample() {
  if (document.hidden) return;
  if (++statsCounter % 3 === 0) void pollAppStats();

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

const todayRows = computed(() => {
  void tick.value;
  return vpnStore.trafficToday;
});

const todayTotal = computed(() => {
  void tick.value;
  const rows = todayRows.value;
  return { total: rows.reduce((acc, r) => acc + r.total, 0) };
});

const weekTotal = computed(() => {
  void tick.value;
  return vpnStore.trafficWeekTotal;
});

interface WeekBar {
  key: string;
  label: string;
  rxH: number;
  txH: number;
  title: string;
}

const weekBars = computed<WeekBar[]>(() => {
  void tick.value;
  
  const perDay = new Map<string, { rx: number; tx: number }>();
  const now = Date.now();
  for (let i = 6; i >= 0; i--) {
    const dt = new Date(now - i * 86_400_000);
    const m = String(dt.getMonth() + 1).padStart(2, '0');
    const d = String(dt.getDate()).padStart(2, '0');
    perDay.set(`${dt.getFullYear()}-${m}-${d}`, { rx: 0, tx: 0 });
  }
  for (const entry of Object.values(vpnStore.trafficHistory)) {
    for (const day of entry.days) {
      const slot = perDay.get(day.d);
      if (slot) {
        slot.rx += day.rx;
        slot.tx += day.tx;
      }
    }
    if (entry.liveRx > 0 || entry.liveTx > 0) {
      const slot = perDay.get(weekBarsTodayKey());
      if (slot) {
        slot.rx += entry.liveRx;
        slot.tx += entry.liveTx;
      }
    }
  }

  let peak = 1;
  for (const v of perDay.values()) peak = Math.max(peak, v.rx + v.tx);

  const names = weekdayNames();
  const bars: WeekBar[] = [];
  let idx = 0;
  for (const [key, val] of perDay.entries()) {
    const dateMs = now - (6 - idx) * 86_400_000;
    bars.push({
      key,
      label: names[new Date(dateMs).getDay()],
      rxH: Math.max(val.rx > 0 ? 4 : 0, (val.rx / peak) * 100),
      txH: Math.max(val.tx > 0 ? 4 : 0, (val.tx / peak) * 100),
      title: `${key} — ${fmtBytes(val.rx + val.tx)}`,
    });
    idx++;
  }
  return bars;
});

function weekBarsTodayKey(): string {
  const dt = new Date();
  const m = String(dt.getMonth() + 1).padStart(2, '0');
  const d = String(dt.getDate()).padStart(2, '0');
  return `${dt.getFullYear()}-${m}-${d}`;
}

let _weekdayCache: string[] | null = null;
function weekdayNames(): string[] {
  if (_weekdayCache) return _weekdayCache;
  const fmt = new Intl.DateTimeFormat(vpnStore.settings.language === 'ru' ? 'ru' : 'en', {
    weekday: 'short',
  });
  
  _weekdayCache = [0, 1, 2, 3, 4, 5, 6].map((dow) =>
    fmt.format(new Date(2024, 0, 7 + dow)),
  );
  return _weekdayCache;
}

function rowWidth(total: number): number {
  const max = Math.max(...todayRows.value.map((r) => r.total), 1);
  return Math.max(3, (total / max) * 100);
}

const quotaPct = computed(() => {
  const q = vpnStore.trafficQuota;
  if (!q || q.total <= 0) return 100;
  return Math.max(2, Math.min(100, (q.left / q.total) * 100));
});

const quotaTone = computed(() => {
  const q = vpnStore.trafficQuota;
  if (!q || q.total <= 0) return 'ok';
  const ratio = q.left / q.total;
  if (q.left === 0) return 'empty';
  if (ratio <= 0.05) return 'critical';
  if (ratio <= 0.2) return 'low';
  return 'ok';
});

const quotaCaption = computed(() => {
  const q = vpnStore.trafficQuota;
  if (!q) return '';
  return `${fmtBytes(q.used)} / ${fmtBytes(q.total)}`;
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
    <div class="page-content" data-sensitive>
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

      <div class="chart-card rise" style="animation-delay: 190ms">
        <div class="chart-header">
          <h2 class="chart-title" v-text="t('analysis.appStats')"></h2>
          <span class="chart-sub mono" v-text="t('analysis.appSession')"></span>
        </div>
        <div v-if="!appStats.length" class="apps-empty mono" v-text="t('analysis.appEmpty')"></div>
        <div v-else class="apps-list">
          <div v-for="(a, i) in appStats.slice(0, 6)" :key="a.name" class="app-row">
            <span class="app-rank mono" v-text="String(i + 1).padStart(2, '0')"></span>
            <span class="app-name" v-text="a.name"></span>
            <div class="app-bar"><i :style="{ width: barWidth(a) + '%' }"></i></div>
            <span class="app-val mono" v-text="fmtBytes(a.rx + a.tx)"></span>
          </div>
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

      <div class="chart-card rise" style="animation-delay: 270ms">
        <div class="chart-header">
          <h2 class="chart-title" v-text="t('analysis.historyTitle')"></h2>
          <div class="history-summary">
            <span class="foot-pill">
              <span v-text="t('analysis.historyToday')"></span>
              <span class="mono foot-val" v-text="fmtBytes(todayTotal.total)"></span>
            </span>
            <span class="foot-pill">
              <span v-text="t('analysis.historyWeek')"></span>
              <span class="mono foot-val" v-text="fmtBytes(weekTotal.total)"></span>
            </span>
          </div>
        </div>

        
        <div class="hist-chart">
          <div
            v-for="(col, i) in weekBars"
            :key="col.key"
            class="hist-col"
            :title="col.title"
          >
            <div class="hist-stack">
              <i class="hist-bar hist-bar--tx" :style="{ height: col.txH + '%' }"></i>
              <i class="hist-bar hist-bar--rx" :style="{ height: col.rxH + '%' }"></i>
            </div>
            <span class="hist-label mono" :class="{ 'hist-label--today': i === weekBars.length - 1 }" v-text="col.label"></span>
          </div>
        </div>

        
        <div v-if="todayRows.length > 0" class="apps-list hist-rows">
          <div v-for="row in todayRows" :key="row.name" class="app-row">
            <span class="app-name" v-text="row.name"></span>
            <div class="app-bar"><i :style="{ width: rowWidth(row.total) + '%' }"></i></div>
            <span class="app-val mono" v-text="fmtBytes(row.total)"></span>
          </div>
        </div>

        
        <div v-if="vpnStore.trafficQuota" class="quota-block">
          <div class="quota-head">
            <span v-text="t('analysis.quotaLeft')"></span>
            <span class="mono quota-val" :class="quotaTone" v-text="fmtBytes(vpnStore.trafficQuota.left)"></span>
          </div>
          <div class="quota-track">
            <i
              class="quota-fill"
              :class="'quota-fill--' + quotaTone"
              :style="{ width: quotaPct + '%' }"
            ></i>
          </div>
          <p class="quota-sub mono" v-text="quotaCaption"></p>
        </div>
      </div>

      <div class="chart-card rise" style="animation-delay: 310ms">
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

.chart-title { font-size: 13px; font-weight: 500; }\n\n.chart-sub { font-size: 10.5px; letter-spacing: 0.08em; text-transform: uppercase; color: var(--muted-foreground); }

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

.apps-empty {
  padding: 18px 4px;
  font-size: 12px;
  color: var(--muted-foreground);
}

.apps-list {
  display: flex;
  flex-direction: column;
}

.app-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 9px 2px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
}

.app-row:last-child {
  border-bottom: none;
}

.app-rank {
  font-size: 10px;
  color: var(--muted-foreground);
  opacity: 0.7;
}

.app-name {
  width: 180px;
  font-size: 13px;
  color: var(--foreground);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.app-bar {
  flex: 1;
  height: 4px;
  background: rgba(255, 255, 255, 0.06);
  overflow: hidden;
}

.app-bar i {
  display: block;
  height: 100%;
  background: linear-gradient(90deg, var(--success), #7ab8ff);
}

.app-val {
  min-width: 84px;
  text-align: right;
  font-size: 12px;
  color: var(--foreground);
  font-variant-numeric: tabular-nums;
}

.history-summary {
  display: inline-flex;
  gap: 6px;
}

.hist-chart {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 8px;
  height: 108px;
  padding: 12px 14px 8px;
  border-radius: 13px;
  border: 1px solid rgba(255, 255, 255, 0.06);
  background: rgba(0, 0, 0, 0.22);
}

.hist-col {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  flex: 1;
  min-width: 0;
}

.hist-stack {
  display: flex;
  flex-direction: column-reverse;
  align-items: stretch;
  justify-content: flex-start;
  width: 100%;
  max-width: 34px;
  height: 62px;
  border-radius: 6px;
  overflow: hidden;
  gap: 1px;
}

.hist-bar {
  display: block;
  width: 100%;
  transition: height 600ms cubic-bezier(0.22, 0.9, 0.3, 1);
}

.hist-bar--rx {
  background: linear-gradient(180deg, rgba(56, 224, 178, 0.85), rgba(56, 224, 178, 0.5));
}

.hist-bar--tx {
  background: linear-gradient(180deg, rgba(255, 158, 100, 0.75), rgba(255, 158, 100, 0.45));
}

.hist-label {
  font-size: 9.5px;
  color: var(--muted-foreground);
  letter-spacing: 0.02em;
}

.hist-label--today {
  color: #7ab8ff;
}

.hist-rows {
  margin-top: 10px;
}

.quota-block {
  margin-top: 14px;
  padding: 12px 14px;
  border-radius: 13px;
  border: 1px solid rgba(255, 255, 255, 0.07);
  background: rgba(0, 0, 0, 0.18);
}

.quota-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 11.5px;
  color: var(--muted-foreground);
}

.quota-val {
  font-size: 12.5px;
  color: var(--foreground);
  font-variant-numeric: tabular-nums;
}

.quota-track {
  margin-top: 8px;
  height: 7px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.07);
  overflow: hidden;
}

.quota-fill {
  display: block;
  height: 100%;
  border-radius: inherit;
  transition: width 700ms cubic-bezier(0.22, 0.9, 0.3, 1), background 300ms ease;
}

.quota-fill--ok {
  background: linear-gradient(90deg, var(--success), #7ab8ff);
}

.quota-fill--low {
  background: linear-gradient(90deg, oklch(0.78 0.16 80), oklch(0.82 0.16 65));
}

.quota-fill--critical {
  background: linear-gradient(90deg, oklch(0.72 0.17 45), oklch(0.78 0.17 35));
}

.quota-fill--empty {
  background: var(--destructive);
}

.quota-sub {
  margin: 7px 0 0;
  font-size: 10px;
  color: var(--muted-foreground);
}

</style>
