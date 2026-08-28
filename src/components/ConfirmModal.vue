<template>
  <Teleport to="body">
    <Transition name="confirm-veil">
      <div v-if="state.open" class="confirm-overlay" @click.self="cancel">
        <Transition name="confirm-pop" appear>
          <div v-if="state.open" class="confirm-card" :class="{ 'confirm-card--danger': state.danger }" role="alertdialog" aria-modal="true">
            <div class="confirm-badge" :class="{ 'confirm-badge--danger': state.danger }">
              <ShieldAlert v-if="state.danger" :size="22" />
              <ShieldCheck v-else :size="22" />
            </div>
            <h3 class="confirm-heading">{{ state.title }}</h3>
            <p v-if="state.description" class="confirm-text">{{ state.description }}</p>
            <div class="confirm-buttons">
              <button type="button" class="confirm-btn-cancel" @click="cancel">{{ state.cancelLabel }}</button>
              <button
                type="button"
                class="confirm-btn-ok"
                :class="{ 'confirm-btn-ok--danger': state.danger }"
                @click="confirm"
              >
                {{ state.confirmLabel }}
              </button>
            </div>
          </div>
        </Transition>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ShieldAlert, ShieldCheck } from '../lib/appIcons';
import { useConfirmState, settleConfirm } from '../composables/useConfirm';

const state = useConfirmState();

function cancel() { settleConfirm(false); }
function confirm() { settleConfirm(true); }
</script>

<style scoped>
.confirm-overlay {
  position: fixed;
  inset: 0;
  z-index: 200;
  background: rgba(0, 0, 0, 0.68);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
}

.confirm-card {
  width: 100%;
  max-width: 360px;
  padding: 32px 26px 24px;
  border-radius: 24px;
  border: 1px solid var(--border);
  background: var(--card);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  text-align: center;
  box-shadow: 0 24px 64px rgba(0, 0, 0, 0.5);
}

.confirm-card--danger {
  border-color: color-mix(in oklch, var(--destructive) 30%, transparent);
}

.confirm-badge {
  width: 60px;
  height: 60px;
  border-radius: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: color-mix(in oklch, var(--success) 12%, transparent);
  color: var(--success);
}

.confirm-badge--danger {
  background: color-mix(in oklch, var(--destructive) 14%, transparent);
  color: var(--destructive);
}

.confirm-heading {
  font-size: 17.5px;
  font-weight: 700;
  letter-spacing: -0.01em;
}

.confirm-text {
  font-size: 13px;
  color: var(--muted-foreground);
  line-height: 1.6;
}

.confirm-buttons {
  display: flex;
  gap: 8px;
  width: 100%;
  margin-top: 10px;
}

.confirm-btn-cancel {
  flex: 1;
  padding: 12px;
  border-radius: 14px;
  border: 1px solid var(--border);
  background: var(--secondary);
  color: var(--foreground);
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: background 150ms;
}

.confirm-btn-cancel:hover { background: var(--muted); }

.confirm-btn-ok {
  flex: 1;
  padding: 12px;
  border-radius: 14px;
  border: none;
  background: var(--primary);
  color: var(--primary-foreground);
  font-size: 13px;
  font-weight: 700;
  cursor: pointer;
  transition: opacity 150ms, transform 100ms;
}

.confirm-btn-ok:hover { opacity: 0.85; }
.confirm-btn-ok:active { transform: scale(0.97); }

.confirm-btn-ok--danger {
  background: var(--destructive);
  color: #fff;
}

.confirm-veil-enter-active, .confirm-veil-leave-active { transition: opacity 180ms ease; }
.confirm-veil-enter-from, .confirm-veil-leave-to { opacity: 0; }

.confirm-pop-enter-active { transition: all 320ms cubic-bezier(0.34, 1.56, 0.64, 1); }
.confirm-pop-leave-active { transition: all 150ms ease; }
.confirm-pop-enter-from { opacity: 0; transform: scale(0.85) translateY(12px); }
.confirm-pop-leave-to { opacity: 0; transform: scale(0.92) translateY(6px); }
</style>