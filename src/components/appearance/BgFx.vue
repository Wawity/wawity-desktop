<template>
  <div v-if="a.state.fx.enabled" class="bgfx" aria-hidden="true">
    <span v-if="a.state.fx.tint > 0" class="bgfx-tint" />
    <span v-if="a.state.fx.vignette > 0" class="bgfx-vig" />
    <span v-if="a.state.fx.grain > 0" class="bgfx-grain" />
  </div>
</template>

<script setup lang="ts">
// Film layer over any background (live scenes or custom picture):
// accent tint, vignette, animated grain. Put right after .stage-scrim in Layout.vue.
import { useAppearance } from '../../appearance/store';
const a = useAppearance();
</script>

<style scoped>
.bgfx { position: fixed; inset: 0; z-index: 0; pointer-events: none; }
.bgfx > span { position: absolute; inset: 0; }
.bgfx-tint { background: var(--ember); mix-blend-mode: soft-light; opacity: calc(var(--fx-tint) * 0.85); }
.bgfx-vig {
  background: radial-gradient(ellipse at 50% 45%, transparent 38%, rgba(0, 0, 0, 0.95) 100%);
  opacity: var(--fx-vignette);
  transition: opacity 0.8s ease;
}
:global(html.fx-reactive.wawity-vpn-off) .bgfx-vig { opacity: min(1, calc(var(--fx-vignette) + 0.25)); }
.bgfx-grain {
  inset: -50%;
  background-image: url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='180' height='180'><filter id='n'><feTurbulence type='fractalNoise' baseFrequency='.9' numOctaves='2' stitchTiles='stitch'/><feColorMatrix values='0 0 0 0 1  0 0 0 0 1  0 0 0 0 1  0 0 0 .55 0'/></filter><rect width='100%' height='100%' filter='url(%23n)'/></svg>");
  opacity: calc(var(--fx-grain) * 0.6);
  mix-blend-mode: overlay;
  animation: grain 0.9s steps(6) infinite;
}
@keyframes grain {
  0% { transform: translate(0, 0); }
  20% { transform: translate(-6%, 4%); }
  40% { transform: translate(5%, -7%); }
  60% { transform: translate(-3%, 8%); }
  80% { transform: translate(7%, 2%); }
  100% { transform: translate(0, 0); }
}
:global(html.motion-simple) .bgfx-grain { animation: none; }
</style>
