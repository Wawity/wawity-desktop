<template>
  <div :class="['lk-row', { 'lk-row--stack': stack }]">
    <div class="lk-row-left">
      <component :is="icon" v-if="icon" :size="16" class="lk-row-icon" aria-hidden="true" />
      <div class="lk-row-text">
        <p class="lk-row-title">{{ title }}</p>
        <p v-if="desc" class="lk-row-desc">{{ desc }}</p>
      </div>
    </div>
    <div v-if="$slots.default" class="lk-row-ctrl"><slot /></div>
  </div>
</template>

<script setup lang="ts">
import type { Component } from 'vue';
defineProps<{ title: string; desc?: string; icon?: Component; stack?: boolean }>();
</script>

<style scoped>
.lk-row { display: flex; align-items: center; justify-content: space-between; gap: 16px; padding: 17px 2px; border-bottom: 1px solid var(--border); }
.lk-row:last-child { border-bottom: 0; }
.lk-row-left { display: flex; align-items: flex-start; gap: 12px; min-width: 0; }
.lk-row-icon { color: var(--muted-foreground); flex-shrink: 0; margin-top: 2px; }
.lk-row-text { display: flex; flex-direction: column; gap: 2px; min-width: 0; }
.lk-row-title { margin: 0; font-family: var(--font-display); font-size: 14.5px; font-weight: 400; letter-spacing: -0.015em; overflow-wrap: anywhere; }
.lk-row-desc { margin: 0; font-size: 12px; line-height: 1.5; color: var(--muted-foreground); max-width: 52ch; }
.lk-row-ctrl { display: flex; align-items: center; gap: 8px; flex-shrink: 0; max-width: 100%; }
.lk-row--stack { flex-direction: column; align-items: stretch; gap: 12px; }
.lk-row--stack .lk-row-ctrl { flex-wrap: wrap; flex-shrink: 1; }
@media (max-width: 560px) {
  .lk-row:has(.lk-slider) { flex-direction: column; align-items: stretch; gap: 10px; }
  .lk-row:has(.lk-slider) .lk-row-ctrl { width: 100%; }
}
</style>
