import { listen } from '@tauri-apps/api/event';
import { appWindow } from '@tauri-apps/api/window';

/**
 * WebView2 under Tauri 1 does not reliably flip `document.hidden` when the
 * host window is hidden to tray or minimized. Everything that pauses on
 * `visibilitychange` (WebGL scenes, status polling, ping, charts) kept running
 * inside an invisible window for hours, so Windows trimmed the process and
 * reopening the window stalled while it paged back in.
 *
 * We fold the real host-window state into `document.hidden` /
 * `document.visibilityState` and fire `visibilitychange` ourselves, so every
 * existing listener keeps working without changes.
 */

let hostHidden = false;
let installed = false;
let lastReported: boolean | null = null;

const nativeHidden = Object.getOwnPropertyDescriptor(Document.prototype, 'hidden');
const nativeState = Object.getOwnPropertyDescriptor(Document.prototype, 'visibilityState');

function readNativeHidden(): boolean {
  try {
    return nativeHidden?.get ? Boolean(nativeHidden.get.call(document)) : false;
  } catch {
    return false;
  }
}

function readNativeState(): DocumentVisibilityState {
  try {
    return nativeState?.get
      ? (nativeState.get.call(document) as DocumentVisibilityState)
      : 'visible';
  } catch {
    return 'visible';
  }
}

function announce() {
  const now = document.hidden;
  if (now === lastReported) return;
  lastReported = now;
  document.dispatchEvent(new Event('visibilitychange'));
}

export function setHostHidden(hidden: boolean) {
  if (hostHidden === hidden) return;
  hostHidden = hidden;
  if (installed) announce();
}

export function isHostHidden(): boolean {
  return hostHidden;
}

type Callable = (...args: unknown[]) => Promise<unknown>;

export function installHostVisibility() {
  if (installed) return;
  try {
    Object.defineProperty(document, 'hidden', {
      configurable: true,
      get: () => hostHidden || readNativeHidden(),
    });
    Object.defineProperty(document, 'visibilityState', {
      configurable: true,
      get: () => (hostHidden ? 'hidden' : readNativeState()),
    });
  } catch {
    return;
  }
  installed = true;
  lastReported = document.hidden;

  // Native visibilitychange events keep our bookkeeping in sync.
  document.addEventListener(
    'visibilitychange',
    () => {
      lastReported = document.hidden;
    },
    true,
  );

  // Rust side emits this whenever it shows/hides the main window.
  listen<boolean>('wawity-main-visibility', (e) => setHostHidden(e.payload === false)).catch(
    () => {},
  );
  // A focused window is by definition on screen (restore from taskbar, hotkeys).
  listen('tauri://focus', () => setHostHidden(false)).catch(() => {});

  // Hides/minimizes triggered from JS (close button, panic hotkey, minimize).
  const target = appWindow as unknown as Record<string, Callable | undefined>;
  const plan: Array<[string, boolean]> = [
    ['hide', true],
    ['minimize', true],
    ['show', false],
    ['unminimize', false],
  ];
  for (const [name, hidden] of plan) {
    const original = target[name];
    if (typeof original !== 'function') continue;
    const bound = original.bind(appWindow);
    target[name] = async (...args: unknown[]) => {
      const result = await bound(...args);
      setHostHidden(hidden);
      return result;
    };
  }

  announce();
}
