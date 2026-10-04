<template>
  <span
    v-if="visible"
    class="pa"
    :class="{
      'pa--on': connected,
      'pa--busy': loading && cfg.pulse,
      'pa--gray': cfg.grayscaleOff && !connected,
      'pa--spin': cfg.spin,
      'pa--fade': cfg.edgeFade,
    }"
    :style="vars"
    aria-hidden="true"
  >
    <video
      v-if="videoSrc"
      ref="videoRef"
      class="pa-video"
      :src="videoSrc"
      autoplay
      loop
      muted
      playsinline
      preload="auto"
    />
    <span v-else class="pa-img" :style="imgStyle" />
  </span>
</template>

<script setup lang="ts">
// Faint picture/pattern layer inside the round connect button.
// Place it as the first child of <button class="power-btn">; the button
// already has overflow:hidden and its svg/label sit at z-index 1.
import { computed, nextTick, ref, watch } from 'vue';
import { useAppearance, type PowerArtSettings } from '../../appearance/store';
import { patternHas, patternKind, patternUrl, phaseOf } from '../../appearance/patterns';

const props = defineProps<{ connected?: boolean; loading?: boolean; override?: PowerArtSettings }>();
const a = useAppearance();
const cfg = computed(() => props.override ?? a.state.power);

const videoRef = ref<HTMLVideoElement | null>(null);

const phase = computed(() => phaseOf(props.connected, props.loading));

const videoSrc = computed(() => {
  const src = cfg.value.source;
  if (src === 'custom') return a.powerVideoUrl;
  if (src === 'none') return '';
  if (patternKind(src) !== 'video') return '';
  // A pack with only some of its clips must not point a <video> at a file
  // that was never dropped in; that state falls back to nothing instead.
  return patternHas(src, phase.value) ? patternUrl(src, phase.value) : '';
});

const visible = computed(() => {
  const src = cfg.value.source;
  if (src === 'none') return false;
  if (src === 'custom') return !!a.powerUrl || !!a.powerVideoUrl;
  // A pattern without its file on disk must not paint a broken tile. The
  // picker already filters those out; this covers a stale saved choice.
  return patternKind(src) !== null;
});

// Swapping src does not autoplay by itself once the element exists, and button
// art frozen on a first frame reads as a still image.
watch(videoSrc, (src) => {
  if (!src) return;
  void nextTick(() => {
    const el = videoRef.value;
    if (!el) return;
    el.currentTime = 0;
    void el.play().catch(() => {});
  });
});

const vars = computed(() => ({
  '--pa-off': String(cfg.value.opacityOff / 100),
  '--pa-on': String(cfg.value.opacityOn / 100),
  '--pa-blur': `${cfg.value.blur}px`,
  '--pa-zoom': String(cfg.value.zoom),
  '--pa-x': `${cfg.value.posX}%`,
  '--pa-y': `${cfg.value.posY}%`,
  mixBlendMode: cfg.value.blend as any,
}));

const imgStyle = computed(() => {
  const src = cfg.value.source;
  if (src === 'custom') {
    return a.powerUrl ? { backgroundImage: `url("${a.powerUrl}")`, backgroundSize: 'cover' } : {};
  }
  if (src !== 'none') {
    const url = patternUrl(src, phase.value);
    if (!url) return {};
    // cover, not repeat: these are pictures, and tiling one reads as a bug.
    return { backgroundImage: `url("${url}")`, backgroundSize: 'cover', backgroundRepeat: 'no-repeat' };
  }
  return {};
});
</script>

<style scoped>
.pa {
  position: absolute;
  inset: 0;
  border-radius: 50%;
  overflow: hidden;
  pointer-events: none;
  z-index: 0;
  opacity: var(--pa-off);
  transition: opacity 0.7s ease;
}
.pa--fade {
  -webkit-mask-image: radial-gradient(circle at 50% 50%, #000 42%, transparent 71%);
  mask-image: radial-gradient(circle at 50% 50%, #000 42%, transparent 71%);
}
.pa--on { opacity: var(--pa-on); }
.pa-img {
  position: absolute;
  inset: -10%;
  background-position: var(--pa-x) var(--pa-y);
  background-repeat: no-repeat;
  transform: scale(var(--pa-zoom));
  filter: blur(var(--pa-blur));
  transition: filter 0.7s ease, transform 0.7s ease;
}
.pa--gray .pa-img { filter: blur(var(--pa-blur)) grayscale(1); }
.pa-video {
  position: absolute;
  inset: -10%;
  width: calc(100% + 20%);
  height: calc(100% + 20%);
  object-fit: cover;
  object-position: var(--pa-x) var(--pa-y);
  transform: scale(var(--pa-zoom));
  filter: blur(var(--pa-blur));
  transition: filter 0.7s ease, transform 0.7s ease;
}
.pa--gray .pa-video { filter: blur(var(--pa-blur)) grayscale(1); }
.pa--spin .pa-video { animation: pa-spin 40s linear infinite; }
.pa--busy { animation: pa-breathe 1.5s ease-in-out infinite; }
.pa--spin .pa-img { animation: pa-spin 40s linear infinite; }
@keyframes pa-breathe {
  0%, 100% { opacity: var(--pa-off); }
  50% { opacity: var(--pa-on); }
}
@keyframes pa-spin { to { rotate: 360deg; } }
</style>
