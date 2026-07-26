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
import { CheckCircle2, XCircle, AlertTriangle, Info, X } from 'lucide-vue-next';
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
  border-radius: 14px;
  border: 1px solid var(--border);
  background: var(--card);
  min-width: 280px;
  max-width: 360px;
  pointer-events: all;
  position: relative;
  overflow: hidden;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  backdrop-filter: blur(12px);
}

.toast--success {
  border-color: color-mix(in oklch, var(--success) 30%, transparent);
  background: color-mix(in oklch, var(--card) 95%, var(--success));
}

.toast--error {
  border-color: color-mix(in oklch, var(--destructive) 30%, transparent);
  background: color-mix(in oklch, var(--card) 95%, var(--destructive));
}

.toast--warning {
  border-color: color-mix(in oklch, oklch(0.82 0.18 70) 30%, transparent);
  background: color-mix(in oklch, var(--card) 95%, oklch(0.82 0.18 70));
}

.toast--info {
  border-color: color-mix(in oklch, oklch(0.72 0.15 240) 30%, transparent);
  background: color-mix(in oklch, var(--card) 95%, oklch(0.72 0.15 240));
}

.toast-icon-wrap {
  flex-shrink: 0;
  margin-top: 1px;
}

.toast--success .toast-icon-wrap { color: var(--success); }
.toast--error .toast-icon-wrap { color: var(--destructive); }
.toast--warning .toast-icon-wrap { color: oklch(0.82 0.18 70); }
.toast--info .toast-icon-wrap { color: oklch(0.72 0.15 240); }

.toast-body { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 2px; }
.toast-title { font-size: 13px; font-weight: 600; color: var(--foreground); }
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
.toast--info .toast-progress { background: oklch(0.72 0.15 240); }

@keyframes shrink {
  from { transform: scaleX(1); }
  to { transform: scaleX(0); }
}

.toast-enter-active {
  transition: all 300ms cubic-bezier(0.34, 1.56, 0.64, 1);
}

.toast-leave-active {
  transition: all 200ms ease;
}

.toast-enter-from {
  opacity: 0;
  transform: translateX(100%) scale(0.9);
}

.toast-leave-to {
  opacity: 0;
  transform: translateX(60%) scale(0.95);
}

.toast-move {
  transition: transform 250ms ease;
}
</style>