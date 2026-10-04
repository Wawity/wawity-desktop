<template>
  <button
    type="button"
    role="switch"
    class="wsw"
    :class="{ 'wsw--on': modelValue }"
    :aria-checked="modelValue"
    @click="emit('update:modelValue', !modelValue)"
  >
    <span class="wsw-text">{{ label }}</span>
    <span class="wsw-box"><span class="wsw-cell wsw-cell--a" /><span class="wsw-cell wsw-cell--b" /></span>
  </button>
</template>

<script setup lang="ts">
defineProps<{ modelValue: boolean; label?: string }>();
const emit = defineEmits<{ (e: 'update:modelValue', v: boolean): void }>();
</script>

<style scoped>
.wsw {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  width: 100%;
  padding: 5px 0;
  background: none;
  border: 0;
  color: var(--muted-foreground);
  font-size: 12px;
  cursor: pointer;
  text-align: left;
}
.wsw:hover { color: var(--foreground); }
.wsw-box {
  flex: none;
  display: flex;
  gap: 2px;
  padding: 2px;
  border: 1px solid var(--border);
  border-radius: 4px;
}
.wsw-cell { width: 10px; height: 10px; border-radius: 2px; transition: background 0.18s, box-shadow 0.18s; }
.wsw-cell--a { background: oklch(1 0 0 / 22%); }
.wsw-cell--b { background: transparent; }
.wsw--on { color: var(--paper); }
.wsw--on .wsw-box { border-color: color-mix(in oklch, var(--ember) 60%, transparent); }
.wsw--on .wsw-cell--a { background: transparent; }
.wsw--on .wsw-cell--b { background: var(--ember); box-shadow: 0 0 8px var(--ember); }
</style>
