import { defineStore } from 'pinia';
import { reactive, ref, watch } from 'vue';
import { delImage, delVideo, getImage, getVideo, putImage, putVideo } from './idb';
import { FONT_PRESETS } from './fonts';
import type { PatternId } from './patterns';

export type PowerSource = 'none' | 'custom' | PatternId;
export type Blend = 'normal' | 'screen' | 'overlay' | 'luminosity';
export type SoundPack = 'orbit' | 'pulsar' | 'crystal' | 'retro' | 'void' | 'minecraft' | 'moans';

export interface PowerArtSettings {
  source: PowerSource;
  opacityOff: number;
  opacityOn: number;
  blur: number;
  zoom: number;
  posX: number;
  posY: number;
  blend: Blend;
  grayscaleOff: boolean;
  pulse: boolean;
  spin: boolean;
  edgeFade: boolean;
}

export interface AppearanceState {
  accent: string;
  accentCustom: string;
  accentStatus: boolean;
  font: string;
  parallax: { enabled: boolean; strength: number };
  reactive: boolean;
  power: PowerArtSettings;
  sound: {
    enabled: boolean;
    pack: SoundPack;
    volume: number;
    connect: boolean;
    disconnect: boolean;
    error: boolean;
    clicks: boolean;
  };
  openModules: Record<string, boolean>;
}

export const DEFAULT_APPEARANCE: AppearanceState = {
  accent: 'ember', accentCustom: '#fe2c02', accentStatus: false, font: 'wawity',
  parallax: { enabled: false, strength: 14 }, reactive: false,
  power: {
    // Tuned for pictures, not for the thin white lines these slots used to be
    // filled with. A one-pixel gradient stroke still reads at 9% alpha; a
    // photograph does not — add the mask, the desaturation and screen
    // blending and it disappears entirely, which is what made "the picture
    // does not turn on" look like a broken file rather than a default.
    source: 'none', opacityOff: 30, opacityOn: 62, blur: 1, zoom: 1,
    posX: 50, posY: 50, blend: 'screen', grayscaleOff: false, pulse: true, spin: false, edgeFade: true,
  },
  sound: { enabled: false, pack: 'orbit', volume: 45, connect: true, disconnect: true, error: true, clicks: false },
  openModules: { 'bg-source': true, 'power-art': true, fonts: true },
};

const KEY = 'wawity_appearance';
function isObj(v: unknown): v is Record<string, unknown> { return !!v && typeof v === 'object' && !Array.isArray(v); }

export function mergeInto(dst: Record<string, any>, src: unknown, shape: Record<string, any> = dst) {
  if (!isObj(src)) return;
  for (const key of Object.keys(shape)) {
    const incoming = src[key];
    if (incoming === undefined) continue;
    const base = shape[key];
    if (key === 'openModules' && isObj(incoming)) {
      dst[key] = Object.fromEntries(Object.entries(incoming).filter(([name, value]) => name !== '__proto__' && name !== 'constructor' && name !== 'prototype' && typeof value === 'boolean'));
    } else if (isObj(base)) {
      if (!isObj(dst[key])) dst[key] = { ...base };
      mergeInto(dst[key], incoming, base);
    } else if (typeof incoming === typeof base && (typeof incoming !== 'number' || Number.isFinite(incoming))) {
      dst[key] = incoming;
    }
  }
}

function fresh(): AppearanceState { return JSON.parse(JSON.stringify(DEFAULT_APPEARANCE)); }
function normalize(state: AppearanceState) {
  if (!FONT_PRESETS.some((p) => p.id === state.font)) state.font = 'wawity';
}
function load(): AppearanceState {
  const out = fresh();
  try {
    const raw = localStorage.getItem(KEY);
    if (raw) mergeInto(out, JSON.parse(raw), DEFAULT_APPEARANCE);
  } catch {}
  normalize(out);
  return out;
}

export type ImageSlot = 'power' | 'bg';
export type VideoSlot = 'power' | 'bg';
export const IDB_BG = 'idb:bg';

export const useAppearance = defineStore('appearance', () => {
  const state = reactive<AppearanceState>(load());
  const powerUrl = ref('');
  const bgUrl = ref('');
  let hydration: Promise<void> | null = null;
  const revisions: Record<ImageSlot, number> = { power: 0, bg: 0 };
  const queues: Record<ImageSlot, Promise<void>> = { power: Promise.resolve(), bg: Promise.resolve() };

  watch(state, () => {
    try { localStorage.setItem(KEY, JSON.stringify(state)); } catch {}
  }, { deep: true });

  function slotRef(slot: ImageSlot) { return slot === 'power' ? powerUrl : bgUrl; }

  function hydrateImages(): Promise<void> {
    if (hydration) return hydration;
    hydration = Promise.all((['power', 'bg'] as ImageSlot[]).map(async (slot) => {
      const stamp = revisions[slot];
      const blob = await getImage(slot).catch(() => null);
      if (blob && revisions[slot] === stamp && !slotRef(slot).value) slotRef(slot).value = URL.createObjectURL(blob);
    })).then(() => {});
    return hydration;
  }

  function setImage(slot: ImageSlot, blob: Blob | null): Promise<void> {
    ++revisions[slot];
    const run = queues[slot].catch(() => {}).then(async () => {
      if (blob) await putImage(slot, blob);
      else await delImage(slot);
      const target = slotRef(slot);
      const previous = target.value;
      target.value = blob ? URL.createObjectURL(blob) : '';
      if (previous) URL.revokeObjectURL(previous);
    });
    queues[slot] = run.catch(() => {});
    return run;
  }

  function reset() {
    const keepOpen = { ...state.openModules };
    mergeInto(state, fresh(), DEFAULT_APPEARANCE);
    state.openModules = keepOpen;
  }
  function patch(p: unknown) {
    mergeInto(state, p, DEFAULT_APPEARANCE);
    normalize(state);
  }

  // ── video slots ──
  // Kept beside the picture slots rather than inside them: a background can be
  // a clip or a still, and replacing one must not silently discard the other.
  const powerVideoUrl = ref('');
  const bgVideoUrl = ref('');
  const videoRev: Record<VideoSlot, number> = { power: 0, bg: 0 };
  const videoQueues: Record<VideoSlot, Promise<void>> = {
    power: Promise.resolve(),
    bg: Promise.resolve(),
  };
  let videoHydrated: Promise<void> | null = null;

  function videoRef(slot: VideoSlot) {
    return slot === 'power' ? powerVideoUrl : bgVideoUrl;
  }

  function hydrateVideos(): Promise<void> {
    if (videoHydrated) return videoHydrated;
    videoHydrated = Promise.all(
      (['power', 'bg'] as VideoSlot[]).map(async (slot) => {
        const stamp = videoRev[slot];
        const blob = await getVideo(slot).catch(() => null);
        if (blob && videoRev[slot] === stamp && !videoRef(slot).value) {
          videoRef(slot).value = URL.createObjectURL(blob);
        }
      }),
    ).then(() => {});
    return videoHydrated;
  }

  function setVideo(slot: VideoSlot, blob: Blob | null): Promise<void> {
    ++videoRev[slot];
    const run = videoQueues[slot].catch(() => {}).then(async () => {
      if (blob) await putVideo(slot, blob);
      else await delVideo(slot);
      const target = videoRef(slot);
      const previous = target.value;
      target.value = blob ? URL.createObjectURL(blob) : '';
      if (previous) URL.revokeObjectURL(previous);
    });
    videoQueues[slot] = run.catch(() => {});
    return run;
  }

  return {
    state,
    powerUrl,
    bgUrl,
    powerVideoUrl,
    bgVideoUrl,
    hydrateImages,
    setImage,
    hydrateVideos,
    setVideo,
    reset,
    patch,
  };
});
