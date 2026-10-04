<template>
  <div class="ws" :class="{ 'ws--drag': dragging, 'ws--off': disabled }">
    <div v-if="label" class="ws-head">
      <span class="ws-label">{{ label }}</span>
      <span class="ws-val">{{ display }}</span>
    </div>
    <div
      ref="track"
      class="ws-track"
      role="slider"
      tabindex="0"
      :aria-label="label"
      :aria-valuemin="min"
      :aria-valuemax="max"
      :aria-valuenow="modelValue"
      :title="defaultValue !== undefined ? hint : undefined"
      @pointerdown="onDown"
      @pointermove="onMove"
      @pointerup="onUp"
      @pointercancel="onUp"
      @pointerleave="hover = -1"
      @wheel.prevent="onWheel"
      @keydown="onKey"
      @dblclick="resetDefault"
    >
      <span
        v-for="i in TICKS"
        :key="i"
        class="ws-tick"
        :class="{
          'ws-tick--on': i - 1 < filled,
          'ws-tick--head': i - 1 === headIndex,
          'ws-tick--major': (i - 1) % 6 === 0,
        }"
        :style="tickStyle(i - 1)"
      />
      <span v-if="defaultValue !== undefined" class="ws-def" :style="{ left: defPct + '%' }" />
    </div>
  </div>
</template>

<script setup lang="ts">
// Custom "ruler / equalizer" slider: segmented ticks fill with the accent,
// the head tick glows, ticks near the cursor lift. Drag, wheel, arrows,
// double-click = back to default.
import { computed, ref } from 'vue';
import { playSound } from '../../appearance/sounds';

const props = withDefaults(
  defineProps<{
    modelValue: number;
    min?: number;
    max?: number;
    step?: number;
    label?: string;
    unit?: string;
    defaultValue?: number;
    disabled?: boolean;
    format?: (v: number) => string;
  }>(),
  { min: 0, max: 100, step: 1, unit: '' },
);
const emit = defineEmits<{ (e: 'update:modelValue', v: number): void }>();

const TICKS = 30;
const track = ref<HTMLElement | null>(null);
const dragging = ref(false);
const hover = ref(-1);
const hint = 'Двойной клик — по умолчанию · колесо — точнее';

const ratio = computed(() => {
  const r = (props.modelValue - props.min) / (props.max - props.min || 1);
  return Math.max(0, Math.min(1, r));
});
const filled = computed(() => ratio.value * TICKS);
const headIndex = computed(() => Math.max(0, Math.ceil(filled.value) - 1));
const defPct = computed(() =>
  props.defaultValue === undefined ? 0 : ((props.defaultValue - props.min) / (props.max - props.min || 1)) * 100,
);
const decimals = computed(() => {
  const s = String(props.step);
  return s.includes('.') ? s.split('.')[1].length : 0;
});
const display = computed(() =>
  props.format ? props.format(props.modelValue) : `${props.modelValue.toFixed(decimals.value)}${props.unit}`,
);

function tickStyle(i: number) {
  if (hover.value < 0 || props.disabled) return undefined;
  const d = Math.abs(i - hover.value);
  if (d > 3) return undefined;
  return { transform: `scaleY(${1 + (3 - d) * 0.14})` };
}

function commit(v: number) {
  const stepped = Math.round((v - props.min) / props.step) * props.step + props.min;
  const clamped = Math.max(props.min, Math.min(props.max, Number(stepped.toFixed(decimals.value))));
  if (clamped !== props.modelValue) {
    emit('update:modelValue', clamped);
    playSound('tick');
  }
}

function fromEvent(e: PointerEvent) {
  const el = track.value;
  if (!el) return;
  const r = el.getBoundingClientRect();
  const x = Math.max(0, Math.min(1, (e.clientX - r.left) / r.width));
  hover.value = Math.floor(x * TICKS);
  return props.min + x * (props.max - props.min);
}

function onDown(e: PointerEvent) {
  if (props.disabled || e.button !== 0) return;
  dragging.value = true;
  (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  const v = fromEvent(e);
  if (v !== undefined) commit(v);
}
function onMove(e: PointerEvent) {
  const v = fromEvent(e);
  if (dragging.value && v !== undefined) commit(v);
}
function onUp(e: PointerEvent) {
  dragging.value = false;
  try {
    (e.currentTarget as HTMLElement).releasePointerCapture(e.pointerId);
  } catch {}
}
function onWheel(e: WheelEvent) {
  if (props.disabled) return;
  commit(props.modelValue + (e.deltaY < 0 ? 1 : -1) * props.step);
}
function onKey(e: KeyboardEvent) {
  if (props.disabled) return;
  const big = (props.max - props.min) / 10;
  const map: Record<string, number> = {
    ArrowRight: props.step,
    ArrowUp: props.step,
    ArrowLeft: -props.step,
    ArrowDown: -props.step,
    PageUp: big,
    PageDown: -big,
  };
  if (e.key === 'Home') commit(props.min);
  else if (e.key === 'End') commit(props.max);
  else if (map[e.key] !== undefined) commit(props.modelValue + map[e.key]);
  else return;
  e.preventDefault();
}
function resetDefault() {
  if (props.defaultValue !== undefined && !props.disabled) commit(props.defaultValue);
}
</script>

<style scoped>
.ws { display: flex; flex-direction: column; gap: 6px; user-select: none; }
.ws--off { opacity: 0.4; pointer-events: none; }
.ws-head { display: flex; align-items: baseline; justify-content: space-between; gap: 8px; }
.ws-label { font-size: 11.5px; color: var(--muted-foreground); }
.ws-val {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--paper);
  padding: 1px 6px;
  border: 1px solid var(--border);
  border-radius: 4px;
  min-width: 44px;
  text-align: center;
  transition: border-color 0.2s, color 0.2s;
}
.ws--drag .ws-val { border-color: var(--ember); color: var(--ember-soft); }

.ws-track {
  position: relative;
  height: 22px;
  display: grid;
  grid-template-columns: repeat(30, 1fr);
  align-items: center;
  gap: 2px;
  padding: 0 1px;
  cursor: ew-resize;
  outline: none;
  border-radius: 4px;
}
.ws-track:focus-visible { box-shadow: 0 0 0 1px var(--ember); }
.ws-tick {
  height: 8px;
  border-radius: 1px;
  background: oklch(1 0 0 / 9%);
  transform-origin: center;
  transition: transform 0.18s cubic-bezier(0.22, 1, 0.36, 1), background 0.15s, height 0.18s, box-shadow 0.2s;
}
.ws-tick--major { height: 12px; }
.ws-tick--on { background: color-mix(in oklch, var(--ember) 70%, transparent); height: 12px; }
.ws-tick--on.ws-tick--major { height: 15px; }
.ws-tick--head {
  height: 20px !important;
  background: var(--ember);
  box-shadow: 0 0 10px color-mix(in oklch, var(--ember) 80%, transparent), 0 0 2px var(--ember-soft);
}
.ws-def {
  position: absolute;
  bottom: -3px;
  width: 0;
  height: 0;
  border-left: 3px solid transparent;
  border-right: 3px solid transparent;
  border-bottom: 4px solid oklch(1 0 0 / 35%);
  transform: translateX(-50%);
  pointer-events: none;
}
</style>
