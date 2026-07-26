<template>
  <div class="shell">
  <div class="ambient" aria-hidden="true">
    <span class="ambient-blob ambient-blob--violet"></span>
    <span class="ambient-blob ambient-blob--blue"></span>
    <span class="ambient-blob ambient-blob--ember"></span>
  </div>
  <template v-if="vpnStore.settings.black_hole_bg">
    <BlackHole v-if="spawned.home" v-show="backdrop.home" :active="backdrop.home" :detail="vpnStore.settings.black_hole_detail" />
    <Pulsar v-if="spawned.settings" v-show="backdrop.settings" :active="backdrop.settings" />
    <NebulaEye v-if="spawned.analysis" v-show="backdrop.analysis" :active="backdrop.analysis" />
    <Earth v-if="spawned.servers" v-show="backdrop.servers" :active="backdrop.servers" />
  </template>
    <div :class="['titlebar', vpnStore.settings.liquid_glass ? 'titlebar--glass' : 'titlebar--clear']" data-tauri-drag-region>
      <div class="titlebar-left" data-tauri-drag-region>
        <img :src="iconSrc" class="titlebar-icon" alt="" aria-hidden="true" />
        <span class="titlebar-name" data-tauri-drag-region>wawity</span>
      </div>
      <div class="titlebar-controls">
        <button class="ctrl-btn ctrl-btn--minimize" @click="minimize" :aria-label="t('layout.minimize')">
          <svg width="10" height="1" viewBox="0 0 10 1" fill="none">
            <rect width="10" height="1" rx="0.5" fill="currentColor"/>
          </svg>
        </button>
        <button class="ctrl-btn ctrl-btn--maximize" @click="toggleMaximize" :aria-label="t('layout.maximize')">
          <svg width="9" height="9" viewBox="0 0 9 9" fill="none">
            <rect x="0.5" y="0.5" width="8" height="8" rx="1.5" stroke="currentColor"/>
          </svg>
        </button>
        <button class="ctrl-btn ctrl-btn--close" @click="closeWindow" :aria-label="t('layout.close')">
          <svg width="10" height="10" viewBox="0 0 10 10" fill="none">
            <path d="M1 1L9 9M9 1L1 9" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
          </svg>
        </button>
      </div>
    </div>

    <div class="body">
      <aside :class="['sidebar', vpnStore.settings.liquid_glass ? 'sidebar--glass' : 'sidebar--clear']">
        <nav class="nav" :aria-label="t('layout.mainNav')">
          <span class="nav-pill" :style="pillStyle" aria-hidden="true"></span>
          <router-link
            v-for="item in navItems"
            :key="item.to"
            :to="item.to"
            class="nav-link"
            :class="{ 'nav-link--active': $route.path === item.to }"
            :aria-current="$route.path === item.to ? 'page' : undefined"
          >
            <component :is="item.icon" class="nav-icon" :size="16" aria-hidden="true" />
            <span class="nav-text">{{ t(`nav.${item.key}`) }}</span>
          </router-link>
        </nav>

        <div class="sidebar-spacer" />

        <div class="sidebar-version">v0.1.0</div>
      </aside>

      <main class="main-content">
        <div class="content-inner">
          <router-view v-slot="{ Component }">
            <Transition name="route" mode="out-in">
              <KeepAlive :include="CACHED_VIEWS" :max="2">
                <component :is="Component" />
              </KeepAlive>
            </Transition>
          </router-view>
        </div>
      </main>
    </div>

    <nav class="mobile-nav" :aria-label="t('layout.mobileNav')">
      <router-link
        v-for="item in navItems"
        :key="item.to"
        :to="item.to"
        class="mobile-link"
        :class="{ 'mobile-link--active': $route.path === item.to }"
        :aria-current="$route.path === item.to ? 'page' : undefined"
      >
        <component :is="item.icon" :size="20" aria-hidden="true" />
        <span>{{ t(`nav.${item.key}`) }}</span>
      </router-link>
    </nav>

    <ToastContainer />
    <ConfirmModal />
  </div>
</template>

<script setup lang="ts">
import { computed, inject, onMounted, reactive, ref, watchEffect } from 'vue';
import { useRoute } from 'vue-router';
import { appWindow } from '@tauri-apps/api/window';
import { Shield, Activity, Globe, Settings } from 'lucide-vue-next';
import { useVpnStore } from '../stores/vpn';
import { useAppIcon } from '../composables/useAppIcon';
import { initNotificationWatcher } from '../composables/useNotifications';
import { t } from '../i18n';
import ToastContainer from './ToastContainer.vue';
import ConfirmModal from './ConfirmModal.vue';
import BlackHole from './BlackHole.vue';
import Pulsar from './Pulsar.vue';
import Earth from './Earth.vue';
import NebulaEye from './NebulaEye.vue';

const vpnStore = useVpnStore();
const route = useRoute();
const { iconSrc } = useAppIcon();
const onboardingOpen = inject('onboardingOpen', ref(false));

const navItems = [
  { to: '/', key: 'connection', icon: Shield },
  { to: '/analysis', key: 'analysis', icon: Activity },
  { to: '/servers', key: 'servers', icon: Globe },
  { to: '/settings', key: 'settings', icon: Settings },
];

const CACHED_VIEWS = ['AnalysisView'];

const backdrop = computed(() => {
  const on = vpnStore.settings.black_hole_bg;
  const path = route.path;
  return {
    home: on && path === '/' && !onboardingOpen.value,
    settings: on && path === '/settings',
    analysis: on && path === '/analysis',
    servers: on && path === '/servers' && vpnStore.settings.server_view !== 'globe',
  };
});

const spawned = reactive({
  home: false,
  settings: false,
  analysis: false,
  servers: false,
});

watchEffect(() => {
  const want = backdrop.value;
  if (want.home) spawned.home = true;
  if (want.settings) spawned.settings = true;
  if (want.analysis) spawned.analysis = true;
  if (want.servers) spawned.servers = true;
});

const activeIndex = computed(() => navItems.findIndex(item => item.to === route.path));

const pillStyle = computed(() => ({
  transform: `translateY(${Math.max(activeIndex.value, 0) * 40}px)`,
  opacity: activeIndex.value < 0 ? '0' : '1',
}));

onMounted(() => {
  initNotificationWatcher();
});

async function minimize() { await appWindow.minimize(); }
async function toggleMaximize() { await appWindow.toggleMaximize(); }
async function closeWindow() { await appWindow.hide(); }
</script>

<style scoped>
.shell {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: var(--background);
  color: var(--foreground);
  user-select: none;
}

.titlebar {
  position: relative;
  z-index: 2;
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 40px;
  padding: 0 12px 0 16px;
  flex-shrink: 0;
  -webkit-app-region: drag;
  app-region: drag;
}

.titlebar--glass {
  border-bottom: 1px solid color-mix(in oklch, var(--foreground) 10%, transparent);
  background: linear-gradient(
    180deg,
    color-mix(in oklch, var(--foreground) 7%, transparent),
    color-mix(in oklch, var(--foreground) 3%, transparent)
  );
  backdrop-filter: blur(44px) saturate(1.7) brightness(1.06);
  box-shadow: inset 0 1px 0 color-mix(in oklch, var(--foreground) 12%, transparent);
}

.titlebar--clear {
  border-bottom: 1px solid color-mix(in oklch, var(--border) 55%, transparent);
  background: transparent;
}

.titlebar-left {
  display: flex;
  align-items: center;
  gap: 8px;
  -webkit-app-region: drag;
  app-region: drag;
}

.titlebar-icon { width: 18px; height: 18px; object-fit: contain; flex-shrink: 0; }

.titlebar-name {
  font-size: 13px;
  font-weight: 600;
  letter-spacing: -0.01em;
  color: var(--foreground);
}

.titlebar-controls {
  display: flex;
  align-items: center;
  gap: 4px;
  -webkit-app-region: no-drag;
  app-region: no-drag;
}

.ctrl-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: 8px;
  border: none;
  background: transparent;
  color: var(--muted-foreground);
  cursor: pointer;
  transition: background 150ms, color 150ms;
  flex-shrink: 0;
}

.ctrl-btn:hover { color: var(--foreground); background: var(--secondary); }
.ctrl-btn--close:hover {
  background: color-mix(in oklch, var(--destructive) 15%, transparent);
  color: var(--destructive);
}

.body {
  position: relative;
  z-index: 1;
  display: flex;
  flex: 1;
  overflow: hidden;
}

.sidebar {
  width: 200px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  padding: 16px 10px 12px;
}

.sidebar--glass {
  border-right: 1px solid color-mix(in oklch, var(--foreground) 10%, transparent);
  background: linear-gradient(
    180deg,
    color-mix(in oklch, var(--foreground) 6%, transparent),
    color-mix(in oklch, var(--foreground) 2%, transparent)
  );
  backdrop-filter: blur(44px) saturate(1.7) brightness(1.06);
  box-shadow: inset 0 1px 0 color-mix(in oklch, var(--foreground) 10%, transparent);
}

.sidebar--clear {
  border-right: 1px solid color-mix(in oklch, var(--border) 55%, transparent);
  background: transparent;
}

.nav {
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.nav-pill {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 36px;
  border-radius: 8px;
  background: color-mix(in oklch, var(--foreground) 8%, transparent);
  transition: transform 220ms cubic-bezier(0.4, 0, 0.2, 1), opacity 160ms ease;
  pointer-events: none;
}

.nav-link {
  position: relative;
  z-index: 1;
  display: flex;
  align-items: center;
  gap: 10px;
  height: 36px;
  padding: 0 10px;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 500;
  color: var(--muted-foreground);
  text-decoration: none;
  transition: color 150ms, background 150ms;
}

.nav-link:hover {
  color: var(--foreground);
  background: color-mix(in oklch, var(--foreground) 5%, transparent);
}

.nav-link--active { color: var(--foreground); }
.nav-link--active:hover { background: transparent; }

.nav-icon { flex-shrink: 0; }

.nav-text { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }

.sidebar-spacer { flex: 1; }

.sidebar-version {
  padding: 0 10px;
  font-size: 11px;
  color: color-mix(in oklch, var(--muted-foreground) 70%, transparent);
  font-variant-numeric: tabular-nums;
}

.main-content {
  flex: 1;
  overflow-y: auto;
  padding: 28px 28px 40px;
}

.content-inner {
  max-width: 860px;
  margin: 0 auto;
  width: 100%;
}

.mobile-nav { display: none; }

@media (max-width: 768px) {
  .sidebar { display: none; }
  .main-content { padding: 16px 16px 80px; }

  .mobile-nav {
    display: flex;
    position: fixed;
    inset-inline: 0;
    bottom: 0;
    z-index: 50;
    border-top: 1px solid var(--border);
    background: color-mix(in oklch, var(--background) 92%, transparent);
    backdrop-filter: blur(12px);
    align-items: center;
    justify-content: space-around;
    padding: 8px 0;
  }

  .mobile-link {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 3px;
    padding: 6px 16px;
    font-size: 11px;
    font-weight: 500;
    color: var(--muted-foreground);
    text-decoration: none;
    border-radius: 8px;
    transition: color 150ms;
  }

  .mobile-link--active { color: var(--foreground); }
}
.ambient {
  position: fixed;
  inset: 0;
  z-index: 0;
  overflow: hidden;
  pointer-events: none;
}

.ambient-blob {
  position: absolute;
  border-radius: 50%;
  filter: blur(70px);
  will-change: transform;
}

.ambient-blob--violet {
  width: 55vw;
  height: 55vw;
  left: -12vw;
  top: -18vh;
  background: radial-gradient(circle, rgba(139, 92, 246, 0.17), transparent 65%);
  animation: drift-a 26s ease-in-out infinite alternate;
}

.ambient-blob--blue {
  width: 60vw;
  height: 60vw;
  right: -20vw;
  top: 30vh;
  background: radial-gradient(circle, rgba(56, 116, 203, 0.13), transparent 65%);
  animation: drift-b 34s ease-in-out infinite alternate;
}

.ambient-blob--ember {
  width: 40vw;
  height: 40vw;
  left: 25vw;
  bottom: -22vh;
  background: radial-gradient(circle, rgba(190, 100, 60, 0.08), transparent 65%);
  animation: drift-c 30s ease-in-out infinite alternate;
}
.route-enter-active {
  transition: transform 260ms cubic-bezier(0.34, 1.3, 0.64, 1);
}

.route-leave-active {
  transition: opacity 120ms ease, transform 120ms ease;
}

.route-enter-from {
  transform: translateY(16px) scale(0.985);
}

.route-leave-to {
  opacity: 0;
  transform: translateY(-8px) scale(0.99);
}
@keyframes drift-a {
  from { transform: translate(0, 0) scale(1); }
  to { transform: translate(7vw, 5vh) scale(1.12); }
}

@keyframes drift-b {
  from { transform: translate(0, 0) scale(1.08); }
  to { transform: translate(-6vw, -6vh) scale(1); }
}

@keyframes drift-c {
  from { transform: translate(0, 0) scale(1); }
  to { transform: translate(5vw, -4vh) scale(1.15); }
}
</style>
