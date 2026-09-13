<template>
  <ExtraPage :title="t('dnsbench.title')" :subtitle="t('dnsbench.subtitle')">
    <template #actions>
      <button type="button" class="tool-action" :class="{ 'tool-action--active': help }" @click="help = !help">
        <span v-text="t('dnsbench.helpBtn')" />
      </button>
      <button
        v-if="!busy"
        type="button"
        class="tool-action tool-action--primary"
        @click="run"
      >
        <span v-text="rows.length ? t('dnsbench.again') : t('dnsbench.run')" />
      </button>
      <button v-else type="button" class="tool-action" disabled>
        <span v-text="t('dnsbench.running')" />
      </button>
    </template>

    <p v-if="error" class="tool-error" v-text="error" />

    <div v-if="help" class="glass help-block">
      <p v-for="(line, i) in helpLines" :key="i" v-text="line" />
    </div>

    <div v-if="rows.length" class="glass bench-list">
      <div
        v-for="(row, i) in rows"
        :key="row.key"
        class="bench-row"
        :class="{ 'bench-row--best': i === 0 && row.ms !== null }"
      >
        <span class="bench-pos mono" v-text="String(i + 1).padStart(2, '0')" />
        <span class="bench-name" v-text="row.name" />
        <span class="bench-host mono" v-text="row.host" />
        <span class="bench-ok mono" :title="t('dnsbench.attempts')" v-text="row.attempts" />
        <span class="bench-ms mono" :class="msTone(row.ms)" v-text="msText(row.ms)" />
        <button
          v-if="row.preset && i === 0 && row.ms !== null && !appliedKey"
          type="button"
          class="tool-action tool-action--primary bench-apply"
          :disabled="busy"
          @click="apply(row)"
        >
          <span v-text="t('dnsbench.apply')" />
        </button>
        <span v-if="appliedKey === row.key" class="bench-applied" v-text="t('dnsbench.applied')" />
      </div>
    </div>

    <p v-else-if="!busy && !error" class="tool-empty" v-text="t('dnsbench.empty')" />

    <p class="tool-hint" v-text="t('dnsbench.hint')" />
  </ExtraPage>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { useVpnStore } from '../../stores/vpn';
import ExtraPage from '../../components/ExtraPage.vue';
import { t } from '../../i18n';

const vpnStore = useVpnStore();

interface BenchRow {
  key: string;
  name: string;
  host: string;
  probe: string;
  mode: 'json' | 'wire';
  preset: string | null;
  ms: number | null;
  attempts: string;
}

// mode 'json' — provider JSON API (accept: application/dns-json);
// mode 'wire' — RFC 8484 DoH (GET ?dns=<base64url>), for providers without
// a JSON endpoint. Wire queries use a distinct DNS id per attempt so CDNs
// cannot serve a cached response.
const PROVIDERS: BenchRow[] = [
  { key: 'cloudflare', name: 'Cloudflare', host: '1.1.1.1', probe: 'https://1.1.1.1/dns-query?name=example.com&type=A', mode: 'json', preset: 'cloudflare', ms: null },
  { key: 'google', name: 'Google', host: '8.8.8.8', probe: 'https://dns.google/resolve?name=example.com&type=A', mode: 'json', preset: 'google', ms: null },
  { key: 'quad9', name: 'Quad9', host: '9.9.9.9', probe: 'https://dns.quad9.net:5053/dns-query?name=example.com&type=A', mode: 'json', preset: 'quad9', ms: null },
  { key: 'adguard', name: 'AdGuard', host: '94.140.14.14', probe: 'https://dns.adguard-dns.com/resolve?name=example.com&type=A', mode: 'json', preset: 'adguard', ms: null },
  { key: 'nextdns', name: 'NextDNS', host: 'dns.nextdns.io', probe: 'https://dns.nextdns.io/dns-query', mode: 'wire', preset: 'dns_sby', ms: null },
  { key: 'dnssb', name: 'DNS.SB', host: 'dns.sb', probe: 'https://dns.sb/dns-query', mode: 'wire', preset: 'dns_sbi', ms: null },
  { key: 'mullvad', name: 'Mullvad', host: 'dns.mullvad.net', probe: 'https://dns.mullvad.net/dns-query', mode: 'wire', preset: 'mullvad', ms: null },
  { key: 'digitale', name: 'Digitale Gesellschaft', host: 'dns.digitale-gesellschaft.ch', probe: 'https://dns.digitale-gesellschaft.ch/dns-query', mode: 'wire', preset: 'digitale', ms: null },
  { key: 'dns0', name: 'dns0.eu', host: 'dns0.eu', probe: 'https://dns0.eu/dns-query', mode: 'wire', preset: null, ms: null },
  { key: 'opendns', name: 'OpenDNS', host: 'doh.opendns.com', probe: 'https://doh.opendns.com/dns-query', mode: 'wire', preset: null, ms: null },
  { key: 'controld', name: 'ControlD', host: 'freedns.controld.com', probe: 'https://freedns.controld.com/dns-query', mode: 'wire', preset: null, ms: null },
  { key: 'comss', name: 'Comss', host: 'dns.comss.one', probe: 'https://dns.comss.one/dns-query', mode: 'wire', preset: null, ms: null },
];

let wireId = 0;

// Minimal A-record query for example.com (RFC 8484 wire format).
function encodeWireQuery(): string {
  const bytes = new Uint8Array(29);
  const view = new DataView(bytes.buffer);
  view.setUint16(0, ++wireId & 0xffff); // id — varies per attempt
  view.setUint16(2, 0x0100); // RD
  view.setUint16(4, 1); // QDCOUNT
  let off = 12;
  for (const label of ['example', 'com']) {
    bytes[off++] = label.length;
    for (let i = 0; i < label.length; i++) bytes[off++] = label.charCodeAt(i);
  }
  bytes[off++] = 0;
  view.setUint16(off, 1); off += 2; // QTYPE = A
  view.setUint16(off, 1); // QCLASS = IN
  let bin = '';
  for (const b of bytes) bin += String.fromCharCode(b);
  return btoa(bin).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

const customHost = computed(() => {
  const raw = vpnStore.settings.dns_custom_doh?.trim();
  if (!raw) return null;
  try {
    return new URL(raw).host;
  } catch {
    return null;
  }
});

const rows = ref<BenchRow[]>([]);
const busy = ref(false);
const error = ref('');
const appliedKey = ref('');
  const help = ref(true);
  const helpLines = computed(() => [1, 2, 3].map((n) => t('dnsbench.help' + n)));

const activeKey = computed(() => {
  const s = vpnStore.settings;
  return (s.dns_custom_doh ?? '').trim() ? 'custom' : s.dns_remote ?? 'cloudflare';
});

async function timeOnce(row: BenchRow): Promise<number | null> {
  const ctrl = new AbortController();
  const t = setTimeout(() => ctrl.abort(), 3000);
  const started = performance.now();
  try {
    if (row.mode === 'wire') {
      await fetch(row.probe + '?dns=' + encodeWireQuery(), {
        signal: ctrl.signal,
        headers: { accept: 'application/dns-message' },
        cache: 'no-store',
      });
    } else {
      await fetch(row.probe, {
        signal: ctrl.signal,
        headers: { accept: 'application/dns-json' },
        cache: 'no-store',
      });
    }
    return Math.round(performance.now() - started);
  } catch {
    return null;
  } finally {
    clearTimeout(t);
  }
}

function median(values: number[]): number {
  const sorted = [...values].sort((a, b) => a - b);
  return sorted[Math.floor(sorted.length / 2)];
}

async function run() {
  if (busy.value) return;
  busy.value = true;
  error.value = '';
  appliedKey.value = '';

  const list: BenchRow[] = PROVIDERS.map((p) => ({ ...p, ms: null }));
  if (customHost.value) {
    list.push({
      key: 'custom',
      name: t('dnsbench.custom'),
      host: customHost.value,
      probe: (vpnStore.settings.dns_custom_doh || '').trim() + '?name=example.com&type=A',
      mode: 'json',
      preset: null,
      ms: null,
    });
  }
  rows.value = list;

  const ROUNDS = 5;
  const samples: Record<string, number[]> = {};
  for (const row of list) samples[row.key] = [];

  for (let round = 0; round < ROUNDS; round++) {
    for (const row of list) {
      const ms = await timeOnce(row);
      if (ms !== null) samples[row.key].push(ms);
    }
  }

  rows.value = list.map((row) => ({
    ...row,
    ms: samples[row.key].length ? median(samples[row.key]) : null,
    attempts: samples[row.key].length + '/5',
  }));

  rows.value.sort((a, b) => {
    if (a.ms === null) return 1;
    if (b.ms === null) return -1;
    return a.ms - b.ms;
  });

  if (rows.value.every((r) => r.ms === null)) {
    error.value = t('dnsbench.fail');
  }
  busy.value = false;
}

function msText(ms: number | null): string {
  return ms === null ? '—' : ms + ' ms';
}

function msTone(ms: number | null): string {
  if (ms === null) return '';
  if (ms < 60) return 'ms-good';
  if (ms < 150) return 'ms-ok';
  return 'ms-slow';
}

async function apply(row: BenchRow) {
  if (!row.preset) return;
  vpnStore.updateSettings({ dns_remote: row.preset as never, dns_custom_doh: '' });
  appliedKey.value = row.key;
}

onMounted(() => {
  if (rows.value.length === 0) void run();
});
</script>

<style scoped>
.bench-list {
  padding: 6px 0;
}

.bench-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 18px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
}

.bench-row:last-child {
  border-bottom: none;
}

.bench-row--best .bench-name {
  color: var(--success);
}

.bench-pos {
  width: 26px;
  flex-shrink: 0;
  font-size: 11px;
  color: var(--muted-foreground);
}

.bench-name {
  flex: 1;
  min-width: 0;
  font-size: 13px;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.bench-host {
  font-size: 11px;
  color: var(--muted-foreground);
  white-space: nowrap;
}

.bench-ms {
  min-width: 76px;
  text-align: right;
  font-size: 12.5px;
  font-variant-numeric: tabular-nums;
}

.ms-good { color: var(--success); }
.ms-ok { color: oklch(0.85 0.14 95); }
.ms-slow { color: var(--destructive); }

.bench-apply {
  flex-shrink: 0;
}

.bench-applied {
  flex-shrink: 0;
  font-size: 11px;
  color: var(--success);
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
