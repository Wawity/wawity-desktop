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
    gsap.fromTo(el, { y: -8, opacity: 0 }, {
      y: 0,
      opacity: 1,
      duration: 0.28,
      ease: 'power2.out',
    });
  },
);
</script>

<style scoped>
/* A quiet confirmation, not an alert. This was a violet gradient pill with an
   inset highlight on top of a blur — the last of the old palette on this
   surface. Flat onyx and a hairline now, with the ember tick carrying the
   only colour. */
.copy-hint {
  position: fixed;
  top: 18px;
  left: 50%;
  z-index: 1200;
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 8px 14px;
  border-radius: 999px;
  border: 1px solid var(--border);
  background: var(--background);
  color: var(--paper);
  font-family: var(--font-sans);
  font-size: 12px;
  font-weight: 500;
  letter-spacing: -0.01em;
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.5);
  pointer-events: none;
  transform: translate(-50%, 0);
}

.copy-hint svg {
  color: var(--ember);
  flex-shrink: 0;
}

.hint-drop-leave-active {
  transition:
    opacity 220ms ease,
    transform 220ms ease;
}

.hint-drop-leave-to {
  opacity: 0;
  transform: translate(-50%, -6px);
}
</style>
