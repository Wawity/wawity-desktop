<template>
  <span
    ref="host"
    class="thinking-orb-host"
    :style="{ width: preset + 'px', height: preset + 'px' }"
  ></span>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, watch } from 'vue';
import { createElement } from 'react';
import { ThinkingOrb as ThinkingOrbReact } from 'thinking-orbs';
import { mountReactRoot } from '../lib/reactMount';
import type { ReactHandle } from '../lib/reactMount';

type OrbState =
  | 'working'
  | 'searching'
  | 'solving'
  | 'listening'
  | 'connecting'
  | 'weaving'
  | 'composing'
  | 'breathing'
  | 'shaping';

const props = withDefaults(
  defineProps<{
    state?: OrbState;
    size?: number;
    speed?: number;
    paused?: boolean;
    label?: string;
  }>(),
  {
    state: 'solving',
    size: 20,
    speed: 1,
    paused: false,
    label: undefined,
  },
);

const host = ref<HTMLElement | null>(null);
const preset = computed(() => (props.size >= 42 ? 64 : 20));
let handle: ReactHandle | null = null;

function paint() {
  if (!handle) return;
  handle.render(
    createElement(ThinkingOrbReact, {
      state: props.state,
      size: preset.value,
      theme: 'dark',
      speed: props.speed,
      paused: props.paused,
      'aria-label': props.label,
    }),
  );
}

onMounted(() => {
  if (!host.value) return;
  handle = mountReactRoot(host.value);
  paint();
});

watch(() => [props.state, preset.value, props.speed, props.paused, props.label], paint);

onBeforeUnmount(() => {
  handle?.unmount();
  handle = null;
});
</script>

<style scoped>
.thinking-orb-host {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  line-height: 0;
}
</style>
