import { reactive } from 'vue';

interface CopyHintState {
  visible: boolean;
  text: string;
}

const state = reactive<CopyHintState>({ visible: false, text: '' });
let hideTimer = 0;

export function showCopyHint(text: string) {
  if (hideTimer) window.clearTimeout(hideTimer);
  state.text = text;
  state.visible = true;
  hideTimer = window.setTimeout(() => {
    state.visible = false;
  }, 1800);
}

export function useCopyHint() {
  return state;
}
