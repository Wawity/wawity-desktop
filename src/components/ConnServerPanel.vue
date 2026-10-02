<template>
  <div class="csp" data-sensitive>
    <div class="csp-head">
      <div class="csp-search">
        <Search :size="13" class="csp-search-icon" aria-hidden="true" />
        <input
          v-model="query"
          type="text"
          :placeholder="t('servers.searchServers')"
          class="csp-search-input"
          spellcheck="false"
        />
        <button v-if="query" type="button" class="csp-search-clear" @click="query = ''">
          <X :size="12" aria-hidden="true" />
        </button>
      </div>
      <button
        type="button"
        class="csp-act"
        :disabled="vpnStore.latencyLoading"
        :title="t('servers.pingAllTitle')"
        @click="vpnStore.measureLatencies()"
      >
        <Loader2 v-if="vpnStore.latencyLoading" :size="13" class="csp-act-spin" />
        <Radio v-else :size="13" aria-hidden="true" />
      </button>
      <button
        type="button"
        class="csp-act"
        :disabled="refreshing"
        :title="t('servers.refreshAllTitle')"
        @click="refreshAll"
      >
        <Loader2 v-if="refreshing" :size="13" class="csp-act-spin" />
        <RotateCw v-else :size="13" aria-hidden="true" />
      </button>
      <span class="csp-count mono" v-text="t('servers.serversFound', { count: totalCount })" />
    </div>

    <div class="csp-body">
      <template v-if="buckets.length">
        <section v-for="bucket in buckets" :key="bucket.code" class="csp-group">
          <header class="csp-group-head">
            <span v-if="bySub" class="csp-group-badge" :style="subBadgeStyle(bucket.code)">
              <component :is="subBadgeGlyph(bucket.code)" :size="11" aria-hidden="true" />
            </span>
            <CountryFlag v-else :code="bucket.code" :width="16" :height="11" />
            <span class="csp-group-name" v-text="bucket.name" />
            <span class="csp-group-rule" aria-hidden="true" />
            <span class="csp-group-count mono" v-text="bucket.servers.length" />
          </header>
          <ul class="csp-list">
            <li
              v-for="srv in bucket.servers"
              :key="srv.id"
              class="csp-row"
              :class="{
                'csp-row--selected': vpnStore.selectedServerId === srv.id,
                'csp-row--expired': vpnStore.isServerExpired(srv.id),
                'csp-row--disabled': switching || vpnStore.loading,
              }"
              @click="hop(srv.id)"
            >
              <CountryFlag :code="srv.countryCode ?? 'UN'" :width="20" :height="14" class="csp-flag" />
              <span class="csp-row-name" v-text="srv.name" />
              <span class="csp-row-right">
                <button
                  type="button"
                  class="csp-fav"
                  :class="{ 'csp-fav--on': vpnStore.isFavorite(srv.id) }"
                  :title="t('servers.favTitle')"
                  @click.stop="vpnStore.toggleFavorite(srv.id)"
                >
                  <Star :size="12" />
                </button>
                <span
                  class="csp-ping mono"
                  :class="pingTier(srv.latencyMs)"
                >
                  <template v-if="srv.latencyMs !== null && srv.latencyMs !== undefined">
                    {{ srv.latencyMs }}<span class="csp-ping-unit">ms</span>
                  </template>
                  <template v-else>
                    <span class="csp-ping-none">&mdash;</span>
                  </template>
                </span>
                <Transition name="csp-check">
                  <Check v-if="vpnStore.selectedServerId === srv.id" :size="15" class="csp-check" />
                </Transition>
              </span>
            </li>
          </ul>
        </section>
      </template>
      <div v-else class="csp-empty" v-text="t('servers.noServersMatch', { query })" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { iconByKey, tintSoft } from '../lib/subicons';
import { ref, computed } from 'vue';
import { Search, X, Star, Check, Loader2, Radio, RotateCw } from '../lib/appIcons';
import { useVpnStore } from '../stores/vpn';
import { useNotifications } from '../composables/useNotifications';
import { t } from '../i18n';
import {
  groupServers,
  groupServersBySubscription,
  pingTier,
  type CountryBucket,
  type SubBucket,
} from '../lib/geo';
import CountryFlag from './CountryFlag.vue';

const vpnStore = useVpnStore();
const { pushToast } = useNotifications();
const query = ref('');
const switching = ref(false);
const refreshing = ref(false);

async function refreshAll() {
  if (refreshing.value) return;
  refreshing.value = true;
  try {
    const r = await vpnStore.refreshAllSubscriptions();
    if (r.failed > 0) {
      pushToast(
        'warning',
        t('servers.refreshCompletedErrors'),
        t('servers.refreshCompletedErrorsDesc', { count: r.failed }),
        5000,
      );
    } else if (r.added === 0 && r.removed === 0) {
      pushToast('info', t('servers.allUpToDate'), t('servers.allUpToDateDesc'));
    } else {
      const parts: string[] = [];
      if (r.added > 0) parts.push(t('servers.newCount', { count: r.added }));
      if (r.removed > 0) parts.push(t('servers.removedCount', { count: r.removed }));
      pushToast('success', t('servers.allSubscriptionsUpdated'), parts.join(', '));
    }
  } finally {
    refreshing.value = false;
  }
}

const bySub = computed(() => vpnStore.settings.server_group === 'subscription');

type AnyBucket = Omit<CountryBucket, 'lat' | 'lon'> | SubBucket;

const buckets = computed<AnyBucket[]>(() =>
  bySub.value
    ? groupServersBySubscription(vpnStore.subscriptions, query.value, vpnStore.hiddenSubIds)
    : groupServers(
        vpnStore.subscriptions,
        query.value,
        vpnStore.settings.language,
        vpnStore.hiddenSubIds,
      ),
);

const totalCount = computed(() =>
  buckets.value.reduce((a, b) => a + b.servers.length, 0),
);

async function hop(id: string) {
  if (switching.value || vpnStore.loading) return;
  if (vpnStore.isServerExpired(id)) return;
  switching.value = true;
  try {
    await vpnStore.switchServer(id);
  } finally {
    switching.value = false;
  }
}

function subBadgeGlyph(id: string) {
  const badge = (vpnStore.badgeBySubId || {})[id];
  return iconByKey(badge ? badge.icon : 'shield');
}

function subBadgeStyle(id: string) {
  const badge = (vpnStore.badgeBySubId || {})[id];
  const tone = badge ? badge.color : '#ff7a5c';
  return { color: tone, background: tintSoft(tone, 0.2) };
}
</script>

<style scoped>
/* A hairline frame rather than a glass card. The old panel also clipped its
   own list with overflow: hidden, which is why the last row looked cut off
   mid-way. The list scrolls instead, and the frame is only a border. */
.csp {
  display: flex;
  flex-direction: column;
  height: 100%;
  max-height: 560px;
  min-height: 320px;
  border-radius: 14px;
  border: 1px solid var(--border);
  background: transparent;
  backdrop-filter: none;
  -webkit-backdrop-filter: none;
  box-shadow: none;
  overflow: hidden;
}

.csp-head {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 14px 10px;
}

.csp-search {
  position: relative;
  display: flex;
  align-items: center;
  flex: 1;
  min-width: 0;
  height: 34px;
  padding: 0 2px;
  border-radius: 0;
  border: 0;
  border-bottom: 1px solid var(--border);
  background: transparent;
  transition: border-color 200ms ease;
}

.csp-search:focus-within {
  border-color: color-mix(in oklch, var(--foreground) 40%, transparent);
}

.csp-search-icon {
  color: rgba(235, 238, 250, 0.4);
  flex-shrink: 0;
}

.csp-search-input {
  flex: 1;
  min-width: 0;
  border: none;
  background: transparent;
  outline: none;
  color: #eef1fb;
  font-size: 12px;
  font-family: var(--font-sans);
  padding: 0 7px;
}

.csp-search-input::placeholder {
  color: rgba(235, 238, 250, 0.32);
}

.csp-search-clear {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  border-radius: 9999px;
  border: none;
  background: color-mix(in oklch, var(--foreground) 8%, transparent);
  color: var(--muted-foreground);
  cursor: pointer;
  flex-shrink: 0;
}

.csp-search-clear:hover {
  color: var(--foreground);
}

.csp-count {
  font-family: var(--font-label);
  font-size: 9px;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  color: var(--muted-foreground);
  opacity: 0.7;
  white-space: nowrap;
  flex-shrink: 0;
}

.csp-act {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: 9999px;
  border: 1px solid var(--border);
  background: transparent;
  color: var(--muted-foreground);
  cursor: pointer;
  flex-shrink: 0;
  transition:
    background 200ms,
    color 200ms,
    border-color 200ms;
}

.csp-act:hover:not(:disabled) {
  background: color-mix(in oklch, var(--foreground) 6%, transparent);
  border-color: color-mix(in oklch, var(--foreground) 24%, transparent);
  color: var(--foreground);
}

.csp-act:active:not(:disabled) {
  background: rgba(255, 255, 255, 0.1);
}

.csp-act:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.csp-act-spin {
  animation: csp-rotate 0.8s linear infinite;
}

@keyframes csp-rotate {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.csp-body {
  flex: 1;
  overflow-y: auto;
  padding: 2px 10px 14px;
}

.csp-group + .csp-group {
  margin-top: 16px;
}

.csp-group-head {
  position: sticky;
  top: 0;
  z-index: 2;
  display: flex;
  align-items: center;
  gap: 9px;
  padding: 12px 6px 10px;
  /* Sticky needs a backdrop, but an opaque fill would be a black bar over
     the scene. */
  background: color-mix(in oklch, var(--background) 72%, transparent);
  backdrop-filter: blur(10px);
}

.csp-group-badge {
  display: grid;
  place-items: center;
  width: 20px;
  height: 20px;
  border-radius: 6px;
  flex-shrink: 0;
}

.csp-group-name {
  flex: 0 1 auto;
  min-width: 0;
  font-family: var(--font-label);
  font-size: 9px;
  font-weight: 400;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--muted-foreground);
  opacity: 0.8;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.csp-group-rule {
  flex: 1 0 12px;
  height: 1px;
  background: var(--border);
}

.csp-group-count {
  font-size: 9.5px;
  color: var(--muted-foreground);
  opacity: 0.6;
  min-width: 16px;
  text-align: right;
}

.csp-list {
  list-style: none;
  display: flex;
  flex-direction: column;
}

/* Rows between rules, selection as an edge rule, no rounded chips. */
.csp-row {
  position: relative;
  display: flex;
  align-items: center;
  gap: 12px;
  height: 48px;
  padding: 0 12px 0 14px;
  border-bottom: 1px solid var(--border);
  background: transparent;
  cursor: pointer;
  transition: background 180ms ease;
}

.csp-row:hover {
  background: color-mix(in oklch, var(--foreground) 4%, transparent);
}

.csp-row--selected {
  background: color-mix(in oklab, var(--success) 7%, transparent);
  box-shadow: inset 2px 0 0 var(--success);
}

.csp-row--selected:hover {
  background: color-mix(in oklab, var(--success) 11%, transparent);
}

.csp-row--disabled {
  pointer-events: none;
  opacity: 0.55;
}

.csp-row--expired {
  opacity: 0.35;
  filter: grayscale(0.7);
  pointer-events: none;
}

.csp-flag {
  flex-shrink: 0;
  border-radius: 2.5px;
}

.csp-row-name {
  flex: 1;
  min-width: 0;
  font-size: 13px;
  font-weight: 500;
  color: rgba(238, 241, 251, 0.82);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  transition: color 140ms ease;
}

.csp-row--selected .csp-row-name {
  color: #fff;
  font-weight: 600;
}

.csp-row-right {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-shrink: 0;
}

.csp-fav {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  flex-shrink: 0;
  border-radius: 5px;
  border: none;
  background: transparent;
  color: rgba(235, 238, 250, 0.35);
  opacity: 0;
  cursor: pointer;
  transition:
    color 150ms ease,
    background 150ms ease,
    opacity 180ms ease;
}

.csp-row:hover .csp-fav {
  opacity: 1;
}

.csp-fav:hover {
  color: #ffd47a;
  background: rgba(255, 200, 90, 0.12);
}

.csp-fav--on {
  opacity: 1;
  color: #ffd47a;
}

.csp-fav--on > svg {
  fill: currentColor;
}

@media (hover: none) {
  .csp-fav {
    opacity: 1;
  }
}

/* the hero: oversized telemetry numerals */
.csp-ping {
  font-size: 19px;
  font-weight: 650;
  letter-spacing: -0.02em;
  font-variant-numeric: tabular-nums;
  line-height: 1;
  color: rgba(238, 241, 251, 0.9);
  min-width: 56px;
  text-align: right;
  white-space: nowrap;
  transition: color 200ms ease;
}

.csp-ping-unit {
  font-size: 9.5px;
  font-weight: 500;
  letter-spacing: 0.04em;
  margin-left: 3px;
  opacity: 0.55;
}

.csp-ping.tier-good {
  color: #5ee69a;
}
.csp-ping.tier-ok {
  color: #f0d36a;
}
.csp-ping.tier-slow {
  color: #ff9f6b;
}
.csp-ping.tier-bad {
  color: #ff8a92;
}

.csp-ping-none {
  font-size: 15px;
  font-weight: 400;
  opacity: 0.22;
}

.csp-check {
  color: #5ee69a;
  flex-shrink: 0;
}

.csp-check-enter-active {
  transition: all 160ms cubic-bezier(0.34, 1.56, 0.64, 1);
}
.csp-check-leave-active {
  transition: all 100ms ease;
}
.csp-check-enter-from,
.csp-check-leave-to {
  opacity: 0;
  transform: scale(0.4);
}

.csp-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 140px;
  padding: 20px;
  font-size: 12.5px;
  color: var(--muted-foreground);
  text-align: center;
}

.mono {
  font-family: var(--font-mono);
}

@media (prefers-reduced-motion: reduce) {
  .csp-ping {
    transition: none;
  }
  .csp-act-spin {
    animation-duration: 1.6s;
  }
}
</style>
