<template>
  <span
    class="sm-logo"
    :style="{ width: size + 'px', height: size + 'px' }"
    aria-hidden="true"
  ></span>
</template>

<script setup lang="ts">
import { computed, defineProps, withDefaults } from 'vue';
import { BRAND_LOGOS } from '../lib/brandLogos';

const props = withDefaults(
  defineProps<{
    domain: string;
    tint: string;
    size?: number;
  }>(),
  { size: 21 },
);

const ALIASES: Record<string, string> = {
  'twitter.com': 'x.com',
};

function resolveKey(domain: string): string {
  const d = (domain || '').toLowerCase();
  if (BRAND_LOGOS[d]) return d;
  if (ALIASES[d] && BRAND_LOGOS[ALIASES[d]]) return ALIASES[d];
  const parts = d.split('.');
  for (let i = 1; i < parts.length - 1; i++) {
    const candidate = parts.slice(i).join('.');
    if (BRAND_LOGOS[candidate]) return candidate;
  }
  return '';
}

const GLOBE_FALLBACK =
  '<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><g fill="none" stroke="#ffffff" stroke-width="1.6"><circle cx="12" cy="12" r="8.4"/><ellipse cx="12" cy="12" rx="3.5" ry="8.4"/><path d="M4 9.4h16M4 14.6h16"/></g></svg>';

const logo = computed(() => {
  const key = resolveKey(props.domain);
  const raw = key ? BRAND_LOGOS[key] : GLOBE_FALLBACK;
  
  return raw.replace(/<svg\b/, `<svg width="${props.size}" height="${props.size}"`);
});
</script>

<style scoped>
.sm-logo :global(svg) {
  display: block;
}
</style>
