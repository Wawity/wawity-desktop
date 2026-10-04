

export function preloadUiFlags(): void {
  try {
    const raw = localStorage.getItem('wawity_settings');
    if (!raw) return;
    const settings = JSON.parse(raw) as {
      liquid_glass?: boolean;
      motion_level?: 'simple' | 'fancy';
    };
    if (settings.liquid_glass) {
      document.documentElement.classList.add('liquid-glass-on');
    }
    const root = document.documentElement;
    if (settings.motion_level === 'simple') {
      root.classList.add('motion-simple');
      root.classList.remove('motion-fancy');
    } else {
      root.classList.add('motion-fancy');
      root.classList.remove('motion-simple');
    }
  } catch {}
}
