<template>
  <div class="lk-slider">
    <input
      type="range"
      class="lk-range"
      :min="min"
      :max="max"
      :step="step"
      :value="modelValue"
      :disabled="disabled"
      :aria-label="label"
      :aria-valuetext="shown"
      :style="{ '--fill': fill }"
      :title="defaultValue === undefined ? undefined : tr('Двойной клик — сброс', 'Double-click to reset')"
      @input="onInput"
      @dblclick="resetValue"
    />
    <output class="lk-val">{{ shown }}</output>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { tr } from '../../appearance/i18n';

const props = withDefaults(defineProps<{
  modelValue: number;
  label: string;
  min?: number;
  max?: number;
  step?: number;
  unit?: string;
  disabled?: boolean;
  defaultValue?: number;
}>(), { min: 0, max: 100, step: 1, unit: '', disabled: false });

const emit = defineEmits<{ (e: 'update:modelValue', value: number): void }>();
const value = computed(() => Number.isFinite(props.modelValue) ? props.modelValue : props.min);
const fill = computed(() => `${Math.max(0, Math.min(100, (value.value - props.min) / Math.max(0.001, props.max - props.min) * 100))}%`);
const shown = computed(() => {
  const places = String(props.step).split('.')[1]?.length ?? 0;
  return `${Number(value.value.toFixed(places))}${props.unit}`;
});
function emitValue(v: number) {
  if (!Number.isFinite(v) || props.disabled) return;
  emit('update:modelValue', Math.min(props.max, Math.max(props.min, v)));
}
function onInput(e: Event) { emitValue((e.target as HTMLInputElement).valueAsNumber); }
function resetValue() { if (props.defaultValue !== undefined) emitValue(props.defaultValue); }
</script>

<style scoped>
.lk-slider { display: flex; align-items: center; gap: 10px; width: 220px; max-width: 100%; }
.lk-range { flex: 1; min-width: 0; width: 160px; height: 24px; appearance: none; -webkit-appearance: none; background: transparent; outline: none; cursor: pointer; }
.lk-range::-webkit-slider-runnable-track { height: 2px; background: linear-gradient(90deg, var(--ember) 0 var(--fill), color-mix(in oklch, var(--foreground) 14%, transparent) var(--fill) 100%); }
.lk-range::-webkit-slider-thumb { appearance: none; -webkit-appearance: none; width: 4px; height: 14px; margin-top: -6px; border: 0; border-radius: 1px; background: var(--ember); cursor: pointer; transition: height 140ms ease, margin-top 140ms ease; }
.lk-range:hover::-webkit-slider-thumb, .lk-range:active::-webkit-slider-thumb { height: 18px; margin-top: -8px; }
.lk-range:focus-visible::-webkit-slider-thumb { outline: 1px solid var(--foreground); outline-offset: 3px; }
.lk-range::-moz-range-track { height: 2px; background: color-mix(in oklch, var(--foreground) 14%, transparent); }
.lk-range::-moz-range-progress { height: 2px; background: var(--ember); }
.lk-range::-moz-range-thumb { width: 4px; height: 14px; border: 0; border-radius: 1px; background: var(--ember); }
.lk-range:disabled { opacity: 0.45; cursor: not-allowed; }
.lk-val { width: 48px; text-align: right; font-family: var(--font-mono); font-size: 11px; color: var(--muted-foreground); font-variant-numeric: tabular-nums; }
@media (max-width: 560px) { .lk-slider { width: 100%; } }
@media (prefers-reduced-motion: reduce) { .lk-range::-webkit-slider-thumb { transition: none; } }
</style>
