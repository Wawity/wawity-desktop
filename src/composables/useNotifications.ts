import { watch } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import { useVpnStore } from '../stores/vpn';
import { t } from '../i18n';

export type ToastVariant = 'success' | 'error' | 'warning' | 'info';

export interface ToastItem {
  id: number;
  variant: ToastVariant;
  title: string;
  message?: string;
  duration: number;
}

async function sendNative(title: string, body?: string, variant: ToastVariant = 'info') {
  try {
    await invoke('show_notification', { title, body: body ?? null, variant });
  } catch {}
}

let _initialized = false;

export function useNotifications() {
  return {};
}

export async function initNotificationWatcher() {
  if (_initialized) return;
  _initialized = true;

  const vpnStore = useVpnStore();

  let prevConnected: boolean | null = null;
  let prevError: string | null = null;
  let prevKillSwitch: boolean | null = null;

  watch(
    () => vpnStore.status.connected,
    (connected) => {
      if (prevConnected === null) {
        prevConnected = connected;
        return;
      }
      if (connected && !prevConnected) {
        const serverName = vpnStore.status.server_name ?? t('toast.unknownServer');
        if (vpnStore.settings.notifications) {
          sendNative(t('toast.vpnConnected'), serverName, 'success');
        }
      }
      if (!connected && prevConnected) {
        if (vpnStore.settings.notifications) {
          sendNative(t('toast.vpnDisconnected'), t('toast.trafficUnencrypted'), 'info');
        }
      }
      prevConnected = connected;
    }
  );

  watch(
    () => vpnStore.connectError,
    (err) => {
      if (!err || err === prevError) return;
      prevError = err;
      if (vpnStore.settings.notifications) {
        sendNative(t('toast.connectionFailed'), err, 'error');
      }
    }
  );

  watch(
    () => vpnStore.status.kill_switch,
    (active) => {
      if (prevKillSwitch === null) {
        prevKillSwitch = active;
        return;
      }
      if (active && !prevKillSwitch) {
        if (vpnStore.settings.notifications) {
          sendNative(t('toast.killSwitchActive'), t('toast.killSwitchActiveDesc'), 'warning');
        }
      }
      prevKillSwitch = active;
    }
  );

  watch(
    () => vpnStore.autoSelectLoading,
    (loading) => {
      if (!loading && vpnStore.selectedServer && vpnStore.settings.notifications) {
        const ms = vpnStore.selectedServer.latencyMs;
        const latencyStr = ms !== null && ms !== undefined ? ` (${ms}ms)` : '';
        sendNative(t('toast.autoSelected'), `${vpnStore.selectedServer.name}${latencyStr}`, 'info');
      }
    }
  );
}
