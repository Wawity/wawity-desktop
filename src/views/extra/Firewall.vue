<template>
  <ExtraPage :title="t('firewall.title')" :subtitle="t('firewall.subtitle')">
    <template #actions>
      <button type="button" class="tool-action" :class="{ 'tool-action--active': help }" @click="help = !help">
        <span v-text="t('firewall.helpBtn')" />
      </button>
      <button
        v-if="!loading"
        type="button"
        class="tool-action"
        @click="refresh"
      >
        <span v-text="t('firewall.refresh')" />
      </button>
      <button
        type="button"
        class="tool-action tool-action--primary"
        :disabled="repairing"
        @click="repair"
      >
        <span v-text="repairing ? t('firewall.repairing') : t('firewall.repair')" />
      </button>
    </template>

    <p v-if="error" class="tool-error" v-text="error" />

    <div v-if="help" class="glass help-block">
      <p v-for="(line, i) in helpLines" :key="i" v-text="line" />
    </div>

    <div v-if="!loading && rules.length === 0" class="glass fw-empty">
      <p v-text="t('firewall.empty')" />
    </div>

    <div v-else class="glass fw-list">
      <div v-for="rule in rules" :key="rule.name" class="fw-row">
        <span class="fw-dot" :class="rule.enabled ? 'fw-dot--on' : 'fw-dot--off'" />
        <span class="fw-name mono" v-text="rule.name" />
        <span class="fw-dir mono" v-text="dirLabel(rule.direction)" />
        <span class="fw-profiles mono" :title="t('firewall.profilesLabel')" v-text="rule.profiles" />
        <span
          class="fw-action mono"
          :class="rule.action === 'Allow' ? 'fw-allow' : 'fw-block'"
          v-text="actionLabel(rule.action)"
        />
      </div>
    </div>

    <p class="tool-hint" v-text="t('firewall.hint')" />
  </ExtraPage>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import { useNotifications } from '../../composables/useNotifications';
import ExtraPage from '../../components/ExtraPage.vue';
import { t } from '../../i18n';

interface FirewallRule {
  name: string;
  enabled: boolean;
  direction: string;
  action: string;
  profiles: string;
}

const { pushToast } = useNotifications();
const rules = ref<FirewallRule[]>([]);
const loading = ref(true);
const repairing = ref(false);
const error = ref('');
  const help = ref(true);
  const helpLines = computed(() => [1, 2, 3].map((n) => t('firewall.help' + n)));

async function refresh() {
  loading.value = true;
  error.value = '';
  try {
    rules.value = await invoke<FirewallRule[]>('firewall_wawity_rules');
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

async function repair() {
  if (repairing.value) return;
  repairing.value = true;
  try {
    await invoke('repair_network');
    pushToast('success', t('firewall.repaired'), '', 4000);
    await refresh();
  } catch (e) {
    pushToast('error', t('firewall.repairFail'), String(e), 6000);
  } finally {
    repairing.value = false;
  }
}

function dirLabel(d: string): string {
  return /in/i.test(d) ? 'IN' : /out/i.test(d) ? 'OUT' : d.toUpperCase();
}

function actionLabel(a: string): string {
  return /allow/i.test(a) ? 'ALLOW' : 'BLOCK';
}

onMounted(refresh);
</script>

<style scoped>
.fw-list {
  padding: 4px 0;
}

.fw-empty {
  padding: 20px;
  font-size: 12.5px;
  color: var(--muted-foreground);
  text-align: center;
}

.fw-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 18px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
}

.fw-row:last-child { border-bottom: none; }

.fw-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}
.fw-dot--on { background: var(--success); }
.fw-dot--off { background: var(--muted-foreground); opacity: 0.4; }

.fw-name {
  flex: 1;
  min-width: 0;
  font-size: 11.5px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.fw-dir, .fw-action {
  font-size: 10px;
  letter-spacing: 0.05em;
  color: var(--muted-foreground);
}

.fw-allow { color: var(--success); }
.fw-block { color: var(--destructive); }

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
