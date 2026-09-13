<template>
  <Teleport to="body">
    <Transition name="hint-drop">
      <div v-if="hint.visible" class="copy-hint" role="status" aria-live="polite">
        <Check :size="12" />
        <span v-text="hint.text" />
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { watch, nextTick } from 'vue';
import { gsap } from 'gsap';
import { Check } from '../lib/appIcons';
import { useCopyHint } from '../composables/useCopyHint';
import { prefersReduced } from '../lib/motion';

const hint = useCopyHint();

watch(
  () => hint.visible,
  async (visible) => {
    if (!visible) return;
    await nextTick();
    const el = document.querySelector('.copy-hint');
    if (!el || prefersReduced()) return;
    gsap.fromTo(
      el,
      { y: -16, opacity: 0, filter: 'blur(6px)', scale: 0.92 },
      {
        y: 0,
        opacity: 1,
        filter: 'blur(0px)',
        scale: 1,
        duration: 0.42,
        ease: 'expo.out',
        clearProps: 'filter',
      },
    );
  },
);
</script>

<style scoped>
.copy-hint {
  position: fixed;
  top: 18px;
  left: 50%;
  z-index: 1200;
  display: inline-flex;
  align-items: center;
  gap: 7px;
  padding: 7px 14px;
  border-radius: 999px;
  border: 1px solid rgba(167, 139, 250, 0.4);
  background: linear-gradient(180deg, rgba(48, 42, 74, 0.95), rgba(30, 27, 48, 0.95));
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.12),
    0 8px 24px rgba(0, 0, 0, 0.45);
  color: #d9ccff;
  font-size: 11.5px;
  font-weight: 500;
  pointer-events: none;
  transform: translate(-50%, 0);
}

.hint-drop-leave-active {
  transition:
    opacity 300ms ease,
    filter 300ms ease,
    transform 300ms ease;
}

.hint-drop-leave-to {
  opacity: 0;
  filter: blur(6px);
  transform: translate(-50%, -10px);
}
</style>
