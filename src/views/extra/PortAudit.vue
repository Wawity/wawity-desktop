<template>
  <ExtraPage :title="t('portaudit.title')" :subtitle="t('portaudit.subtitle')">
    <template #actions>
      <button type="button" class="tool-action" :class="{ 'tool-action--active': help }" @click="help = !help">
        <span v-text="t('portaudit.helpBtn')" />
      </button>
      <button
        v-if="!scanning"
        type="button"
        class="tool-action tool-action--primary"
        :disabled="scanning"
        @click="scan"
      >
        <span v-text="scanned ? t('portaudit.again') : t('portaudit.run')" />
      </button>
      <button v-else type="button" class="tool-action" disabled>
        <span v-text="t('portaudit.scanning')" />
      </button>
    </template>

    <div class="glass pa-ip" data-sensitive>
      <div class="pa-ip-row">
        <span class="pa-label" v-text="t('portaudit.yourIp')" />
        <span class="mono pa-ip-value" v-text="ip ?? '…'" />
      </div>
      <div v-if="scanned" class="pa-ip-row">
        <span class="pa-label" v-text="t('portaudit.route')" />
        <span class="mono pa-ip-value" v-text="scannedVia === 'vpn' ? t('portaudit.viaVpn') : t('portaudit.viaDirect')" />
      </div>
      <div class="pa-ip-row">
        <span class="pa-label" v-text="t('portaudit.verdict')" />
        <span
          v-if="scanned"
          class="mono pa-verdict"
          :class="openPorts.length === 0 ? 'pa-good' : 'pa-bad'"
          v-text="openPorts.length === 0 ? t('portaudit.stealth') : t('portaudit.exposed', { n: openPorts.length })"
        />
        <span v-else class="mono pa-verdict" v-text="'—'" />
      </div>
    </div>

    <p v-if="error" class="tool-error" v-text="error" />

    <div v-if="help" class="glass help-block">
      <p v-for="(line, i) in helpLines" :key="i" v-text="line" />
    </div>

    <div v-if="scanned && openPorts.length > 0" class="glass pa-list">
      <div v-for="port in openPorts" :key="port.port + port.service" class="pa-row pa-row--bad">
        <span class="mono pa-port" v-text="port.port + '/tcp'" />
        <span class="pa-service" v-text="port.service" />
      </div>
    </div>

    <div v-else-if="scanned" class="glass pa-list">
      <div class="pa-row">
        <span class="pa-service" v-text="t('portaudit.nothingOpen')" />
      </div>
    </div>

    <p class="tool-hint" v-text="t('portaudit.hint')" />
  </ExtraPage>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import ExtraPage from '../../components/ExtraPage.vue';
import { useVpnStore } from '../../stores/vpn';
import { t } from '../../i18n';

function vpnConnected(): boolean {
  try {
    return useVpnStore().status.connected;
  } catch {
    return false;
  }
}

interface PortEntry {
  port: string;
  service: string;
}

const vpnStore = useVpnStore();
const ip = ref<string | null>(null);
const scanned = ref(false);
const scanning = ref(false);
const error = ref('');
const openPorts = ref<PortEntry[]>([]);
  const help = ref(true);
  const helpLines = computed(() => [1, 2, 3].map((n) => t('portaudit.help' + n)));
  const scannedVia = ref<'' | 'vpn' | 'direct'>('');

async function fetchIp(): Promise<string | null> {
  try {
    const res = await fetch('https://api.ipify.org?format=json', { cache: 'no-store' });
    const json = (await res.json()) as { ip?: string };
    return json.ip ?? null;
  } catch {
    return null;
  }
}

async function scan() {
  if (scanning.value) return;
  scanning.value = true;
  error.value = '';
  scanned.value = false;
  openPorts.value = [];

  try {
    const wasConnected = vpnConnected();
    if (!ip.value) ip.value = await fetchIp();
    scannedVia.value = wasConnected ? 'vpn' : 'direct';
    if (!ip.value) {
      error.value = t('portaudit.noIp');
      scanning.value = false;
      return;
    }

    const res = await fetch('https://api.hackertarget.com/nmap/?q=' + encodeURIComponent(ip.value), {
      cache: 'no-store',
    });
    if (!res.ok) throw new Error('HTTP ' + res.status);
    const text = await res.text();

    if (/API count exceeded|API rate limit/i.test(text)) {
      error.value = t('portaudit.rateLimited');
      scanned.value = true;
      return;
    }

    const ports: PortEntry[] = [];
    for (const line of text.split('\n')) {
      const m = line.match(/^(\d+)\/tcp\s+open\s+(.+)$/i);
      if (m) ports.push({ port: m[1], service: m[2].trim() });
    }
    openPorts.value = ports;
    scanned.value = true;
  } catch (e) {
    error.value = String(e);
  } finally {
    scanning.value = false;
  }
}
</script>

<style scoped>
.pa-ip {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 14px 18px;
  margin-bottom: 14px;
}

.pa-ip-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}

.pa-label {
  font-size: 11.5px;
  color: var(--muted-foreground);
}

.pa-ip-value {
  font-size: 14px;
  color: var(--foreground);
  font-variant-numeric: tabular-nums;
}

.pa-verdict {
  font-size: 12.5px;
  font-weight: 600;
}

.pa-good { color: var(--success); }
.pa-bad { color: var(--destructive); }

.pa-list {
  padding: 4px 0;
  margin-bottom: 14px;
}

.pa-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 9px 18px;
}

.pa-row--bad .pa-port { color: var(--destructive); font-weight: 600; }

.pa-port {
  min-width: 84px;
  font-size: 12.5px;
}

.pa-service {
  font-size: 12px;
  color: var(--muted-foreground);
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
