<template>
  <div class="chart-wrap">
    <svg
      :viewBox="`0 0 ${W} ${H}`"
      class="chart-svg"
      preserveAspectRatio="none"
      role="img"
      aria-label="Download and upload throughput over the last 24 hours"
    >
      <line
        v-for="t in [0.25, 0.5, 0.75]"
        :key="t"
        x1="0"
        :x2="W"
        :y1="H * t"
        :y2="H * t"
        class="grid-line"
        stroke-width="1"
        stroke-dasharray="4 6"
      />
      <path :d="downloadArea" class="dl-area" />
      <path :d="downloadLine" class="dl-line" stroke-width="2" fill="none" />
      <path :d="uploadLine" class="ul-line" stroke-width="1.5" fill="none" />
    </svg>
    <div class="chart-labels">
      <span>00:00</span>
      <span>06:00</span>
      <span>12:00</span>
      <span>18:00</span>
      <span>24:00</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';

const props = withDefaults(defineProps<{
  download?: number[];
  upload?: number[];
}>(), {
  download: () => [22, 35, 28, 48, 62, 55, 74, 68, 82, 71, 88, 79, 64, 58, 70, 84, 92, 76, 61, 55, 68, 73, 66, 58],
  upload: () => [8, 12, 10, 15, 20, 18, 24, 22, 28, 25, 30, 27, 21, 19, 23, 28, 32, 26, 20, 18, 22, 25, 21, 19],
});

const W = 600;
const H = 180;
const MAX = 100;

function buildPath(data: number[]): string {
  const step = W / (data.length - 1);
  return data.map((v, i) => `${i === 0 ? 'M' : 'L'}${(i * step).toFixed(1)},${(H - (v / MAX) * H).toFixed(1)}`).join(' ');
}

const downloadLine = computed(() => buildPath(props.download));
const uploadLine = computed(() => buildPath(props.upload));
const downloadArea = computed(() => `${buildPath(props.download)} L${W},${H} L0,${H} Z`);
</script>

<style scoped>
.chart-wrap {
  width: 100%;
}

.chart-svg {
  width: 100%;
  height: 160px;
  display: block;
}

.grid-line {
  stroke: var(--border);
}

.dl-area {
  fill: color-mix(in oklch, var(--success) 10%, transparent);
}

.dl-line {
  stroke: var(--success);
}

.ul-line {
  stroke: var(--muted-foreground);
}

.chart-labels {
  display: flex;
  justify-content: space-between;
  margin-top: 8px;
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--muted-foreground);
}
</style>