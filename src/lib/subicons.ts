import {
  Shield,
  Zap,
  Rocket,
  Globe2,
  Flame,
  Star,
  Heart,
  Crown,
  Gem,
  Anchor,
  Atom,
  Cloud,
  Compass,
  Feather,
  Ghost,
  Leaf,
} from './appIcons';
import type { Component } from 'vue';

export const SUB_ICONS: { key: string; icon: Component }[] = [
  { key: 'shield', icon: Shield },
  { key: 'zap', icon: Zap },
  { key: 'rocket', icon: Rocket },
  { key: 'globe', icon: Globe2 },
  { key: 'flame', icon: Flame },
  { key: 'star', icon: Star },
  { key: 'heart', icon: Heart },
  { key: 'crown', icon: Crown },
  { key: 'gem', icon: Gem },
  { key: 'anchor', icon: Anchor },
  { key: 'atom', icon: Atom },
  { key: 'cloud', icon: Cloud },
  { key: 'compass', icon: Compass },
  { key: 'feather', icon: Feather },
  { key: 'ghost', icon: Ghost },
  { key: 'leaf', icon: Leaf },
];

export const SUB_COLORS: string[] = [
  '#a78bfa',
  '#8fb6ff',
  '#5ee69a',
  '#f0d36a',
  '#ff9f6b',
  '#ff8a92',
  '#f472b6',
  '#22d3ee',
  '#c084fc',
  '#34d399',
  '#fbbf24',
  '#94a3b8',
];

export function iconByKey(key: string | null | undefined): Component {
  const hit = SUB_ICONS.find((item) => item.key === key);
  return hit ? hit.icon : Shield;
}

export function badgeForIndex(index: number): { icon: string; color: string } {
  const safe = Math.max(0, index);
  return {
    icon: SUB_ICONS[safe % SUB_ICONS.length].key,
    color: SUB_COLORS[safe % SUB_COLORS.length],
  };
}

export function tintSoft(color: string, alpha = 0.16): string {
  const raw = (color || '#a78bfa').replace('#', '');
  const full =
    raw.length === 3
      ? raw
          .split('')
          .map((c) => c + c)
          .join('')
      : raw;
  const r = parseInt(full.slice(0, 2), 16) || 0;
  const g = parseInt(full.slice(2, 4), 16) || 0;
  const b = parseInt(full.slice(4, 6), 16) || 0;
  return `rgba(${r}, ${g}, ${b}, ${alpha})`;
}
