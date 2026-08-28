<template>
  <ExtraPage :title="t('leaks.title')" :subtitle="t('leaks.subtitle')">
    <template #actions>
      <button v-if="!busy" type="button" class="tool-action tool-action--primary" @click="run">
        <span v-text="tested ? t('leaks.retest') : t('leaks.testing')" />
      </button>
      <button v-else type="button" class="tool-action" disabled>
        <span v-text="t('leaks.collecting')" />
      </button>
    </template>

    <p v-if="failed" class="tool-error" v-text="failed" />

    
    <section class="glass lk-verdict">
      <span class="lk-word mono" :class="'lk-' + overall">
        {{ overall === 'pass' ? '✓' : overall === 'fail' ? '✕' : '…' }}
        {{ tested ? (overall === 'pass' ? 'OK' : overall === 'fail' ? 'LEAK' : '—') : '—' }}
      </span>
      <span class="lk-hint">{{ tested ? '' : t('leaks.subtitle') }}</span>
    </section>

    
    <section class="glass lk-card">
      <header class="lk-head"><span class="mono" v-text="t('leaks.exit')"></span></header>
      <div class="lk-rows">
        <div class="lk-row">
          <span class="lk-k mono" v-text="t('leaks.exit')"></span>
          <span class="lk-v mono">{{ audit.exitIp || '—' }}</span>
        </div>
        <div class="lk-row">
          <span class="lk-k mono" v-text="t('leaks.country')"></span>
          <span class="lk-v mono">{{ audit.exitCountry || '—' }}<template v-if="zone"> / {{ zone }}</template></span>
        </div>
        <div class="lk-row" v-if="audit.carrier">
          <span class="lk-k mono" v-text="t('leaks.carrier')"></span>
          <span class="lk-v mono">{{ audit.carrier }}</span>
        </div>
      </div>
    </section>

    
    <div class="lk-grid">
      <section class="glass lk-check">
        <div class="lk-check-head">
          <span class="mono">DNS</span>
          <span class="lk-state mono" :class="'st-' + dnsState">{{ dnsState.toUpperCase() }}</span>
        </div>
        <p class="lk-note">{{ audit.dnsOutsideTunnel ? t('leaks.outsideTunnel') : t('leaks.dnsNote') }}</p>
        <div class="lk-resolvers mono">
          <div v-for="(r, idx) in audit.resolvers.slice(0, 4)" :key="idx" class="lk-resolver-row">
            <span>{{ r.ip }}</span><span v-if="r.country" class="lk-res-country">{{ r.country }}</span>
          </div>
          <span v-if="!audit.resolvers.length">{{ t('leaks.noResolvers') }}</span>
        </div>
      </section>

      <section class="glass lk-check">
        <div class="lk-check-head">
          <span class="mono">IPv6</span>
          <span class="lk-state mono" :class="'st-' + ipv6State">{{ ipv6State.toUpperCase() }}</span>
        </div>
        <p class="lk-note"><code>{{ audit.ipv6 || '—' }}</code></p>
      </section>

      <section class="glass lk-check">
        <div class="lk-check-head">
          <span class="mono">WebRTC</span>
          <span class="lk-state mono" :class="'st-' + rtcState">{{ rtcState.toUpperCase() }}</span>
        </div>
        <p class="lk-note">{{ rtcHosts.length ? rtcHosts.slice(0, 3).join(', ') : t('leaks.webrtcNote') }}</p>
        <button type="button" class="tool-action" style="margin-top: 12px" :disabled="rtcBusy" @click="sniffWebRtc">
          <span>WebRTC</span>
        </button>
      </section>

      <section class="glass lk-check">
        <div class="lk-check-head">
          <span class="mono" v-text="t('leaks.timezone')"></span>
          <span class="lk-state mono" :class="'st-' + tzState">{{ tzState.toUpperCase() }}</span>
        </div>
        <p class="lk-note"><code>{{ zone || '—' }}</code></p>
      </section>
    </div>
  </ExtraPage>
</template><script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import {
  Binary,
  Clock,
  Globe2,
  Network,
  Radio,
  RefreshCw,
  Shield,
  ShieldAlert,
  ShieldCheck,
} from '../../lib/appIcons';
import ExtraPage from '../../components/ExtraPage.vue';
import { useTilt } from '../../lib/useTilt';
import { t } from '../../i18n';
import type { LeakAudit } from '../../types/vpn';

const blank: LeakAudit = {
  exitIp: '',
  exitCountry: '',
  carrier: '',
  colo: '',
  ipv6: '',
  ipv6Exposed: false,
  resolvers: [],
  resolverCountries: [],
  dnsOutsideTunnel: false,
  resolverCount: 0,
  tookMs: 0,
};

const audit = ref<LeakAudit>({ ...blank });
const busy = ref(false);
const tested = ref(false);
const failed = ref('');
const rtcHosts = ref<string[]>([]);
const rtcBusy = ref(false);
const zone = ref('');

const { tiltStyle, handleMove, handleLeave } = useTilt(6);

let peer: RTCPeerConnection | null = null;
let rtcTimer = 0;

const exitState = computed(() => (tested.value && audit.value.exitIp ? 'pass' : 'unknown'));

const dnsState = computed(() => {
  if (!tested.value || !audit.value.resolvers.length) return 'unknown';
  return audit.value.dnsOutsideTunnel ? 'fail' : 'pass';
});

const ipv6State = computed(() => {
  if (!tested.value) return 'unknown';
  return audit.value.ipv6Exposed ? 'fail' : 'pass';
});

const rtcState = computed(() => {
  if (rtcBusy.value) return 'unknown';
  if (!rtcHosts.value.length) return tested.value ? 'pass' : 'unknown';
  return rtcHosts.value.every((ip) => ip === audit.value.exitIp) ? 'pass' : 'fail';
});

const tzState = computed(() => {
  if (!tested.value || !audit.value.exitCountry || !zone.value) return 'unknown';
  const region = zone.value.split('/')[0].toLowerCase();
  const country = audit.value.exitCountry.toLowerCase();
  if (
    region === 'europe' &&
    ['russia', 'germany', 'netherlands', 'france', 'finland', 'sweden', 'poland'].includes(country)
  ) {
    return 'pass';
  }
  return 'warn';
});

const overall = computed(() => {
  const pool = [dnsState.value, ipv6State.value, rtcState.value, tzState.value];
  if (pool.includes('fail')) return 'fail';
  if (pool.includes('warn') || pool.includes('unknown')) return 'warn';
  return 'pass';
});

const shieldIcon = computed(() => {
  if (busy.value) return Shield;
  if (overall.value === 'pass') return ShieldCheck;
  if (overall.value === 'fail') return ShieldAlert;
  return Shield;
});

function closePeer() {
  if (rtcTimer) {
    window.clearTimeout(rtcTimer);
    rtcTimer = 0;
  }
  if (peer) {
    peer.close();
    peer = null;
  }
}

function sniffWebRtc() {
  closePeer();
  rtcHosts.value = [];
  if (typeof RTCPeerConnection === 'undefined') {
    rtcBusy.value = false;
    return;
  }

  rtcBusy.value = true;
  try {
    peer = new RTCPeerConnection({
      iceServers: [{ urls: 'stun:stun.l.google.com:19302' }],
    });
    peer.createDataChannel('probe');
    peer.onicecandidate = (event) => {
      if (!event.candidate) {
        rtcBusy.value = false;
        return;
      }
      const found = /([0-9]{1,3}(?:\.[0-9]{1,3}){3}|[a-f0-9]{1,4}(?::[a-f0-9]{0,4}){2,7})/i.exec(
        event.candidate.candidate,
      );
      if (!found) return;
      const ip = found[1];
      if (
        ip.startsWith('10.') ||
        ip.startsWith('192.168.') ||
        ip.startsWith('127.') ||
        ip.endsWith('.local')
      )
        return;
      if (!rtcHosts.value.includes(ip)) rtcHosts.value.push(ip);
    };
    peer
      .createOffer()
      .then((offer) => peer && peer.setLocalDescription(offer))
      .catch(() => {});
    rtcTimer = window.setTimeout(() => {
      rtcBusy.value = false;
      rtcTimer = 0;
    }, 4000);
  } catch {
    rtcBusy.value = false;
  }
}

async function run() {
  if (busy.value) return;
  busy.value = true;
  tested.value = false;
  failed.value = '';
  sniffWebRtc();
  try {
    audit.value = await invoke<LeakAudit>('audit_leaks');
    tested.value = true;
  } catch (error) {
    failed.value = String(error);
  } finally {
    busy.value = false;
  }
}

onMounted(() => {
  zone.value = Intl.DateTimeFormat().resolvedOptions().timeZone || '';
  run();
});

onUnmounted(closePeer);
</script>

<style scoped>
.lk-verdict {
  display: flex;
  align-items: center;
  gap: 18px;
  padding: 22px 26px;
  margin-bottom: 14px;
}

.lk-word {
  font-size: 30px;
  font-weight: 300;
  letter-spacing: 0.04em;
  color: var(--foreground);
  white-space: nowrap;
}

.lk-pass { color: var(--success); }
.lk-fail { color: var(--destructive); }
.lk-warn { color: #f0d36a; }

.lk-hint {
  font-size: 13px;
  color: var(--muted-foreground);
  line-height: 1.5;
}

.lk-card {
  margin-bottom: 14px;
}

.lk-head {
  padding: 13px 18px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  font-size: 10px;
  letter-spacing: 0.2em;
  text-transform: uppercase;
  color: var(--muted-foreground);
}

.lk-rows {
  padding: 6px 18px 10px;
}

.lk-row {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 16px;
  padding: 9px 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
}

.lk-row:last-child {
  border-bottom: none;
}

.lk-k {
  font-size: 10.5px;
  letter-spacing: 0.16em;
  text-transform: uppercase;
  color: var(--muted-foreground);
}

.lk-v {
  font-size: 13.5px;
  color: var(--foreground);
  text-align: right;
  word-break: break-all;
}

.lk-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 14px;
}

.lk-check {
  padding: 18px 20px 20px;
}

.lk-check-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.lk-check-head > span:first-child {
  font-size: 11px;
  letter-spacing: 0.2em;
  text-transform: uppercase;
  color: var(--foreground);
}

.lk-state {
  font-size: 10px;
  letter-spacing: 0.14em;
}

.st-pass { color: var(--success); }
.st-fail { color: var(--destructive); }
.st-warn { color: #f0d36a; }
.st-unknown { color: var(--muted-foreground); }

.lk-note {
  margin: 0;
  font-size: 12.5px;
  line-height: 1.6;
  color: var(--muted-foreground);
  overflow-wrap: anywhere;
}

.lk-note code {
  font-family: var(--font-mono);
  font-size: 12px;
  opacity: 0.85;
}

.lk-resolvers {
  margin-top: 12px;
  display: flex;
  flex-direction: column;
  gap: 5px;
  font-size: 11.5px;
  color: var(--fog, var(--muted-foreground));
}

.lk-resolver-row {
  display: flex;
  justify-content: space-between;
  gap: 12px;
}

.lk-res-country {
  opacity: 0.55;
}

@media (max-width: 720px) {
  .lk-grid {
    grid-template-columns: 1fr;
  }
}
</style>