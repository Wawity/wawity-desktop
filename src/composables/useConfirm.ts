import { reactive } from 'vue';

interface ConfirmOptions {
  title: string;
  description?: string;
  confirmLabel?: string;
  cancelLabel?: string;
  danger?: boolean;
}

interface ConfirmState extends Required<Omit<ConfirmOptions, 'description'>> {
  description: string;
  open: boolean;
  resolveFn: ((value: boolean) => void) | null;
}

const state = reactive<ConfirmState>({
  open: false,
  title: '',
  description: '',
  confirmLabel: 'Confirm',
  cancelLabel: 'Cancel',
  danger: false,
  resolveFn: null,
});

export function useConfirmState() {
  return state;
}

export function askConfirm(options: ConfirmOptions): Promise<boolean> {
  return new Promise((resolve) => {
    state.title = options.title;
    state.description = options.description ?? '';
    state.confirmLabel = options.confirmLabel ?? 'Confirm';
    state.cancelLabel = options.cancelLabel ?? 'Cancel';
    state.danger = options.danger ?? false;
    state.resolveFn = resolve;
    state.open = true;
  });
}

export function settleConfirm(result: boolean) {
  if (state.resolveFn) {
    state.resolveFn(result);
    state.resolveFn = null;
  }
  state.open = false;
}