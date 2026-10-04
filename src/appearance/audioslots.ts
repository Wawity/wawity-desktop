// User-supplied sounds, one slot per event.
//
// Everything is capped at 3 seconds. A naive truncate clicks audibly: the
// waveform stops mid-cycle and the discontinuity is a step, which is exactly
// what a short percussive UI sound must not have. So a longer file keeps its
// first 3 seconds and the last 120ms of that ramp linearly to silence. The
// sound fades out instead of stopping.

import { delAudio, getAudio, putAudio } from './idb';

export const SOUND_MAX_SECONDS = 3;
/** Length of the ramp at the cut point. Long enough to kill the click,
 *  short enough that the sound still reads as ending rather than fading. */
export const SOUND_FADE_SECONDS = 0.12;

export type AudioEvent = 'connect' | 'disconnect' | 'error' | 'click' | 'toggle' | 'tick';

export const AUDIO_EVENTS: AudioEvent[] = [
  'connect',
  'disconnect',
  'error',
  'click',
  'toggle',
  'tick',
];

export const EVENT_LABEL: Record<AudioEvent, [string, string]> = {
  connect: ['Подключение', 'Connect'],
  disconnect: ['Отключение', 'Disconnect'],
  error: ['Ошибка', 'Error'],
  click: ['Клик', 'Click'],
  toggle: ['Переключатель', 'Toggle'],
  tick: ['Отметка', 'Tick'],
};

/**
 * Copy the head of `buf` into a new buffer of at most SOUND_MAX_SECONDS,
 * ramping the tail to zero when the source was cut. Returns the input
 * untouched when it already fits.
 */
export function trimToLimit(buf: AudioBuffer, ctx: BaseAudioContext): AudioBuffer {
  if (buf.duration <= SOUND_MAX_SECONDS + 0.001) return buf;

  const rate = buf.sampleRate;
  const keep = Math.floor(SOUND_MAX_SECONDS * rate);
  const fadeN = Math.min(Math.floor(SOUND_FADE_SECONDS * rate), keep);
  const out = ctx.createBuffer(buf.numberOfChannels, keep, rate);

  for (let ch = 0; ch < buf.numberOfChannels; ch++) {
    const src = buf.getChannelData(ch);
    const dst = out.getChannelData(ch);
    dst.set(src.subarray(0, keep));
    // Divide by fadeN - 1, not fadeN: that puts the last sample exactly on
    // zero rather than one step above it, so the tail ends in true silence.
    const denom = Math.max(1, fadeN - 1);
    for (let i = 0; i < fadeN; i++) {
      const idx = keep - fadeN + i;
      dst[idx] *= i >= denom ? 0 : (denom - i) / denom;
    }
  }
  return out;
}

export async function decodeTrimmed(
  bytes: ArrayBuffer,
  ctx: BaseAudioContext,
): Promise<{ buffer: AudioBuffer; trimmed: boolean }> {
  // decodeAudioData detaches the ArrayBuffer, so hand it a copy.
  const decoded = await ctx.decodeAudioData(bytes.slice(0));
  if (decoded.duration <= SOUND_MAX_SECONDS + 0.001) {
    return { buffer: decoded, trimmed: false };
  }
  return { buffer: trimToLimit(decoded, ctx), trimmed: true };
}

const cache = new Map<AudioEvent, AudioBuffer>();
let hydration: Promise<void> | null = null;

export function customBuffer(ev: AudioEvent): AudioBuffer | undefined {
  return cache.get(ev);
}

export function hasCustom(ev: AudioEvent): boolean {
  return cache.has(ev);
}

export function loadedEvents(): AudioEvent[] {
  return AUDIO_EVENTS.filter((e) => cache.has(e));
}

/** Decode every stored custom sound once at boot. */
export function hydrateAudio(ctx: BaseAudioContext): Promise<void> {
  if (hydration) return hydration;
  hydration = Promise.all(
    AUDIO_EVENTS.map(async (ev) => {
      try {
        const blob = await getAudio(ev);
        if (!blob) return;
        const { buffer } = await decodeTrimmed(await blob.arrayBuffer(), ctx);
        cache.set(ev, buffer);
      } catch {
        /* a sound that will not decode is simply absent */
      }
    }),
  ).then(() => undefined);
  return hydration;
}

export async function setCustom(
  ev: AudioEvent,
  file: File,
  ctx: BaseAudioContext,
): Promise<{ trimmed: boolean; duration: number }> {
  const { buffer, trimmed } = await decodeTrimmed(await file.arrayBuffer(), ctx);
  await putAudio(ev, file);
  cache.set(ev, buffer);
  return { trimmed, duration: buffer.duration };
}

export async function clearCustom(ev: AudioEvent): Promise<void> {
  cache.delete(ev);
  await delAudio(ev);
}