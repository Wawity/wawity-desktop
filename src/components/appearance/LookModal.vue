<template>
  <Teleport to="body">
    <Transition name="lk-veil">
      <div v-if="open" class="lk-overlay" :style="{ zIndex: layer }" @pointerdown="onPointerDown" @click="onBackdropClick">
        <section
          ref="panel"
          :class="['lk-modal', `lk-modal--${size}`]"
          role="dialog"
          aria-modal="true"
          :aria-labelledby="titleId"
          :aria-describedby="subtitle ? descId : undefined"
          tabindex="-1"
        >
          <header class="lk-head">
            <div class="lk-head-text">
              <span v-if="icon" class="lk-caption"><component :is="icon" :size="14" aria-hidden="true" /> {{ tr('НАСТРОЙКИ', 'SETTINGS') }}</span>
              <h2 :id="titleId" class="lk-title">{{ title }}</h2>
              <p v-if="subtitle" :id="descId" class="lk-sub">{{ subtitle }}</p>
            </div>
            <button type="button" class="lk-close" :aria-label="tr('Закрыть', 'Close')" @click="emit('close')"><X :size="16" aria-hidden="true" /></button>
          </header>
          <div v-if="$slots.tabs" class="lk-tabs-wrap"><slot name="tabs" /></div>
          <div class="lk-body"><slot /></div>
          <footer v-if="$slots.footer" class="lk-foot"><slot name="footer" /></footer>
        </section>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { nextTick, onBeforeUnmount, ref, watch, type Component } from 'vue';
import { X } from '../../lib/appIcons';
import { tr } from '../../appearance/i18n';
import { claimModalId, isTopModal, popModal, pushModal } from '../../appearance/modalStack';

const props = withDefaults(defineProps<{ open: boolean; title: string; subtitle?: string; icon?: Component; size?: 'sm' | 'md' | 'lg' }>(), { size: 'md', subtitle: '' });
const emit = defineEmits<{ (e: 'close'): void }>();
const id = claimModalId();
const titleId = `look-title-${id}`;
const descId = `look-desc-${id}`;
const panel = ref<HTMLElement | null>(null);
const layer = ref(1200);
let active = false;
let version = 0;
let backdropDown = false;
let returnFocus: HTMLElement | null = null;

function focusables() {
  if (!panel.value) return [];
  return Array.from(panel.value.querySelectorAll<HTMLElement>('button:not(:disabled), a[href], input:not(:disabled), select:not(:disabled), textarea:not(:disabled), [tabindex]:not([tabindex="-1"])')).filter((el) => el.tabIndex >= 0 && el.getClientRects().length > 0 && getComputedStyle(el).visibility !== 'hidden');
}
function onKey(e: KeyboardEvent) {
  if (!active || !isTopModal(id)) return;
  if (e.key === 'Escape') {
    e.preventDefault();
    e.stopImmediatePropagation();
    emit('close');
    return;
  }
  if (e.key !== 'Tab') return;
  const items = focusables();
  if (!items.length) { e.preventDefault(); panel.value?.focus(); return; }
  const index = items.indexOf(document.activeElement as HTMLElement);
  if (index === -1 || (!e.shiftKey && index === items.length - 1) || (e.shiftKey && index === 0)) {
    e.preventDefault();
    (e.shiftKey ? items[items.length - 1] : items[0]).focus();
  }
}
function onFocus(e: FocusEvent) {
  if (!active || !isTopModal(id) || !panel.value || panel.value.contains(e.target as Node)) return;
  (focusables()[0] ?? panel.value).focus({ preventScroll: true });
}
function onPointerDown(e: PointerEvent) { backdropDown = e.target === e.currentTarget && e.button === 0; }
function onBackdropClick(e: MouseEvent) {
  const close = backdropDown && e.target === e.currentTarget && isTopModal(id);
  backdropDown = false;
  if (close) emit('close');
}
async function activate() {
  if (active) return;
  active = true;
  const stamp = ++version;
  returnFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null;
  layer.value = pushModal(id);
  window.addEventListener('keydown', onKey, true);
  window.addEventListener('focusin', onFocus, true);
  await nextTick();
  if (!active || version !== stamp || !isTopModal(id)) return;
  panel.value?.focus({ preventScroll: true });
}
function deactivate() {
  if (!active) return;
  const wasTop = isTopModal(id);
  active = false;
  ++version;
  backdropDown = false;
  window.removeEventListener('keydown', onKey, true);
  window.removeEventListener('focusin', onFocus, true);
  popModal(id);
  const previous = returnFocus;
  returnFocus = null;
  if (wasTop && previous?.isConnected) previous.focus({ preventScroll: true });
}
watch(() => props.open, (open) => { if (open) void activate(); else deactivate(); }, { immediate: true, flush: 'post' });
onBeforeUnmount(deactivate);
</script>

<style>
html.wawity-look-open .main-content { overflow: hidden !important; }
.lk-overlay { position: fixed; inset: 0; display: flex; align-items: center; justify-content: center; padding: 52px 20px 20px; background: rgba(8, 7, 7, 0.78); backdrop-filter: blur(12px); -webkit-backdrop-filter: blur(12px); overscroll-behavior: contain; }
.lk-modal { position: relative; display: flex; flex-direction: column; width: 100%; max-height: calc(100dvh - 72px); border: 1px solid var(--border); border-top: 2px solid var(--foreground); border-radius: 0; background: var(--popover); color: var(--foreground); box-shadow: 0 30px 80px rgba(0, 0, 0, 0.6); outline: none; overflow: hidden; font-family: var(--font-sans); }
.lk-modal--sm { max-width: 520px; }
.lk-modal--md { max-width: 720px; }
.lk-modal--lg { max-width: 860px; }
.lk-modal .lk-head { display: flex; align-items: flex-start; justify-content: space-between; gap: 20px; padding: 26px 28px 20px; }
.lk-modal .lk-head-text { min-width: 0; }
.lk-modal .lk-caption { display: inline-flex; align-items: center; gap: 8px; margin-bottom: 10px; font-family: var(--font-label); font-size: 9px; letter-spacing: 0.14em; color: var(--muted-foreground); }
.lk-modal .lk-title { margin: 0; font-family: var(--font-display); font-size: 24px; font-weight: 400; letter-spacing: -0.03em; line-height: 1.2; }
.lk-modal .lk-sub { margin: 8px 0 0; max-width: 58ch; font-size: 12px; line-height: 1.5; color: var(--muted-foreground); }
.lk-modal .lk-close { display: grid; place-items: center; width: 32px; height: 32px; padding: 0; flex-shrink: 0; border: 1px solid var(--border); border-radius: 50%; background: transparent; color: var(--muted-foreground); cursor: pointer; }
.lk-modal .lk-close:hover { border-color: var(--foreground); color: var(--foreground); }
.lk-modal .lk-tabs-wrap { padding: 0 28px; }
.lk-modal .lk-tabs { display: flex; gap: 24px; border-bottom: 1px solid var(--border); }
.lk-modal .lk-tab { position: relative; display: inline-flex; align-items: center; gap: 8px; min-height: 42px; padding: 0 0 12px; border: 0; background: transparent; color: var(--muted-foreground); font: inherit; font-size: 12.5px; cursor: pointer; }
.lk-modal .lk-tab:hover, .lk-modal .lk-tab--on { color: var(--foreground); }
.lk-modal .lk-tab--on::after { content: ''; position: absolute; bottom: -1px; left: 0; right: 0; height: 2px; background: var(--foreground); }
.lk-modal .lk-body { flex: 1; min-height: 0; overflow-y: auto; scrollbar-gutter: stable; overscroll-behavior: contain; padding: 0 28px 22px; scrollbar-width: thin; scrollbar-color: var(--border) transparent; }
.lk-modal .lk-foot { display: flex; align-items: center; justify-content: flex-end; flex-wrap: wrap; gap: 8px; padding: 16px 28px; border-top: 1px solid var(--border); }
.lk-modal .lk-pills { display: flex; flex-wrap: wrap; gap: 6px; }
.lk-modal .lk-pill { display: inline-flex; align-items: center; justify-content: center; gap: 6px; min-height: 30px; padding: 5px 12px; border: 1px solid var(--border); border-radius: var(--r-pill, 9999px); background: transparent; color: var(--muted-foreground); font: inherit; font-size: 11.5px; cursor: pointer; transition: background 180ms, color 180ms, border-color 180ms; }
.lk-modal .lk-pill:hover:not(:disabled) { color: var(--foreground); border-color: color-mix(in oklch, var(--foreground) 24%, transparent); background: color-mix(in oklch, var(--foreground) 6%, transparent); }
.lk-modal .lk-pill--on { background: var(--foreground); border-color: var(--foreground); color: var(--background); }
.lk-modal .lk-pill--on:hover:not(:disabled) { background: color-mix(in oklch, var(--foreground) 88%, var(--background)); color: var(--background); }
.lk-modal .lk-pill:disabled { opacity: 0.4; cursor: not-allowed; }
.lk-modal .lk-pill--danger:hover:not(:disabled) { color: var(--destructive); border-color: var(--destructive); }
.lk-modal .lk-input { flex: 1; min-width: 0; height: 32px; padding: 0 11px; border: 1px solid var(--input); border-radius: 9px; background: transparent; color: var(--foreground); font-family: var(--font-mono); font-size: 11.5px; outline: none; }
.lk-modal .lk-input:focus { border-color: var(--ember); }
.lk-modal .lk-input--bad { border-color: var(--destructive); }
.lk-modal .lk-url { display: flex; flex-wrap: wrap; gap: 6px; width: 100%; }
.lk-modal .lk-note { margin: 14px 0 0; padding: 10px 0; border-bottom: 1px solid var(--border); color: var(--muted-foreground); font-size: 12px; line-height: 1.5; }
.lk-modal .lk-swatches { display: flex; flex-wrap: wrap; gap: 10px; }
.lk-modal .lk-swatch { width: 26px; height: 26px; padding: 0; border: 1px solid color-mix(in oklch, var(--foreground) 20%, transparent); border-radius: 50%; background: var(--c); cursor: pointer; }
.lk-modal .lk-swatch--on { outline: 1px solid var(--foreground); outline-offset: 3px; }
.lk-modal button:focus-visible { outline: 2px solid var(--foreground); outline-offset: 3px; }
.lk-veil-enter-active, .lk-veil-leave-active { transition: opacity 180ms ease; }
.lk-veil-enter-active .lk-modal, .lk-veil-leave-active .lk-modal { transition: transform 220ms cubic-bezier(0.22, 1, 0.36, 1); }
.lk-veil-enter-from, .lk-veil-leave-to { opacity: 0; }
.lk-veil-enter-from .lk-modal, .lk-veil-leave-to .lk-modal { transform: translateY(8px); }
html.motion-simple .lk-veil-enter-active, html.motion-simple .lk-veil-leave-active, html.motion-simple .lk-veil-enter-active .lk-modal, html.motion-simple .lk-veil-leave-active .lk-modal { transition: none; }
@media (prefers-reduced-motion: reduce) { .lk-veil-enter-active, .lk-veil-leave-active, .lk-veil-enter-active .lk-modal, .lk-veil-leave-active .lk-modal { transition: none; } }
@media (max-width: 560px) {
  .lk-overlay { padding: 48px 12px 12px; }
  .lk-modal { max-height: calc(100dvh - 60px); }
  .lk-modal .lk-head { padding: 20px 18px 16px; }
  .lk-modal .lk-tabs-wrap { padding: 0 18px; }
  .lk-modal .lk-tabs { gap: 18px; }
  .lk-modal .lk-body { padding: 0 18px 18px; }
  .lk-modal .lk-foot { padding: 14px 18px; }
  .lk-modal .lk-url .lk-input { flex-basis: 100%; }
}
</style>
