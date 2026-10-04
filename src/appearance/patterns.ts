// Pictures for the connect button, and the clips that replace them per state.
//
// Which files exist is decided at build time by the asset-manifest plugin in
// vite.config.ts. Nothing here loads an image to find out: the picker is a
// list, and asking the browser whether it can fetch a file before offering it
// in a menu is how the list came up empty in the first place.

import manifest from 'virtual:wawity-assets';

export type PatternId = 'hentai' | 'anime' | 'mellstroy' | 'mellstroy-animated';

/** What the button shows while connecting. A clip that exists for all three
 *  states swaps its src, so the state change is visible without a crossfade
 *  that a 96px circle cannot show anyway. */
export type PowerPhase = 'offline' | 'connecting' | 'connected';

export function phaseOf(connected?: boolean, loading?: boolean): PowerPhase {
  if (connected) return 'connected';
  if (loading) return 'connecting';
  return 'offline';
}

export interface PatternDef {
  id: PatternId;
  kind: 'image' | 'video';
  ru: string;
  en: string;
  /** Stills: one file. Clips: one per state, under ART_DIR. */
  file?: string;
  states?: Record<PowerPhase, string>;
  advise: { ru: string; en: string };
}

const LANDSCAPE = {
  ru: 'Лучше горизонтальная: круглая маска срезает углы.',
  en: 'Landscape works best; the round mask crops the corners.',
};

const SPECS: Array<{
  id: PatternId;
  kind: 'image' | 'video';
  ru: string;
  en: string;
  need: string[];
  states?: Record<PowerPhase, string>;
}> = [
  { id: 'hentai', kind: 'image', ru: 'Хентай', en: 'Hentai', need: ['hentai.png'], },
  { id: 'anime', kind: 'image', ru: 'Аниме', en: 'Anime', need: ['anime.png'], },
  { id: 'mellstroy', kind: 'image', ru: 'Меллстрой', en: 'Mellstroy', need: ['mellstroy.png'], },
  {
    id: 'mellstroy-animated',
    kind: 'video',
    ru: 'Меллстрой (анимация)',
    en: 'Mellstroy animated',
    need: ['mellstroy-offline.mp4', 'mellstroy-connecting.mp4', 'mellstroy-connected.mp4'],
    states: {
      offline: 'mellstroy-offline.mp4',
      connecting: 'mellstroy-connecting.mp4',
      connected: 'mellstroy-connected.mp4',
    },
  },
];

// Root-relative, the way the rest of the app reaches public/ — Earth.vue asks
// for '/earth/day.jpg'. A bare relative path resolves against the current
// document rather than the bundle root.
export const ART_DIR = '/art/';

const onDisk = new Set(manifest.art);

/** A clip counts as present when at least one of its states is there, so a
 *  half-filled pack still shows up instead of vanishing from the list. */
function has(spec: (typeof SPECS)[number]): boolean {
  if (spec.kind === 'image') return onDisk.has(spec.need[0]);
  const present = spec.need.filter((f) => onDisk.has(f));
  if (present.length) return true;
  return false;
}

export const PATTERNS: PatternDef[] = SPECS.filter(has).map((spec) => ({
  id: spec.id,
  kind: spec.kind,
  ru: spec.ru,
  en: spec.en,
  file: spec.kind === 'image' ? spec.need[0] : undefined,
  states: spec.states,
  advise:
    spec.kind === 'video'
      ? { ru: 'Три ролика: офлайн, подключение, подключено. Без звука.', en: 'Three clips: offline, connecting, connected. Muted.' }
      : LANDSCAPE,
}));

export function patternFile(def: PatternDef): string {
  return def.file ?? '';
}

export function patternUrl(id: PatternId, phase?: PowerPhase): string {
  const def = PATTERNS.find((p) => p.id === id);
  if (!def) return '';
  if (def.kind === 'video') {
    const f = def.states?.[phase ?? 'offline'];
    return f ? ART_DIR + f : '';
  }
  return def.file ? ART_DIR + def.file : '';
}

export function patternKind(id: PatternId): 'image' | 'video' | null {
  return PATTERNS.find((p) => p.id === id)?.kind ?? null;
}

/** True when the state has a file of its own. A video pack with only some
 *  clips falls back to nothing for the rest rather than showing a stale frame. */
export function patternHas(id: PatternId, phase: PowerPhase): boolean {
  const def = PATTERNS.find((p) => p.id === id);
  if (!def) return false;
  return onDisk.has(def.kind === 'video' ? def.states?.[phase] ?? '' : def.file ?? '');
}
