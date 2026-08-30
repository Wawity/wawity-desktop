<template>
  <div ref="pageRef" class="page">
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
        ref="powerBtnRef"
        type="button"
        class="power-btn"
        :class="{ 'power-btn--on': vpnStore.status.connected, 'connect-flash': flash }"
        :aria-pressed="vpnStore.status.connected"
        :disabled="vpnStore.loading"
        @mouseenter="onPowerEnter"
        @mouseleave="onPowerLeave"
        @mousedown="pressPower"
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
          v-text="
            vpnStore.settings.kill_switch
              ? t('connection.killSwitchOn')
              : t('connection.killSwitchOff')
          "
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
          v-text="
            vpnStore.settings.always_on ? t('connection.alwaysOnOn') : t('connection.alwaysOnOff')
          "
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
          v-text="
            vpnStore.settings.multihop_enabled
              ? t('connection.multihopOn')
              : t('connection.multihopOff')
          "
        ></span>
      </button>
    </div>

    <p v-if="vpnStore.status.multihop" data-sensitive class="hop-route mono" v-text="hopRoute"></p>

    <div class="server-card" data-sensitive>
      <router-link to="/servers" class="server-main">
        <div class="pill-left">
          <CountryFlag
            :code="vpnStore.selectedServer?.countryCode ?? 'UN'"
            :width="30"
            :height="20"
          />
          <div class="pill-info">
            <span class="pill-name" v-text="pillName"></span>
            <span class="pill-sub mono" v-text="pillSub"></span>
          </div>
        </div>
        <ChevronRight :size="15" class="chevron" aria-hidden="true" />
      </router-link>
      <button
        ref="copyBtnRef"
        type="button"
        class="server-copy-btn"
        :disabled="!vpnStore.selectedServer"
        :title="t('connection.copyIpTitle')"
        @click="copyServerAddress"
      >
        <Check v-if="serverCopied" :size="15" aria-hidden="true" />
        <Copy v-else :size="15" aria-hidden="true" />
      </button>
    </div>

    <div class="stats-grid" data-sensitive>
      <div class="stat-card">
        <Signal :size="15" class="stat-icon" :class="pingColorClass" aria-hidden="true" />
        <span
          class="stat-value mono"
          :class="pingColorClass"
          v-text="vpnStore.currentPingDisplay"
        ></span>
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

    <div class="ip-pill" data-sensitive :class="vpnStore.status.connected ? 'ip-pill--secure' : 'ip-pill--exposed'">
      <span class="ip-orb">
        <Transition name="orb-swap" mode="out-in">
          <ThinkingOrb
            v-if="vpnStore.loading"
            key="orb-loading"
            :size="18"
            state="connecting"
          />
          <ThinkingOrb
            v-else-if="vpnStore.status.connected"
            key="orb-on"
            :size="18"
            state="breathing"
          />
          <EyeOff v-else key="ico-off" :size="14" />
        </Transition>
      </span>
      <Transition name="stage-swap" mode="out-in">
        <span ref="ipPillTextRef" :key="ipPillLabel" class="ip-pill-text" v-text="ipPillLabel"></span>
      </Transition>
    </div>

    <p v-if="vpnStore.connectError" class="error-msg" v-text="vpnStore.connectError"></p>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onUnmounted, onMounted, nextTick } from 'vue';
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
  Activity,
  Copy,
  Check,
  EyeOff,
} from '../lib/appIcons';
import { useVpnStore } from '../stores/vpn';
import { useAppIcon } from '../composables/useAppIcon';
import { writeText } from '@tauri-apps/api/clipboard';
import { showCopyHint } from '../composables/useCopyHint';
import { t } from '../i18n';
import { gsap } from 'gsap';
import { pressPop, isFancy, staggerChildren } from '../lib/motion';
import CountryFlag from '../components/CountryFlag.vue';
import ThinkingOrb from '../components/ThinkingOrb.vue';
  
const vpnStore = useVpnStore();
const { iconSrc } = useAppIcon();
const router = useRouter();

const flash = ref(false);
const powerBtnRef = ref<HTMLElement | null>(null);
const pageRef = ref<HTMLElement | null>(null);
const burst = ref(false);
let flashTimer = 0;
let burstTimer = 0;

const CONNECT_STAGES = [
  'connection.stageInit',
  'connection.stageResolve',
  'connection.stageHandshake',
  'connection.stageRoute',
  'connection.stageVerify',
  'connection.stageAlmost',
];
const stageIdx = ref(0);
let stageTimer = 0;

onMounted(() => {
  staggerChildren(pageRef.value, '.feature-pill, .stat-card', { per: 0.045 });
});

const stageLabel = computed(() => {
  if (vpnStore.status.connected) return t('connection.stageDisconnecting');
  const idx = Math.min(stageIdx.value, CONNECT_STAGES.length - 1);
  return t(CONNECT_STAGES[idx]);
});

const serverCopied = ref(false);
const copyBtnRef = ref<HTMLElement | null>(null);
const ipPillTextRef = ref<HTMLElement | null>(null);
let serverCopyTimer = 0;

const ipPillLabel = computed(() => {
  if (vpnStore.loading) return stageLabel.value;
  if (vpnStore.status.connected) return t('connection.ipHidden');
  return t('connection.ipExposed');
});

watch(ipPillLabel, async () => {
  await nextTick();
  if (!isFancy()) return;
  const el = ipPillTextRef.value;
  if (!el) return;
  gsap.fromTo(
    el,
    { filter: 'blur(6px)', opacity: 0.2, y: 3 },
    { filter: 'blur(0px)', opacity: 1, y: 0, duration: 0.38, ease: 'expo.out', clearProps: 'filter' },
  );
});

async function copyServerAddress() {
  const server = vpnStore.selectedServer?.server;
  if (!server) return;
  try {
    await writeText(server);
    serverCopied.value = true;
    showCopyHint(t('connection.ipCopied'));
    pressPop(copyBtnRef.value);
    if (serverCopyTimer) window.clearTimeout(serverCopyTimer);
    serverCopyTimer = window.setTimeout(() => {
      serverCopied.value = false;
    }, 1600);
  } catch {}
}

function startStages() {
  stopStages();
  stageIdx.value = 0;
  stageTimer = window.setInterval(() => {
    if (stageIdx.value < CONNECT_STAGES.length - 1) {
      stageIdx.value += 1;
    } else {
      stopStages();
    }
  }, 950);
}

function stopStages() {
  if (stageTimer) {
    window.clearInterval(stageTimer);
    stageTimer = 0;
  }
}

watch(
  () => vpnStore.loading,
  (now) => {
    if (now && !vpnStore.status.connected) {
      startStages();
    } else {
      stopStages();
    }
  },
);

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

  
  if (isFancy()) {
    const btn = powerBtnRef.value;
    if (btn) {
      gsap.killTweensOf(btn);
      gsap
        .timeline()
        .fromTo(
          btn,
          { scale: 1 },
          { scale: 1.07, duration: 0.16, ease: 'power2.out' },
        )
        .to(btn, { scale: 1, duration: 0.6, ease: 'elastic.out(1, 0.45)' });
    }
  }

  flashTimer = window.setTimeout(() => {
    flash.value = false;
  }, 900);
  burstTimer = window.setTimeout(() => {
    burst.value = false;
  }, 750);
}

function onPowerEnter() {
  if (!isFancy()) return;
  const btn = powerBtnRef.value;
  if (!btn) return;
  gsap.killTweensOf(btn);
  gsap.to(btn, { scale: 1.04, duration: 0.3, ease: 'power2.out' });
}

function onPowerLeave() {
  if (!isFancy()) return;
  const btn = powerBtnRef.value;
  if (!btn) return;
  gsap.killTweensOf(btn);
  gsap.to(btn, { scale: 1, duration: 0.4, ease: 'power2.out' });
}

function pressPower() {
  pressPop(powerBtnRef.value);
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
  vpnStore.clearDisconnectIntent();
  if (!vpnStore.selectedServerId) {
    vpnStore.connectError = t('connection.selectServerFirst');
    return;
  }
  await vpnStore.connectWithChain();
}

onUnmounted(() => {
  window.clearTimeout(flashTimer);
  window.clearTimeout(burstTimer);
  window.clearTimeout(serverCopyTimer);
  stopStages();
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
  backdrop-filter: blur(11px);
  -webkit-backdrop-filter: blur(11px);
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
  animation: arc-spin 1500ms linear infinite;
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
  background: linear-gradient(
    165deg,
    rgba(255, 255, 255, 0.1),
    rgba(255, 255, 255, 0.03) 45%,
    rgba(0, 0, 0, 0.12)
  );
  backdrop-filter: blur(15px);
  -webkit-backdrop-filter: blur(15px);
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

.power-btn > svg,
.power-label {
  position: relative;
  z-index: 1;
}

.power-btn:hover:not(:disabled) {
  color: rgba(255, 255, 255, 0.95);
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
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
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

.server-card {
  position: relative;
  display: flex;
  align-items: stretch;
  width: 316px;
  height: 56px;
  box-sizing: border-box;
  border-radius: 18px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  background: linear-gradient(
    170deg,
    rgba(255, 255, 255, 0.08),
    rgba(255, 255, 255, 0.03) 55%,
    rgba(0, 0, 0, 0.1)
  );
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.14),
    0 10px 26px rgba(0, 0, 0, 0.35);
  overflow: hidden;
  transition:
    border-color 250ms ease,
    box-shadow 350ms ease,
    transform 180ms cubic-bezier(0.34, 1.56, 0.64, 1);
}

.server-card::before {
  content: '';
  position: absolute;
  top: 0;
  left: 8%;
  width: 84%;
  height: 1px;
  background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.35), transparent);
  pointer-events: none;
}

.server-card:hover {
  transform: translateY(-1px);
  border-color: rgba(255, 255, 255, 0.2);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.16),
    0 16px 38px rgba(0, 0, 0, 0.45);
}

.server-main {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex: 1;
  min-width: 0;
  gap: 10px;
  padding: 0 11px 0 11px;
  color: var(--foreground);
  text-decoration: none;
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

.server-main:hover .chevron {
  transform: translateX(3px);
}

.server-copy-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 44px;
  flex-shrink: 0;
  border: none;
  border-left: 1px solid rgba(255, 255, 255, 0.08);
  background:
    linear-gradient(
      180deg,
      rgba(255, 255, 255, 0.05),
      rgba(255, 255, 255, 0.015)
    );
  color: var(--muted-foreground);
  cursor: pointer;
  transition:
    color 200ms ease,
    background 200ms ease,
    border-color 250ms ease;
}

.server-copy-btn:hover:not(:disabled) {
  color: #d9ccff;
  background: rgba(167, 139, 250, 0.14);
}

.server-copy-btn:disabled {
  opacity: 0.35;
  cursor: default;
}

.server-copy-btn > svg {
  transition: transform 200ms cubic-bezier(0.34, 1.56, 0.64, 1);
}

.server-copy-btn:hover:not(:disabled) > svg {
  transform: scale(1.12);
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
  backdrop-filter: blur(11px);
  -webkit-backdrop-filter: blur(11px);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.12),
    0 8px 22px rgba(0, 0, 0, 0.3);
  color: var(--muted-foreground);
  transition:
    border-color 250ms ease,
    transform 180ms ease;
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

.ip-pill {
  display: inline-flex;
  align-items: center;
  gap: 10px;
  margin-top: 2px;
  padding: 8px 18px;
  border-radius: 999px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.05), rgba(255, 255, 255, 0.015));
  backdrop-filter: blur(13px);
  -webkit-backdrop-filter: blur(13px);
  box-shadow:
    inset 0 1px 1px rgba(255, 255, 255, 0.09),
    0 8px 24px rgba(0, 0, 0, 0.35);
  color: var(--muted-foreground);
  font-size: 12px;
  font-weight: 500;
  transition:
    border-color 400ms ease,
    color 400ms ease,
    box-shadow 500ms ease,
    background 400ms ease;
}

.ip-orb {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  flex-shrink: 0;
  border-radius: 50%;
}

.ip-pill-text {
  white-space: nowrap;
  filter: blur(0);
  transition: filter 300ms ease;
}

.ip-pill--exposed {
  color: rgba(235, 238, 250, 0.55);
}

.ip-pill--secure {
  color: var(--success);
  border-color: color-mix(in oklab, var(--success) 40%, transparent);
  background: linear-gradient(
    180deg,
    color-mix(in oklab, var(--success) 12%, transparent),
    color-mix(in oklab, var(--success) 3%, transparent)
  );
  box-shadow:
    inset 0 1px 1px rgba(255, 255, 255, 0.1),
    0 0 28px color-mix(in oklab, var(--success) 16%, transparent),
    0 8px 24px rgba(0, 0, 0, 0.35);
}

.orb-swap-enter-active,
.orb-swap-leave-active {
  transition:
    opacity 180ms ease,
    filter 220ms ease,
    transform 200ms ease;
}
.orb-swap-enter-from {
  opacity: 0;
  transform: scale(0.5);
  filter: blur(4px);
}
.orb-swap-leave-to {
  opacity: 0;
  transform: scale(1.4);
  filter: blur(4px);
}

@media (max-width: 560px) {
  .stats-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}
.status-pill {
  display: inline-flex;
  align-items: center;
  gap: 12px;
  margin-top: 6px;
  padding: 10px 22px 10px 12px;
  border-radius: 999px;
  border: 1px solid rgba(255, 255, 255, 0.09);
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.06), rgba(255, 255, 255, 0.02));
  backdrop-filter: blur(13px);
  -webkit-backdrop-filter: blur(13px);
  box-shadow:
    inset 0 1px 1px rgba(255, 255, 255, 0.1),
    0 10px 30px rgba(0, 0, 0, 0.4);
}

.status-track {
  position: relative;
  display: inline-flex;
  min-width: 0;
}

.stage-word {
  font-size: 13.5px;
  font-weight: 500;
  letter-spacing: 0.01em;
  white-space: nowrap;
}

.t-shimmer {
  position: relative;
  display: inline-block;
  color: #6e6e6e;
}

.t-shimmer::before {
  content: attr(data-text);
  position: absolute;
  inset: 0;
  pointer-events: none;
  background-image: linear-gradient(
    90deg,
    transparent 0%,
    transparent 40%,
    #ededed 50%,
    transparent 60%,
    transparent 100%
  );
  background-size: 400% 100%;
  background-repeat: no-repeat;
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
  -webkit-text-fill-color: transparent;
  animation: t-shimmer 2000ms linear infinite;
}

@keyframes t-shimmer {
  0% {
    background-position: 100% 0;
  }
  100% {
    background-position: 0% 0;
  }
}

.stage-swap-enter-active,
.stage-swap-leave-active {
  transition:
    transform 150ms ease-in-out,
    filter 150ms ease-in-out,
    opacity 150ms ease-in-out;
}

.stage-swap-enter-from {
  transform: translateY(4px);
  filter: blur(2px);
  opacity: 0;
}

.stage-swap-leave-to {
  transform: translateY(-4px);
  filter: blur(2px);
  opacity: 0;
}

.status-pill-enter-active {
  transition:
    opacity 260ms ease,
    transform 320ms cubic-bezier(0.34, 1.56, 0.64, 1);
}

.status-pill-leave-active {
  transition:
    opacity 200ms ease,
    transform 200ms ease;
}

.status-pill-enter-from {
  opacity: 0;
  transform: translateY(10px) scale(0.96);
}

.status-pill-leave-to {
  opacity: 0;
  transform: translateY(6px) scale(0.98);
}

@media (prefers-reduced-motion: reduce) {
  .t-shimmer::before {
    animation: none !important;
  }
  .status-pill-enter-active,
  .status-pill-leave-active,
  .stage-swap-enter-active,
  .stage-swap-leave-active {
    transition: none !important;
  }
}
</style>
