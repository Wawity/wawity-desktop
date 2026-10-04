// Synthesised UI sounds (WebAudio) — no audio files, no network. Two packs are
// the exception: those are plain files in public/sounds/ that the user supplies.
import type { SoundPack } from './store';
import { customBuffer, decodeTrimmed, hydrateAudio, type AudioEvent } from './audioslots';

export type SoundEvent = AudioEvent;

export interface SoundConfig {
  enabled: boolean;
  pack: SoundPack;
  volume: number;
  connect: boolean;
  disconnect: boolean;
  error: boolean;
  clicks: boolean;
}

let cfg: SoundConfig = {
  enabled: false,
  pack: 'orbit',
  volume: 45,
  connect: true,
  disconnect: true,
  error: true,
  clicks: false,
};

export function configureSound(c: SoundConfig) {
  cfg = { ...c };
}

let ctx: AudioContext | null = null;
function audio(): AudioContext | null {
  try {
    if (!ctx) {
      const AC = window.AudioContext || (window as any).webkitAudioContext;
      ctx = new AC();
    }
    if (ctx.state === 'suspended') void ctx.resume();
    return ctx;
  } catch {
    return null;
  }
}
export function primeAudio() {
  audio();
}

/** The shared context, for callers that need to decode against the same
 *  sample rate they will play back at. */
export function sharedAudioContext(): AudioContext | null {
  return audio();
}

interface Voice {
  type?: OscillatorType;
  f: number;
  f2?: number;
  at?: number;
  dur: number;
  gain?: number;
  attack?: number;
  fm?: { ratio: number; depth: number };
  filter?: { type: BiquadFilterType; f: number; f2?: number; q?: number };
}

function voice(c: AudioContext, out: AudioNode, v: Voice) {
  const t0 = c.currentTime + (v.at ?? 0) + 0.005;
  const end = t0 + v.dur;
  const osc = c.createOscillator();
  osc.type = v.type ?? 'sine';
  osc.frequency.setValueAtTime(v.f, t0);
  if (v.f2) osc.frequency.exponentialRampToValueAtTime(v.f2, end);
  const g = c.createGain();
  const peak = v.gain ?? 0.3;
  g.gain.setValueAtTime(0.0001, t0);
  g.gain.exponentialRampToValueAtTime(peak, t0 + (v.attack ?? 0.008));
  g.gain.exponentialRampToValueAtTime(0.0001, end);
  if (v.fm) {
    const m = c.createOscillator();
    m.frequency.value = v.f * v.fm.ratio;
    const mg = c.createGain();
    mg.gain.setValueAtTime(v.f * v.fm.depth, t0);
    mg.gain.exponentialRampToValueAtTime(1, end);
    m.connect(mg);
    mg.connect(osc.frequency);
    m.start(t0);
    m.stop(end + 0.05);
  }
  let node: AudioNode = osc;
  if (v.filter) {
    const bq = c.createBiquadFilter();
    bq.type = v.filter.type;
    bq.frequency.setValueAtTime(v.filter.f, t0);
    if (v.filter.f2) bq.frequency.exponentialRampToValueAtTime(v.filter.f2, end);
    bq.Q.value = v.filter.q ?? 0.7;
    node.connect(bq);
    node = bq;
  }
  node.connect(g);
  g.connect(out);
  osc.start(t0);
  osc.stop(end + 0.05);
}

let noiseBuf: AudioBuffer | null = null;
function noise(
  c: AudioContext,
  out: AudioNode,
  o: { at?: number; dur: number; gain?: number; type?: BiquadFilterType; f: number; f2?: number; q?: number },
) {
  if (!noiseBuf) {
    noiseBuf = c.createBuffer(1, c.sampleRate * 1.5, c.sampleRate);
    const d = noiseBuf.getChannelData(0);
    for (let i = 0; i < d.length; i++) d[i] = Math.random() * 2 - 1;
  }
  const t0 = c.currentTime + (o.at ?? 0) + 0.005;
  const end = t0 + o.dur;
  const src = c.createBufferSource();
  src.buffer = noiseBuf;
  const bq = c.createBiquadFilter();
  bq.type = o.type ?? 'bandpass';
  bq.frequency.setValueAtTime(o.f, t0);
  if (o.f2) bq.frequency.exponentialRampToValueAtTime(o.f2, end);
  bq.Q.value = o.q ?? 1;
  const g = c.createGain();
  g.gain.setValueAtTime(0.0001, t0);
  g.gain.exponentialRampToValueAtTime(o.gain ?? 0.3, t0 + Math.min(0.08, o.dur / 3));
  g.gain.exponentialRampToValueAtTime(0.0001, end);
  src.connect(bq);
  bq.connect(g);
  g.connect(out);
  src.start(t0);
  src.stop(end + 0.05);
}

type Player = (c: AudioContext, out: AudioNode) => void;
const seq = (c: AudioContext, out: AudioNode, vs: Voice[]) => vs.forEach((v) => voice(c, out, v));

const PACKS: Record<SoundPack, Record<SoundEvent, Player>> = {
  // Smooth sine sweeps — the default, closest to the app's calm look.
  orbit: {
    connect: (c, o) =>
      seq(c, o, [
        { f: 330, f2: 660, dur: 0.35, gain: 0.22, attack: 0.04 },
        { f: 495, f2: 990, at: 0.12, dur: 0.45, gain: 0.18, attack: 0.04 },
        { f: 1320, at: 0.32, dur: 0.6, gain: 0.07 },
      ]),
    disconnect: (c, o) =>
      seq(c, o, [
        { f: 660, f2: 330, dur: 0.35, gain: 0.2, attack: 0.03 },
        { f: 440, f2: 220, at: 0.1, dur: 0.45, gain: 0.15 },
      ]),
    error: (c, o) =>
      seq(c, o, [
        { type: 'triangle', f: 220, f2: 170, dur: 0.18, gain: 0.3 },
        { type: 'triangle', f: 220, f2: 150, at: 0.2, dur: 0.28, gain: 0.3 },
      ]),
    click: (c, o) => voice(c, o, { f: 1150, f2: 900, dur: 0.05, gain: 0.08 }),
    toggle: (c, o) =>
      seq(c, o, [
        { f: 880, dur: 0.06, gain: 0.08 },
        { f: 1320, at: 0.05, dur: 0.08, gain: 0.07 },
      ]),
    tick: (c, o) => voice(c, o, { f: 2200, dur: 0.02, gain: 0.035, attack: 0.002 }),
  },
  // Rhythmic filtered pulses, like the Pulsar backdrop.
  pulsar: {
    connect: (c, o) =>
      [0, 0.09, 0.18, 0.27].forEach((at, i) =>
        voice(c, o, {
          type: 'sawtooth',
          f: 110 * (i + 2),
          at,
          dur: 0.14,
          gain: 0.14,
          filter: { type: 'lowpass', f: 600 + i * 700, q: 6 },
        }),
      ),
    disconnect: (c, o) =>
      [0, 0.1, 0.2].forEach((at, i) =>
        voice(c, o, {
          type: 'sawtooth',
          f: 110 * (4 - i),
          at,
          dur: 0.16,
          gain: 0.13,
          filter: { type: 'lowpass', f: 2200 - i * 700, q: 6 },
        }),
      ),
    error: (c, o) =>
      seq(c, o, [{ type: 'square', f: 96, dur: 0.35, gain: 0.12, filter: { type: 'lowpass', f: 900, f2: 200, q: 8 } }]),
    click: (c, o) => voice(c, o, { type: 'square', f: 300, dur: 0.035, gain: 0.05, filter: { type: 'lowpass', f: 1800 } }),
    toggle: (c, o) => voice(c, o, { type: 'sawtooth', f: 220, f2: 440, dur: 0.08, gain: 0.06, filter: { type: 'lowpass', f: 2400, q: 4 } }),
    tick: (c, o) => voice(c, o, { type: 'square', f: 1600, dur: 0.015, gain: 0.025 }),
  },
  // FM bells — glassy and bright.
  crystal: {
    connect: (c, o) =>
      seq(c, o, [
        { f: 784, dur: 1.0, gain: 0.16, fm: { ratio: 3.5, depth: 2 } },
        { f: 1175, at: 0.11, dur: 1.1, gain: 0.13, fm: { ratio: 3.5, depth: 2 } },
        { f: 1568, at: 0.22, dur: 1.2, gain: 0.1, fm: { ratio: 3.5, depth: 1.5 } },
      ]),
    disconnect: (c, o) =>
      seq(c, o, [
        { f: 1175, dur: 0.8, gain: 0.13, fm: { ratio: 2.5, depth: 1.6 } },
        { f: 784, at: 0.12, dur: 1.0, gain: 0.13, fm: { ratio: 2.5, depth: 1.6 } },
      ]),
    error: (c, o) =>
      seq(c, o, [
        { f: 466, dur: 0.6, gain: 0.16, fm: { ratio: 1.41, depth: 3 } },
        { f: 440, at: 0.16, dur: 0.7, gain: 0.14, fm: { ratio: 1.41, depth: 3 } },
      ]),
    click: (c, o) => voice(c, o, { f: 2637, dur: 0.12, gain: 0.05, fm: { ratio: 3.5, depth: 1 } }),
    toggle: (c, o) => voice(c, o, { f: 1976, dur: 0.25, gain: 0.07, fm: { ratio: 3.5, depth: 1.2 } }),
    tick: (c, o) => voice(c, o, { f: 3520, dur: 0.04, gain: 0.025, fm: { ratio: 2, depth: 0.5 } }),
  },
  // 8-bit square arpeggios.
  retro: {
    connect: (c, o) =>
      [523, 659, 784, 1047].forEach((f, i) => voice(c, o, { type: 'square', f, at: i * 0.07, dur: 0.09, gain: 0.08, attack: 0.002 })),
    disconnect: (c, o) =>
      [784, 659, 523, 392].forEach((f, i) => voice(c, o, { type: 'square', f, at: i * 0.07, dur: 0.09, gain: 0.08, attack: 0.002 })),
    error: (c, o) =>
      seq(c, o, [
        { type: 'square', f: 150, dur: 0.12, gain: 0.1, attack: 0.002 },
        { type: 'square', f: 110, at: 0.14, dur: 0.22, gain: 0.1, attack: 0.002 },
      ]),
    click: (c, o) => voice(c, o, { type: 'square', f: 880, dur: 0.03, gain: 0.05, attack: 0.001 }),
    toggle: (c, o) =>
      seq(c, o, [
        { type: 'square', f: 660, dur: 0.04, gain: 0.05, attack: 0.001 },
        { type: 'square', f: 990, at: 0.04, dur: 0.05, gain: 0.05, attack: 0.001 },
      ]),
    tick: (c, o) => voice(c, o, { type: 'square', f: 1760, dur: 0.012, gain: 0.02, attack: 0.001 }),
  },
  // Airy noise whooshes — almost silent, cinematic.
  void: {
    connect: (c, o) => {
      noise(c, o, { dur: 0.7, gain: 0.25, type: 'bandpass', f: 300, f2: 3200, q: 2 });
      voice(c, o, { f: 55, f2: 110, dur: 0.8, gain: 0.25, attack: 0.1 });
    },
    disconnect: (c, o) => {
      noise(c, o, { dur: 0.6, gain: 0.22, type: 'bandpass', f: 2800, f2: 250, q: 2 });
      voice(c, o, { f: 110, f2: 45, dur: 0.7, gain: 0.22, attack: 0.05 });
    },
    error: (c, o) => {
      noise(c, o, { dur: 0.3, gain: 0.3, type: 'lowpass', f: 500, q: 4 });
      voice(c, o, { type: 'triangle', f: 70, dur: 0.4, gain: 0.35 });
    },
    click: (c, o) => noise(c, o, { dur: 0.04, gain: 0.12, type: 'highpass', f: 3000 }),
    toggle: (c, o) => noise(c, o, { dur: 0.12, gain: 0.12, type: 'bandpass', f: 900, f2: 2400, q: 3 }),
    tick: (c, o) => noise(c, o, { dur: 0.015, gain: 0.06, type: 'highpass', f: 5000 }),
  },
};

export const SOUND_PACKS: Array<{ id: SoundPack; ru: string; en: string; hint: { ru: string; en: string } }> = [
  { id: 'orbit', ru: 'Орбита', en: 'Orbit', hint: { ru: 'Мягкие свипы', en: 'Soft sweeps' } },
  { id: 'pulsar', ru: 'Пульсар', en: 'Pulsar', hint: { ru: 'Синт-импульсы', en: 'Synth pulses' } },
  { id: 'crystal', ru: 'Кристалл', en: 'Crystal', hint: { ru: 'Стеклянные колокола', en: 'Glass bells' } },
  { id: 'retro', ru: 'Ретро 8-бит', en: 'Retro 8-bit', hint: { ru: 'Как на приставке', en: 'Console chiptune' } },
  { id: 'void', ru: 'Пустота', en: 'Void', hint: { ru: 'Кино-шорохи', en: 'Cinematic air' } },
  { id: 'minecraft', ru: 'Minecraft', en: 'Minecraft', hint: { ru: 'Файл: public/sounds/minecraft.ogg', en: 'File: public/sounds/minecraft.ogg' } },
  { id: 'moans', ru: 'Стоны', en: 'Moans', hint: { ru: 'Файл: public/sounds/moans.ogg', en: 'File: public/sounds/moans.ogg' } },
];

const lastAt: Partial<Record<SoundEvent, number>> = {};
const MIN_GAP: Record<SoundEvent, number> = { connect: 300, disconnect: 300, error: 400, click: 40, toggle: 40, tick: 45 };

/** Packs backed by files in public/sounds/<pack>/<event>.<ext> instead of a
 *  synthesiser. Each event gets its own file, so one pack can have a
 *  different sound for connecting than for a click. A flat public/sounds/<pack>.<ext>
 *  is still honoured as the sound for every event. */
const FILE_PACKS: SoundPack[] = ['minecraft', 'moans'];
export const SOUND_DIR = '/sounds/';

const packFiles = new Map<string, AudioBuffer>();
const packFlat = new Map<SoundPack, AudioBuffer>();
const tried = new Set<string>();

const EXT = ['ogg', 'mp3', 'wav'];

async function fetchDecoded(url: string, ctx: AudioContext): Promise<AudioBuffer | null> {
  try {
    const res = await fetch(url);
    if (!res.ok) return null;
    const { buffer } = await decodeTrimmed(await res.arrayBuffer(), ctx);
    return buffer;
  } catch {
    return null;
  }
}

export function packSoundUrl(pack: SoundPack, ev: SoundEvent): string[] {
  return EXT.map((x) => `${SOUND_DIR}${pack}/${ev}.${x}`);
}

export function packFlatUrl(pack: SoundPack): string[] {
  return EXT.map((x) => `${SOUND_DIR}${pack}.${x}`);
}

/** Load one event of a file pack. Missing files stay missing so the event
 *  falls through to the synthesised default instead of going quiet. */
export async function loadPackEvent(
  pack: SoundPack,
  ev: SoundEvent,
  ctx: AudioContext,
): Promise<boolean> {
  const key = `${pack}/${ev}`;
  if (packFiles.has(key)) return true;
  if (tried.has(key)) return false;
  for (const url of packSoundUrl(pack, ev)) {
    const buf = await fetchDecoded(url, ctx);
    if (buf) {
      packFiles.set(key, buf);
      tried.add(key);
      return true;
    }
  }
  tried.add(key);
  return false;
}

export async function loadPackFlat(pack: SoundPack, ctx: AudioContext): Promise<boolean> {
  if (packFlat.has(pack)) return true;
  if (tried.has(`flat:${pack}`)) return false;
  for (const url of packFlatUrl(pack)) {
    const buf = await fetchDecoded(url, ctx);
    if (buf) {
      packFlat.set(pack, buf);
      tried.add(`flat:${pack}`);
      return true;
    }
  }
  tried.add(`flat:${pack}`);
  return false;
}

/** True when the pack has any audio of its own, so the picker can hide it. */
export function packAvailable(pack: SoundPack): boolean {
  if (packFlat.has(pack)) return true;
  return (['connect', 'disconnect', 'error', 'click', 'toggle', 'tick'] as SoundEvent[]).some((ev) =>
    packFiles.has(`${pack}/${ev}`),
  );
}

export async function hydrateAll(ctx: AudioContext): Promise<void> {
  await hydrateAudio(ctx);
  const events: SoundEvent[] = ['connect', 'disconnect', 'error', 'click', 'toggle', 'tick'];
  await Promise.all(
    FILE_PACKS.map(async (p) => {
      await Promise.all(events.map((ev) => loadPackEvent(p, ev, ctx).catch(() => false)));
      await loadPackFlat(p, ctx).catch(() => false);
    }),
  );
}

/** Decode everything on the shared context, once the user has interacted. */
export function warmSounds(): void {
  const c = audio();
  if (!c) return;
  void hydrateAll(c).catch(() => {});
}

/** One buffer plays the same way for all events, which is what a pack of
 *  short effects ends up being. Reusing it keeps every event to one buffer. */
function playBuffer(c: AudioContext, master: GainNode, buf: AudioBuffer) {
  const src = c.createBufferSource();
  src.buffer = buf;
  src.connect(master);
  src.start();
}

function render(pack: SoundPack, ev: SoundEvent, volume: number) {
  const c = audio();
  if (!c) return;
  const now = performance.now();
  if (now - (lastAt[ev] ?? 0) < MIN_GAP[ev]) return;
  lastAt[ev] = now;
  const master = c.createGain();
  master.gain.value = Math.max(0, Math.min(1, volume / 100)) * 0.9;
  master.connect(c.destination);
  try {
    // A sound the user attached to this exact event wins over everything.
    const own = customBuffer(ev);
    const fromPack = FILE_PACKS.includes(pack)
      ? packFiles.get(`${pack}/${ev}`) ?? packFlat.get(pack)
      : undefined;
    if (own) playBuffer(c, master, own);
    else if (fromPack) playBuffer(c, master, fromPack);
    else (PACKS[pack] ?? PACKS.orbit)[ev](c, master);
  } catch {}
  setTimeout(() => master.disconnect(), 2500);
}

/** Respect user config (master switch + per-event switches). */
export function playSound(ev: SoundEvent) {
  if (!cfg.enabled) return;
  if ((ev === 'click' || ev === 'toggle' || ev === 'tick') && !cfg.clicks) return;
  if (ev === 'connect' && !cfg.connect) return;
  if (ev === 'disconnect' && !cfg.disconnect) return;
  if (ev === 'error' && !cfg.error) return;
  render(cfg.pack, ev, cfg.volume);
}

/** Always plays — used by the preview buttons on the visual page. */
export function previewSound(pack: SoundPack, ev: SoundEvent, volume = cfg.volume) {
  render(pack, ev, volume || 40);
}
