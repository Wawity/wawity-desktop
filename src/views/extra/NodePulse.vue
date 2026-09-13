<template>
  <ExtraPage :title="t('pulse.title')" :subtitle="t('pulse.subtitle')">
    <template #actions>
      <button type="button" class="tool-action" :class="{ 'tool-action--active': help }" @click="help = !help">
        <span v-text="t('pulse.helpBtn')" />
      </button>
      <button
        v-if="!busy"
        type="button"
        class="tool-action tool-action--primary"
        :disabled="vpnStore.allServers.length === 0"
        @click="run"
      >
        <span v-text="rows.length ? t('pulse.again') : t('pulse.run')" />
      </button>
      <button v-else type="button" class="tool-action" disabled>
        <span v-text="t('pulse.running')" />
      </button>
    </template>

    <p v-if="vpnStore.allServers.length === 0" class="tool-empty" v-text="t('pulse.noServers')" />

    <template v-else>
      <div v-if="help" class="glass help-block">
        <p v-for="(line, i) in helpLines" :key="i" v-text="line" />
      </div>

      <div v-if="rows.length" class="rc-meta mono">
        <span><b class="ok">{{ alive }}</b> {{ t('pulse.alive') }}</span>
        <span><b class="warn">{{ degraded }}</b> {{ t('pulse.degraded') }}</span>
        <span><b class="dead">{{ dead }}</b> {{ t('pulse.dead') }}</span>
      </div>

      <div v-if="rows.length" class="glass pulse-list">
        <div
          v-for="row in rows"
          :key="row.id"
          class="pulse-row"
          :class="['pulse-row--' + row.grade, { 'pulse-row--hidden': isHidden(row.id) }]"
        >
          <span class="pulse-dot" :class="'pulse-dot--' + row.grade" />
          <span class="pulse-name" v-text="row.name" />
          <span class="pulse-ms mono" v-text="row.msText" />
          <button
            type="button"
            class="tool-action pulse-hide"
            :class="{ 'tool-action--active': isHidden(row.id) }"
            @click="vpnStore.toggleHideServer(row.id)"
          >
            <span v-text="isHidden(row.id) ? t('pulse.unhide') : t('pulse.hide')" />
          </button>
        </div>
      </div>

      <p v-else-if="!busy" class="tool-empty" v-text="t('pulse.empty')" />

      <p class="tool-hint" v-text="t('pulse.hint')" />
    </template>
  </ExtraPage>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import { useVpnStore } from '../../stores/vpn';
import ExtraPage from '../../components/ExtraPage.vue';
import { t } from '../../i18n';

const vpnStore = useVpnStore();

interface PulseRow {
  id: string;
  name: string;
  ms: number | null;
  msText: string;
  grade: 'alive' | 'degraded' | 'dead';
}

const rows = ref<PulseRow[]>([]);
const busy = ref(false);
  const help = ref(true);
  const helpLines = computed(() => [1, 2, 3].map((n) => t('pulse.help' + n)));

const alive = computed(() => rows.value.filter((r) => r.grade === 'alive').length);
const degraded = computed(() => rows.value.filter((r) => r.grade === 'degraded').length);
const dead = computed(() => rows.value.filter((r) => r.grade === 'dead').length);

function isHidden(id: string): boolean {
  return vpnStore.hiddenServers.includes(id);
}

async function run() {
  if (busy.value) return;
  busy.value = true;
  try {
    const pool = vpnStore.allServers.slice(0, 120);
    const targets = pool.map((s) => ({ host: s.server, port: parsePort(s.url) }));
    const results = await invoke<{ host: string; port: number; latency_ms: number | null }[]>(
      'ping_servers',
      { targets },
    );
    const byHost = new Map<string, number | null>();
    for (const r of results) byHost.set(r.host, r.latency_ms);

    rows.value = pool.map((s) => {
      const ms = byHost.get(s.server) ?? null;
      const grade: PulseRow['grade'] =
        ms === null ? 'dead' : ms >= 800 ? 'degraded' : 'alive';
      return {
        id: s.id,
        name: s.name,
        ms,
        msText: ms === null ? '—' : ms + ' ms',
        grade,
      };
    });
  } finally {
    busy.value = false;
  }
}

function parsePort(url: string): number | null {
  try {
    const parsed = new URL(url);
    const port = Number(parsed.port);
    return Number.isFinite(port) && port > 0 ? port : null;
  } catch {
    return null;
  }
}

onMounted(() => {
  if (rows.value.length === 0 && vpnStore.allServers.length > 0) void run();
});
</script>

<style scoped>
.rc-meta {
  display: flex;
  gap: 16px;
  font-size: 12px;
  color: var(--muted-foreground);
}
.rc-meta b { color: var(--foreground); }
.rc-meta .ok { color: var(--success); }
.rc-meta .warn { color: oklch(0.8 0.15 80); }
.rc-meta .dead { color: var(--destructive); }

.pulse-list {
  padding: 4px 0;
  max-height: 460px;
  overflow-y: auto;
}

.pulse-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 9px 18px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
}

.pulse-row:last-child { border-bottom: none; }

.pulse-row--hidden-later { opacity: 0.55; }

.pulse-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}
.pulse-dot--alive { background: var(--success); box-shadow: 0 0 8px color-mix(in oklab, var(--success) 60%, transparent); }
.pulse-dot--degraded { background: oklch(0.8 0.15 80); }
.pulse-dot--dead { background: var(--destructive); opacity: 0.6; }

.pulse-name {
  flex: 1;
  min-width: 0;
  font-size: 12.5px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.pulse-row--dead .pulse-name { opacity: 0.5; text-decoration: line-through; }
.pulse-row--degraded .pulse-name { color: oklch(0.85 0.12 80); }

.pulse-ms {
  font-size: 11.5px;
  color: var(--muted-foreground);
  font-variant-numeric: tabular-nums;
  min-width: 64px;
  text-align: right;
}

.pulse-row--hidden .pulse-name,
.pulse-row--hidden .pulse-ms,
.pulse-row--hidden .pulse-host {
  opacity: 0.4;
  text-decoration: line-through;
}

.pulse-row--hidden .pulse-dot {
  opacity: 0.35;
}

.pulse-hide {
  flex-shrink: 0;
  padding: 4px 10px;
  font-size: 10.5px;
}

.tool-hint {
  font-size: 11px;
  line-height: 1.5;
  color: var(--muted-foreground);
}

/* ---------- shared extra-tool styles ---------- */
.tool-action {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 7px 14px;
  border-radius: 10px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  background: rgba(255, 255, 255, 0.05);
  color: var(--foreground);
  font-size: 12px;
  cursor: pointer;
  transition:
    background 160ms ease,
    border-color 160ms ease,
    color 160ms ease;
}

.tool-action:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.1);
  border-color: rgba(255, 255, 255, 0.2);
}

.tool-action:disabled {
  opacity: 0.5;
  cursor: default;
}

.tool-action--primary {
  border-color: rgba(167, 139, 250, 0.45);
  background: linear-gradient(180deg, rgba(167, 139, 250, 0.3), rgba(124, 92, 255, 0.18));
  color: #efe9ff;
}

.tool-action--primary:hover:not(:disabled) {
  background: linear-gradient(180deg, rgba(167, 139, 250, 0.42), rgba(124, 92, 255, 0.26));
}

.tool-action--active {
  border-color: rgba(167, 139, 250, 0.5);
  background: rgba(167, 139, 250, 0.14);
  color: #d9ccff;
}

.tool-error {
  margin: 0 0 12px;
  padding: 10px 14px;
  border-radius: 10px;
  border: 1px solid rgba(255, 108, 120, 0.3);
  background: rgba(255, 108, 120, 0.08);
  color: #ff9aa2;
  font-size: 12px;
}

.tool-hint {
  margin: 14px 0 0;
  font-size: 11px;
  line-height: 1.5;
  color: var(--muted-foreground);
}

.rc-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 14px;
  margin-bottom: 12px;
  font-size: 12px;
  color: var(--muted-foreground);
}

.rc-meta b {
  color: var(--foreground);
}

.help-block {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 14px;
  padding: 12px 16px;
  border-radius: 12px;
  border: 1px solid rgba(167, 139, 250, 0.25);
  background: rgba(124, 92, 255, 0.07);
}

.help-block p {
  margin: 0;
  font-size: 12px;
  line-height: 1.55;
  color: rgba(235, 238, 250, 0.78);
}
</style>
