<template>
  <Teleport to="body">
    <div class="toast-root" aria-live="polite" aria-label="Notifications">
      <TransitionGroup name="toast" tag="div" class="toast-stack">
        <div
          v-for="toast in toasts"
          :key="toast.id"
          class="toast"
          :class="`toast--${toast.variant}`"
          role="alert"
        >
          <div class="toast-icon-wrap">
            <CheckCircle2 v-if="toast.variant === 'success'" :size="16" />
            <XCircle v-else-if="toast.variant === 'error'" :size="16" />
            <AlertTriangle v-else-if="toast.variant === 'warning'" :size="16" />
            <Info v-else :size="16" />
          </div>
          <div class="toast-body">
            <p class="toast-title">{{ toast.title }}</p>
            <p v-if="toast.message" class="toast-message">{{ toast.message }}</p>
          </div>
          <button class="toast-close" type="button" @click="removeToast(toast.id)" aria-label="Dismiss">
            <X :size="12" />
          </button>
          <div
            class="toast-progress"
            :style="{ animationDuration: `${toast.duration}ms` }"
          />
        </div>
      </TransitionGroup>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { CheckCircle2, XCircle, AlertTriangle, Info, X } from '../lib/appIcons';
import { useNotifications } from '../composables/useNotifications';

const { toasts, removeToast } = useNotifications();
</script>

<style scoped>
.toast-root {
  position: fixed;
  bottom: 24px;
  right: 20px;
  z-index: 9999;
  pointer-events: none;
  display: flex;
  flex-direction: column;
  align-items: flex-end;
}

.toast-stack {
  display: flex;
  flex-direction: column;
  gap: 8px;
  align-items: flex-end;
}

.toast {
  display: flex;
  align-items: flex-start;
  gap: 11px;
  padding: 13px 14px;
  border-radius: 12px;
  border: 1px solid var(--border);
  background: var(--card);
  min-width: 260px;
  max-width: 360px;
  pointer-events: all;
  position: relative;
  overflow: hidden;
  box-shadow: 0 16px 40px rgba(0, 0, 0, 0.5);
}

.toast--success {
  border-color: color-mix(in oklch, var(--success) 30%, transparent);
}

/* Flat tint only. The background used to be 95% card mixed toward the accent,
   which tinted the whole card and fought the text on top of it. */
.toast--error {
  border-color: color-mix(in oklch, var(--destructive) 30%, transparent);
}

.toast--warning {
  border-color: color-mix(in oklch, oklch(0.82 0.18 70) 30%, transparent);
}

/* Neutral on purpose: this was an off-palette blue, the last colour in the
   app that did not come from the site palette or a semantic token. */
.toast--info {
  border-color: var(--border);
}

.toast-icon-wrap {
  flex-shrink: 0;
  margin-top: 1px;
}

.toast--success .toast-icon-wrap { color: var(--success); }
.toast--error .toast-icon-wrap { color: var(--destructive); }
.toast--warning .toast-icon-wrap { color: oklch(0.82 0.18 70); }
.toast--info .toast-icon-wrap { color: var(--muted-foreground); }

.toast-body { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 2px; }
.toast-title {
  font-family: var(--font-display);
  font-size: 14px;
  font-weight: 400;
  letter-spacing: -0.015em;
  color: var(--foreground);
}
.toast-message {
  font-size: 11.5px;
  color: var(--muted-foreground);
  line-height: 1.4;
  word-break: break-word;
}

.toast-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  border-radius: 6px;
  border: none;
  background: transparent;
  color: var(--muted-foreground);
  cursor: pointer;
  flex-shrink: 0;
  transition: color 120ms, background 120ms;
  margin-top: -1px;
}

.toast-close:hover { color: var(--foreground); background: var(--secondary); }

.toast-progress {
  position: absolute;
  bottom: 0;
  left: 0;
  height: 2px;
  width: 100%;
  transform-origin: left;
  animation: shrink linear forwards;
}

.toast--success .toast-progress { background: var(--success); }
.toast--error .toast-progress { background: var(--destructive); }
.toast--warning .toast-progress { background: oklch(0.82 0.18 70); }
.toast--info .toast-progress { background: var(--muted-foreground); }

@keyframes shrink {
  from { transform: scaleX(1); }
  to { transform: scaleX(0); }
}

/* No overshoot: the old spring pushed the card past its final width, which
   read as a bounce on a notification that is already secondary to the
   action that triggered it. */
.toast-enter-active {
  transition:
    opacity 260ms ease,
    transform 260ms cubic-bezier(0.22, 1, 0.36, 1);
}

.toast-leave-active {
  transition:
    opacity 180ms ease,
    transform 180ms ease;
}

.toast-enter-from {
  opacity: 0;
  transform: translateX(16px);
}

.toast-leave-to {
  opacity: 0;
  transform: translateX(8px);
}

.toast-move {
  transition: transform 250ms ease;
}
</style>