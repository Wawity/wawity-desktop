<template>
<div class="tray" :class="{ 'tray--streamer': streamerActive }">
<div class="tray-head">
      <span class="dot" :class="{ 'dot--on': vpnStore.status.connected }" />
      <div class="tray-head-text">
        <span class="tray-status" v-text="statusText" />
        <span
          v-if="vpnStore.status.connected && vpnStore.status.server_name"
          class="tray-server mono"
          v-text="vpnStore.status.server_name"
        />
      </div>
      <button class="head-btn" type="button" :title="t('tray.openApp')" @click="openApp">
        <Maximize2 :size="13" />
      </button>
    </div>

    <div v-if="subs.length > 0" class="tray-subs" data-sensitive>
      <button
        v-for="sub in subs"
        :key="sub.id"
        type="button"
        class="sub-pill"
        :class="{
          'sub-pill--active': sub.id === vpnStore.selectedSubId,
          'sub-pill--dead': isDead(sub),
        }"
        @click="pickSub(sub)"
      >
        <span class="sub-pill-name" v-text="sub.name" />
        <span v-if="isDead(sub)" class="sub-pill-x" v-text="t('servers.expired')" />
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
        <div class="srv-text">
          <span class="srv-name" v-text="srv.name" />
          <span class="srv-meta mono" v-text="srv.server" />
        </div>
        <span
          v-if="srv.latencyMs !== null && srv.latencyMs !== undefined"
          class="srv-ping mono"
          v-text="srv.latencyMs + 'ms'"
        />
        <Check v-if="srv.id === vpnStore.selectedServerId" :size="13" class="srv-check" />
      </button>
    </div>

    <div class="tray-actions">
      <button type="button" class="act-btn" :disabled="!vpnStore.status.connected" @click="reconnect">
        <RotateCw :size="12" />
        <span v-text="t('tray.reconnect')" />
      </button>
      <button type="button" class="act-btn" @click="repair">
        <Wrench :size="12" />
        <span v-text="t('tray.repair')" />
      </button>
    </div>

    <div class="tray-foot">
      <button
        type="button"
        class="foot-btn foot-btn--accent"
        @click="toggle"
        v-text="vpnStore.status.connected ? t('tray.disconnect') : t('tray.connect')"
      />
      <button type="button" class="foot-btn" @click="openApp" v-text="t('tray.openApp')" />
      <button type="button" class="foot-btn foot-btn--danger" @click="quit" v-text="t('tray.exit')" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { Check, Maximize2, RotateCw, Wrench } from './lib/appIcons';
import { invoke } from '@tauri-apps/api/tauri';
import { emit, listen, type UnlistenFn } from '@tauri-apps/api/event';
import { appWindow } from '@tauri-apps/api/window';
import { useVpnStore } from './stores/vpn';
import { t } from './i18n';
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

.tray {
  display: flex;
  flex-direction: column;
  width: 100vw;
  height: 100vh;
  box-sizing: border-box;
  background: rgba(33, 33, 33, 0.97);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  overflow: hidden;
  color: #f2f2f2;
  font-family: 'Segoe UI Variable Text', 'Segoe UI', system-ui, sans-serif;
  font-size: 13px;
  box-shadow: 0 18px 46px rgba(0, 0, 0, 0.55);
  backdrop-filter: blur(16px);
  user-select: none;
}

.mono {
  font-family: 'Cascadia Mono', 'Consolas', monospace;
}

.tray-head {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 14px 10px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
}

.dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #ff5b60;
  box-shadow: 0 0 8px rgba(255, 91, 96, 0.5);
  flex-shrink: 0;
}

.dot--on {
  background: #6ccb5f;
  box-shadow: 0 0 8px rgba(108, 203, 95, 0.55);
}

.tray-head-text {
  display: flex;
  flex-direction: column;
  gap: 1px;
  min-width: 0;
  flex: 1;
}

.tray-status {
  font-size: 13px;
  font-weight: 600;
}

.tray-server {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.55);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.head-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: rgba(255, 255, 255, 0.65);
  cursor: pointer;
}

.head-btn:hover {
  background: rgba(255, 255, 255, 0.06);
  color: #fff;
}

.tray-subs {
  display: flex;
  gap: 6px;
  padding: 10px 12px 8px;
  overflow-x: auto;
  scrollbar-width: none;
}

.tray-subs::-webkit-scrollbar {
  display: none;
}

.sub-pill {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  border: 1px solid rgba(255, 255, 255, 0.09);
  border-radius: 14px;
  background: rgba(255, 255, 255, 0.04);
  color: rgba(255, 255, 255, 0.75);
  font-size: 12px;
  font-family: inherit;
  white-space: nowrap;
  cursor: pointer;
}

.sub-pill:hover {
  background: rgba(255, 255, 255, 0.08);
  color: #fff;
}

.sub-pill--active {
  background: rgba(76, 194, 255, 0.16);
  border-color: rgba(76, 194, 255, 0.45);
  color: #4cc2ff;
}

.sub-pill--dead {
  opacity: 0.4;
  cursor: default;
}

.sub-pill--dead:hover {
  background: rgba(255, 255, 255, 0.04);
  color: rgba(255, 255, 255, 0.75);
}

.sub-pill-name {
  max-width: 120px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.sub-pill-x {
  font-size: 10px;
  color: #ff8a92;
}

.tray-list {
  flex: 1;
  overflow-y: auto;
  padding: 4px 8px;
}

.tray-list::-webkit-scrollbar {
  width: 6px;
}

.tray-list::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.14);
  border-radius: 3px;
}

.tray-empty {
  padding: 24px 12px;
  text-align: center;
  color: rgba(255, 255, 255, 0.4);
  font-size: 12px;
}

.srv-row {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 7px 10px;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: #f2f2f2;
  font-family: inherit;
  text-align: left;
  cursor: pointer;
}

.srv-row:hover {
  background: rgba(255, 255, 255, 0.06);
}

.srv-row--active {
  background: rgba(255, 255, 255, 0.08);
  box-shadow: inset 3px 0 0 #4cc2ff;
}

.srv-text {
  display: flex;
  flex-direction: column;
  gap: 1px;
  min-width: 0;
  flex: 1;
}

.srv-name {
  font-size: 12.5px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.srv-meta {
  font-size: 10.5px;
  color: rgba(255, 255, 255, 0.45);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.srv-ping {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.55);
  flex-shrink: 0;
}

.srv-check {
  color: #4cc2ff;
  flex-shrink: 0;
}

.tray-actions {
  display: flex;
  gap: 6px;
  padding: 8px 12px 10px;
  border-top: 1px solid rgba(255, 255, 255, 0.06);
}

.act-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  flex: 1;
  padding: 6px 8px;
  border: 1px solid rgba(255, 255, 255, 0.09);
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.04);
  color: rgba(255, 255, 255, 0.75);
  font-size: 11.5px;
  font-family: inherit;
  cursor: pointer;
  white-space: nowrap;
}

.act-btn:hover {
  background: rgba(255, 255, 255, 0.09);
  color: #fff;
}

.act-btn:disabled {
  opacity: 0.35;
  cursor: default;
}

.act-btn:disabled:hover {
  background: rgba(255, 255, 255, 0.04);
  color: rgba(255, 255, 255, 0.75);
}

.tray-foot {
  display: flex;
  gap: 6px;
  padding: 0 12px 12px;
}

.foot-btn {
  flex: 1;
  padding: 7px 8px;
  border: 1px solid rgba(255, 255, 255, 0.09);
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.05);
  color: #f2f2f2;
  font-size: 12px;
  font-family: inherit;
  cursor: pointer;
  white-space: nowrap;
}

.foot-btn:hover {
  background: rgba(255, 255, 255, 0.1);
}

.foot-btn--accent {
  background: rgba(76, 194, 255, 0.16);
  border-color: rgba(76, 194, 255, 0.4);
  color: #4cc2ff;
}

.foot-btn--accent:hover {
  background: rgba(76, 194, 255, 0.26);
}

.foot-btn--danger:hover {
  background: rgba(255, 91, 96, 0.16);
  border-color: rgba(255, 91, 96, 0.4);
  color: #ff8a92;
}

.tray--streamer [data-sensitive] {
  filter: blur(10px) saturate(0.8);
  pointer-events: none;
  user-select: none;
}
</style>