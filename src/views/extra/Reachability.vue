<template>
  <ExtraPage :title="t('reachability.title')" :subtitle="t('reachability.subtitle')">
    <template #actions>
      <button
        type="button"
        class="tool-action"
        :class="{ 'tool-action--active': onlyBad }"
        @click="onlyBad = !onlyBad"
      >
        <span v-text="t('reachability.onlyBlocked')" />
      </button>
      <button v-if="!busy" type="button" class="tool-action tool-action--primary" @click="run">
        <span v-text="rows.length ? t('reachability.rescan') : t('reachability.rescan')" />
      </button>
      <button v-else type="button" class="tool-action" disabled>
        <span v-text="t('reachability.scanning')" />
      </button>
    </template>

    <p v-if="failed" class="tool-error" v-text="failed" />

    <div class="rc-meta mono">
      <span><b>{{ okCount }}</b> {{ t('reachability.open') }}</span>
      <span class="rc-bad"><b>{{ blockedCount }}</b> {{ t('reachability.blocked') }}</span>
      <span><b>{{ availability }}%</b> {{ t('reachability.availability') }}</span>
      <span><b>{{ medianMs || '—' }}</b> ms</span>
    </div>

    <section
      v-for="group in grouped"
      :key="group.key"
      class="glass rc-group"
    >
      <header class="rc-group-head">
        <span class="rc-group-name mono" v-text="t('reachability.cat.' + group.key)"></span>
        <span class="rc-group-count mono">{{ group.rows.length }}</span>
      </header>

      <div
        v-for="row in group.rows"
        :key="row.domain"
        class="rc-row"
        :class="{ 'rc-row--bad': row.blocked }"
      >
        <ServiceMark :domain="row.domain" :tint="tintOf(row)" :size="20" />
        <span class="rc-name" v-text="row.label || row.domain"></span>
        <span class="rc-domain mono" v-text="row.domain"></span>
        <span class="rc-state mono">
          {{ row.blocked ? t('reachability.blocked') : row.elapsedMs + ' ms' }}
        </span>
      </div>
    </section>
  </ExtraPage>
</template><script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import { Filter, Radar, RefreshCw, Timer } from '../../lib/appIcons';
import ExtraPage from '../../components/ExtraPage.vue';
import ServiceMark from '../../components/ServiceMark.vue';
import { useTilt } from '../../lib/useTilt';
import { t } from '../../i18n';
import type { BlockReport } from '../../types/vpn';

const ORDER = ['media', 'messaging', 'social', 'ai', 'dev', 'gaming', 'privacy', 'infra', 'other'];

const CATEGORY_TINT: Record<string, string> = {
  media: '#ff7a90',
  messaging: '#8fb6ff',
  social: '#c084fc',
  ai: '#5ee6c8',
  dev: '#aeb9c8',
  gaming: '#ffb46b',
  privacy: '#5ee69a',
  infra: '#7fd7ff',
  other: '#9aa6b6',
};

const BRAND_TINT: Record<string, string> = {
  'youtube.com': '#ff5c5c',
  'googlevideo.com': '#ff7a7a',
  'netflix.com': '#e5484d',
  'spotify.com': '#4ade80',
  'twitch.tv': '#a970ff',
  'soundcloud.com': '#ff8a3d',
  'vimeo.com': '#5fd0ff',
  'rutracker.org': '#8fb6ff',
  'discord.com': '#8b9cff',
  'gateway.discord.gg': '#8b9cff',
  'telegram.org': '#3fb6f0',
  'web.telegram.org': '#3fb6f0',
  'signal.org': '#6f8bff',
  'whatsapp.com': '#4ee082',
  'viber.com': '#b48bff',
  'instagram.com': '#e56cc6',
  'facebook.com': '#5b8def',
  'x.com': '#e6ebf2',
  'twitter.com': '#5fc4ff',
  'linkedin.com': '#4aa3e0',
  'reddit.com': '#ff8a54',
  'tiktok.com': '#4fe3d2',
  'pinterest.com': '#f0576b',
  'chatgpt.com': '#5ee6c8',
  'openai.com': '#7fe8cf',
  'claude.ai': '#ff9f6b',
  'gemini.google.com': '#8fb6ff',
  'perplexity.ai': '#4fd6d0',
  'huggingface.co': '#ffd15c',
  'github.com': '#c9d1e0',
  'gitlab.com': '#ff8a54',
  'npmjs.com': '#ff6a6a',
  'crates.io': '#e0b25c',
  'docker.com': '#5fb6ff',
  'stackoverflow.com': '#ff9f6b',
  'steamcommunity.com': '#8fb6ff',
  'epicgames.com': '#d5dbe6',
  'riotgames.com': '#ff5c5c',
  'battle.net': '#5fa8ff',
  'proton.me': '#a78bfa',
  'torproject.org': '#b48bff',
  'mullvad.net': '#ffd15c',
  'cloudflare.com': '#ff9f4d',
  'medium.com': '#d5dbe6',
  'patreon.com': '#ff6a5c',
  'speakerdeck.com': '#5fb6ff',
};

const rows = ref<BlockReport[]>([]);
const busy = ref(false);
const failed = ref('');
const onlyBad = ref(false);

const { tiltStyle, handleMove, handleLeave } = useTilt(6);

const okCount = computed(() => rows.value.filter((row) => !row.blocked).length);
const blockedCount = computed(() => rows.value.filter((row) => row.blocked).length);

const availability = computed(() => {
  if (!rows.value.length) return 0;
  return Math.round((okCount.value / rows.value.length) * 100);
});

const medianMs = computed(() => {
  const live = rows.value
    .filter((row) => !row.blocked)
    .map((row) => row.elapsedMs)
    .sort((a, b) => a - b);
  if (!live.length) return 0;
  return live[Math.floor(live.length / 2)];
});

const radarDots = computed(() => {
  const pool = rows.value.length ? rows.value.slice(0, 9) : Array.from({ length: 6 });
  return pool.map((entry, index) => {
    const report = entry as BlockReport | undefined;
    const angle = (index / Math.max(pool.length, 1)) * Math.PI * 2 + index * 0.55;
    const radius = 20 + ((index * 37) % 30);
    const tone = !report ? 'idle' : report.blocked ? 'bad' : 'good';
    return {
      key: report ? report.domain : 'seed-' + index,
      x: 50 + Math.cos(angle) * radius,
      y: 50 + Math.sin(angle) * radius,
      tone,
      delay: ((index * 140) % 900) + 'ms',
    };
  });
});

const grouped = computed(() => {
  const pool = onlyBad.value ? rows.value.filter((row) => row.blocked) : rows.value;
  const bag = new Map<string, BlockReport[]>();
  for (const row of pool) {
    const key = row.category || 'other';
    const found = bag.get(key);
    if (found) found.push(row);
    else bag.set(key, [row]);
  }
  return ORDER.filter((key) => bag.has(key)).map((key) => ({
    key,
    rows: bag
      .get(key)!
      .slice()
      .sort((a, b) => Number(b.blocked) - Number(a.blocked) || a.elapsedMs - b.elapsedMs),
  }));
});

function groupIndex(key: string) {
  return ORDER.indexOf(key) + 1;
}

function toneOf(row: BlockReport) {
  if (!row.blocked) return 'good';
  if (row.verdict === 'unresolved') return 'warn';
  return 'bad';
}

function tintOf(row: BlockReport) {
  return BRAND_TINT[row.domain] || CATEGORY_TINT[row.category] || '#9aa6b6';
}

async function run() {
  if (busy.value) return;
  busy.value = true;
  failed.value = '';
  try {
    rows.value = await invoke<BlockReport[]>('probe_reachability', { targets: null });
  } catch (error) {
    failed.value = String(error);
  } finally {
    busy.value = false;
  }
}

onMounted(run);
</script>

<style scoped>
.rc-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 0;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 999px;
  width: max-content;
  max-width: 100%;
  margin-bottom: 18px;
  font-size: 11px;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: var(--muted-foreground);
}

.rc-meta span {
  padding: 9px 18px;
  border-right: 1px solid rgba(255, 255, 255, 0.08);
}

.rc-meta span:last-child {
  border-right: none;
}

.rc-meta b {
  color: var(--foreground);
  font-weight: 600;
}

.rc-bad b {
  color: var(--destructive);
}

.rc-group {
  margin-bottom: 14px;
  overflow: hidden;
}

.rc-group-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 13px 18px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
}

.rc-group-name {
  font-size: 10px;
  letter-spacing: 0.2em;
  text-transform: uppercase;
  color: var(--muted-foreground);
}

.rc-group-count {
  font-size: 10px;
  color: var(--muted-foreground);
}

.rc-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 18px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
}

.rc-row:last-child {
  border-bottom: none;
}

.rc-row--bad .rc-name,
.rc-row--bad .rc-domain {
  color: var(--destructive);
  opacity: 0.75;
}

.rc-name {
  font-size: 13.5px;
  color: var(--foreground);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.rc-domain {
  font-size: 11px;
  color: var(--muted-foreground);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  opacity: 0.7;
}

.rc-state {
  margin-left: auto;
  flex-shrink: 0;
  font-size: 11.5px;
  color: var(--success);
  font-variant-numeric: tabular-nums;
}

.rc-row--bad .rc-state {
  color: var(--destructive);
}
</style>