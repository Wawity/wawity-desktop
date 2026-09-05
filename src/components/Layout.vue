<template>
  <div class="shell">
    <div v-if="customBgOn" class="custom-bg" :style="customBgVars" aria-hidden="true">
      <img
        v-if="!bgFailed"
        :src="vpnStore.settings.bg_custom_url"
        alt=""
        decoding="async"
        @error="onBgImgError"
      />
      <span v-else class="custom-bg-note" v-text="t('settings.bgLoadFail')" />
      <span class="custom-bg-dim" />
    </div>
    <template v-if="!isMaterial && !customBgOn">
      <BlackHole
        v-if="spawned.home"
        v-show="backdrop.home"
        :active="backdrop.home || scenePrewarm"
      />
      <Pulsar
        v-if="spawned.settings"
        v-show="backdrop.settings"
        :active="backdrop.settings || scenePrewarm"
      />
      <NebulaEye
        v-if="spawned.analysis"
        v-show="backdrop.analysis"
        :active="backdrop.analysis || scenePrewarm"
      />
      <Earth
        v-if="spawned.servers"
        v-show="backdrop.servers"
        :active="backdrop.servers || scenePrewarm"
      />
    </template>

    <div
      :class="['titlebar', glassOn ? 'titlebar--glass' : 'titlebar--clear']"
      data-tauri-drag-region
    >
      <div class="titlebar-left" data-tauri-drag-region>
        <img :src="iconSrc" class="titlebar-icon" alt="" aria-hidden="true" />
        <span class="titlebar-name" data-tauri-drag-region>wawity</span>
        <div ref="searchWrapRef" class="tb-search" :class="{ 'tb-search--open': searchOpen }">
          <button
            type="button"
            class="tb-search-toggle"
            :aria-label="t('layout.searchOpen')"
            :aria-expanded="searchOpen"
            @click="toggleSearch"
          >
            <Search :size="15" aria-hidden="true" />
          </button>
          <div class="tb-search-field">
            <input
              ref="searchInputRef"
              v-model="searchQuery"
              type="text"
              class="tb-search-input"
              :placeholder="t('layout.searchPlaceholder')"
              :aria-label="t('layout.searchPlaceholder')"
              spellcheck="false"
              @keydown="onSearchKeydown"
            />
            <button
              v-if="searchQuery"
              type="button"
              class="tb-search-clear"
              :aria-label="t('layout.searchClear')"
              @click="clearSearch"
            >
              <X :size="13" aria-hidden="true" />
            </button>
          </div>
          <Transition name="tb-results">
            <div v-if="searchOpen && searchResults.length" class="tb-results" role="listbox">
              <button
                v-for="(item, idx) in searchResults"
                :key="(item.to ?? item.tab ?? '') + item.labelKey"
                type="button"
                :class="['tb-result', { 'tb-result--active': idx === activeIdx }]"
                role="option"
                :aria-selected="idx === activeIdx"
                @mouseenter="activeIdx = idx"
                @click="goTo(item)"
              >
                <component :is="item.icon" :size="14" class="tb-result-icon" aria-hidden="true" />
                <span class="tb-result-label" v-text="t(item.labelKey)"></span>
                <span class="tb-result-kind" v-text="t(item.kindKey)"></span>
              </button>
            </div>
            <div v-else-if="searchOpen && searchQuery && !searchResults.length" class="tb-results tb-results--empty">
              <span v-text="t('layout.searchEmpty')"></span>
            </div>
          </Transition>
        </div>
      </div>
      <div class="titlebar-controls">
        <button class="ctrl-btn ctrl-btn--minimize" @click="minimizeWin" :aria-label="t('layout.minimize')">
          <svg width="10" height="1" viewBox="0 0 10 1" fill="none"><rect width="10" height="1" rx="0.5" fill="currentColor"/></svg>
        </button>
        <button class="ctrl-btn ctrl-btn--maximize" @click="maximizeWin" :aria-label="t('layout.maximize')">
          <svg width="9" height="9" viewBox="0 0 9 9" fill="none"><rect x="0.5" y="0.5" width="8" height="8" rx="1.5" stroke="currentColor"/></svg>
        </button>
        <button class="ctrl-btn ctrl-btn--close" @click="hideWin" :aria-label="t('layout.close')">
          <svg width="10" height="10" viewBox="0 0 10 10" fill="none"><path d="M1 1L9 9M9 1L1 9" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>
        </button>
      </div>
    </div>

    <div :class="['body', { 'body--solo': isStandalone }]">
      <aside
        v-if="!isStandalone"
        :class="['sidebar', glassOn ? 'sidebar--glass' : 'sidebar--clear']"
      >
        <nav class="nav" :aria-label="t('layout.mainNav')">
          <span class="nav-pill" :style="pillStyle" aria-hidden="true"></span>
          <router-link
            v-for="item in navItems"
            :key="item.to"
            :to="item.to"
            :class="['nav-link', { 'nav-link--active': route.path === item.to }]"
            :aria-current="route.path === item.to ? 'page' : undefined"
          >
            <component :is="item.icon" class="nav-icon" :size="16" aria-hidden="true" />
            <span class="nav-text">{{ t(`nav.${item.key}`) }}</span>
          </router-link>

          <div class="extra-anchor" @mouseenter="openExtra" @mouseleave="closeExtra">
            <button
              type="button"
              :class="['nav-link extra-btn', {
                'extra-btn--open': extraOpen,
                'extra-btn--active': isExtra,
              }]"
              :title="t('nav.extra')"
              :aria-expanded="extraOpen"
              @click="extraOpen = !extraOpen"
            >
              <Sparkles class="nav-icon" :size="16" aria-hidden="true" />
              <span class="extra-label">{{ t('nav.extra') }}</span>
              <ChevronRight class="extra-caret" :size="12" aria-hidden="true" />
            </button>
            <Transition name="flyout">
              <div v-if="extraOpen" class="flyout" role="menu">
                <button
                  v-for="item in extraItems"
                  :key="item.to"
                  type="button"
                  :class="['flyout-item', { 'flyout-item--active': route.path === item.to }]"
                  role="menuitem"
                  @click="goExtra(item.to)"
                >
                  <component :is="item.icon" class="flyout-icon" :size="14" aria-hidden="true" />
                  <span class="flyout-title">{{ t(`nav.${item.key}`) }}</span>
                </button>
              </div>
            </Transition>
          </div>
        </nav>
        <div class="sidebar-spacer"></div>
        <div class="sidebar-version">v0.2.1</div>
      </aside>

      <main class="main-content">
        <div class="content-inner">
          <router-view v-slot="{ Component }">
            <Transition name="route" mode="out-in">
              <KeepAlive :include="keepAlive" :max="2">
                <component :is="Component" />
              </KeepAlive>
            </Transition>
          </router-view>
        </div>
      </main>
    </div>

    <nav v-if="!isStandalone" class="mobile-nav" :aria-label="t('layout.mobileNav')">
      <router-link
        v-for="item in navItems"
        :key="item.to"
        :to="item.to"
        :class="['mobile-link', { 'mobile-link--active': route.path === item.to }]"
        :aria-current="route.path === item.to ? 'page' : undefined"
      >
        <component :is="item.icon" :size="20" aria-hidden="true" />
        <span>{{ t(`nav.${item.key}`) }}</span>
      </router-link>
    </nav>

    <ToastContainer />
    <ConfirmModal />
    <CopyHint />

    <Transition name="dnd-veil">
      <div v-if="dragActive" class="dnd-veil">
        <div class="dnd-card">
          <AppWindow :size="30" class="dnd-icon" />
          <p class="dnd-title" v-text="t('layout.dropTitle')" />
          <p class="dnd-sub" v-text="t('layout.dropSub')" />
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import {
  computed,
  inject,
  onBeforeUnmount,
  onMounted,
  reactive,
  ref,
  watch,
  watchEffect } from 'vue';
import { useRoute,
  useRouter } from 'vue-router';
import { appWindow } from '@tauri-apps/api/window';
import BlackHole from './BlackHole.vue';
import Pulsar from './Pulsar.vue';
import NebulaEye from './NebulaEye.vue';
import Earth from './Earth.vue';
import ToastContainer from './ToastContainer.vue';
import ConfirmModal from './ConfirmModal.vue';
import CopyHint from './CopyHint.vue';
import { useVpnStore } from '../stores/vpn';
import { initNotificationWatcher,
  pushToast } from '../composables/useNotifications';
import { showCopyHint } from '../composables/useCopyHint';
import { staggerChildren } from '../lib/motion';
import { useAppIcon } from '../composables/useAppIcon';
import {
  Activity,
  ChevronRight,
  Gauge,
  Globe2,
  Info,
  Layers,
  Power,
  Radar,
  Search,
  Settings,
  ShieldAlert,
  ShieldCheck,
  Shuffle,
  Sparkles,
  X,
  AppWindow,
  Crosshair,
  Wrench,
} from '../lib/appIcons';
import { t } from '../i18n';

const vpnStore = useVpnStore();
const route = useRoute();
const router = useRouter();
const { iconSrc } = useAppIcon();
const onboardingOpen = inject('onboardingOpen', ref(false));

const dragActive = ref(false);
let dragDepth = 0;
let unlistenDrop: Array<() => void> = [];

onMounted(async () => {
  try {
    const { listen } = await import('@tauri-apps/api/event');
    unlistenDrop.push(
      await listen<string[]>('tauri://file-drop', async (event) => {
        dragDepth = 0;
        dragActive.value = false;
        const paths = (event.payload ?? []).filter((p) => p.toLowerCase().endsWith('.exe'));
        if (paths.length === 0) return;
        const added = await vpnStore.addBypassApps(paths);
        if (added > 0) {
          showCopyHint(t('toast.appsAddedShort', { count: added }));
          pushToast('success', t('toast.appsAdded'), t('toast.appsAddedDesc', { count: added }));
        } else {
          pushToast('info', t('toast.nothingToAdd'), t('toast.nothingToAddDesc'));
        }
      }),
    );
    unlistenDrop.push(
      await listen('tauri://file-drop-hover', () => {
        dragDepth += 1;
        dragActive.value = true;
      }),
    );
    unlistenDrop.push(
      await listen('tauri://file-drop-cancelled', () => {
        dragDepth = 0;
        dragActive.value = false;
      }),
    );
  } catch {}
});

onBeforeUnmount(() => {
  unlistenDrop.forEach((fn) => fn());
  unlistenDrop = [];
});

const isMaterial = computed(() => vpnStore.settings.ui_style === 'material');

watchEffect(() => {
  document.documentElement.classList.toggle('theme-material', isMaterial.value);
});

const customBgOn = computed(
  () => !isMaterial.value && vpnStore.settings.bg_custom_enabled && !!vpnStore.settings.bg_custom_url,
);

const bgFailed = ref(false);
watch(
  () => vpnStore.settings.bg_custom_url,
  () => {
    bgFailed.value = false;
  },
);

const customBgVars = computed(() => ({
  '--bg-dim': String(Math.min(90, Math.max(0, vpnStore.settings.bg_custom_dim)) / 100),
  '--bg-blur': `${Math.min(40, Math.max(0, vpnStore.settings.bg_custom_blur))}px`,
  '--bg-scale': String(
    1 + Math.min(40, Math.max(0, vpnStore.settings.bg_custom_blur)) / 90,
  ),
}));

function onBgImgError() {
  bgFailed.value = true;
}

const glassOn = computed(() => vpnStore.settings.liquid_glass);

watchEffect(() => {
  document.documentElement.classList.toggle('liquid-glass-on', glassOn.value);
});

onMounted(() => {
  if (!glassOn.value) return;
  const probe = document.createElement('div');
  // On-screen (WebView2 skips backdrop-filter compilation for offscreen layers),
  // nearly invisible, with the exact blur radius used by the glass panels.
  probe.style.cssText =
    'position:fixed;right:0;bottom:0;width:180px;height:180px;z-index:-1;pointer-events:none;opacity:0.02;backdrop-filter:blur(14px);-webkit-backdrop-filter:blur(14px);transform:translateZ(0);contain:strict;';
  document.body.appendChild(probe);

  const radii = [13, 15, 14];
  radii.forEach((r, i) => {
    window.setTimeout(() => {
      probe.style.backdropFilter = `blur(${r}px)`;
      probe.style.webkitBackdropFilter = `blur(${r}px)`;
      probe.style.transform = i % 2 ? 'translateZ(0)' : 'translateZ(0.5px)';
    }, 60 * i);
  });

  window.setTimeout(() => {
    probe.style.transform = 'scale(1.04)';
  }, 220);
  window.setTimeout(() => probe.remove(), 900);
});

const scenePrewarm = ref(false);

function startScenePrewarm() {
  window.setTimeout(() => {
    scenePrewarm.value = true;
    
    window.setTimeout(() => {
      scenePrewarm.value = false;
    }, 900);
  }, 120);
}

onMounted(() => {
  if (isMaterial.value) return; 
  if (document.visibilityState === 'visible') {
    startScenePrewarm();
  } else {
    document.addEventListener('visibilitychange', startScenePrewarm, { once: true });
  }
});

const backdropsVisible = computed(() => {
  
  const p = route.path;
  return {
    home: p === '/' && !onboardingOpen.value,
    settings: p === '/settings',
    analysis: p === '/analysis',
    servers: p === '/servers' && vpnStore.settings.server_view !== 'globe',
  };
});

const spawned = reactive({ home: true, settings: true, analysis: true, servers: true });
const backdrop = reactive({ home: false, settings: false, analysis: false, servers: false });

watch(
  backdropsVisible,
  (visible) => {
    if (visible.home) spawned.home = true;
    if (visible.settings) spawned.settings = true;
    if (visible.analysis) spawned.analysis = true;
    if (visible.servers) spawned.servers = true;
    backdrop.home = visible.home;
    backdrop.settings = visible.settings;
    backdrop.analysis = visible.analysis;
    backdrop.servers = visible.servers;
  },
  { immediate: true },
);

const navItems = [
  { to: '/', key: 'connection', icon: Power },
  { to: '/analysis', key: 'analysis', icon: Activity },
  { to: '/servers', key: 'servers', icon: Globe2 },
  { to: '/settings', key: 'settings', icon: Settings },
];

const extraItems = [
  { to: '/extra/reachability', key: 'reachability', icon: Radar },
  { to: '/extra/speedtest', key: 'speedtest', icon: Gauge },
  { to: '/extra/leaks', key: 'leaks', icon: ShieldCheck },
  { to: '/extra/dns-bench', key: 'dnsBench', icon: Gauge },
  { to: '/extra/node-pulse', key: 'nodePulse', icon: Activity },
  { to: '/extra/port-audit', key: 'portAudit', icon: Crosshair },
  { to: '/extra/firewall', key: 'firewall', icon: ShieldAlert },
  { to: '/extra/snippets', key: 'snippets', icon: Wrench },
];

const keepAlive = ['AnalysisView'];

const searchOpen = ref(false);
const searchQuery = ref('');
const activeIdx = ref(0);
const searchInputRef = ref<HTMLInputElement | null>(null);
const searchWrapRef = ref<HTMLElement | null>(null);

const SEARCH_ITEMS = [
  { labelKey: 'nav.connection', to: '/', icon: Power, kindKey: 'layout.searchKindPage', keywords: 'home connect power vpn' },
  { labelKey: 'nav.analysis', to: '/analysis', icon: Activity, kindKey: 'layout.searchKindPage', keywords: 'traffic graph stats throughput' },
  { labelKey: 'nav.servers', to: '/servers', icon: Globe2, kindKey: 'layout.searchKindPage', keywords: 'locations countries ping globe list' },
  { labelKey: 'nav.settings', to: '/settings', icon: Settings, kindKey: 'layout.searchKindPage', keywords: 'preferences options config' },
  { labelKey: 'nav.reachability', to: '/extra/reachability', icon: Radar, kindKey: 'layout.searchKindTool', keywords: 'reachability probe tcp connectivity' },
  { labelKey: 'nav.speedtest', to: '/extra/speedtest', icon: Gauge, kindKey: 'layout.searchKindTool', keywords: 'speed test bandwidth download upload' },
  { labelKey: 'nav.leaks', to: '/extra/leaks', icon: ShieldCheck, kindKey: 'layout.searchKindTool', keywords: 'dns webrtc ip leak test' },
  { labelKey: 'settings.security', tab: 'security', icon: ShieldAlert, kindKey: 'layout.searchKindTab', keywords: 'security section' },
  { labelKey: 'settings.connectionSection', tab: 'connection', icon: Gauge, kindKey: 'layout.searchKindTab', keywords: 'connection section' },
  { labelKey: 'settings.privacy', tab: 'privacy', icon: Info, kindKey: 'layout.searchKindTab', keywords: 'privacy section' },
  { labelKey: 'settings.appearance', tab: 'appearance', icon: Sparkles, kindKey: 'layout.searchKindTab', keywords: 'appearance theme section ui style material' },
  { labelKey: 'settings.splitTunneling', tab: 'split', icon: Layers, kindKey: 'layout.searchKindTab', keywords: 'split tunneling apps process' },
  { labelKey: 'settings.about', tab: 'about', icon: Info, kindKey: 'layout.searchKindTab', keywords: 'about version license' },
  { labelKey: 'settings.dpiTitle', tab: 'security', descKey: 'settings.dpiDesc', icon: ShieldCheck, kindKey: 'layout.searchKindSetting', keywords: 'dpi fragmentation tls evasion' },
  { labelKey: 'settings.killSwitch', tab: 'security', highlight: 'killswitch', descKey: 'settings.killSwitchDesc', icon: ShieldAlert, kindKey: 'layout.searchKindSetting', keywords: 'killswitch block leak' },
  { labelKey: 'settings.alwaysOn', tab: 'security', highlight: 'alwayson', descKey: 'settings.alwaysOnDesc', icon: ShieldAlert, kindKey: 'layout.searchKindSetting', keywords: 'always on lockdown autostart' },
  { labelKey: 'settings.quantum', tab: 'security', descKey: 'settings.quantumDesc', icon: ShieldCheck, kindKey: 'layout.searchKindSetting', keywords: 'quantum post quantum encryption' },
  { labelKey: 'settings.strictRoute', tab: 'security', descKey: 'settings.strictRouteDesc', icon: Layers, kindKey: 'layout.searchKindSetting', keywords: 'strict route routing' },
  { labelKey: 'settings.dnsLeakGuard', tab: 'security', descKey: 'settings.dnsLeakGuardDesc', icon: ShieldCheck, kindKey: 'layout.searchKindSetting', keywords: 'dns leak guard' },
  { labelKey: 'settings.tunnelOwnTraffic', tab: 'connection', descKey: 'settings.tunnelOwnTrafficDesc', icon: Globe2, kindKey: 'layout.searchKindSetting', keywords: 'tunnel own traffic' },
  { labelKey: 'settings.insecureTls', tab: 'security', descKey: 'settings.insecureTlsDesc', icon: ShieldCheck, kindKey: 'layout.searchKindSetting', keywords: 'insecure tls certificate' },
  { labelKey: 'settings.bootstrapDns', tab: 'security', descKey: 'settings.bootstrapDnsDesc', icon: Info, kindKey: 'layout.searchKindSetting', keywords: 'bootstrap dns cloudflare quad9 google' },
  { labelKey: 'settings.smartConnect', tab: 'connection', descKey: 'settings.smartConnectDesc', icon: Gauge, kindKey: 'layout.searchKindSetting', keywords: 'smart connect fastest auto' },
  { labelKey: 'settings.failover', tab: 'connection', descKey: 'settings.failoverDesc', icon: Gauge, kindKey: 'layout.searchKindSetting', keywords: 'failover fallback retry' },
  { labelKey: 'settings.startOnBoot', tab: 'connection', descKey: 'settings.startOnBootDesc', icon: Gauge, kindKey: 'layout.searchKindSetting', keywords: 'start on boot autostart launch' },
  { labelKey: 'settings.autoConnect', tab: 'connection', descKey: 'settings.autoConnectDesc', icon: Gauge, kindKey: 'layout.searchKindSetting', keywords: 'auto connect startup' },
  { labelKey: 'settings.lanAccess', tab: 'connection', descKey: 'settings.lanAccessDesc', icon: Globe2, kindKey: 'layout.searchKindSetting', keywords: 'lan local network access' },
  { labelKey: 'settings.autoPing', tab: 'connection', descKey: 'settings.autoPingDesc', icon: Activity, kindKey: 'layout.searchKindSetting', keywords: 'auto ping latency refresh' },
  { labelKey: 'settings.multihop', tab: 'connection', highlight: 'multihop', descKey: 'settings.multihopDesc', icon: Shuffle, kindKey: 'layout.searchKindSetting', keywords: 'multihop double chain' },
  { labelKey: 'settings.hotkeys', tab: 'connection', descKey: 'settings.hotkeysDesc', icon: Settings, kindKey: 'layout.searchKindSetting', keywords: 'hotkeys shortcuts keyboard' },
  { labelKey: 'settings.hotkeyToggle', tab: 'connection', descKey: 'settings.hotkeyToggleDesc', icon: Settings, kindKey: 'layout.searchKindSetting', keywords: 'hotkey toggle shortcut' },
  { labelKey: 'settings.blockTrackers', tab: 'privacy', descKey: 'settings.blockTrackersDesc', icon: ShieldCheck, kindKey: 'layout.searchKindSetting', keywords: 'block trackers ads' },
  { labelKey: 'settings.onlineGeo', tab: 'privacy', descKey: 'settings.onlineGeoDesc', icon: Info, kindKey: 'layout.searchKindSetting', keywords: 'geolocation geo online' },
  { labelKey: 'settings.notifications', tab: 'privacy', descKey: 'settings.notificationsDesc', icon: Info, kindKey: 'layout.searchKindSetting', keywords: 'notifications alerts' },
  { labelKey: 'settings.telemetry', tab: 'privacy', descKey: 'settings.telemetryDesc', icon: Info, kindKey: 'layout.searchKindSetting', keywords: 'telemetry analytics statistics' },
  { labelKey: 'settings.deviceId', tab: 'privacy', descKey: 'settings.deviceIdDesc', icon: ShieldCheck, kindKey: 'layout.searchKindSetting', keywords: 'hwid device id hardware fingerprint' },
  { labelKey: 'settings.discordRpc', tab: 'privacy', descKey: 'settings.discordRpcDesc', icon: Sparkles, kindKey: 'layout.searchKindSetting', keywords: 'discord rich presence rpc status' },
  { labelKey: 'settings.liquidGlass', tab: 'appearance', descKey: 'settings.liquidGlassDesc', icon: Sparkles, kindKey: 'layout.searchKindSetting', keywords: 'liquid glass blur transparency' },
  { labelKey: 'settings.serverView', tab: 'appearance', descKey: 'settings.serverViewDesc', icon: Globe2, kindKey: 'layout.searchKindSetting', keywords: 'server view globe list' },
  { labelKey: 'settings.language', tab: 'appearance', descKey: 'settings.languageDesc', icon: Info, kindKey: 'layout.searchKindSetting', keywords: 'language locale translation' },
];

import { Shuffle as Shuffle } from '../lib/appIcons';

const searchResults = computed(() => {
  const q = searchQuery.value.trim().toLowerCase();
  if (!q) return SEARCH_ITEMS;
  return SEARCH_ITEMS.filter((item) =>
    [t(item.labelKey), item.descKey ? t(item.descKey) : '', item.keywords ?? '']
      .join(' ')
      .toLowerCase()
      .includes(q),
  );
});

watch(searchResults, () => {
  activeIdx.value = 0;
});

function openSearch() {
  searchOpen.value = true;
  void nextTickFn(() => {
    searchInputRef.value?.focus();
  });
}

function nextTickFn(fn: () => void) {
  import('vue').then(({ nextTick }) => nextTick().then(fn));
}

function closeSearch() {
  searchOpen.value = false;
  searchQuery.value = '';
}

function toggleSearch() {
  searchOpen.value ? closeSearch() : openSearch();
}

function clearSearch() {
  searchQuery.value = '';
  searchInputRef.value?.focus();
}

function goTo(item: (typeof SEARCH_ITEMS)[number]) {
  if (item.to) {
    router.push(item.to);
    closeSearch();
    return;
  }
  const query: Record<string, string> = {};
  if (item.tab) query.tab = item.tab;
  if (item.highlight) query.highlight = item.highlight;
  router.push({ path: '/settings', query });
  closeSearch();
}

function onSearchKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    e.preventDefault();
    closeSearch();
    return;
  }
  if (e.key === 'ArrowDown') {
    e.preventDefault();
    activeIdx.value = Math.min(activeIdx.value + 1, searchResults.value.length - 1);
    return;
  }
  if (e.key === 'ArrowUp') {
    e.preventDefault();
    activeIdx.value = Math.max(activeIdx.value - 1, 0);
    return;
  }
  if (e.key === 'Enter') {
    e.preventDefault();
    const item = searchResults.value[activeIdx.value];
    if (item) goTo(item);
  }
}

function onOutsidePointer(e: PointerEvent) {
  const wrap = searchWrapRef.value;
  if (wrap && !wrap.contains(e.target as Node)) closeSearch();
}

watch(searchOpen, (open) => {
  if (open) window.addEventListener('pointerdown', onOutsidePointer, true);
  else window.removeEventListener('pointerdown', onOutsidePointer, true);
});

onBeforeUnmount(() => {
  window.removeEventListener('pointerdown', onOutsidePointer, true);
});

const extraOpen = ref(false);
const isStandalone = computed(() => route.meta.standalone === true);
const isExtra = computed(() => route.path.startsWith('/extra/'));

let extraTimer: ReturnType<typeof setTimeout> | null = null;

function openExtra() {
  if (extraTimer) {
    clearTimeout(extraTimer);
    extraTimer = null;
  }
  extraOpen.value = true;
}

function closeExtra() {
  if (extraTimer) clearTimeout(extraTimer);
  extraTimer = setTimeout(() => {
    extraOpen.value = false;
  }, 190);
}

function goExtra(to: string) {
  extraOpen.value = false;
  router.push(to);
}

const activeIndex = computed(() => navItems.findIndex((item) => item.to === route.path));

const pillStyle = computed(() => ({
  transform: `translateY(${Math.max(activeIndex.value, 0) * 40}px)`,
  opacity: activeIndex.value < 0 ? '0' : '1',
}));

onMounted(() => {
  initNotificationWatcher();
});

async function minimizeWin() {
  await appWindow.minimize();
}
async function maximizeWin() {
  await appWindow.toggleMaximize();
}
async function hideWin() {
  await appWindow.hide();
}
</script>

<style scoped>
.shell{display:flex;flex-direction:column;height:100vh;background:var(--background);color:var(--foreground);-webkit-user-select:none;user-select:none}

.titlebar{position:relative;z-index:2;display:flex;align-items:center;justify-content:space-between;height:40px;padding:0 12px 0 16px;flex-shrink:0;-webkit-app-region:drag;app-region:drag}

.titlebar--glass{border-bottom:1px solid color-mix(in oklch,var(--foreground) 10%,transparent);background:linear-gradient(180deg,color-mix(in oklch,var(--foreground) 7%,transparent),color-mix(in oklch,var(--foreground) 3%,transparent)),rgba(13,14,20,.42);backdrop-filter:blur(14px);box-shadow:inset 0 1px color-mix(in oklch,var(--foreground) 12%,transparent)}

.titlebar--clear{border-bottom:1px solid color-mix(in oklch,var(--border) 55%,transparent);background:transparent}

.titlebar-left{display:flex;align-items:center;gap:8px;-webkit-app-region:drag;app-region:drag}

.titlebar-icon{width:18px;height:18px;object-fit:contain;flex-shrink:0}

.titlebar-name{font-size:13px;font-weight:600;letter-spacing:-.01em;color:var(--foreground)}

.tb-search{position:relative;display:flex;align-items:center;margin-left:4px;-webkit-app-region:no-drag;app-region:no-drag}

.tb-search-toggle{display:flex;align-items:center;justify-content:center;width:26px;height:26px;border-radius:8px;border:1px solid transparent;background:transparent;color:var(--muted-foreground);cursor:pointer;flex-shrink:0;transition:background .16s ease,color .16s ease,border-color .16s ease}

.tb-search-toggle:hover{color:var(--foreground);background:color-mix(in oklch,var(--foreground) 7%,transparent)}

.tb-search--open .tb-search-toggle{color:var(--foreground)}

.tb-search-field{position:relative;display:flex;align-items:center;width:0;opacity:0;overflow:hidden;transition:width .28s cubic-bezier(.22,1,.36,1),opacity .2s ease}

.tb-search--open .tb-search-field{width:216px;opacity:1}

.tb-search-input{width:100%;height:26px;padding:0 26px 0 10px;border-radius:8px;border:1px solid color-mix(in oklch,var(--foreground) 12%,transparent);background:color-mix(in oklch,var(--foreground) 6%,transparent);color:var(--foreground);font-size:12.5px;outline:none;transition:border-color .16s ease,box-shadow .16s ease}

.tb-search-input:focus{border-color:color-mix(in oklch,var(--success) 45%,transparent);box-shadow:0 0 0 3px color-mix(in oklch,var(--success) 14%,transparent)}

.tb-search-clear{position:absolute;right:6px;display:flex;align-items:center;justify-content:center;width:18px;height:18px;border-radius:6px;border:none;background:transparent;color:var(--muted-foreground);cursor:pointer}

.tb-search-clear:hover{color:var(--foreground)}

.tb-results{position:absolute;top:34px;left:0;z-index:40;display:flex;flex-direction:column;gap:2px;min-width:252px;max-height:360px;overflow-y:auto;padding:6px;border-radius:12px;border:1px solid color-mix(in oklch,var(--foreground) 10%,transparent);background:color-mix(in oklch,var(--background) 82%,transparent);backdrop-filter:blur(10px);box-shadow:inset 0 1px #ffffff14,0 20px 44px #00000080}

.tb-results--empty{padding:12px 14px;font-size:12px;color:var(--muted-foreground)}

.tb-result{display:flex;align-items:center;gap:9px;padding:8px 10px;border-radius:9px;border:none;background:transparent;color:var(--foreground);font-size:12.5px;text-align:left;cursor:pointer;transition:background .14s ease}

.tb-result--active{background:color-mix(in oklch,var(--foreground) 9%,transparent)}

.tb-result-icon{color:var(--muted-foreground);flex-shrink:0}

.tb-result-label{flex:1;min-width:0;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}

.tb-result-kind{flex-shrink:0;font-size:10.5px;letter-spacing:.04em;text-transform:uppercase;color:color-mix(in oklch,var(--muted-foreground) 75%,transparent)}

.tb-results-enter-active,.tb-results-leave-active{transition:opacity .16s ease,transform .18s cubic-bezier(.22,1,.36,1)}

.tb-results-enter-from,.tb-results-leave-to{opacity:0;transform:translateY(-6px) scale(.98)}

@media (prefers-reduced-motion:reduce){.tb-search-field{transition:opacity .2s ease}}

@media (prefers-reduced-motion:reduce){.tb-results-enter-active,.tb-results-leave-active{transition:none}}

.titlebar-controls{display:flex;align-items:center;gap:4px;-webkit-app-region:no-drag;app-region:no-drag}

.ctrl-btn{display:flex;align-items:center;justify-content:center;width:28px;height:28px;border-radius:8px;border:none;background:transparent;color:var(--muted-foreground);cursor:pointer;transition:background .15s,color .15s;flex-shrink:0}

.ctrl-btn:hover{color:var(--foreground);background:var(--secondary)}

.ctrl-btn--close:hover{background:color-mix(in oklch,var(--destructive) 15%,transparent);color:var(--destructive)}

.body{position:relative;z-index:1;display:flex;flex:1;overflow:hidden}

.sidebar{width:200px;flex-shrink:0;display:flex;flex-direction:column;padding:16px 10px 12px}

.sidebar--glass{border-right:1px solid color-mix(in oklch,var(--foreground) 10%,transparent);background:linear-gradient(180deg,color-mix(in oklch,var(--foreground) 6%,transparent),color-mix(in oklch,var(--foreground) 2%,transparent)),rgba(13,14,20,.42);backdrop-filter:blur(14px);box-shadow:inset 0 1px color-mix(in oklch,var(--foreground) 10%,transparent)}

.sidebar--clear{border-right:1px solid color-mix(in oklch,var(--border) 55%,transparent);background:transparent}

.nav{position:relative;display:flex;flex-direction:column;gap:4px}

.nav-pill{position:absolute;top:0;left:0;right:0;height:36px;border-radius:8px;background:color-mix(in oklch,var(--foreground) 8%,transparent);transition:transform .22s cubic-bezier(.4,0,.2,1),opacity .16s ease;pointer-events:none}

.nav-link{position:relative;z-index:1;display:flex;align-items:center;gap:10px;height:36px;padding:0 10px;border-radius:8px;font-size:13px;font-weight:500;color:var(--muted-foreground);text-decoration:none;transition:color .15s,background .15s}

.nav-link:hover{color:var(--foreground);background:color-mix(in oklch,var(--foreground) 5%,transparent)}

.nav-link--active{color:var(--foreground)}

.nav-link--active:hover{background:transparent}

.nav-icon{flex-shrink:0}

.nav-text{min-width:0;flex:1;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}

.sidebar-spacer{flex:1}

.sidebar-version{padding:0 10px;font-size:11px;color:color-mix(in oklch,var(--muted-foreground) 70%,transparent);font-variant-numeric:tabular-nums}

.main-content{flex:1;overflow-y:auto;padding:28px 28px 40px}

.content-inner{max-width:860px;margin:0 auto;width:100%}

.mobile-nav{display:none}

@media(max-width:768px){.sidebar{display:none}}
@media(max-width:768px){.main-content{padding:16px 16px 80px}}
@media(max-width:768px){.mobile-nav{display:flex;position:fixed;inset-inline:0;bottom:0;z-index:50;border-top:1px solid var(--border);background:color-mix(in oklch,var(--background) 92%,transparent);backdrop-filter:blur(10px);align-items:center;justify-content:space-around;padding:8px 0}}
@media(max-width:768px){.mobile-link{display:flex;flex-direction:column;align-items:center;gap:3px;padding:6px 16px;font-size:11px;font-weight:500;color:var(--muted-foreground);text-decoration:none;border-radius:8px;transition:color .15s}}
@media(max-width:768px){.mobile-link--active{color:var(--foreground)}}

.custom-bg{position:fixed;top:0;right:0;bottom:0;left:0;z-index:0;overflow:hidden;pointer-events:none}
.custom-bg img{width:100%;height:100%;object-fit:cover;filter:blur(var(--bg-blur,0px));transform:scale(var(--bg-scale,1))}
.custom-bg-dim{position:absolute;inset:0;background:rgba(5,6,10,var(--bg-dim,0.45))}
.custom-bg-note{position:absolute;inset:0;display:flex;align-items:center;justify-content:center;font-size:12px;color:rgba(235,238,250,.4);background:var(--background)}
.route-enter-active{animation:routeInSimple .26s cubic-bezier(.34,1.3,.64,1) both}

.route-leave-active{animation:routeOut .12s ease both}

@keyframes routeInSimple{from{opacity:0;transform:translateY(16px) scale(.985)}to{opacity:1;transform:none}}

@keyframes routeOut{from{opacity:1}to{opacity:0;transform:translateY(-8px) scale(.99)}}

html.motion-fancy .route-enter-active{animation:routeInFancy .44s cubic-bezier(.22,.9,.3,1) both}

html.motion-fancy .route-leave-active{animation:routeOutFancy .16s ease-in both}

@keyframes routeInFancy{from{opacity:0;transform:translateY(20px) scale(.988);filter:blur(7px)}60%{filter:blur(2px)}to{opacity:1;transform:none;filter:blur(0)}}

@keyframes routeOutFancy{to{opacity:0;transform:translateY(-10px);filter:blur(5px)}}

.body--solo .main-content{display:flex;overflow:hidden;padding:20px 26px 0}

.body--solo .content-inner{display:flex;min-height:0;max-width:1040px;flex:1;flex-direction:column}

.extra-anchor{position:relative;width:100%;min-width:0}

.extra-btn{display:grid;width:100%;min-width:0;box-sizing:border-box;grid-template-columns:16px minmax(0,1fr) 12px;align-items:center;column-gap:8px;padding:0 9px;border:0;background:transparent;font:inherit;text-align:left;cursor:pointer;overflow:hidden}

.extra-label{min-width:0;overflow:hidden;font-size:12.5px;text-overflow:ellipsis;white-space:nowrap}

.extra-btn--active,.extra-btn--open{color:var(--foreground);background:color-mix(in oklch,var(--foreground) 7%,transparent)}

.extra-caret{justify-self:end;opacity:.42;transition:transform .15s ease,opacity .15s ease}

.extra-btn--open .extra-caret{transform:translateX(1px);opacity:.78}

.flyout{position:absolute;top:0;left:calc(100% + 5px);z-index:80;display:flex;width:174px;flex-direction:column;gap:1px;padding:4px;border:1px solid color-mix(in oklch,var(--foreground) 9%,transparent);border-radius:9px;background:color-mix(in oklch,var(--background) 96%,transparent);box-shadow:0 12px 30px #00000061,inset 0 1px color-mix(in oklch,var(--foreground) 4%,transparent);backdrop-filter:blur(12px)}

.flyout-item{display:grid;height:34px;grid-template-columns:15px minmax(0,1fr);align-items:center;gap:8px;padding:0 9px;border:0;border-radius:7px;background:transparent;color:var(--muted-foreground);font:inherit;font-size:12px;font-weight:500;text-align:left;cursor:pointer;transition:color .14s ease,background .14s ease}

.flyout-item:hover,.flyout-item--active{background:color-mix(in oklch,var(--foreground) 6%,transparent);color:var(--foreground)}

.flyout-icon{flex-shrink:0}

.flyout-title{min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}

.flyout-enter-active,.flyout-leave-active{transition:opacity .14s ease,transform .14s cubic-bezier(.22,1,.36,1)}

.flyout-enter-from,.flyout-leave-to{opacity:0;transform:translate(-4px)}

.dnd-veil {
  position: fixed;
  inset: 0;
  z-index: 1100;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(10, 10, 16, 0.62);
  backdrop-filter: blur(5px);
  -webkit-backdrop-filter: blur(5px);
  pointer-events: none;
}

.dnd-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  padding: 30px 42px;
  border-radius: 22px;
  border: 1.5px dashed rgba(167, 139, 250, 0.55);
  background: linear-gradient(170deg, rgba(40, 36, 66, 0.92), rgba(24, 22, 40, 0.94));
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.12),
    0 26px 70px rgba(0, 0, 0, 0.55),
    0 0 44px rgba(124, 92, 255, 0.22);
  text-align: center;
}

.dnd-icon {
  color: #a78bfa;
  margin-bottom: 4px;
}

.dnd-title {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: #efe9ff;
}

.dnd-sub {
  margin: 0;
  font-size: 11px;
  color: var(--muted-foreground);
  max-width: 240px;
  line-height: 1.45;
}

.dnd-veil-enter-active,
.dnd-veil-leave-active {
  transition:
    opacity 200ms ease,
    transform 260ms cubic-bezier(0.34, 1.56, 0.64, 1);
}
.dnd-veil-enter-from,
.dnd-veil-leave-to {
  opacity: 0;
  transform: scale(0.985);
}
</style>
