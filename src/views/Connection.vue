<template>
  <div class="page">
    <div class="wordmark">
      <img :src="iconSrc" class="wm-logo" alt="Wawity" />
      <h1 class="wm-text">wawity</h1>
    </div>

    <div v-if="vpnStore.status.always_on_locked" class="lockdown-banner">
      <Lock :size="14" aria-hidden="true" />
      <span v-text="t('connection.lockdownBanner')"></span>
    </div>

    <div class="btn-wrap">
      <template v-if="vpnStore.status.connected">
        <span class="ring-ping" aria-hidden="true"></span>
        <span class="ring-static" aria-hidden="true"></span>
      </template>
      <span v-if="vpnStore.loading" class="busy-arc" aria-hidden="true"></span>
      <span v-if="burst" class="burst-ring" aria-hidden="true"></span>
      <button
        type="button"
        class="power-btn"
        :class="{ 'power-btn--on': vpnStore.status.connected, 'connect-flash': flash }"
        :aria-pressed="vpnStore.status.connected"
        :disabled="vpnStore.loading"
        @click="toggle"
      >
        <Power :size="40" :class="{ 'icon-pulse': vpnStore.loading }" aria-hidden="true" />
        <span class="power-label" v-text="powerLabel"></span>
      </button>
    </div>

    <div class="feature-pills">
      <button
        type="button"
        class="feature-pill"
        :class="{ 'feature-pill--on': vpnStore.settings.kill_switch }"
        @click="goToSetting('killswitch')"
      >
        <ShieldAlert :size="13" aria-hidden="true" />
        <span
          v-text="vpnStore.settings.kill_switch ? t('connection.killSwitchOn') : t('connection.killSwitchOff')"
        ></span>
      </button>
      <button
        type="button"
        class="feature-pill"
        :class="{ 'feature-pill--on': vpnStore.settings.always_on }"
        @click="goToSetting('alwayson')"
      >
        <Lock :size="13" aria-hidden="true" />
        <span
          v-text="vpnStore.settings.always_on ? t('connection.alwaysOnOn') : t('connection.alwaysOnOff')"
        ></span>
      </button>
      <button
        type="button"
        class="feature-pill"
        :class="{ 'feature-pill--on': vpnStore.settings.multihop_enabled }"
        @click="goToSetting('multihop')"
      >
        <Shuffle :size="13" aria-hidden="true" />
        <span
          v-text="vpnStore.settings.multihop_enabled ? t('connection.multihopOn') : t('connection.multihopOff')"
        ></span>
      </button>
    </div>

    <p v-if="vpnStore.status.multihop" class="hop-route mono" v-text="hopRoute"></p>

    <router-link to="/servers" class="server-pill">
      <div class="pill-left">
        <CountryFlag
          :code="vpnStore.selectedServer?.countryCode ?? 'UN'"
          :width="36"
          :height="24"
        />
        <div class="pill-info">
          <span class="pill-name" v-text="pillName"></span>
          <span class="pill-sub mono" v-text="pillSub"></span>
        </div>
      </div>
      <ChevronRight :size="16" class="chevron" aria-hidden="true" />
    </router-link>

    <div class="stats-grid">
      <div class="stat-card">
        <Signal :size="15" class="stat-icon" :class="pingColorClass" aria-hidden="true" />
        <span class="stat-value mono" :class="pingColorClass" v-text="vpnStore.currentPingDisplay"></span>
        <span class="stat-label" v-text="t('connection.ping')"></span>
      </div>
      <div class="stat-card">
        <ArrowDown :size="15" class="stat-icon stat-icon--down" aria-hidden="true" />
        <span class="stat-value mono" v-text="vpnStore.speedRxFormatted"></span>
        <span class="stat-label" v-text="t('connection.down')"></span>
      </div>
      <div class="stat-card">
        <ArrowUp :size="15" class="stat-icon stat-icon--up" aria-hidden="true" />
        <span class="stat-value mono" v-text="vpnStore.speedTxFormatted"></span>
        <span class="stat-label" v-text="t('connection.up')"></span>
      </div>
      <div class="stat-card">
        <Timer :size="15" class="stat-icon" aria-hidden="true" />
        <span class="stat-value mono" v-text="vpnStore.sessionDuration"></span>
        <span class="stat-label" v-text="t('connection.session')"></span>
      </div>
    </div>

    <p v-if="vpnStore.connectError" class="error-msg" v-text="vpnStore.connectError"></p>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onUnmounted } from 'vue';
import { useRouter } from 'vue-router';
import {
  Power,
  ChevronRight,
  ArrowDown,
  ArrowUp,
  Timer,
  ShieldAlert,
  Shuffle,
  Signal,
  Lock,
} from 'lucide-vue-next';
import { useVpnStore } from '../stores/vpn';
import { useAppIcon } from '../composables/useAppIcon';
import { t } from '../i18n';
import CountryFlag from '../components/CountryFlag.vue';

const vpnStore = useVpnStore();
const { iconSrc } = useAppIcon();
const router = useRouter();

const flash = ref(false);
const burst = ref(false);
let flashTimer = 0;
let burstTimer = 0;

const powerLabel = computed(() => {
  if (vpnStore.status.connected) return t('connection.disconnect');
  if (vpnStore.loading) return t('connection.connectingButton');
  return t('connection.connect');
});

const hopRoute = computed(() => {
  const entry = vpnStore.status.entry_server_name;
  const exit = vpnStore.status.server_name;
  if (!entry || !exit) return '';
  return entry + ' \u2192 ' + exit;
});

const pillName = computed(() => vpnStore.selectedServer?.name ?? t('connection.noServerSelected'));

const pillSub = computed(() => {
  const picked = vpnStore.selectedServer;
  if (!picked) return t('connection.goToServers');
  return picked.protocol + ' \u00b7 ' + picked.server;
});

const pingColorClass = computed(() => {
  const ms = vpnStore.currentPingMs;
  if (ms === null || ms === undefined) return '';
  if (ms < 80) return 'ping-good';
  if (ms < 200) return 'ping-ok';
  if (ms < 500) return 'ping-slow';
  return 'ping-bad';
});

watch(
  () => vpnStore.status.connected,
  (now, before) => {
    if (now && !before) celebrate();
  },
);

function celebrate() {
  window.clearTimeout(flashTimer);
  window.clearTimeout(burstTimer);
  flash.value = true;
  burst.value = true;
  flashTimer = window.setTimeout(() => {
    flash.value = false;
  }, 900);
  burstTimer = window.setTimeout(() => {
    burst.value = false;
  }, 750);
}

function goToSetting(key: string) {
  router.push({ path: '/settings', query: { highlight: key } });
}

async function toggle() {
  if (vpnStore.loading) return;
  if (vpnStore.status.connected) {
    await vpnStore.disconnect();
    return;
  }
  if (!vpnStore.selectedServerId) {
    vpnStore.connectError = t('connection.selectServerFirst');
    return;
  }
  await vpnStore.connect();
}

onUnmounted(() => {
  window.clearTimeout(flashTimer);
  window.clearTimeout(burstTimer);
});
</script>

<style scoped>
.page {
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  padding: 24px 20px 40px;
  min-height: 100%;
}

.wordmark {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 4px;
}

.wm-logo {
  width: 28px;
  height: 28px;
}

.wm-text {
  font-size: 20px;
  font-weight: 700;
  letter-spacing: 0.02em;
  margin: 0;
}

.lockdown-banner {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  max-width: 520px;
  padding: 11px 16px;
  border-radius: 16px;
  border: 1px solid rgba(255, 90, 90, 0.28);
  background: linear-gradient(180deg, rgba(255, 70, 70, 0.14), rgba(255, 70, 70, 0.07));
  backdrop-filter: blur(20px) saturate(150%);
  -webkit-backdrop-filter: blur(20px) saturate(150%);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.1),
    0 8px 24px rgba(0, 0, 0, 0.35);
  color: #ff8a8a;
  font-size: 12.5px;
  line-height: 1.4;
}

.btn-wrap {
  position: relative;
  width: 148px;
  height: 148px;
  margin-top: 12px;
}

.ring-static {
  position: absolute;
  inset: -12px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 50%;
  pointer-events: none;
}

.ring-ping {
  position: absolute;
  inset: -12px;
  border: 1px solid color-mix(in oklab, var(--success) 60%, transparent);
  border-radius: 50%;
  pointer-events: none;
  animation: ring-swell 3s ease-out infinite;
}

.busy-arc {
  position: absolute;
  inset: -12px;
  border: 2px solid transparent;
  border-top-color: var(--primary);
  border-radius: 50%;
  pointer-events: none;
  animation: arc-spin 900ms linear infinite;
}

.burst-ring {
  position: absolute;
  inset: 0;
  border: 2px solid var(--success);
  border-radius: 50%;
  pointer-events: none;
  animation: burst-out 700ms ease-out forwards;
}

.power-btn {
  position: relative;
  width: 148px;
  height: 148px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  border-radius: 50%;
  border: 1px solid rgba(255, 255, 255, 0.14);
  background: linear-gradient(165deg, rgba(255, 255, 255, 0.1), rgba(255, 255, 255, 0.03) 45%, rgba(0, 0, 0, 0.12));
  backdrop-filter: blur(28px) saturate(170%);
  -webkit-backdrop-filter: blur(28px) saturate(170%);
  box-shadow:
    inset 0 1.5px 0 rgba(255, 255, 255, 0.22),
    inset 0 -14px 28px rgba(0, 0, 0, 0.22),
    0 18px 44px rgba(0, 0, 0, 0.45);
  color: rgba(255, 255, 255, 0.65);
  cursor: pointer;
  overflow: hidden;
  transition:
    color 300ms ease,
    border-color 300ms ease,
    box-shadow 500ms ease,
    transform 180ms cubic-bezier(0.34, 1.56, 0.64, 1);
}

.power-btn::before {
  content: '';
  position: absolute;
  top: -35%;
  left: 12%;
  width: 76%;
  height: 58%;
  border-radius: 50%;
  background: radial-gradient(ellipse at center, rgba(255, 255, 255, 0.28), transparent 68%);
  pointer-events: none;
}

.power-btn:hover:not(:disabled) {
  transform: scale(1.04);
  color: rgba(255, 255, 255, 0.95);
}

.power-btn:active:not(:disabled) {
  transform: scale(0.96);
}

.power-btn:disabled {
  cursor: default;
  opacity: 0.8;
}

.power-btn--on {
  color: var(--success);
  border-color: color-mix(in oklab, var(--success) 50%, rgba(255, 255, 255, 0.14));
  box-shadow:
    inset 0 1.5px 0 rgba(255, 255, 255, 0.22),
    inset 0 -14px 28px color-mix(in oklab, var(--success) 14%, transparent),
    0 0 52px color-mix(in oklab, var(--success) 30%, transparent),
    0 18px 44px rgba(0, 0, 0, 0.45);
}

.connect-flash {
  animation: flash-fade 900ms ease-out;
}

.icon-pulse {
  animation: icon-throb 1s ease-in-out infinite;
}

.power-label {
  font-family: var(--font-sans);
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 0.05em;
  text-transform: uppercase;
}

.feature-pills {
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  gap: 8px;
}

.feature-pill {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 13px;
  border-radius: 999px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.08), rgba(255, 255, 255, 0.02));
  backdrop-filter: blur(18px) saturate(150%);
  -webkit-backdrop-filter: blur(18px) saturate(150%);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.14),
    0 4px 14px rgba(0, 0, 0, 0.25);
  font-size: 11.5px;
  color: var(--muted-foreground);
  cursor: pointer;
  transition:
    border-color 250ms ease,
    color 250ms ease,
    box-shadow 250ms ease,
    transform 180ms cubic-bezier(0.34, 1.56, 0.64, 1);
}

.feature-pill:hover {
  transform: translateY(-1px);
  border-color: rgba(255, 255, 255, 0.2);
  color: var(--foreground);
}

.feature-pill:active {
  transform: translateY(0) scale(0.97);
}

.feature-pill--on {
  border-color: color-mix(in oklab, var(--success) 40%, rgba(255, 255, 255, 0.1));
  color: var(--success);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.14),
    0 0 18px color-mix(in oklab, var(--success) 16%, transparent),
    0 4px 14px rgba(0, 0, 0, 0.25);
}

.feature-pill--on:hover {
  color: var(--success);
}

.hop-route {
  font-size: 12px;
  color: var(--muted-foreground);
  margin: 0;
}

.server-pill {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  width: 100%;
  max-width: 380px;
  padding: 13px 16px;
  border-radius: 20px;
  border: 1px solid rgba(255, 255, 255, 0.11);
  background: linear-gradient(170deg, rgba(255, 255, 255, 0.09), rgba(255, 255, 255, 0.03) 55%, rgba(0, 0, 0, 0.08));
  backdrop-filter: blur(24px) saturate(160%);
  -webkit-backdrop-filter: blur(24px) saturate(160%);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.16),
    0 12px 32px rgba(0, 0, 0, 0.38);
  color: var(--foreground);
  text-decoration: none;
  overflow: hidden;
  transition:
    border-color 250ms ease,
    box-shadow 350ms ease,
    transform 180ms cubic-bezier(0.34, 1.56, 0.64, 1);
}

.server-pill::before {
  content: '';
  position: absolute;
  top: 0;
  left: 8%;
  width: 84%;
  height: 1px;
  background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.35), transparent);
  pointer-events: none;
}

.server-pill:hover {
  transform: translateY(-1px);
  border-color: rgba(255, 255, 255, 0.2);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.16),
    0 16px 38px rgba(0, 0, 0, 0.45);
}

.server-pill:active {
  transform: translateY(0) scale(0.99);
}

.pill-left {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
}

.pill-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.pill-name {
  font-size: 13.5px;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.pill-sub {
  font-size: 11px;
  color: var(--muted-foreground);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.chevron {
  color: var(--muted-foreground);
  flex-shrink: 0;
  transition: transform 200ms ease;
}

.server-pill:hover .chevron {
  transform: translateX(3px);
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 10px;
  width: 100%;
  max-width: 520px;
}

.stat-card {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 13px 8px;
  border-radius: 18px;
  border: 1px solid rgba(255, 255, 255, 0.09);
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.07), rgba(255, 255, 255, 0.02));
  backdrop-filter: blur(20px) saturate(150%);
  -webkit-backdrop-filter: blur(20px) saturate(150%);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.12),
    0 8px 22px rgba(0, 0, 0, 0.3);
  color: var(--muted-foreground);
  transition: border-color 250ms ease, transform 180ms ease;
}

.stat-card:hover {
  border-color: rgba(255, 255, 255, 0.16);
  transform: translateY(-1px);
}

.stat-icon {
  color: var(--muted-foreground);
}

.stat-icon--down {
  color: var(--success);
}

.stat-icon--up {
  color: var(--primary);
}

.stat-value {
  font-size: 13px;
  color: var(--foreground);
}

.stat-label {
  font-size: 10.5px;
  letter-spacing: 0.06em;
  text-transform: uppercase;
}

.error-msg {
  max-width: 380px;
  font-size: 12.5px;
  color: var(--destructive);
  text-align: center;
  margin: 0;
}

.ping-good {
  color: var(--success);
}

.ping-ok {
  color: oklch(0.85 0.16 95);
}

.ping-slow {
  color: oklch(0.75 0.16 55);
}

.ping-bad {
  color: var(--destructive);
}

@keyframes ring-swell {
  0% {
    transform: scale(1);
    opacity: 0.8;
  }
  70% {
    transform: scale(1.35);
    opacity: 0;
  }
  100% {
    transform: scale(1.35);
    opacity: 0;
  }
}

@keyframes arc-spin {
  to {
    transform: rotate(360deg);
  }
}

@keyframes burst-out {
  0% {
    transform: scale(1);
    opacity: 1;
  }
  100% {
    transform: scale(1.9);
    opacity: 0;
  }
}

@keyframes flash-fade {
  0% {
    box-shadow:
      inset 0 1.5px 0 rgba(255, 255, 255, 0.22),
      0 0 90px color-mix(in oklab, var(--success) 55%, transparent);
  }
  100% {
    box-shadow:
      inset 0 1.5px 0 rgba(255, 255, 255, 0.22),
      0 0 52px color-mix(in oklab, var(--success) 30%, transparent);
  }
}

@keyframes icon-throb {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.45;
  }
}

@media (max-width: 560px) {
  .stats-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}
</style>