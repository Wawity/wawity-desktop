<template>
  <div class="tool-stage">
    <header class="tool-topbar">
      <button class="nav-back" type="button" :aria-label="t('extra.back')" @click="leave">
        <ArrowLeft :size="15" aria-hidden="true" />
        <span v-text="t('extra.back')" />
      </button>
      <span class="crumb mono" aria-hidden="true">
        <span class="crumb-led" />
        wawity<span class="crumb-slash">/</span>tools
      </span>
    </header>

    <section class="tool-hero">
      <div class="hero-copy">
        <h1 class="hero-title" v-text="title" />
        <p v-if="subtitle" class="hero-subtitle" v-text="subtitle" />
      </div>
      <div v-if="$slots.actions" class="hero-actions">
        <slot name="actions" />
      </div>
    </section>

    <main class="tool-scroll">
      <div class="tool-content">
        <slot />
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import type { Component } from 'vue';
import { useRouter } from 'vue-router';
import { ArrowLeft } from '../lib/appIcons';
import { t } from '../i18n';

withDefaults(
  defineProps<{
    title: string;
    subtitle?: string;
    icon?: Component;
  }>(),
  { subtitle: '' },
);

const router = useRouter();

function leave() {
  if (window.history.length > 1) {
    router.back();
    return;
  }
  router.push('/');
}
</script>

<style scoped>
.tool-stage {
  --good: var(--success);
  --warn: #f0d36a;
  --bad: var(--destructive);
  --surface: #171615;
  --surface-2: #201f1e;
  --hairline: rgba(255, 255, 255, 0.08);
  --hairline-2: rgba(255, 255, 255, 0.16);
  position: relative;
  z-index: 1;
  display: flex;
  min-height: 0;
  flex: 1;
  flex-direction: column;
}

.mono {
  font-family: var(--font-mono);
  font-variant-numeric: tabular-nums;
}

.tool-topbar {
  display: flex;
  flex-shrink: 0;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 4px 2px 16px;
  animation: hero-rise 420ms cubic-bezier(0.22, 0.61, 0.36, 1) both;
}

.nav-back {
  display: inline-flex;
  height: 34px;
  align-items: center;
  gap: 7px;
  padding: 0 15px 0 11px;
  border-radius: 999px;
  border: 1px solid var(--hairline);
  background: var(--surface);
  color: var(--muted-foreground);
  font: inherit;
  font-size: 12.5px;
  font-weight: 500;
  cursor: pointer;
  transition:
    color 180ms ease,
    background 180ms ease,
    transform 180ms ease;
}

.nav-back:hover {
  transform: translateX(-2px);
  background: var(--surface-2);
  color: var(--foreground);
}

.nav-back:active {
  transform: translateX(0) scale(0.97);
}

.crumb {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  font-family: var(--font-mono);
  font-size: 10px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--muted-foreground);
  opacity: 0.75;
}

.crumb-led {
  width: 5px;
  height: 5px;
  border-radius: 50%;
  background: var(--success);
}

.crumb-slash {
  margin: 0 3px;
  opacity: 0.5;
}

.tool-hero {
  display: flex;
  flex-shrink: 0;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 2px 2px 22px;
  animation: hero-rise 460ms 50ms cubic-bezier(0.22, 0.61, 0.36, 1) both;
}

.hero-lead {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: 14px;
}

.hero-badge {
  position: relative;
  display: grid;
  width: 46px;
  height: 46px;
  flex-shrink: 0;
  place-items: center;
  border-radius: 14px;
  border: 1px solid var(--hairline);
  background: var(--surface);
  color: var(--foreground);
}

.hero-copy {
  display: flex;
  min-width: 0;
  flex-direction: column;
  gap: 3px;
}

.hero-title {
  overflow: hidden;
  margin: 0;
  font-size: 22px;
  font-weight: 600;
  letter-spacing: -0.02em;
  line-height: 1.15;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.hero-subtitle {
  overflow: hidden;
  margin: 0;
  color: var(--muted-foreground);
  font-size: 12.5px;
  line-height: 1.45;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.hero-actions {
  display: flex;
  min-width: 0;
  flex-shrink: 0;
  align-items: center;
  gap: 8px;
}

.tool-scroll {
  min-height: 0;
  flex: 1;
  overflow-x: hidden;
  overflow-y: auto;
  scrollbar-gutter: stable;
}

.tool-content {
  width: 100%;
  padding: 0 2px 40px;
  animation: hero-rise 500ms 90ms cubic-bezier(0.22, 0.61, 0.36, 1) both;
}

.tool-scroll::-webkit-scrollbar {
  width: 5px;
}

.tool-scroll::-webkit-scrollbar-thumb {
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.12);
}

@keyframes hero-rise {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.tool-stage :deep(.glass) {
  position: relative;
  border-radius: 16px;
  border: 1px solid var(--hairline);
  background: var(--surface);
  box-shadow: none;
  backdrop-filter: none;
  -webkit-backdrop-filter: none;
}

.tool-stage :deep(.glass::before) {
  display: none;
}

.tool-stage :deep(.glass-pill) {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  padding: 7px 13px;
  border-radius: 999px;
  border: 1px solid var(--hairline);
  background: var(--surface);
  backdrop-filter: none;
  -webkit-backdrop-filter: none;
  box-shadow: none;
  color: var(--muted-foreground);
  font-size: 11.5px;
}

.tool-stage :deep(.tilt3d) {
  transform: none !important;
  transition: none;
}

.tool-stage :deep(.tilt-pop),
.tool-stage :deep(.tilt-pop-soft) {
  transform: none;
}

.tool-stage :deep(.tool-action) {
  display: inline-flex;
  height: 34px;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  gap: 7px;
  padding: 0 16px;
  border-radius: 999px;
  border: 1px solid var(--hairline-2);
  background: transparent;
  backdrop-filter: none;
  -webkit-backdrop-filter: none;
  box-shadow: none;
  color: var(--foreground);
  font: inherit;
  font-size: 12px;
  font-weight: 500;
  white-space: nowrap;
  cursor: pointer;
  transition:
    background 160ms ease,
    border-color 160ms ease,
    opacity 160ms ease,
    transform 160ms ease;
}

.tool-stage :deep(.tool-action:hover:not(:disabled)) {
  background: var(--surface-2);
  border-color: var(--hairline-2);
  color: var(--foreground);
}

.tool-stage :deep(.tool-action:active:not(:disabled)) {
  transform: scale(0.97);
}

.tool-stage :deep(.tool-action--primary) {
  background: #ffffff;
  border-color: #ffffff;
  color: #000000;
  font-weight: 600;
  box-shadow: none;
}

.tool-stage :deep(.tool-action--primary:hover:not(:disabled)) {
  background: #e9e7e4;
  border-color: #e9e7e4;
  color: #000000;
  box-shadow: none;
}

.tool-stage :deep(.tool-action--active) {
  background: var(--surface-2);
  color: var(--foreground);
}

.tool-stage :deep(.tool-action:disabled) {
  opacity: 0.45;
  cursor: default;
}

.tool-stage :deep(.tool-error) {
  margin: 0 0 14px;
  padding: 11px 15px;
  border-radius: 12px;
  border: 1px solid rgba(255, 91, 96, 0.25);
  border-left: 2px solid #ff5b60;
  background: rgba(255, 91, 96, 0.07);
  backdrop-filter: none;
  -webkit-backdrop-filter: none;
  box-shadow: none;
  color: #ff8a92;
  font-size: 12.5px;
  line-height: 1.45;
}

.tool-stage :deep(.tool-spin) {
  animation: tool-spin 850ms linear infinite;
}

@keyframes tool-spin {
  to {
    transform: rotate(360deg);
  }
}

.tool-stage :deep(.hero-glow),
.tool-stage :deep(.aurora),
.tool-stage :deep(.grain) {
  display: none;
}

@media (max-width: 720px) {
  .tool-hero {
    flex-wrap: wrap;
  }

  .hero-actions {
    width: 100%;
  }

  .hero-subtitle {
    white-space: normal;
  }
}

@media (prefers-reduced-motion: reduce) {
  .tool-topbar,
  .tool-hero,
  .tool-content {
    animation: none;
  }
}
</style>