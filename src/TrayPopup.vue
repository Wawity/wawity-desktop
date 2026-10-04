<template>
<div class="tray" :class="{ 'tray--streamer': streamerActive }">
    <div class="tray-head">
      <span class="tick" :class="{ 'tick--on': vpnStore.status.connected }">
        <span class="tick-mark"><Check :size="13" /></span>
      </span>
      <div class="tray-head-text">
        <span class="tray-status" v-text="statusText" />
        <span
          v-if="vpnStore.status.connected && vpnStore.status.server_name"
          class="tray-server"
          v-text="vpnStore.status.server_name"
        />
      </div>
    </div>

    <div v-if="subs.length > 1" class="tray-subs" data-sensitive>
      <button
        v-for="sub in subs"
        :key="sub.id"
        type="button"
        class="sub-chip"
        :class="{
          'sub-chip--active': sub.id === vpnStore.selectedSubId,
          'sub-chip--dead': isDead(sub),
        }"
        @click="pickSub(sub)"
      >
        <span class="sub-chip-name" v-text="sub.name" />
        <span v-if="isDead(sub)" class="sub-chip-x" v-text="t('servers.expired')" />
      </button>
    </div>

    <div class="tray-list" data-sensitive>
      <div v-if="servers.length === 0" class="tray-empty" v-text="t('tray.noServers')" />
      <button
        v-for="srv in servers"
        :key="srv.id"
        type="button"
        class="srv-row"
        :class="{ 'srv-row--active': srv.id === vpnStore.selectedServerId }"
        @click="pickServer(srv.id)"
      >
        <CountryFlag :code="srv.countryCode" :size="16" />
        <span class="srv-name" v-text="srv.name" />
        <span
          class="srv-ping mono"
          :class="srv.latencyMs != null ? pingTier(srv.latencyMs) : ''"
          v-text="srv.latencyMs != null ? srv.latencyMs + 'ms' : ''"
        />
        <Check v-if="srv.id === vpnStore.selectedServerId" :size="13" class="srv-check" />
      </button>
    </div>

    <div class="tray-foot">
      <button
        type="button"
        class="connect-btn"
        @click="toggle"
        v-text="vpnStore.status.connected ? t('tray.disconnect') : t('tray.connect')"
      />
      <button type="button" class="icon-btn" :title="t('tray.reconnect')" :disabled="!vpnStore.status.connected" @click="reconnect">
        <RotateCw :size="14" />
      </button>
      <button type="button" class="icon-btn" :title="t('tray.repair')" @click="repair">
        <Wrench :size="14" />
      </button>
      <button type="button" class="icon-btn" :title="t('tray.openApp')" @click="openApp">
        <Maximize2 :size="14" />
      </button>
      <button type="button" class="icon-btn icon-btn--danger" :title="t('tray.exit')" @click="quit">
        <X :size="14" />
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { Check, Maximize2, RotateCw, Wrench, X } from './lib/appIcons';
import { invoke } from '@tauri-apps/api/tauri';
import { emit, listen, type UnlistenFn } from '@tauri-apps/api/event';
import { appWindow } from '@tauri-apps/api/window';
import { useVpnStore } from './stores/vpn';
import { setLanguage, t } from './i18n';
import { pingTier } from './lib/geo';
import type { SubscriptionGroup, VpnStatus } from './types/vpn.d';
import CountryFlag from './components/CountryFlag.vue';

const vpnStore = useVpnStore();

const subs = computed(() => vpnStore.subscriptions);
const servers = computed(() => vpnStore.trayServers);
const statusText = computed(() =>
  vpnStore.status.connected ? t('tray.connected') : t('tray.disconnected'),
);

// The popup keeps its own store instance. It must stay passive: calling
// refreshStatus() here used to start session/ping timers, write traffic
// history over the main window's copy and push a second Discord presence.
let shown = false;

function isDead(sub: SubscriptionGroup): boolean {
  return sub.expiresAt !== null && sub.expiresAt <= Date.now();
}

function notifyMain() {
  emit('wawity-tray-sync').catch(() => {});
}

function pickSub(sub: SubscriptionGroup) {
  if (isDead(sub)) return;
  vpnStore.selectSubscription(sub.id);
  notifyMain();
}

function pickServer(id: string) {
  if (vpnStore.isServerExpired(id)) return;
  vpnStore.selectServer(id);
  notifyMain();
  invoke('tray_connect_server', { serverId: id }).catch(() => {});
}

function toggle() {
  if (vpnStore.status.connected) {
    emit('wawity-tray-disconnect').catch(() => {});
  }
  invoke('tray_toggle_connection').catch(() => {});
}

function openApp() {
  invoke('tray_open_main').catch(() => {});
}

function quit() {
  invoke('tray_quit').catch(() => {});
}

function reconnect() {
  invoke('tray_reconnect').catch(() => {});
}

function repair() {
  invoke('tray_repair').catch(() => {});
}

function hidePopup() {
  shown = false;
  appWindow.hide().catch(() => {});
}

const streamerActive = ref(false);

async function refreshStreamer() {
  if (!vpnStore.settings.streamer_mode) {
    streamerActive.value = false;
    return;
  }
  try {
    streamerActive.value = await invoke<boolean>('stream_capture_state');
  } catch {
    streamerActive.value = false;
  }
}

async function pullStatus() {
  try {
    const raw = await invoke<VpnStatus>('get_vpn_status');
    Object.assign(vpnStore.status, raw);
  } catch {}
}

function pullSettings() {
  try {
    const raw = localStorage.getItem('wawity_settings');
    if (raw) vpnStore.settings = { ...vpnStore.settings, ...JSON.parse(raw) };
  } catch {}
  setLanguage(vpnStore.settings.language);
}

function reload() {
  pullSettings();
  vpnStore.loadSelectedServer();
  vpnStore.loadSubscriptions();
  void pullStatus();
  void refreshStreamer();
}

function onShown() {
  shown = true;
  reload();
}

function onKey(e: KeyboardEvent) {
  if (e.key === 'Escape') hidePopup();
}

let unlisten: UnlistenFn | null = null;
let unlistenBlur: UnlistenFn | null = null;
let timer: ReturnType<typeof setInterval> | null = null;

onMounted(async () => {
  reload();
  unlisten = await listen('tray-popup-shown', onShown);
  // The popup hides itself on blur, so blur == off screen.
  unlistenBlur = await listen('tauri://blur', () => {
    shown = false;
  });
  timer = setInterval(() => {
    if (!shown || document.hidden) return;
    void pullStatus();
  }, 2000);
  window.addEventListener('keydown', onKey);
});

onUnmounted(() => {
  if (unlisten) unlisten();
  if (unlistenBlur) unlistenBlur();
  if (timer) clearInterval(timer);
  window.removeEventListener('keydown', onKey);
});
</script>

<style scoped>
:global(html),
:global(body),
:global(#app) {
  margin: 0;
  padding: 0;
  background: transparent !important;
  overflow: hidden;
}

/* Flat onyx flyout on the site's tokens. This used to be the Windows 11
   acrylic idiom: #0067c0 primary, #4cc2ff selection, Segoe UI, 8px radius. */
.tray {
  display: flex;
  flex-direction: column;
  width: 100vw;
  height: 100vh;
  box-sizing: border-box;
  background: rgba(11, 10, 10, 0.9);
  border: 1px solid var(--border);
  border-radius: 14px;
  overflow: hidden;
  color: var(--paper);
  font-family: var(--font-sans);
  font-size: 13px;
  box-shadow: 0 18px 48px rgba(0, 0, 0, 0.55);
  backdrop-filter: blur(30px) saturate(1.1);
  -webkit-backdrop-filter: blur(30px) saturate(1.1);
  user-select: none;
}

.mono {
  font-family: var(--font-mono);
}

/* ---------- header ---------- */

.tray-head {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px 16px 12px;
}

/* The connection state reads as the same tick the settings toggles use rather
   than a red/green dot: a filled hairline circle with the Check glyph drawn
   in. Same motion, so the two surfaces agree on what "on" looks like. */
.tick {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border-radius: 50%;
  flex-shrink: 0;
  border: 1px solid var(--input);
  background: transparent;
  transition:
    background 200ms ease,
    border-color 200ms ease;
}

.tick--on {
  background: var(--paper);
  border-color: var(--paper);
}

.tick-mark {
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--background);
  transform: scale(0.5);
  opacity: 0;
  transition:
    transform 220ms cubic-bezier(0.34, 1.5, 0.64, 1),
    opacity 160ms ease;
}

.tick--on .tick-mark {
  transform: scale(1);
  opacity: 1;
}

/* The path is 21.2 units long, so 22 fully hides the stroke. */
.tick-mark svg path {
  stroke-dasharray: 22;
  stroke-dashoffset: 22;
  transition: stroke-dashoffset 300ms cubic-bezier(0.22, 1, 0.36, 1);
}

.tick--on .tick-mark svg path {
  stroke-dashoffset: 0;
  transition: stroke-dashoffset 340ms cubic-bezier(0.19, 1, 0.22, 1);
}

html.motion-simple .tick-mark,
html.motion-simple .tick--on .tick-mark {
  transform: none;
  opacity: 0;
  transition: opacity 120ms linear;
}

html.motion-simple .tick--on .tick-mark {
  opacity: 1;
}

html.motion-simple .tick-mark svg path {
  stroke-dasharray: none;
  stroke-dashoffset: 0;
  transition: none;
}

.tray-head-text {
  display: flex;
  flex-direction: column;
  gap: 1px;
  min-width: 0;
  flex: 1;
}

.tray-status {
  font-family: var(--font-display);
  font-size: 15px;
  font-weight: 400;
  letter-spacing: -0.025em;
}

.tray-server {
  font-size: 11.5px;
  color: var(--muted-foreground);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* ---------- subscription chips ---------- */

.tray-subs {
  display: flex;
  gap: 6px;
  padding: 0 12px 10px;
  overflow-x: auto;
  scrollbar-width: none;
}

.tray-subs::-webkit-scrollbar {
  display: none;
}

.sub-chip {
  display: flex;
  align-items: center;
  gap: 6px;
  height: 26px;
  padding: 0 10px;
  border: 1px solid var(--border);
  border-radius: 999px;
  background: transparent;
  color: var(--muted-foreground);
  font-size: 12px;
  font-family: inherit;
  white-space: nowrap;
  cursor: pointer;
  transition:
    background 120ms ease,
    color 120ms ease,
    border-color 120ms ease;
}

.sub-chip:hover {
  background: color-mix(in oklch, var(--paper) 6%, transparent);
  color: var(--paper);
}

.sub-chip--active {
  background: color-mix(in oklch, var(--ember) 12%, transparent);
  border-color: color-mix(in oklch, var(--ember) 40%, transparent);
  color: var(--ember-soft);
}

.sub-chip--dead {
  opacity: 0.4;
  cursor: default;
}

.sub-chip--dead:hover {
  background: transparent;
  color: var(--muted-foreground);
}

.sub-chip-name {
  max-width: 120px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.sub-chip-x {
  font-size: 10px;
  color: var(--destructive);
}

/* ---------- server list ---------- */

.tray-list {
  flex: 1;
  overflow-y: auto;
  padding: 2px 8px 8px;
}

.tray-list::-webkit-scrollbar {
  width: 6px;
}

.tray-list::-webkit-scrollbar-thumb {
  background: color-mix(in oklch, var(--paper) 12%, transparent);
  border-radius: 3px;
}

.tray-empty {
  padding: 28px 12px;
  text-align: center;
  color: var(--muted-foreground);
  font-size: 12px;
}

.srv-row {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  height: 38px;
  padding: 0 10px;
  border: none;
  border-radius: 8px;
  background: transparent;
  color: var(--paper);
  font-family: inherit;
  text-align: left;
  cursor: pointer;
  transition: background 100ms ease;
}

.srv-row:hover {
  background: color-mix(in oklch, var(--paper) 6%, transparent);
}

.srv-row:active {
  background: color-mix(in oklch, var(--paper) 4%, transparent);
}

.srv-row--active {
  background: color-mix(in oklch, var(--ember) 10%, transparent);
}

.srv-row--active:hover {
  background: color-mix(in oklch, var(--ember) 15%, transparent);
}

.srv-name {
  flex: 1;
  min-width: 0;
  font-size: 12.5px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.srv-ping {
  font-family: var(--font-mono);
  font-size: 10.5px;
  color: var(--muted-foreground);
  flex-shrink: 0;
  font-variant-numeric: tabular-nums;
}

.srv-ping.tier-good { color: var(--success); }
.srv-ping.tier-ok { color: var(--muted-foreground); }
.srv-ping.tier-slow { color: var(--ember-soft); }
.srv-ping.tier-bad { color: var(--destructive); }

.srv-check {
  color: var(--ember);
  flex-shrink: 0;
}

/* ---------- bottom bar: primary + icon actions ---------- */

.tray-foot {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 10px 12px 12px;
  border-top: 1px solid var(--border);
}

/* The site's primary is inverted white on near-black, not a filled blue. */
.connect-btn {
  flex: 1;
  height: 34px;
  border: 1px solid transparent;
  border-radius: 8px;
  background: var(--paper);
  color: var(--background);
  font-size: 12.5px;
  font-weight: 500;
  font-family: var(--font-display);
  letter-spacing: -0.01em;
  cursor: pointer;
  white-space: nowrap;
  transition:
    opacity 120ms ease,
    background 120ms ease;
}

.connect-btn:hover {
  opacity: 0.88;
}

.connect-btn:active {
  opacity: 0.72;
}

.icon-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  flex-shrink: 0;
  border: none;
  border-radius: 8px;
  background: transparent;
  color: var(--muted-foreground);
  cursor: pointer;
  transition:
    background 120ms ease,
    color 120ms ease;
}

.icon-btn:hover:not(:disabled) {
  background: color-mix(in oklch, var(--paper) 7%, transparent);
  color: var(--paper);
}

.icon-btn:disabled {
  opacity: 0.35;
  cursor: default;
}

.icon-btn--danger:hover:not(:disabled) {
  background: color-mix(in oklch, var(--destructive) 14%, transparent);
  color: var(--destructive);
}

.tray--streamer [data-sensitive] {
  filter: blur(10px) saturate(0.8);
  pointer-events: none;
  user-select: none;
}
</style>
