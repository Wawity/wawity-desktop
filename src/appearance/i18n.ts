import { useVpnStore } from '../stores/vpn';

/** Inline RU/EN pick for the visual page, keyed on the app language. */
export function tr(ru: string, en: string): string {
  try {
    const lang = String(useVpnStore().settings.language || 'en');
    return lang.toLowerCase().startsWith('ru') ? ru : en;
  } catch {
    return en;
  }
}
