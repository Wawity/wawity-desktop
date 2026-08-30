import { reactive } from 'vue';
import en from './en';
import ru from './ru';

const dictionaries = { en, ru } as const;
type LangCode = keyof typeof dictionaries;

const state = reactive<{ lang: LangCode }>({ lang: 'en' });

function resolvePath(obj: unknown, path: string): unknown {
  return path.split('.').reduce<unknown>((acc, key) => {
    if (acc && typeof acc === 'object' && key in (acc as Record<string, unknown>)) {
      return (acc as Record<string, unknown>)[key];
    }
    return undefined;
  }, obj);
}

function interpolate(template: string, params?: Record<string, string | number>): string {
  if (!params) return template;
  return template.replace(/\{(\w+)\}/g, (match, key) => {
    const value = params[key];
    return value !== undefined ? String(value) : match;
  });
}

export function setLanguage(lang: string): void {
  if (lang === 'ru' || lang === 'en') {
    state.lang = lang;
  }
}

export function getLanguage(): LangCode {
  return state.lang;
}

export function t(key: string, params?: Record<string, string | number>): string {
  const primary = resolvePath(dictionaries[state.lang], key);
  if (typeof primary === 'string') return interpolate(primary, params);

  const fallback = resolvePath(dictionaries.en, key);
  if (typeof fallback === 'string') return interpolate(fallback, params);

  return key;
}

function loadPersistedLanguage(): void {
  try {
    const raw = localStorage.getItem('wawity_settings');
    if (!raw) return;
    const parsed = JSON.parse(raw) as { language?: string };
    if (parsed.language === 'ru' || parsed.language === 'en') {
      state.lang = parsed.language;
    }
  } catch {
    
  }
}

loadPersistedLanguage();