<template>
  <div class="duo">
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
        <span class="land-dot" :class="pingTier(c.best)"></span>
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
            }"
            @click="hop(srv.id)"
          >
            <div class="duo-srv-text">
              <span class="duo-srv-name" v-text="srv.name"></span>
              <span class="duo-srv-meta mono" v-text="srvMeta(srv)"></span>
            </div>
            <div class="duo-srv-right">
              <button
                v-if="vpnStore.settings.multihop_enabled"
                type="button"
                class="entry-btn"
                :class="{ 'entry-btn--active': vpnStore.selectedEntryServerId === srv.id }"
                :title="t('servers.entryTitle')"
                @click.stop="vpnStore.selectEntryServer(vpnStore.selectedEntryServerId === srv.id ? null : srv.id)"
              >
                <Shuffle :size="12" />
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
import { ref, computed, watchEffect } from 'vue';
import { Shuffle, Check } from 'lucide-vue-next';
import { useVpnStore } from '../stores/vpn';
import { t } from '../i18n';
import { groupServers, pingTier, type ServerEntry } from '../lib/geo';
import CountryFlag from './CountryFlag.vue';

const props = defineProps<{ query: string }>();

const vpnStore = useVpnStore();
const picked = ref<string | null>(null);
const switching = ref(false);

const buckets = computed(() =>
  groupServers(vpnStore.subscriptions, props.query, vpnStore.settings.language)
);

const current = computed(() => buckets.value.find(b => b.code === picked.value) ?? null);

const noMatchText = computed(() => t('servers.noServersMatch', { query: props.query }));

watchEffect(() => {
  if (!buckets.value.some(b => b.code === picked.value)) {
    picked.value = buckets.value.length > 0 ? buckets.value[0].code : null;
  }
});

function srvMeta(srv: ServerEntry): string {
  return `${srv.protocol} · ${srv.server}`;
}

function pingText(ms: number): string {
  return `${ms}ms`;
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
</script>

<style scoped>
.duo {
  display: flex;
  min-height: 320px;
  max-height: 460px;
  border-radius: 18px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(255, 255, 255, 0.045);
  backdrop-filter: blur(22px) saturate(155%);
  -webkit-backdrop-filter: blur(22px) saturate(155%);
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
  transition: background 150ms ease, border-color 150ms ease;
}

.land-row:hover { background: rgba(255, 255, 255, 0.05); }

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

.land-count { font-size: 10px; color: var(--muted-foreground); }

.land-dot { width: 7px; height: 7px; border-radius: 50%; flex-shrink: 0; }
.land-dot.tier-good { background: #5ee69a; box-shadow: 0 0 6px rgba(94, 230, 154, 0.7); }
.land-dot.tier-ok { background: #f0d36a; box-shadow: 0 0 6px rgba(240, 211, 106, 0.6); }
.land-dot.tier-slow { background: #ff9f6b; box-shadow: 0 0 6px rgba(255, 159, 107, 0.6); }
.land-dot.tier-bad { background: #ff8a92; box-shadow: 0 0 6px rgba(255, 138, 146, 0.6); }
.land-dot.tier-none { background: rgba(255, 255, 255, 0.18); }

.duo-right { flex: 1; overflow-y: auto; min-width: 0; }

.duo-list { list-style: none; padding: 6px 0; }

.duo-srv {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 9px 16px;
  cursor: pointer;
  transition: background 150ms ease;
}

.duo-srv:hover { background: rgba(255, 255, 255, 0.05); }

.duo-srv--selected {
  background: rgba(52, 208, 114, 0.08);
  box-shadow: inset 2px 0 0 rgba(52, 208, 114, 0.6);
}

.duo-srv--disabled { opacity: 0.55; pointer-events: none; }

.duo-srv-text { flex: 1; display: flex; flex-direction: column; gap: 2px; min-width: 0; }
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

.duo-srv-meta { font-size: 10.5px; color: var(--muted-foreground); }

.duo-srv-right { display: flex; align-items: center; gap: 8px; flex-shrink: 0; }

.duo-check { color: #5ee69a; }

.duo-hint, .duo-empty {
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
  transition: color 150ms ease, background 150ms ease, border-color 150ms ease;
}

.entry-btn:hover { color: var(--foreground); background: rgba(255, 255, 255, 0.08); }

.entry-btn--active {
  color: #8fb6ff;
  border-color: rgba(96, 150, 240, 0.4);
  background: rgba(70, 120, 220, 0.14);
}

.ping-badge {
  font-family: var(--font-mono);
  font-size: 10px;
  font-weight: 500;
  padding: 2px 6px;
  border-radius: 6px;
  white-space: nowrap;
}

.ping-badge.tier-good { background: rgba(52, 208, 114, 0.14); color: #5ee69a; }
.ping-badge.tier-ok { background: rgba(240, 200, 60, 0.14); color: #f0d36a; }
.ping-badge.tier-slow { background: rgba(240, 130, 60, 0.16); color: #ff9f6b; }
.ping-badge.tier-bad { background: rgba(220, 60, 70, 0.16); color: #ff8a92; }
.ping-badge.tier-none { background: rgba(255, 255, 255, 0.06); color: var(--muted-foreground); }

.swap-enter-active { transition: opacity 180ms ease, transform 180ms ease; }
.swap-leave-active { transition: opacity 120ms ease, transform 120ms ease; }
.swap-enter-from { opacity: 0; transform: translateX(10px); }
.swap-leave-to { opacity: 0; transform: translateX(-6px); }

.check-pop-enter-active { transition: all 160ms cubic-bezier(0.34, 1.56, 0.64, 1); }
.check-pop-leave-active { transition: all 100ms ease; }
.check-pop-enter-from, .check-pop-leave-to { opacity: 0; transform: scale(0.4); }

.mono { font-family: var(--font-mono); }
</style>