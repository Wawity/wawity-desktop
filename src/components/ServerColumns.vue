<template>
  <div class="duo" data-sensitive>
    <div class="duo-left">
      <p class="land-kicker" v-text="regionKicker" />
      <button
        v-for="c in buckets"
        :key="c.code"
        type="button"
        class="land-row"
        :class="{ 'land-row--active': picked === c.code }"
        @click="picked = c.code"
      >
        <span v-if="bySub" class="land-badge" :style="subBadgeStyle(c.code)">
          <component :is="subBadgeGlyph(c.code)" :size="13" aria-hidden="true" />
        </span>
        <CountryFlag v-else :code="c.code" :size="24" />
        <span class="land-name" v-text="c.name"></span>
        <span class="land-count mono" v-text="c.servers.length"></span>
        <span class="land-dot-wrap" :data-tip="dotTip(c)">
          <span class="land-dot" :class="pingTier(c.best)"></span>
        </span>
      </button>
      <div v-if="buckets.length === 0" class="duo-empty" v-text="noMatchText"></div>
    </div>
    <div class="duo-right">
      <p class="land-kicker land-kicker--right" v-text="pickedLabel" />
      <Transition name="swap" mode="out-in">
        <ul v-if="current" :key="current.code" class="duo-list">
          <li
            v-for="srv in current.servers"
            :key="srv.id"
            class="duo-srv"
            :class="{
              'duo-srv--selected': vpnStore.selectedServerId === srv.id,
              'duo-srv--disabled': switching || vpnStore.loading,
              'duo-srv--expired': vpnStore.isServerExpired(srv.id),
              'duo-srv--dead': vpnStore.hiddenServers.includes(srv.id),
            }"
            @click="hop(srv.id)"
          >
            <div class="duo-srv-text">
              <span class="duo-srv-name">
                <CountryFlag
                  v-if="bySub"
                  :code="srv.countryCode ?? 'UN'"
                  :width="16"
                  :height="11"
                  class="srv-flag"
                />
                <span v-else class="srv-badge" :style="badgeStyle(srv.id)">
                  <component :is="badgeGlyph(srv.id)" :size="10" aria-hidden="true" />
                </span>
                <span class="srv-label" v-text="srv.name"></span>
              </span>
              <span class="duo-srv-meta mono" v-text="srvMeta(srv)"></span>
              <span
                v-if="srv.udp && srv.udp !== 'tcp'"
                class="udp-chip"
                :class="udpClass(srv)"
                :title="udpTip(srv)"
                v-text="udpLabel(srv)"
              ></span>
            </div>
            <div class="duo-srv-right">
              <button
                type="button"
                class="fav-btn"
                :class="{ 'fav-btn--on': vpnStore.isFavorite(srv.id) }"
                :title="t('servers.favTitle')"
                @click.stop="vpnStore.toggleFavorite(srv.id)"
              >
                <Star :size="12" />
              </button>
              <button
                v-if="vpnStore.settings.multihop_enabled"
                type="button"
                class="entry-btn"
                :class="{ 'entry-btn--active': vpnStore.selectedEntryServerId === srv.id }"
                :title="t('servers.entryTitle')"
                @click.stop="
                  vpnStore.selectEntryServer(
                    vpnStore.selectedEntryServerId === srv.id ? null : srv.id,
                  )
                "
              >
                <Shuffle :size="12" />
              </button>
              <button
                type="button"
                class="copy-ip-btn"
                :class="{ 'copy-ip-btn--done': copiedId === srv.id }"
                :title="t('connection.copyIpTitle')"
                @click.stop="copyServerIp(srv)"
              >
                <Check v-if="copiedId === srv.id" :size="12" />
                <Copy v-else :size="12" />
              </button>
              <span
                v-if="srv.latencyMs !== null && srv.latencyMs !== undefined"
                class="ping-badge"
                :class="pingTier(srv.latencyMs)"
                v-text="pingText(srv.latencyMs)"
              ></span>
              <span v-else-if="vpnStore.latencyLoading" class="ping-badge tier-none">…</span>
              <Transition name="check-pop">
                <Check v-if="vpnStore.selectedServerId === srv.id" :size="14" class="duo-check" />
              </Transition>
            </div>
          </li>
        </ul>
        <div v-else class="duo-hint" v-text="bySub ? t('servers.pickSubscription') : t('servers.pickCountry')"></div>
      </Transition>
    </div>
  </div>
</template>

<script setup lang="ts">
import { iconByKey, tintSoft } from '../lib/subicons';
import { ref, computed, watchEffect } from 'vue';
import { Shuffle, Check, Copy, Star } from '../lib/appIcons';
import { useVpnStore } from '../stores/vpn';
import { showCopyHint } from '../composables/useCopyHint';
import { writeText } from '@tauri-apps/api/clipboard';
import { t } from '../i18n';
import {
  groupServers,
  groupServersBySubscription,
  pingTier,
  type ServerEntry,
  type CountryBucket,
  type SubBucket,
} from '../lib/geo';
import CountryFlag from './CountryFlag.vue';

const props = defineProps<{ query: string }>();

const vpnStore = useVpnStore();
const picked = ref<string | null>(null);
const switching = ref(false);
const copiedId = ref<string | null>(null);
let copyResetTimer = 0;

const bySub = computed(() => vpnStore.settings.server_group === 'subscription');
// The list shows either regions or subscriptions, so label it accordingly
// instead of always calling it one of them.
const regionKicker = computed(() =>
  bySub.value ? t('servers.kickerSubs') : t('servers.kickerRegions'),
);

type AnyBucket = Omit<CountryBucket, 'lat' | 'lon'> | SubBucket;

async function copyServerIp(srv: ServerEntry) {
  try {
    await writeText(srv.server);
    copiedId.value = srv.id;
    showCopyHint(t('connection.ipCopied'));
    if (copyResetTimer) window.clearTimeout(copyResetTimer);
    copyResetTimer = window.setTimeout(() => {
      copiedId.value = null;
    }, 1600);
  } catch {}
}

const buckets = computed<AnyBucket[]>(() =>
  bySub.value
    ? groupServersBySubscription(
        vpnStore.subscriptions,
        props.query,
        vpnStore.hiddenSubIds,
      )
    : groupServers(
        vpnStore.subscriptions,
        props.query,
        vpnStore.settings.language,
        vpnStore.hiddenSubIds,
      ),
);

const current = computed(() => buckets.value.find((b) => b.code === picked.value) ?? null);

// Both columns carry a kicker so the two sides start on the same line and
// the user can see what the right-hand list is showing.
const pickedLabel = computed(() => {
  const c = current.value;
  if (!c) return noMatchText.value;
  return c.servers.length > 0
    ? `${c.name} · ${c.servers.length}`
    : c.name;
});

const noMatchText = computed(() => t('servers.noServersMatch', { query: props.query }));

watchEffect(() => {
  if (!buckets.value.some((b) => b.code === picked.value)) {
    picked.value = buckets.value.length > 0 ? buckets.value[0].code : null;
  }
});

function srvMeta(srv: ServerEntry): string {
  return `${srv.protocol} · ${srv.server}`;
}

/* Transport badge, shown only when the node can carry UDP. Most panels hand
   out TCP by default, so flagging every server produced a wall of identical
   labels that said nothing. Silence on a TCP node and a mark on a capable
   one makes the badge mean something. */
function udpClass(srv: ServerEntry): string {
  return `udp-chip--${srv.udp === 'full' ? 'full' : 'ok'}`;
}

function udpLabel(srv: ServerEntry): string {
  if (srv.udp === 'full') return 'UDP';
  return `UDP/${srv.transport || 'quic'}`;
}

function udpTip(srv: ServerEntry): string {
  return srv.udp === 'full' ? t('servers.udpFull') : t('servers.udpOk');
}

function pingText(ms: number): string {
  return `${ms}ms`;
}

function dotTip(c: AnyBucket): string {
  return c.best !== null && c.best !== undefined ? `${c.best}ms` : t('servers.pingNoData');
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

function badgeGlyph(id: string) {
  const badge = vpnStore.badgeByServerId[id];
  return iconByKey(badge ? badge.icon : 'shield');
}

function badgeStyle(id: string) {
  const badge = vpnStore.badgeByServerId[id];
  const tone = badge ? badge.color : '#ff7a5c';
  return { color: tone, background: tintSoft(tone, 0.18) };
}
</script>

<style scoped>
.duo {
  display: flex;
  min-height: 320px;
  max-height: 460px;
  border-radius: 0;
  border: 0;
  border-top: 1px solid var(--border);
  border-bottom: 1px solid var(--border);
  background: transparent;
  backdrop-filter: none;
  -webkit-backdrop-filter: none;
  box-shadow: none;
  overflow: hidden;
}

.duo-left {
  width: 224px;
  flex-shrink: 0;
  overflow-y: auto;
  padding: 0 14px 0 0;
  border-right: 1px solid var(--border);
  background: transparent;
  display: flex;
  flex-direction: column;
  gap: 0;
}

/* Same language as the landing page: an Unbounded kicker over a hairline
   list, no filled panel and no rounded rows. The old version put a rounded
   rect with its own hover shift on every region, which turned a navigation
   list into a stack of chips. */
.land-kicker {
  position: sticky;
  top: 0;
  z-index: 2;
  /* Same box on both sides, so the two lists line up under their labels. */
  padding: 15px 2px 11px;
  /* Sticky needs something behind it, but an opaque fill would be a black bar
     over the live scene, so it gets glass instead. */
  background: color-mix(in oklch, var(--background) 72%, transparent);
  backdrop-filter: blur(10px);
  font-family: var(--font-label);
  font-size: 9px;
  font-weight: 400;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--muted-foreground);
  opacity: 0.75;
}

.land-row {
  position: relative;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 11px 10px 11px 12px;
  border-radius: 0;
  border: 0;
  border-bottom: 1px solid var(--border);
  background: transparent;
  color: var(--foreground);
  cursor: pointer;
  text-align: left;
  transition: background 180ms ease;
  content-visibility: auto;
  contain-intrinsic-size: auto 44px;
}

.land-row:hover {
  background: color-mix(in oklch, var(--foreground) 4%, transparent);
  transform: none;
}

.land-badge {
  display: grid;
  place-items: center;
  width: 24px;
  height: 24px;
  border-radius: 8px;
  flex-shrink: 0;
}

.land-row--active {
  background: transparent;
  border-color: var(--border);
  box-shadow: none;
}
/* The active region is marked by a rule on the panel edge rather than a
   filled pill, so the list stays a list. */
.land-row--active::before {
  content: '';
  position: absolute;
  /* Inside the panel: the old -2px sat under the container's overflow:hidden. */
  left: 0;
  top: 0;
  bottom: 0;
  width: 2px;
  background: var(--foreground);
}

.land-name {
  flex: 1;
  min-width: 0;
  font-family: var(--font-display);
  font-size: 13.5px;
  font-weight: 400;
  letter-spacing: -0.015em;
  color: var(--muted-foreground);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  transition: color 180ms ease;
}
.land-row--active .land-name {
  color: var(--foreground);
}

.land-count {
  font-family: var(--font-mono);
  font-size: 10px;
  color: var(--muted-foreground);
  opacity: 0.7;
}

.land-dot-wrap {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  flex-shrink: 0;
  border-radius: 50%;
}

.land-dot-wrap::after {
  content: attr(data-tip);
  position: absolute;
  right: calc(100% + 6px);
  top: 50%;
  transform: translateY(-50%) translateX(-4px);
  padding: 4px 8px;
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  background: rgba(16, 18, 28, 0.96);
  color: var(--foreground);
  font-family: var(--font-mono);
  font-size: 10.5px;
  font-weight: 500;
  white-space: nowrap;
  opacity: 0;
  pointer-events: none;
  transition:
    opacity 140ms ease,
    transform 140ms ease;
  z-index: 5;
}

.land-dot-wrap:hover::after {
  opacity: 1;
  transform: translateY(-50%) translateX(0);
}

.land-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  flex-shrink: 0;
}
.land-dot.tier-good {
  background: #5ee69a;
  box-shadow: 0 0 6px rgba(94, 230, 154, 0.7);
}
.land-dot.tier-ok {
  background: #f0d36a;
  box-shadow: 0 0 6px rgba(240, 211, 106, 0.6);
}
.land-dot.tier-slow {
  background: #ff9f6b;
  box-shadow: 0 0 6px rgba(255, 159, 107, 0.6);
}
.land-dot.tier-bad {
  background: #ff8a92;
  box-shadow: 0 0 6px rgba(255, 138, 146, 0.6);
}
.land-dot.tier-none {
  background: rgba(255, 255, 255, 0.18);
}

.duo-right {
  flex: 1;
  overflow-y: auto;
  min-width: 0;
  padding: 0 0 0 14px;
  display: flex;
  flex-direction: column;
  gap: 0;
}

.duo-list {
  list-style: none;
  padding: 0;
}

/* Server rows follow the region rows: a hairline list, no padding shift on
   hover (the old one nudged padding-left, which made the whole list twitch
   under the cursor) and the selection carried by an edge rule. */
.duo-srv {
  position: relative;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border);
  cursor: pointer;
  transition: background 180ms ease;
  content-visibility: auto;
  contain-intrinsic-size: auto 52px;
}

.duo-srv:hover {
  background: color-mix(in oklch, var(--foreground) 4%, transparent);
}

.duo-srv--selected {
  background: transparent;
  box-shadow: none;
}
.duo-srv--selected::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 2px;
  background: var(--foreground);
}
.duo-srv--selected .duo-srv-name {
  color: var(--foreground);
}

.duo-srv--dead {
  opacity: 0.45;
}

.duo-srv--dead .srv-label {
  text-decoration: line-through;
  text-decoration-color: rgba(255, 108, 120, 0.6);
}

.duo-srv--disabled {
  opacity: 0.55;
  pointer-events: none;
}

.duo-srv-text {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}
.duo-srv--expired {
  opacity: 0.35;
  filter: grayscale(0.7);
  pointer-events: none;
}
.duo-srv-name {
  font-family: var(--font-display);
  font-size: 13.5px;
  font-weight: 400;
  letter-spacing: -0.015em;
  color: var(--muted-foreground);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  transition: color 180ms ease;
}

.duo-srv-meta {
  font-family: var(--font-mono);
  font-size: 10px;
  color: var(--muted-foreground);
  opacity: 0.7;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.udp-chip {
  align-self: flex-start;
  margin-top: 3px;
  padding: 2px 6px;
  border-radius: 5px;
  border: 1px solid transparent;
  font-family: var(--f-mono, monospace);
  font-size: 9px;
  font-weight: 600;
  letter-spacing: 0.02em;
  white-space: nowrap;
}
.udp-chip--full {
  color: #6ee7a8;
  background: rgba(52, 211, 153, 0.12);
  border-color: rgba(52, 211, 153, 0.3);
}
.udp-chip--ok {
  color: #7dd3fc;
  background: rgba(56, 189, 248, 0.1);
  border-color: rgba(56, 189, 248, 0.26);
}
.duo-srv-right {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.duo-check {
  color: #5ee69a;
}

.duo-hint,
.duo-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  min-height: 120px;
  padding: 20px;
  font-size: 12.5px;
  color: var(--muted-foreground);
  text-align: center;
}

.entry-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  flex-shrink: 0;
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  background: transparent;
  color: var(--muted-foreground);
  cursor: pointer;
  transition:
    color 150ms ease,
    background 150ms ease,
    border-color 150ms ease;
}

.entry-btn:hover {
  color: var(--foreground);
  background: color-mix(in oklch, var(--foreground) 6%, transparent);
}

.entry-btn--active {
  color: var(--foreground);
  border-color: color-mix(in oklch, var(--foreground) 40%, transparent);
  background: color-mix(in oklch, var(--foreground) 8%, transparent);
}

.fav-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  flex-shrink: 0;
  border-radius: 8px;
  border: 1px solid transparent;
  background: transparent;
  color: var(--muted-foreground);
  opacity: 0;
  cursor: pointer;
  transition:
    color 150ms ease,
    background 150ms ease,
    border-color 150ms ease,
    opacity 180ms ease,
    transform 160ms cubic-bezier(0.34, 1.56, 0.64, 1);
}

.duo-srv:hover .fav-btn {
  opacity: 1;
}

.fav-btn:hover {
  color: #ffd47a;
  border-color: rgba(255, 212, 122, 0.4);
  background: rgba(255, 200, 90, 0.12);
  transform: scale(1.08);
}

.fav-btn--on {
  opacity: 1;
  color: #ffd47a;
}

.fav-btn--on > svg {
  fill: currentColor;
}

.copy-ip-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  flex-shrink: 0;
  border-radius: 8px;
  border: 1px solid transparent;
  background: transparent;
  color: var(--muted-foreground);
  opacity: 0;
  cursor: pointer;
  transition:
    color 150ms ease,
    background 150ms ease,
    border-color 150ms ease,
    opacity 180ms ease;
}

.duo-srv:hover .copy-ip-btn {
  opacity: 1;
}

.copy-ip-btn:hover {
  color: var(--foreground);
  border-color: color-mix(in oklch, var(--foreground) 26%, transparent);
  background: color-mix(in oklch, var(--foreground) 6%, transparent);
}

.copy-ip-btn--done {
  opacity: 1;
  color: #5ee69a;
}

@media (hover: none) {
  .copy-ip-btn,
  .fav-btn {
    opacity: 1;
  }
}

/* Latency keeps its colour, loses the filled chip: colour alone carries the
   signal and the page stops looking like a wall of confetti. The face is the
   same Unbounded used for the subscription's remaining time, so the numbers
   on this page read as one system. */
.ping-badge {
  font-family: var(--font-label);
  font-size: 9px;
  font-weight: 400;
  letter-spacing: 0.05em;
  padding: 0;
  border-radius: 0;
  white-space: nowrap;
  background: transparent;
  font-variant-numeric: tabular-nums;
}

.ping-badge.tier-good {
  color: var(--success);
}
.ping-badge.tier-ok {
  color: #d8b24a;
}
.ping-badge.tier-slow {
  color: #e08a4a;
}
.ping-badge.tier-bad {
  color: #e0656e;
}
.ping-badge.tier-none {
  color: var(--muted-foreground);
  opacity: 0.5;
}

.swap-enter-active {
  transition:
    opacity 180ms ease,
    transform 180ms ease;
}
.swap-leave-active {
  transition:
    opacity 120ms ease,
    transform 120ms ease;
}
.swap-enter-from {
  opacity: 0;
  transform: translateX(10px);
}
.swap-leave-to {
  opacity: 0;
  transform: translateX(-6px);
}

.check-pop-enter-active {
  transition: all 160ms cubic-bezier(0.34, 1.56, 0.64, 1);
}
.check-pop-leave-active {
  transition: all 100ms ease;
}
.check-pop-enter-from,
.check-pop-leave-to {
  opacity: 0;
  transform: scale(0.4);
}

.mono {
  font-family: var(--font-mono);
}
.duo-srv-name {
  display: flex;
  align-items: center;
  gap: 6px;
}

.srv-label {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.srv-badge {
  display: grid;
  place-items: center;
  width: 17px;
  height: 17px;
  border-radius: 6px;
  flex-shrink: 0;
}

.srv-flag {
  flex-shrink: 0;
  border-radius: 2px;
}
</style>
