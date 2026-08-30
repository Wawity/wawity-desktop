<template>
  <div class="duo" data-sensitive>
    <div class="duo-left">
      <button
        v-for="c in buckets"
        :key="c.code"
        type="button"
        class="land-row"
        :class="{ 'land-row--active': picked === c.code }"
        @click="picked = c.code"
      >
        <CountryFlag :code="c.code" :size="24" />
        <span class="land-name" v-text="c.name"></span>
        <span class="land-count mono" v-text="c.servers.length"></span>
        <span class="land-dot-wrap" :data-tip="dotTip(c)">
          <span class="land-dot" :class="pingTier(c.best)"></span>
        </span>
      </button>
      <div v-if="buckets.length === 0" class="duo-empty" v-text="noMatchText"></div>
    </div>
    <div class="duo-right">
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
                <span class="srv-badge" :style="badgeStyle(srv.id)">
                  <component :is="badgeGlyph(srv.id)" :size="10" aria-hidden="true" />
                </span>
                <span class="srv-label" v-text="srv.name"></span>
              </span>
              <span class="duo-srv-meta mono" v-text="srvMeta(srv)"></span>
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
        <div v-else class="duo-hint" v-text="t('servers.pickCountry')"></div>
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
import { groupServers, pingTier, type ServerEntry, type CountryBucket } from '../lib/geo';
import CountryFlag from './CountryFlag.vue';

const props = defineProps<{ query: string }>();

const vpnStore = useVpnStore();
const picked = ref<string | null>(null);
const switching = ref(false);
const copiedId = ref<string | null>(null);
let copyResetTimer = 0;

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

const buckets = computed(() =>
  groupServers(vpnStore.subscriptions, props.query, vpnStore.settings.language),
);

const current = computed(() => buckets.value.find((b) => b.code === picked.value) ?? null);

const noMatchText = computed(() => t('servers.noServersMatch', { query: props.query }));

watchEffect(() => {
  if (!buckets.value.some((b) => b.code === picked.value)) {
    picked.value = buckets.value.length > 0 ? buckets.value[0].code : null;
  }
});

function srvMeta(srv: ServerEntry): string {
  return `${srv.protocol} · ${srv.server}`;
}

function pingText(ms: number): string {
  return `${ms}ms`;
}

function dotTip(c: CountryBucket): string {
  return c.best !== null && c.best !== undefined ? `${c.best}ms` : t('servers.pingNoData');
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
  const tone = badge ? badge.color : '#a78bfa';
  return { color: tone, background: tintSoft(tone, 0.18) };
}
</script>

<style scoped>
.duo {
  display: flex;
  min-height: 320px;
  max-height: 460px;
  border-radius: 18px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(255, 255, 255, 0.045);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.09),
    inset 0 -1px 0 rgba(0, 0, 0, 0.25),
    0 14px 40px rgba(0, 0, 0, 0.35);
  overflow: hidden;
}

.duo-left {
  width: 200px;
  flex-shrink: 0;
  overflow-y: auto;
  padding: 8px;
  border-right: 1px solid rgba(255, 255, 255, 0.06);
  background: rgba(0, 0, 0, 0.14);
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.land-row {
  display: flex;
  align-items: center;
  gap: 9px;
  padding: 8px 10px;
  border-radius: 11px;
  border: 1px solid transparent;
  background: transparent;
  color: var(--foreground);
  cursor: pointer;
  text-align: left;
  transition:
    background 220ms cubic-bezier(0.22, 1, 0.36, 1),
    border-color 220ms ease,
    transform 200ms cubic-bezier(0.34, 1.3, 0.64, 1);
  content-visibility: auto;
  contain-intrinsic-size: auto 40px;
}

.land-row:hover {
  background: rgba(255, 255, 255, 0.05);
  transform: translateX(2px);
}

.land-row--active {
  border-color: rgba(167, 139, 250, 0.35);
  background: linear-gradient(180deg, rgba(167, 139, 250, 0.16), rgba(139, 92, 246, 0.07));
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.1);
}

.land-name {
  flex: 1;
  min-width: 0;
  font-size: 12.5px;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.land-count {
  font-size: 10px;
  color: var(--muted-foreground);
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
}

.duo-list {
  list-style: none;
  padding: 6px 0;
}

.duo-srv {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 9px 16px;
  cursor: pointer;
  transition:
    background 220ms cubic-bezier(0.22, 1, 0.36, 1),
    box-shadow 320ms ease,
    padding 260ms cubic-bezier(0.34, 1.3, 0.64, 1);
  content-visibility: auto;
  contain-intrinsic-size: auto 44px;
}

.duo-srv:hover {
  background: rgba(255, 255, 255, 0.05);
  padding-left: 19px;
}

.duo-srv--selected {
  background: rgba(52, 208, 114, 0.08);
  box-shadow: inset 2px 0 0 rgba(52, 208, 114, 0.6);
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
  font-size: 13px;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.duo-srv-meta {
  font-size: 10.5px;
  color: var(--muted-foreground);
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
  background: rgba(255, 255, 255, 0.08);
}

.entry-btn--active {
  color: #8fb6ff;
  border-color: rgba(96, 150, 240, 0.4);
  background: rgba(70, 120, 220, 0.14);
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
  color: #d9ccff;
  border-color: rgba(167, 139, 250, 0.4);
  background: rgba(167, 139, 250, 0.12);
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

.ping-badge {
  font-family: var(--font-mono);
  font-size: 10px;
  font-weight: 500;
  padding: 2px 6px;
  border-radius: 6px;
  white-space: nowrap;
}

.ping-badge.tier-good {
  background: rgba(52, 208, 114, 0.14);
  color: #5ee69a;
}
.ping-badge.tier-ok {
  background: rgba(240, 200, 60, 0.14);
  color: #f0d36a;
}
.ping-badge.tier-slow {
  background: rgba(240, 130, 60, 0.16);
  color: #ff9f6b;
}
.ping-badge.tier-bad {
  background: rgba(220, 60, 70, 0.16);
  color: #ff8a92;
}
.ping-badge.tier-none {
  background: rgba(255, 255, 255, 0.06);
  color: var(--muted-foreground);
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
</style>
