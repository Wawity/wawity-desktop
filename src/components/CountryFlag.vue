<template>
  <div class="flag-box" :style="boxStyle">
    <img
      v-if="!broken && resolvedSrc"
      :src="resolvedSrc"
      :width="width"
      :height="height"
      class="flag-img"
      :alt="code"
      loading="lazy"
      decoding="async"
      @error="broken = true"
    />
    <div v-else class="flag-fallback" :style="boxStyle">
      <svg
        :width="width * 0.55"
        :height="height * 0.55"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="1.5"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <circle cx="12" cy="12" r="10"/>
        <path d="M2 12h20M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/>
      </svg>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';

const props = withDefaults(defineProps<{
  code: string;
  width?: number;
  height?: number;
}>(), {
  width: 32,
  height: 24,
});

const broken = ref(false);

const resolvedSrc = computed(() => {
  if (!props.code || props.code === 'UN') return null;
  return `/flags/${props.code.toLowerCase()}.svg`;
});

watch(() => props.code, () => { broken.value = false; });

const boxStyle = computed(() => ({
  width: props.width + 'px',
  height: props.height + 'px',
}));
</script>

<style scoped>
.flag-box {
  flex-shrink: 0;
  border-radius: 4px;
  overflow: hidden;
  position: relative;
  background: var(--secondary);
}

.flag-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}

.flag-fallback {
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--muted-foreground);
  background: var(--secondary);
  border-radius: 4px;
}
</style>