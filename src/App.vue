<template>
  <div id="app-root">
    <AppShell />
    <Onboarding v-if="showOnboarding" @done="completeOnboarding" />
  </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, provide, ref } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import { useRouter } from 'vue-router';
import AppShell from './components/Layout.vue';
import Onboarding from './components/Onboarding.vue';
import { useVpnStore } from './stores/vpn';
import { gsap } from 'gsap';
import { warmUpMotion } from './lib/motion';

const store = useVpnStore();
const router = useRouter();
const showOnboarding = ref(false);
provide('onboardingOpen', showOnboarding);
const MIN_SPLASH_VISIBLE_MS = 400;
const MAX_BOOT_WAIT_MS = 12000;
const mountedAt = Date.now();

let reducedMotion = false;
let pressHandler: ((e: MouseEvent) => void) | null = null;

function bindPressFeedback() {
  reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  if (reducedMotion) return;
  pressHandler = (e: MouseEvent) => {
    const target = e.target as HTMLElement | null;
    const btn = target?.closest('button') as HTMLElement | null;
    if (!btn || btn.disabled) return;
    gsap.killTweensOf(btn);
    gsap
      .timeline()
      .to(btn, { scale: 0.965, duration: 0.07, ease: 'power2.out' })
      .to(btn, { scale: 1, duration: 0.32, ease: 'back.out(2.2)' });
  };
  window.addEventListener('pointerdown', pressHandler, { passive: true });
}

function withHardTimeout<T>(promise: Promise<T>, ms: number): Promise<T | void> {
  return new Promise((resolve) => {
    let settled = false;
    const timer = window.setTimeout(() => {
      if (!settled) {
        settled = true;
        resolve();
      }
    }, ms);

    promise
      .then((value) => {
        if (!settled) {
          settled = true;
          window.clearTimeout(timer);
          resolve(value);
        }
      })
      .catch(() => {
        if (!settled) {
          settled = true;
          window.clearTimeout(timer);
          resolve();
        }
      });
  });
}

async function completeOnboarding(payload: { goToServers: boolean }) {
  showOnboarding.value = false;
  await invoke('finish_first_launch').catch(() => {});
  if (payload.goToServers) {
    router.push('/servers');
  }
}

onMounted(async () => {
  bindPressFeedback();
  warmUpMotion();
  const launchedHidden = await invoke<boolean>('is_launched_hidden').catch(() => false);

  await withHardTimeout(store.boot(), MAX_BOOT_WAIT_MS);

  if (!launchedHidden) {
    const firstLaunch = await invoke<boolean>('is_first_launch').catch(() => false);
    if (firstLaunch) {
      showOnboarding.value = true;
    }
  }

  const elapsed = Date.now() - mountedAt;
  const remaining = Math.max(0, MIN_SPLASH_VISIBLE_MS - elapsed);

  window.setTimeout(() => {
    if (launchedHidden) {
      invoke('close_splash_only').catch(() => {});
    } else {
      invoke('show_main_window').catch(() => {});
    }
  }, remaining);
});

onUnmounted(() => {
  store.stopPolling();
  if (pressHandler) {
    window.removeEventListener('pointerdown', pressHandler);
    pressHandler = null;
  }
});
</script>

<style>
#app-root {
  height: 100vh;
  overflow: hidden;
}
</style>

