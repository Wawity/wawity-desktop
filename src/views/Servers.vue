<template>
  <div ref="pageRoot" class="page">
    <div class="page-header">
      <h1 class="page-title" v-text="t('servers.title')" />
      <p class="page-sub" v-text="subtitleText" />
    </div>

    <div v-if="vpnStore.settings.multihop_enabled" class="hop-banner">
      <Shuffle :size="14" />
      <span>
        <span v-text="t('servers.multihopBannerPart1') + ' '" />
        <Shuffle :size="11" class="inline-icon" />
        <span v-text="' ' + t('servers.multihopBannerPart2') + ' '" />
        <template v-if="vpnStore.entryServer">
          <span v-text="t('servers.currentEntry') + ' '" />
          <strong v-text="vpnStore.entryServer.name" />
        </template>
        <span v-else v-text="t('servers.noEntrySelected')" />
      </span>
    </div>

    <div class="add-section">
      <div class="url-row">
        <input
          v-model="importUrl"
          type="text"
          :placeholder="t('servers.pasteUrl')"
          class="url-input"
          :disabled="fetching"
          @keydown.enter="fetchSubscription"
        />
        <button class="ghost-btn" type="button" @click="pasteUrl">
          <Clipboard :size="14" />
        </button>
        <button
          class="fetch-btn"
          type="button"
          :disabled="!importUrl.trim() || fetching"
          @click="fetchSubscription"
        >
          <Loader2 v-if="fetching" :size="14" class="spin" />
          <span v-else v-text="t('servers.fetch')" />
        </button>
      </div>
      <p v-if="fetchError" class="fetch-error" v-text="fetchError" />
    </div>

    <Transition name="preview-slide">
      <div v-if="preview" class="preview-card">
        <div class="preview-header">
          <div class="preview-meta">
            <span class="preview-name" v-text="preview.name" />
            <span class="preview-count" v-text="previewCountText" />
          </div>
          <button class="ghost-btn" type="button" @click="clearPreview">
            <X :size="14" />
          </button>
        </div>
        <ul class="preview-list">
          <li
            v-for="(srv, i) in preview.servers"
            :key="i"
            class="preview-item"
            :style="{ animationDelay: `${Math.min(i * 16, 500)}ms` }"
          >
            <CountryFlag :code="srv.countryCode" :size="28" />
            <div class="preview-info">
              <span class="preview-srv-name" v-text="srv.name" />
              <span class="preview-proto mono" v-text="srvMeta(srv)" />
            </div>
          </li>
        </ul>
        <button
          class="add-sub-btn"
          type="button"
          @click="openConfirm"
          v-text="t('servers.addSubscription')"
        />
      </div>
    </Transition>

    <Transition name="fade">
      <div v-if="confirmOpen" class="confirm-overlay" @click.self="confirmOpen = false">
        <div class="confirm-modal">
          <div class="confirm-icon"><ShieldCheck :size="26" /></div>
          <h3 class="confirm-title" v-text="t('servers.addSubscriptionConfirm')" />
          <p class="confirm-desc" v-text="addSubDescText" />
          <div class="confirm-actions">
            <button
              class="confirm-cancel"
              type="button"
              @click="confirmOpen = false"
              v-text="t('servers.cancel')"
            />
            <button
              class="confirm-ok"
              type="button"
              @click="addSubscription"
              v-text="t('servers.add')"
            />
          </div>
        </div>
      </div>
    </Transition>

    <Transition name="fade">
      <div v-if="deleteTarget" class="confirm-overlay" @click.self="deleteTarget = null">
        <div class="confirm-modal">
          <div class="confirm-icon confirm-icon--danger"><Trash2 :size="26" /></div>
          <h3 class="confirm-title" v-text="t('servers.removeSubscriptionConfirm')" />
          <p class="confirm-desc" v-text="removeSubDescText" />
          <div class="confirm-actions">
            <button
              class="confirm-cancel"
              type="button"
              @click="deleteTarget = null"
              v-text="t('servers.cancel')"
            />
            <button
              class="confirm-ok confirm-ok--danger"
              type="button"
              @click="removeSubscription"
              v-text="t('servers.remove')"
            />
          </div>
        </div>
      </div>
    </Transition>

    <Transition name="fav-reveal">
      <div v-if="vpnStore.favoriteServers.length > 0" class="fav-strip" data-sensitive>
        <div class="fav-head">
          <Star :size="12" class="fav-star-icon" aria-hidden="true" />
          <span v-text="t('servers.favStrip')" />
          <button
            type="button"
            class="pill-btn fav-auto-btn"
            :class="{ 'pill-btn--active': vpnStore.settings.auto_failover }"
            :title="t('servers.favAutoTitle')"
            @click="
              vpnStore.updateSettings({ auto_failover: !vpnStore.settings.auto_failover })
            "
          >
            <Radar :size="11" />
            <span v-text="t('servers.favAuto')" />
          </button>
        </div>
        <div class="fav-chips">
          <button
            v-for="srv in vpnStore.favoriteServers"
            :key="srv.id"
            type="button"
            class="fav-chip"
            :class="{ 'fav-chip--on': vpnStore.selectedServerId === srv.id }"
            :disabled="switching || vpnStore.loading || vpnStore.isServerExpired(srv.id)"
            @click="activateFav(srv.id)"
          >
            <CountryFlag :code="srv.countryCode ?? 'UN'" :width="16" :height="11" />
            <span class="fav-chip-name mono" v-text="srv.name"></span>
            <span
              v-if="srv.latencyMs !== null && srv.latencyMs !== undefined"
              class="ping-badge"
              :class="pingTierFn(srv.latencyMs)"
              v-text="srv.latencyMs + 'ms'"
            ></span>
            <Check
              v-if="vpnStore.selectedServerId === srv.id"
              :size="11"
              class="fav-chip-check"
              aria-hidden="true"
            />
          </button>
        </div>
      </div>
    </Transition>

    <div v-if="vpnStore.subscriptions.length > 0" class="toolbar">
      <div class="search-bar">
        <Search :size="14" class="search-icon" />
        <input
          v-model="query"
          type="text"
          :placeholder="t('servers.searchServers')"
          class="search-input"
        />
      </div>
      <button
        class="tool-btn"
        type="button"
        :disabled="vpnStore.latencyLoading || vpnStore.autoSelectLoading"
        :title="t('servers.pingAllTitle')"
        @click="pingAll"
      >
        <Loader2 v-if="vpnStore.latencyLoading" :size="13" class="spin" />
        <Radio v-else :size="13" />
        <span v-text="t('servers.ping')" />
      </button>
      <button
        class="tool-btn tool-btn--accent"
        type="button"
        :disabled="vpnStore.autoSelectLoading || vpnStore.latencyLoading"
        :title="t('servers.autoTitle')"
        @click="autoSelect"
      >
        <Loader2 v-if="vpnStore.autoSelectLoading" :size="13" class="spin" />
        <Zap v-else :size="13" />
        <span v-text="t('servers.auto')" />
      </button>
      <button
        class="tool-btn"
        type="button"
        :disabled="anyRefreshing"
        :title="t('servers.refreshAllTitle')"
        @click="refreshAllSubscriptions"
      >
        <Loader2 v-if="anyRefreshing" :size="13" class="spin" />
        <RotateCw v-else :size="13" />
        <span v-text="t('servers.refreshAll')" />
      </button>
    </div>

    <div v-if="vpnStore.subscriptions.length === 0" class="empty-state">
      <Globe :size="32" class="empty-icon" />
      <p v-text="t('servers.noSubscriptions')" />
      <p class="empty-sub" v-text="t('servers.noSubscriptionsSub')" />
    </div>

    <template v-else>
      <div class="subs-strip">
        <div v-for="sub in vpnStore.subscriptions" :key="sub.id" class="sub-chip" data-sensitive>
          <button
            type="button"
            class="sub-badge"
            :style="subBadgeStyle(sub.id)"
            :title="t('servers.badgeTitle')"
            @click.stop="badgeEditId = badgeEditId === sub.id ? null : sub.id"
          >
            <component :is="subBadgeGlyph(sub.id)" :size="12" aria-hidden="true" />
          </button>
          <span class="sub-chip-name" v-text="sub.name" />

          <Transition name="pop">
            <div v-if="badgeEditId === sub.id" class="badge-pop" @click.stop>
              <p class="badge-pop-title" v-text="t('servers.badgeIcon')" />
              <div class="badge-icons">
                <button
                  v-for="item in SUB_ICONS"
                  :key="item.key"
                  type="button"
                  class="badge-cell"
                  :class="{ 'badge-cell--on': currentBadge(sub.id).icon === item.key }"
                  @click="
                    vpnStore.setSubscriptionBadge(sub.id, item.key, currentBadge(sub.id).color)
                  "
                >
                  <component :is="item.icon" :size="14" aria-hidden="true" />
                </button>
              </div>
              <p class="badge-pop-title" v-text="t('servers.badgeColor')" />
              <div class="badge-colors">
                <button
                  v-for="tone in SUB_COLORS"
                  :key="tone"
                  type="button"
                  class="badge-dot"
                  :class="{ 'badge-dot--on': currentBadge(sub.id).color === tone }"
                  :style="{ background: tone }"
                  @click="vpnStore.setSubscriptionBadge(sub.id, currentBadge(sub.id).icon, tone)"
                />
              </div>
              <button
                type="button"
                class="badge-done"
                @click="badgeEditId = null"
                v-text="t('servers.badgeDone')"
              />
            </div>
          </Transition>
          <span class="sub-chip-count mono" v-text="sub.servers.length" />
          <span
            v-if="expiryLabel(sub)"
            class="expiry-badge"
            :class="expiryClass(sub)"
            v-text="expiryLabel(sub)"
          />
          <span v-if="trafficLabel(sub)" class="traffic-badge mono" v-text="trafficLabel(sub)" />
          <button
            class="chip-btn"
            type="button"
            :title="t('servers.copySubTitle')"
            @click="copySubLink(sub)"
          >
            <Copy :size="12" />
          </button>
          <button
            class="chip-btn"
            type="button"
            :disabled="vpnStore.refreshingSubIds.has(sub.id)"
            :title="t('servers.refreshSubTitle')"
            @click="refreshSubscription(sub.id)"
          >
            <Loader2 v-if="vpnStore.refreshingSubIds.has(sub.id)" :size="12" class="spin" />
            <RotateCw v-else :size="12" />
          </button>
          <button
            class="chip-btn chip-btn--danger"
            type="button"
            :title="t('servers.removeSubTitle')"
            @click="promptDelete(sub)"
          >
            <Trash2 :size="12" />
          </button>
        </div>
      </div>

      <ServerGlobe
        v-if="vpnStore.settings.server_view === 'globe'"
        :key="vpnStore.settings.ui_style"
        :query="query"
      />
      <ServerColumns v-else :query="query" />
    </template>
  </div>
</template>

<script setup lang="ts">
import { SUB_COLORS, SUB_ICONS, iconByKey, tintSoft } from '../lib/subicons';
import { ref, computed, onMounted, onUnmounted } from 'vue';
import {
  Clipboard,
  Search,
  Globe,
  Star,
  Copy,
  Check,
  Loader2,
  X,
  ShieldCheck,
  Trash2,
  Zap,
  Radio,
  Shuffle,
  RotateCw,
  Radar,
} from '../lib/appIcons';
import { readText, writeText } from '@tauri-apps/api/clipboard';
import { useVpnStore, formatBytes } from '../stores/vpn';
import { useNotifications } from '../composables/useNotifications';
import { showCopyHint } from '../composables/useCopyHint';
import { t } from '../i18n';
import { staggerChildren } from '../lib/motion';
import { pingTier } from '../lib/geo';
import type { SubscriptionGroup } from '../types/vpn.d';
import CountryFlag from '../components/CountryFlag.vue';
import ServerColumns from '../components/ServerColumns.vue';
import ServerGlobe from '../components/ServerGlobe.vue';

const vpnStore = useVpnStore();
const { pushToast } = useNotifications();

const switching = ref(false);

function pingTierFn(ms: number): string {
  return pingTier(ms);
}

async function activateFav(serverId: string) {
  if (switching.value || vpnStore.loading) return;
  if (serverId === vpnStore.selectedServerId) return;
  switching.value = true;
  try {
    await vpnStore.activateFavorite(serverId);
    const name = vpnStore.selectedServer?.name ?? '';
    if (vpnStore.status.connected) {
      pushToast('info', t('toast.switchedTo'), name, 3000);
    }
  } finally {
    switching.value = false;
  }
}

async function copySubLink(sub: SubscriptionGroup) {
  try {
    await writeText(sub.url);
    showCopyHint(t('servers.subCopied'));
  } catch {}
}

const importUrl = ref('');
const fetching = ref(false);
const fetchError = ref('');
const query = ref('');
const confirmOpen = ref(false);
const deleteTarget = ref<{ id: string; name: string; count: number } | null>(null);
const preview = ref<{ name: string; servers: typeof vpnStore.allServers } | null>(null);

const nowTick = ref(Date.now());
let tickTimer: ReturnType<typeof setInterval> | null = null;
const pageRoot = ref<HTMLElement | null>(null);

onMounted(() => {
  tickTimer = setInterval(() => {
    nowTick.value = Date.now();
  }, 60000);
  staggerChildren(pageRoot.value, '.sub-chip', { per: 0.06 });
});

onUnmounted(() => {
  if (tickTimer) clearInterval(tickTimer);
});

const totalServers = computed(() =>
  vpnStore.subscriptions.reduce((a, s) => a + s.servers.length, 0),
);
const anyRefreshing = computed(() => vpnStore.refreshingSubIds.size > 0);

const subtitleText = computed(() =>
  t('servers.subtitle', { count: totalServers.value, subs: vpnStore.subscriptions.length }),
);

const previewCountText = computed(() =>
  preview.value ? t('servers.serversFound', { count: preview.value.servers.length }) : '',
);

const addSubDescText = computed(() =>
  t('servers.addSubscriptionDesc', {
    name: preview.value?.name ?? '',
    count: preview.value?.servers.length ?? 0,
  }),
);

const removeSubDescText = computed(() =>
  deleteTarget.value
    ? t('servers.removeSubscriptionDesc', {
        name: deleteTarget.value.name,
        count: deleteTarget.value.count,
      })
    : '',
);

function srvMeta(srv: { protocol: string; server: string }): string {
  return `${srv.protocol} · ${srv.server}`;
}

function expiryLabel(sub: SubscriptionGroup): string {
  if (!sub.expiresAt) return '';
  const diff = sub.expiresAt - nowTick.value;
  if (diff <= 0) return t('servers.expired');
  const days = Math.floor(diff / 86400000);
  if (days > 0) return t('servers.daysLeft', { days });
  const hours = Math.floor(diff / 3600000);
  if (hours > 0) return t('servers.hoursLeft', { hours });
  const minutes = Math.floor(diff / 60000);
  return minutes > 0 ? t('servers.minutesLeft', { minutes }) : t('servers.lessThanMinute');
}

function expiryClass(sub: SubscriptionGroup): string {
  if (!sub.expiresAt) return '';
  const diff = sub.expiresAt - nowTick.value;
  if (diff <= 0) return 'expiry-badge--expired';
  const days = diff / 86400000;
  if (days < 1) return 'expiry-badge--critical';
  if (days < 3) return 'expiry-badge--warning';
  return 'expiry-badge--ok';
}

function trafficLabel(sub: SubscriptionGroup): string {
  if (sub.trafficTotalBytes === null || sub.trafficTotalBytes === undefined) return '';
  const used = sub.trafficUsedBytes ?? 0;
  return `${formatBytes(used)} / ${formatBytes(sub.trafficTotalBytes)}`;
}

async function pasteUrl() {
  try {
    const text = await readText();
    if (text) importUrl.value = text.trim();
  } catch {}
}

async function fetchSubscription() {
  if (!importUrl.value.trim() || fetching.value) return;
  fetchError.value = '';
  preview.value = null;
  fetching.value = true;
  try {
    const result = await vpnStore.fetchSubscriptionPreview(importUrl.value);
    if (result.servers.length === 0) {
      fetchError.value = 'No valid servers found in this subscription.';
    } else {
      preview.value = result;
    }
  } catch (e) {
    fetchError.value = String(e);
  } finally {
    fetching.value = false;
  }
}

function clearPreview() {
  preview.value = null;
  importUrl.value = '';
  fetchError.value = '';
}

function openConfirm() {
  confirmOpen.value = true;
}

function addSubscription() {
  if (!preview.value) return;
  vpnStore.addSubscription(importUrl.value.trim(), preview.value.name, preview.value.servers);
  confirmOpen.value = false;
  clearPreview();
}

function promptDelete(sub: SubscriptionGroup) {
  deleteTarget.value = { id: sub.id, name: sub.name, count: sub.servers.length };
}

function removeSubscription() {
  if (!deleteTarget.value) return;
  vpnStore.removeSubscription(deleteTarget.value.id);
  deleteTarget.value = null;
}

async function refreshSubscription(subId: string) {
  const sub = vpnStore.subscriptions.find((s) => s.id === subId);
  const name = sub?.name ?? '';
  const result = await vpnStore.refreshSubscription(subId);

  if (result.error) {
    pushToast('error', t('servers.refreshFailed'), `${name}: ${result.error}`, 5000);
    return;
  }

  if (result.added === 0 && result.removed === 0) {
    pushToast('info', t('servers.upToDate'), t('servers.upToDateDesc', { name }));
  } else {
    const parts: string[] = [];
    if (result.added > 0) parts.push(t('servers.newCount', { count: result.added }));
    if (result.removed > 0) parts.push(t('servers.removedCount', { count: result.removed }));
    pushToast('success', t('servers.subscriptionUpdated'), `${name}: ${parts.join(', ')}`);
  }
}

async function refreshAllSubscriptions() {
  const subs = [...vpnStore.subscriptions];
  let totalAdded = 0;
  let totalRemoved = 0;
  let failCount = 0;

  for (const sub of subs) {
    const result = await vpnStore.refreshSubscription(sub.id);
    if (result.error) {
      failCount++;
    } else {
      totalAdded += result.added;
      totalRemoved += result.removed;
    }
  }

  if (failCount > 0) {
    pushToast(
      'warning',
      t('servers.refreshCompletedErrors'),
      t('servers.refreshCompletedErrorsDesc', { count: failCount }),
      5000,
    );
  } else if (totalAdded === 0 && totalRemoved === 0) {
    pushToast('info', t('servers.allUpToDate'), t('servers.allUpToDateDesc'));
  } else {
    const summary = `${t('servers.newCount', { count: totalAdded })}, ${t('servers.removedCount', { count: totalRemoved })}`;
    pushToast('success', t('servers.allSubscriptionsUpdated'), summary);
  }
}

async function autoSelect() {
  await vpnStore.autoSelectSmartOrFast();
}
async function pingAll() {
  await vpnStore.measureLatencies();
}

const badgeEditId = ref<string | null>(null);

function currentBadge(id: string) {
  return (vpnStore.badgeBySubId || {})[id] || { icon: 'shield', color: '#a78bfa' };
}

function subBadgeGlyph(id: string) {
  return iconByKey(currentBadge(id).icon);
}

function subBadgeStyle(id: string) {
  const tone = currentBadge(id).color;
  return { color: tone, background: tintSoft(tone, 0.2), borderColor: tintSoft(tone, 0.45) };
}
</script>

<style scoped>
.page {
  display: flex;
  flex-direction: column;
  gap: 16px;
  max-width: 760px;
  margin: 0 auto;
}

.page-header {
  display: flex;
  flex-direction: column;
  gap: 3px;
}
.page-title {
  font-size: 21px;
  font-weight: 600;
  letter-spacing: -0.02em;
}
.page-sub {
  font-size: 12px;
  color: rgba(235, 238, 250, 0.55);
}

.hop-banner {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 11px 14px;
  border-radius: 14px;
  border: 1px solid rgba(143, 182, 255, 0.28);
  background: rgba(143, 182, 255, 0.08);
  backdrop-filter: blur(10px);
  color: #8fb6ff;
  font-size: 12px;
  line-height: 1.5;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.06);
}

.hop-banner .inline-icon {
  display: inline;
  vertical-align: -1px;
}

.add-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.url-row {
  display: flex;
  gap: 6px;
  align-items: center;
}

.url-input {
  flex: 1;
  padding: 10px 14px;
  border-radius: 14px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(10px);
  color: #eef1fb;
  font-size: 13px;
  font-family: var(--font-sans);
  outline: none;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.06);
  transition:
    border-color 180ms,
    background 180ms;
}

.url-input:focus {
  border-color: rgba(167, 139, 250, 0.55);
  background: rgba(255, 255, 255, 0.07);
}
.url-input:disabled {
  opacity: 0.5;
}
.url-input::placeholder {
  color: rgba(235, 238, 250, 0.35);
}

.ghost-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 38px;
  height: 38px;
  border-radius: 12px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(10px);
  color: rgba(235, 238, 250, 0.6);
  cursor: pointer;
  flex-shrink: 0;
  transition:
    color 160ms,
    background 160ms,
    transform 160ms;
}

.ghost-btn:hover {
  color: #fff;
  background: rgba(255, 255, 255, 0.1);
}
.ghost-btn:active {
  transform: scale(0.94);
}

.fetch-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  min-width: 74px;
  height: 38px;
  padding: 0 18px;
  border-radius: 12px;
  border: 1px solid rgba(167, 139, 250, 0.45);
  background: linear-gradient(180deg, rgba(167, 139, 250, 0.35), rgba(124, 92, 255, 0.25));
  color: #efeaff;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  flex-shrink: 0;
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.18),
    0 6px 18px rgba(124, 92, 255, 0.25);
  transition:
    opacity 160ms,
    transform 160ms,
    box-shadow 160ms;
}

.fetch-btn:hover:not(:disabled) {
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.22),
    0 8px 24px rgba(124, 92, 255, 0.35);
}
.fetch-btn:active:not(:disabled) {
  transform: scale(0.96);
}
.fetch-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
.fetch-error {
  font-size: 12px;
  color: #ff8a92;
  padding: 0 4px;
}

.spin {
  animation: rotate 0.8s linear infinite;
}
@keyframes rotate {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.preview-card {
  border-radius: 18px;
  border: 1px solid rgba(255, 255, 255, 0.09);
  background: rgba(255, 255, 255, 0.045);
  backdrop-filter: blur(13px);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.08),
    inset 0 -1px 0 rgba(0, 0, 0, 0.25),
    0 18px 44px rgba(0, 0, 0, 0.4);
  overflow: hidden;
}

.preview-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 16px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.07);
}

.preview-meta {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.preview-name {
  font-size: 14px;
  font-weight: 600;
}
.preview-count {
  font-size: 11px;
  color: rgba(235, 238, 250, 0.5);
}

.preview-list {
  list-style: none;
  max-height: 280px;
  overflow-y: auto;
  padding: 6px 0;
}

.preview-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 16px;
  animation: fade-in-up 220ms ease both;
}

@keyframes fade-in-up {
  from {
    opacity: 0;
    transform: translateY(8px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.preview-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
  flex: 1;
}
.preview-srv-name {
  font-size: 12.5px;
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.preview-proto {
  font-size: 10.5px;
  color: rgba(235, 238, 250, 0.4);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.add-sub-btn {
  width: calc(100% - 24px);
  margin: 6px 12px 12px;
  padding: 11px 0;
  border-radius: 12px;
  border: 1px solid rgba(167, 139, 250, 0.45);
  background: linear-gradient(180deg, rgba(167, 139, 250, 0.35), rgba(124, 92, 255, 0.25));
  color: #efeaff;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.18),
    0 6px 18px rgba(124, 92, 255, 0.25);
  transition:
    transform 160ms,
    box-shadow 160ms;
}

.add-sub-btn:active {
  transform: scale(0.98);
}

.confirm-overlay {
  position: fixed;
  inset: 0;
  z-index: 50;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(3, 4, 8, 0.55);
  backdrop-filter: blur(8px);
}

.confirm-modal {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  width: min(360px, calc(100vw - 48px));
  padding: 24px 22px 18px;
  border-radius: 22px;
  border: 1px solid rgba(255, 255, 255, 0.11);
  background: rgba(20, 22, 32, 0.75);
  backdrop-filter: blur(15px);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.1),
    0 30px 80px rgba(0, 0, 0, 0.6);
  text-align: center;
}

.confirm-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 52px;
  height: 52px;
  border-radius: 50%;
  background: rgba(167, 139, 250, 0.14);
  border: 1px solid rgba(167, 139, 250, 0.35);
  color: #a78bfa;
}

.confirm-icon--danger {
  background: rgba(255, 138, 146, 0.12);
  border-color: rgba(255, 138, 146, 0.35);
  color: #ff8a92;
}

.confirm-title {
  font-size: 15px;
  font-weight: 600;
}
.confirm-desc {
  font-size: 12.5px;
  line-height: 1.5;
  color: rgba(235, 238, 250, 0.6);
}
.confirm-actions {
  display: flex;
  gap: 8px;
  width: 100%;
  margin-top: 6px;
}

.confirm-cancel,
.confirm-ok {
  flex: 1;
  padding: 10px 0;
  border-radius: 12px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition:
    transform 150ms,
    background 150ms;
}

.confirm-cancel {
  border: 1px solid rgba(255, 255, 255, 0.12);
  background: rgba(255, 255, 255, 0.06);
  color: rgba(235, 238, 250, 0.75);
}

.confirm-ok {
  border: 1px solid rgba(167, 139, 250, 0.45);
  background: linear-gradient(180deg, rgba(167, 139, 250, 0.4), rgba(124, 92, 255, 0.28));
  color: #efeaff;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.18);
}

.confirm-ok--danger {
  border-color: rgba(255, 138, 146, 0.45);
  background: linear-gradient(180deg, rgba(255, 138, 146, 0.32), rgba(220, 70, 90, 0.24));
  color: #ffe9ec;
}

.confirm-cancel:active,
.confirm-ok:active {
  transform: scale(0.97);
}

.toolbar {
  display: flex;
  gap: 6px;
  align-items: center;
}

.search-bar {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 12px;
  height: 36px;
  border-radius: 999px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(10px);
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.06);
}

.search-icon {
  color: rgba(235, 238, 250, 0.4);
  flex-shrink: 0;
}

.search-input {
  flex: 1;
  border: none;
  background: transparent;
  outline: none;
  color: #eef1fb;
  font-size: 12.5px;
  font-family: var(--font-sans);
}

.search-input::placeholder {
  color: rgba(235, 238, 250, 0.35);
}

.tool-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  height: 36px;
  padding: 0 13px;
  border-radius: 999px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(10px);
  color: rgba(235, 238, 250, 0.75);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  flex-shrink: 0;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.06);
  transition:
    background 160ms,
    color 160ms,
    transform 160ms;
}

.tool-btn:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.1);
  color: #fff;
}
.tool-btn:active:not(:disabled) {
  transform: scale(0.95);
}
.tool-btn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.tool-btn--accent {
  border-color: rgba(167, 139, 250, 0.4);
  background: rgba(167, 139, 250, 0.14);
  color: #cdbcff;
}

.tool-btn--accent:hover:not(:disabled) {
  background: rgba(167, 139, 250, 0.22);
  color: #e6dcff;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  padding: 48px 20px;
  border-radius: 18px;
  border: 1px dashed rgba(255, 255, 255, 0.12);
  background: rgba(255, 255, 255, 0.03);
  color: rgba(235, 238, 250, 0.6);
  font-size: 13px;
  text-align: center;
}

.empty-icon {
  color: rgba(235, 238, 250, 0.3);
  margin-bottom: 4px;
}
.empty-sub {
  font-size: 11.5px;
  color: rgba(235, 238, 250, 0.4);
}

.subs-strip {
  position: relative;
  z-index: 30;
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.sub-chip {
  display: flex;
  align-items: center;
  gap: 8px;
  height: 34px;
  padding: 0 6px 0 13px;
  border-radius: 999px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(10px);
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.06);
  font-size: 12px;
}

.sub-chip-name {
  font-weight: 600;
  max-width: 180px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.sub-chip-count {
  font-size: 10.5px;
  color: rgba(235, 238, 250, 0.45);
}

.expiry-badge {
  font-size: 10px;
  font-weight: 600;
  padding: 2px 7px;
  border-radius: 999px;
}

.expiry-badge--ok {
  background: rgba(94, 230, 154, 0.12);
  color: #5ee69a;
}
.expiry-badge--warning {
  background: rgba(240, 211, 106, 0.12);
  color: #f0d36a;
}
.expiry-badge--critical {
  background: rgba(255, 159, 107, 0.14);
  color: #ff9f6b;
}
.expiry-badge--expired {
  background: rgba(255, 138, 146, 0.14);
  color: #ff8a92;
}

.traffic-badge {
  font-size: 10px;
  color: rgba(235, 238, 250, 0.45);
}

.chip-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border-radius: 50%;
  border: none;
  background: rgba(255, 255, 255, 0.06);
  color: rgba(235, 238, 250, 0.55);
  cursor: pointer;
  transition:
    background 150ms,
    color 150ms,
    transform 150ms;
}

.chip-btn:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.12);
  color: #fff;
}
.chip-btn:active:not(:disabled) {
  transform: scale(0.9);
}
.chip-btn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.chip-btn--danger:hover {
  background: rgba(255, 138, 146, 0.16);
  color: #ff8a92;
}

.preview-slide-enter-active {
  transition: all 260ms cubic-bezier(0.34, 1.4, 0.64, 1);
}
.preview-slide-leave-active {
  transition: all 160ms ease;
}
.preview-slide-enter-from,
.preview-slide-leave-to {
  opacity: 0;
  transform: translateY(-10px) scale(0.98);
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 180ms ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
.sub-chip {
  position: relative;
}

.sub-badge {
  display: grid;
  place-items: center;
  width: 22px;
  height: 22px;
  border-radius: 7px;
  border: 1px solid transparent;
  cursor: pointer;
  flex-shrink: 0;
  transition: transform 0.14s ease;
}

.sub-badge:hover {
  transform: scale(1.08);
}

.badge-pop {
  position: absolute;
  top: calc(100% + 8px);
  left: 0;
  z-index: 50;
  width: 232px;
  padding: 10px;
  border-radius: 14px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  background: rgba(16, 18, 28, 0.96);
  backdrop-filter: blur(13px);
  box-shadow: 0 18px 40px rgba(0, 0, 0, 0.5);
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.badge-pop-title {
  margin: 0;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: rgba(235, 238, 250, 0.45);
}

.badge-icons {
  display: grid;
  grid-template-columns: repeat(8, 1fr);
  gap: 4px;
}

.badge-cell {
  display: grid;
  place-items: center;
  height: 24px;
  border-radius: 7px;
  border: 1px solid transparent;
  background: rgba(255, 255, 255, 0.05);
  color: rgba(235, 238, 250, 0.7);
  cursor: pointer;
}

.badge-cell--on {
  border-color: #a78bfa;
  color: #e6dcff;
  background: rgba(167, 139, 250, 0.18);
}

.badge-colors {
  display: grid;
  grid-template-columns: repeat(12, 1fr);
  gap: 4px;
}

.badge-dot {
  height: 16px;
  border-radius: 5px;
  border: 2px solid transparent;
  cursor: pointer;
  padding: 0;
}

.badge-dot--on {
  border-color: rgba(255, 255, 255, 0.85);
}

.badge-done {
  height: 28px;
  border-radius: 9px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  background: rgba(255, 255, 255, 0.06);
  color: inherit;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
}

.pop-enter-active,
.pop-leave-active {
  transition:
    opacity 0.15s ease,
    transform 0.15s ease;
}

.pop-enter-from,
.pop-leave-to {
  opacity: 0;
  transform: translateY(-6px) scale(0.97);
}

.fav-strip {
  display: flex;
  flex-direction: column;
  gap: 9px;
  padding: 12px 14px;
  border-radius: 16px;
  border: 1px solid rgba(255, 212, 122, 0.22);
  background:
    linear-gradient(
      170deg,
      rgba(255, 200, 90, 0.07),
      rgba(255, 200, 90, 0.02)
    ),
    rgba(12, 14, 20, 0.35);
}

.fav-head {
  display: flex;
  align-items: center;
  gap: 7px;
  font-size: 11.5px;
  font-weight: 600;
  letter-spacing: 0.02em;
  color: #ffd47a;
}

.fav-star-icon {
  color: #ffd47a;
  fill: currentColor;
}

.fav-auto-btn {
  margin-left: auto;
  display: inline-flex;
  align-items: center;
  gap: 5px;
  height: 24px;
  padding: 0 10px;
  border-radius: 999px;
}

.fav-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 7px;
}

.fav-chip {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  max-width: 100%;
  height: 32px;
  padding: 0 12px 0 8px;
  border-radius: 999px;
  border: 1px solid rgba(255, 255, 255, 0.11);
  background: rgba(255, 255, 255, 0.05);
  color: var(--foreground);
  cursor: pointer;
  overflow: hidden;
  transition:
    border-color 220ms ease,
    color 200ms ease,
    background 220ms ease,
    transform 160ms cubic-bezier(0.34, 1.56, 0.64, 1);
}

.fav-chip:hover:not(:disabled) {
  transform: translateY(-1px);
  border-color: rgba(255, 255, 255, 0.24);
}

.fav-chip:disabled {
  opacity: 0.45;
  cursor: default;
}

.fav-chip--on {
  border-color: color-mix(in oklab, var(--success) 45%, transparent);
  background: linear-gradient(
    180deg,
    color-mix(in oklab, var(--success) 15%, transparent),
    color-mix(in oklab, var(--success) 4%, transparent)
  );
  color: var(--success);
}

.fav-chip-name {
  font-size: 11.5px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 150px;
}

.fav-chip .ping-badge {
  opacity: 0.85;
}

.fav-chip-check {
  color: inherit;
}

.fav-reveal-enter-active {
  transition:
    opacity 260ms ease,
    transform 320ms cubic-bezier(0.34, 1.3, 0.64, 1);
}

.fav-reveal-leave-active {
  transition:
    opacity 180ms ease,
    transform 180ms ease;
}

.fav-reveal-enter-from,
.fav-reveal-leave-to {
  opacity: 0;
  transform: translateY(-8px);
}
</style>
