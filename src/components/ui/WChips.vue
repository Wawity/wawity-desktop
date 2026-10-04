<template>
  <div class="wc">
    <span v-if="label" class="wc-label">{{ label }}</span>
    <div class="wc-row" role="radiogroup">
      <button
        v-for="o in options"
        :key="String(o.value)"
        type="button"
        role="radio"
        class="wc-chip"
        :class="{ 'wc-chip--on': o.value === modelValue }"
        :aria-checked="o.value === modelValue"
        @click="emit('update:modelValue', o.value)"
      >
        {{ o.label }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{ modelValue: string | number; options: Array<{ value: string | number; label: string }>; label?: string }>();
const emit = defineEmits<{ (e: 'update:modelValue', v: any): void }>();
</script>

<style scoped>
.wc { display: flex; flex-direction: column; gap: 6px; }
.wc-label { font-size: 11.5px; color: var(--muted-foreground); }
.wc-row { display: flex; flex-wrap: wrap; gap: 4px; }
.wc-chip {
  position: relative;
  padding: 4px 9px 5px;
  font-size: 11px;
  color: var(--muted-foreground);
  background: oklch(1 0 0 / 3%);
  border: 1px solid var(--border);
  border-radius: 4px;
  cursor: pointer;
  transition: color 0.15s, border-color 0.15s, background 0.15s;
}
.wc-chip:hover { color: var(--foreground); border-color: oklch(1 0 0 / 22%); }
.wc-chip--on {
  color: var(--paper);
  border-color: color-mix(in oklch, var(--ember) 65%, transparent);
  background: color-mix(in oklch, var(--ember) 14%, transparent);
}
.wc-chip--on::after {
  content: '';
  position: absolute;
  left: 6px;
  right: 6px;
  bottom: -1px;
  height: 2px;
  border-radius: 2px;
  background: var(--ember);
  box-shadow: 0 0 6px var(--ember);
}
</style>
