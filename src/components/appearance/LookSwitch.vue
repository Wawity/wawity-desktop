<template>
  <button
    type="button"
    role="switch"
    :class="['lk-check', { 'lk-check--on': modelValue }]"
    :aria-checked="modelValue"
    :aria-label="label"
    :disabled="disabled"
    @click="emit('update:modelValue', !modelValue)"
  >
    <span :class="['lk-check-mark', { 'lk-check-mark--on': modelValue }]">
      <Check :size="13" aria-hidden="true" />
    </span>
  </button>
</template>

<script setup lang="ts">
import { Check } from '../../lib/appIcons';

defineProps<{ modelValue: boolean; label: string; disabled?: boolean }>();
const emit = defineEmits<{ (e: 'update:modelValue', value: boolean): void }>();
</script>

<style scoped>
.lk-check {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  padding: 0;
  border-radius: 50%;
  border: 1px solid var(--input);
  background: transparent;
  cursor: pointer;
  flex-shrink: 0;
  transition: background 200ms ease, border-color 200ms ease, transform 160ms ease;
}
.lk-check:hover:not(:disabled) {
  border-color: var(--foreground);
  background: color-mix(in oklch, var(--foreground) 18%, transparent);
}
.lk-check--on {
  background: var(--foreground);
  border-color: var(--foreground);
}
.lk-check--on:hover:not(:disabled) {
  background: color-mix(in oklch, var(--foreground) 84%, transparent);
  border-color: transparent;
}
.lk-check:active:not(:disabled) { transform: scale(0.92); }
.lk-check:disabled { opacity: 0.45; cursor: not-allowed; }
.lk-check:focus-visible { outline: 2px solid var(--foreground); outline-offset: 4px; }
.lk-check-mark {
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--background);
  transform: scale(0.5);
  opacity: 0;
  transition: transform 220ms cubic-bezier(0.34, 1.5, 0.64, 1), opacity 160ms ease;
}
.lk-check-mark--on { transform: scale(1); opacity: 1; }
.lk-check-mark svg path {
  stroke-dasharray: 22;
  stroke-dashoffset: 22;
  transition: stroke-dashoffset 300ms cubic-bezier(0.22, 1, 0.36, 1);
}
.lk-check-mark--on svg path { stroke-dashoffset: 0; }
:global(html.motion-simple) .lk-check-mark { transform: none; transition: none; }
:global(html.motion-simple) .lk-check-mark svg path { stroke-dasharray: none; stroke-dashoffset: 0; transition: none; }
@media (prefers-reduced-motion: reduce) {
  .lk-check, .lk-check-mark, .lk-check-mark svg path { transition: none; }
}
</style>
