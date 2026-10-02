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
  background: rgba(8, 7, 7, 0.78);
  backdrop-filter: blur(12px);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
}

/* Hairline framed sheet rather than a lifted glass card. */
.confirm-card {
  position: relative;
  width: 100%;
  max-width: 380px;
  padding: 30px 28px 24px;
  border-radius: 0;
  border: 1px solid var(--border);
  background: var(--popover);
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 14px;
  text-align: left;
  box-shadow: 0 30px 80px rgba(0, 0, 0, 0.6);
}
/* The accent rule at the top edge is the only emphasis a modal needs. */
.confirm-card::before {
  content: '';
  position: absolute;
  top: -1px;
  left: -1px;
  right: -1px;
  height: 2px;
  background: var(--foreground);
}
.confirm-card--danger::before {
  background: var(--destructive);
}

.confirm-card--danger {
  border-color: color-mix(in oklch, var(--destructive) 30%, transparent);
}

/* Outlined marker, the same shape as the checks in the rules list. */
.confirm-badge {
  width: 38px;
  height: 38px;
  border-radius: 50%;
  border: 1px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  color: var(--foreground);
}

.confirm-badge--danger {
  color: var(--destructive);
  border-color: color-mix(in oklch, var(--destructive) 40%, transparent);
}

.confirm-heading {
  font-family: var(--font-display);
  font-size: 22px;
  font-weight: 400;
  letter-spacing: -0.03em;
  line-height: 1.2;
}

.confirm-text {
  font-size: 13px;
  color: var(--muted-foreground);
  line-height: 1.6;
  max-width: 46ch;
}

.confirm-buttons {
  display: flex;
  gap: 8px;
  width: 100%;
  margin-top: 6px;
  padding-top: 16px;
  border-top: 1px solid var(--border);
}

.confirm-btn-cancel {
  flex: 1;
  padding: 11px 20px;
  border-radius: var(--r-pill, 9999px);
  border: 1px solid var(--border);
  background: transparent;
  color: var(--muted-foreground);
  font-family: var(--f-text);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: background 200ms, color 200ms, border-color 200ms;
}

.confirm-btn-cancel:hover {
  background: color-mix(in oklch, var(--foreground) 6%, transparent);
  color: var(--foreground);
  border-color: color-mix(in oklch, var(--foreground) 24%, transparent);
}

.confirm-btn-ok {
  flex: 1;
  padding: 11px 20px;
  border-radius: var(--r-pill, 9999px);
  border: 1px solid var(--foreground);
  background: var(--foreground);
  color: var(--background);
  font-family: var(--f-text);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: background 200ms, transform 120ms;
}

.confirm-btn-ok:hover {
  background: color-mix(in oklch, var(--foreground) 88%, var(--background));
}
.confirm-btn-ok:active { transform: scale(0.97); }

.confirm-btn-ok--danger {
  background: var(--destructive);
  border-color: var(--destructive);
  color: #fff;
}

.confirm-veil-enter-active, .confirm-veil-leave-active { transition: opacity 180ms ease; }
.confirm-veil-enter-from, .confirm-veil-leave-to { opacity: 0; }

.confirm-pop-enter-active { transition: all 320ms cubic-bezier(0.34, 1.56, 0.64, 1); }
.confirm-pop-leave-active { transition: all 150ms ease; }
.confirm-pop-enter-from { opacity: 0; transform: scale(0.85) translateY(12px); }
.confirm-pop-leave-to { opacity: 0; transform: scale(0.92) translateY(6px); }
</style>