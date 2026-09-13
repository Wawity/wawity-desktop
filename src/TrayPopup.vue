<template>
<div class="tray" :class="{ 'tray--streamer': streamerActive }">
    <div class="tray-head">
      <span class="orb" :class="{ 'orb--on': vpnStore.status.connected }">
        <span class="orb-core" />
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
import { t } from './i18n';
import { pingTier } from './lib/geo';
import type { SubscriptionGroup } from './types/vpn.d';
import CountryFlag from './components/CountryFlag.vue';

const vpnStore = useVpnStore();

const subs = computed(() => vpnStore.subscriptions);
const servers = computed(() => vpnStore.trayServers);
const statusText = computed(() =>
  vpnStore.status.connected ? t('tray.connected') : t('tray.disconnected'),
);

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

function reload() {
  vpnStore.loadSettings();
  vpnStore.loadSelectedServer();
  vpnStore.loadSubscriptions();
  vpnStore.refreshStatus().catch(() => {});
  void refreshStreamer();
}

function onKey(e: KeyboardEvent) {
  if (e.key === 'Escape') hidePopup();
}

let unlisten: UnlistenFn | null = null;
let timer: ReturnType<typeof setInterval> | null = null;

onMounted(async () => {
  reload();
  unlisten = await listen('tray-popup-shown', reload);
  timer = setInterval(() => {
    
    if (document.hidden) return;
    vpnStore.refreshStatus().catch(() => {});
  }, 2000);
  window.addEventListener('keydown', onKey);
});

onUnmounted(() => {
  if (unlisten) unlisten();
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

/* Windows 11 flyout: acrylic surface, 8px outer radius, quiet 1px border */
.tray {
  display: flex;
  flex-direction: column;
  width: 100vw;
  height: 100vh;
  box-sizing: border-box;
  background: rgba(36, 36, 36, 0.78);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  overflow: hidden;
  color: #ffffff;
  font-family: 'Segoe UI Variable Text', 'Segoe UI', system-ui, sans-serif;
  font-size: 13px;
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.45);
  backdrop-filter: blur(28px) saturate(1.3);
  -webkit-backdrop-filter: blur(28px) saturate(1.3);
  user-select: none;
}

.mono {
  font-family: 'Cascadia Mono', 'Consolas', monospace;
}

/* ---------- header ---------- */

.tray-head {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px 16px 12px;
}

.orb {
  position: relative;
  display: grid;
  place-items: center;
  width: 30px;
  height: 30px;
  border-radius: 50%;
  flex-shrink: 0;
  background: rgba(255, 255, 255, 0.06);
}

.orb-core {
  width: 9px;
  height: 9px;
  border-radius: 50%;
  background: #ff5b60;
  transition: background 300ms ease;
}

.orb--on .orb-core {
  background: #6ccb5f;
}

.orb--on {
  background: rgba(108, 203, 95, 0.12);
}

.tray-head-text {
  display: flex;
  flex-direction: column;
  gap: 1px;
  min-width: 0;
  flex: 1;
}

.tray-status {
  font-size: 13.5px;
  font-weight: 600;
  letter-spacing: 0.01em;
}

.tray-server {
  font-size: 11.5px;
  color: rgba(255, 255, 255, 0.55);
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
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.045);
  color: rgba(255, 255, 255, 0.75);
  font-size: 12px;
  font-family: inherit;
  white-space: nowrap;
  cursor: pointer;
  transition: background 120ms ease;
}

.sub-chip:hover {
  background: rgba(255, 255, 255, 0.08);
  color: #fff;
}

.sub-chip--active {
  background: rgba(76, 194, 255, 0.14);
  border-color: rgba(76, 194, 255, 0.4);
  color: #4cc2ff;
}

.sub-chip--dead {
  opacity: 0.4;
  cursor: default;
}

.sub-chip--dead:hover {
  background: rgba(255, 255, 255, 0.045);
  color: rgba(255, 255, 255, 0.75);
}

.sub-chip-name {
  max-width: 120px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.sub-chip-x {
  font-size: 10px;
  color: #ff8a92;
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
  background: rgba(255, 255, 255, 0.14);
  border-radius: 3px;
}

.tray-empty {
  padding: 28px 12px;
  text-align: center;
  color: rgba(255, 255, 255, 0.4);
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
  border-radius: 6px;
  background: transparent;
  color: #f5f5f5;
  font-family: inherit;
  text-align: left;
  cursor: pointer;
  transition: background 100ms ease;
}

.srv-row:hover {
  background: rgba(255, 255, 255, 0.06);
}

.srv-row:active {
  background: rgba(255, 255, 255, 0.04);
}

.srv-row--active {
  background: rgba(76, 194, 255, 0.12);
}

.srv-row--active:hover {
  background: rgba(76, 194, 255, 0.16);
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
  font-size: 11px;
  color: rgba(255, 255, 255, 0.5);
  flex-shrink: 0;
  font-variant-numeric: tabular-nums;
}

.srv-ping.tier-good { color: #6ccb5f; }
.srv-ping.tier-ok { color: #f0d36a; }
.srv-ping.tier-slow { color: #ff9f6b; }
.srv-ping.tier-bad { color: #ff8a92; }

.srv-check {
  color: #4cc2ff;
  flex-shrink: 0;
}

/* ---------- bottom bar: primary + icon actions ---------- */

.tray-foot {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 10px 12px 12px;
}

.connect-btn {
  flex: 1;
  height: 32px;
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 5px;
  background: #0067c0;
  color: #fff;
  font-size: 12.5px;
  font-weight: 600;
  font-family: inherit;
  cursor: pointer;
  white-space: nowrap;
  transition: background 120ms ease;
}

.connect-btn:hover {
  background: #1976d2;
}

.connect-btn:active {
  background: #005ba6;
}

.icon-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  flex-shrink: 0;
  border: none;
  border-radius: 5px;
  background: transparent;
  color: rgba(255, 255, 255, 0.7);
  cursor: pointer;
  transition: background 120ms ease, color 120ms ease;
}

.icon-btn:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.07);
  color: #fff;
}

.icon-btn:disabled {
  opacity: 0.35;
  cursor: default;
}

.icon-btn--danger:hover:not(:disabled) {
  background: rgba(255, 91, 96, 0.14);
  color: #ff8a92;
}

.tray--streamer [data-sensitive] {
  filter: blur(10px) saturate(0.8);
  pointer-events: none;
  user-select: none;
}
</style>
