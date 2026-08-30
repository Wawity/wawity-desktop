import { invoke } from '@tauri-apps/api/tauri';
import type { App } from 'vue';

let allowed = false;
const seen = new Set<string>();

export function setTelemetryAllowed(value: boolean) {
  allowed = value;
}

export function track(name: string, props?: Record<string, string | number | boolean>) {
  if (!allowed) return;
  invoke('track_event', { name, props: props ?? null }).catch(() => {});
}

function reportError(message: string, stack?: string) {
  if (!allowed) return;
  const key = message.slice(0, 200);
  if (seen.has(key) || seen.size > 20) return;
  seen.add(key);
  invoke('report_error', {
    message: message.slice(0, 1000),
    stack: stack ? stack.slice(0, 4000) : null,
  }).catch(() => {});
}

export function initTelemetry() {
  window.addEventListener('error', (e) => {
    const err = e.error as Error | undefined;
    reportError(
      String(e.message || 'window error'),
      err && err.stack ? String(err.stack) : undefined,
    );
  });
  window.addEventListener('unhandledrejection', (e) => {
    const reason = e.reason as { message?: string; stack?: string } | undefined;
    reportError(
      'unhandledrejection: ' + String(reason && reason.message ? reason.message : reason),
      reason && reason.stack ? String(reason.stack) : undefined,
    );
  });
}

export function watchVueErrors(app: App) {
  app.config.errorHandler = (err, _instance, info) => {
    const e = err as Error;
    reportError(
      'vue: ' + String(e && e.message ? e.message : err) + ' [' + info + ']',
      e && e.stack ? e.stack : undefined,
    );
    console.error(err);
  };
}
