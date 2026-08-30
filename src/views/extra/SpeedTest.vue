<template>
  <ExtraPage :title="t('speedtest.title')" :subtitle="t('speedtest.subtitle')">
    <template #actions>
      <button v-if="!busy" type="button" class="tool-action tool-action--primary" @click="start">
        <span v-text="done ? t('speedtest.again') : t('speedtest.start')" />
      </button>
      <button v-else type="button" class="tool-action" @click="stop">
        <span v-text="t('speedtest.stop')" />
      </button>
    </template>

    <p v-if="failed" class="tool-error" v-text="failed" />

    <section class="glass st-hero">
      <span class="st-phase mono" v-text="phaseText"></span>
      <div class="st-value">
        <span class="st-num mono">{{ shown.toFixed(shown >= 100 ? 0 : 1) }}</span>
        <span class="st-unit mono">Mbps</span>
      </div>
      <div class="st-bar"><i :style="{ width: frac * 100 + '%' }"></i></div>
    </section>

    <div class="st-grid">
      <section class="glass st-channel">
        <span class="st-label mono">↓</span>
        <span class="st-num2 mono">{{ fmt(downloadShown) }}</span>
        <span class="st-cap mono">Mbps</span>
      </section>
      <section class="glass st-channel">
        <span class="st-label mono">↑</span>
        <span class="st-num2 mono">{{ fmt(uploadShown) }}</span>
        <span class="st-cap mono">Mbps</span>
      </section>
    </div>
  </ExtraPage>
</template><script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import type { UnlistenFn } from '@tauri-apps/api/event';
import {
  Activity,
  AlertTriangle,
  ArrowDown,
  ArrowUp,
  Gauge,
  MapPin,
  Play,
  Square,
  Waves,
} from '../../lib/appIcons';
import ExtraPage from '../../components/ExtraPage.vue';
import { useTilt } from '../../lib/useTilt';
import { t } from '../../i18n';
import type { SpeedResult, SpeedTick } from '../../types/vpn';

const FULL = 2 * Math.PI * 100;
const SWEEP = FULL * 0.75;
const CEILING = 1000;

const marks = [{ at: 0 }, { at: 0.25 }, { at: 0.5 }, { at: 0.75 }, { at: 1 }];

const blank: SpeedResult = {
  downloadMbps: 0,
  uploadMbps: 0,
  pingMs: 0,
  jitterMs: 0,
  loss: 0,
  colo: '',
  exitIp: '',
  carrier: '',
  country: '',
  downBytes: 0,
  upBytes: 0,
  tookMs: 0,
  aborted: false,
};

const busy = ref(false);
const done = ref(false);
const failed = ref('');
const phase = ref('idle');
const live = ref(0);
const shown = ref(0);
const history = ref<number[]>([]);
const result = ref<SpeedResult>({ ...blank });

const { tiltStyle, handleMove, handleLeave } = useTilt(6);

let drop: UnlistenFn | null = null;
let raf = 0;

function curve(value: number) {
  const capped = Math.max(0, Math.min(CEILING, value));
  return Math.log10(1 + capped) / Math.log10(1 + CEILING);
}

const frac = computed(() => curve(shown.value));
const downloadShown = computed(() =>
  phase.value === 'download' ? shown.value : result.value.downloadMbps,
);
const uploadShown = computed(() =>
  phase.value === 'upload' ? shown.value : result.value.uploadMbps,
);

const phaseText = computed(() => {
  if (phase.value === 'idle') return t('speedtest.idle');
  return t('speedtest.phase.' + phase.value);
});

const tracePath = computed(() => {
  const pool = history.value;
  if (pool.length < 2) return '';
  const peak = Math.max(...pool, 1);
  const step = 600 / (pool.length - 1);
  return pool
    .map((value, index) => `${(index * step).toFixed(1)},${(88 - (value / peak) * 78).toFixed(1)}`)
    .join(' ');
});

const traceFill = computed(() => (tracePath.value ? `0,90 ${tracePath.value} 600,90` : ''));

function fmt(value: number) {
  if (!value) return '0';
  return value >= 100 ? value.toFixed(0) : value.toFixed(1);
}

function glide() {
  const gap = live.value - shown.value;
  shown.value += gap * 0.18;
  if (Math.abs(gap) < 0.05) shown.value = live.value;
  raf = requestAnimationFrame(glide);
}

async function start() {
  if (busy.value) return;
  busy.value = true;
  done.value = false;
  failed.value = '';
  phase.value = 'meta';
  live.value = 0;
  shown.value = 0;
  history.value = [];
  result.value = { ...blank };

  try {
    const outcome = await invoke<SpeedResult>('run_speed_test');
    result.value = outcome;
    live.value = outcome.downloadMbps;
    done.value = true;
    phase.value = 'done';
  } catch (error) {
    failed.value = String(error);
    phase.value = 'idle';
  } finally {
    busy.value = false;
  }
}

async function stop() {
  await invoke('cancel_speed_test').catch(() => {});
}

onMounted(async () => {
  raf = requestAnimationFrame(glide);
  drop = await listen<SpeedTick>('wawity-speed-tick', (event) => {
    const tick = event.payload;
    const phaseChanged = phase.value !== tick.phase;
    phase.value = tick.phase;
    if (tick.phase !== 'download' && tick.phase !== 'upload') return;
    if (phaseChanged) {
      history.value = [];
      live.value = 0;
      shown.value = 0;
    }
    live.value = tick.mbps;
    history.value.push(tick.mbps);
    if (history.value.length > 90) history.value.shift();
  });
});

onUnmounted(() => {
  cancelAnimationFrame(raf);
  if (drop) drop();
  if (busy.value) invoke('cancel_speed_test').catch(() => {});
});
</script>

<style scoped>
.st-hero {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 18px;
  padding: 44px 28px 40px;
}

.st-phase {
  font-size: 10px;
  letter-spacing: 0.22em;
  text-transform: uppercase;
  color: var(--muted-foreground);
  border-bottom: 1px solid rgba(255, 255, 255, 0.16);
  padding: 0 6px 8px;
}

.st-value {
  display: flex;
  align-items: baseline;
  gap: 12px;
}

.st-num {
  font-size: clamp(56px, 9vw, 84px);
  font-weight: 300;
  letter-spacing: -0.04em;
  line-height: 1;
  color: var(--foreground);
  font-variant-numeric: tabular-nums;
}

.st-unit {
  font-size: 13px;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--muted-foreground);
}

.st-bar {
  width: min(420px, 100%);
  height: 2px;
  background: rgba(255, 255, 255, 0.08);
  overflow: hidden;
}

.st-bar i {
  display: block;
  height: 100%;
  background: var(--success);
  transition: width 240ms cubic-bezier(0.22, 0.61, 0.36, 1);
}

.st-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 14px;
  margin-top: 14px;
}

.st-channel {
  display: flex;
  align-items: baseline;
  gap: 14px;
  padding: 20px 24px;
}

.st-label {
  color: var(--muted-foreground);
  font-size: 15px;
}

.st-num2 {
  font-size: 30px;
  font-weight: 400;
  letter-spacing: -0.02em;
  color: var(--foreground);
  font-variant-numeric: tabular-nums;
  margin-left: auto;
}

.st-cap {
  font-size: 10px;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--muted-foreground);
}

@media (max-width: 720px) {
  .st-grid {
    grid-template-columns: 1fr;
  }
}
</style>