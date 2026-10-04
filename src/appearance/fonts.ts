export interface FontPreset {
  id: string;
  ru: string;
  en: string;
  vibe: { ru: string; en: string };
  sans: string;
  display: string;
  label: string;
  mono: string;
  google?: string;
}

const SYS = "system-ui, -apple-system, 'Segoe UI', sans-serif";
const MONO = "Consolas, 'Cascadia Mono', 'Courier New', monospace";

export const FONT_PRESETS: FontPreset[] = [
  {
    id: 'wawity', ru: 'Wawity', en: 'Wawity',
    vibe: { ru: 'Родной стиль', en: 'Stock look' },
    sans: `'Inter', ${SYS}`,
    display: `'Inter Tight', 'Inter', ${SYS}`,
    label: `'Unbounded', 'Inter Tight', ${SYS}`,
    mono: `'JetBrains Mono', 'Fira Code', ${MONO}`,
  },
  {
    id: 'windows', ru: 'Windows 11', en: 'Windows 11',
    vibe: { ru: 'Нативный, без загрузки', en: 'Native, offline' },
    sans: `'Segoe UI Variable Text', 'Segoe UI', ${SYS}`,
    display: `'Segoe UI Variable Display', 'Segoe UI', ${SYS}`,
    label: `'Bahnschrift', 'Segoe UI', ${SYS}`,
    mono: `'Cascadia Mono', 'Cascadia Code', ${MONO}`,
  },
  {
    id: 'soft', ru: 'Мягкий', en: 'Soft',
    vibe: { ru: 'Округлый и добрый', en: 'Round & friendly' },
    sans: `'Nunito', ${SYS}`,
    display: `'Comfortaa', 'Nunito', ${SYS}`,
    label: `'Comfortaa', ${SYS}`,
    mono: `'Ubuntu Mono', ${MONO}`,
    google: 'family=Nunito:wght@400;500;600;700;800&family=Comfortaa:wght@500;700&family=Ubuntu+Mono:wght@400;700',
  },
  {
    id: 'industrial', ru: 'Индастриал', en: 'Industrial',
    vibe: { ru: 'Дорожные знаки, офлайн', en: 'Signage, offline' },
    sans: `'Bahnschrift', ${SYS}`,
    display: `'Bahnschrift SemiBold', 'Bahnschrift', ${SYS}`,
    label: `'Bahnschrift SemiBold Condensed', 'Bahnschrift Condensed', 'Bahnschrift', ${SYS}`,
    mono: `'Consolas', ${MONO}`,
  },
  {
    id: 'geo', ru: 'Геологика', en: 'Geologica',
    vibe: { ru: 'Техно-геометрия', en: 'Techno geometry' },
    sans: `'Geologica', ${SYS}`,
    display: `'Geologica', ${SYS}`,
    label: `'Geologica', ${SYS}`,
    mono: `'Martian Mono', ${MONO}`,
    google: 'family=Geologica:wght@300;400;500;600;700;800&family=Martian+Mono:wght@400;600',
  },
];

export function fontPreset(id: string): FontPreset {
  return FONT_PRESETS.find((p) => p.id === id) ?? FONT_PRESETS[0];
}

function ensureLink(id: string, query: string | undefined) {
  let link = document.getElementById(id) as HTMLLinkElement | null;
  if (!query) { link?.remove(); return; }
  const href = 'https://fonts.googleapis.com/css2?' + query + '&display=swap';
  if (!link) {
    link = document.createElement('link');
    link.id = id;
    link.rel = 'stylesheet';
    document.head.appendChild(link);
  }
  if (link.href !== href) link.href = href;
}

export function applyFontPreset(id: string) {
  const p = fontPreset(id);
  const root = document.documentElement;
  if (p.id === 'wawity') {
    for (const v of ['--font-sans', '--font-display', '--font-label', '--font-mono']) root.style.removeProperty(v);
  } else {
    root.style.setProperty('--font-sans', p.sans);
    root.style.setProperty('--font-display', p.display);
    root.style.setProperty('--font-label', p.label);
    root.style.setProperty('--font-mono', p.mono);
  }
  root.dataset.fontPreset = p.id;
  ensureLink('wawity-font-preset', p.google);
}

export function loadAllPresetFonts() {
  ensureLink('wawity-font-preview', FONT_PRESETS.filter((p) => p.google).map((p) => p.google).join('&'));
}
