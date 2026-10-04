import { watch, watchEffect } from 'vue';
import './appearance.css';
import { useAppearance } from './store';
import { useVpnStore } from '../stores/vpn';
import { applyFontPreset } from './fonts';
import { accentHex } from './accents';
import { configureSound, playSound, primeAudio, warmSounds } from './sounds';

let installed = false;
export function installAppearance() {
  if (installed) return;
  installed = true;
  const a = useAppearance();
  const vpn = useVpnStore();
  const root = document.documentElement;
  const bootAt = performance.now();
  void a.hydrateImages();
  root.style.removeProperty('--radius');
  root.classList.remove('fx-on');
  for (const v of ['--fx-grain', '--fx-vignette', '--fx-tint']) root.style.removeProperty(v);

  watchEffect(() => {
    const hex = accentHex(a.state.accent, a.state.accentCustom);
    if (a.state.accent === 'ember') {
      for (const v of ['--ember', '--ember-soft', '--ember-deep']) root.style.removeProperty(v);
    } else {
      root.style.setProperty('--ember', hex);
      root.style.setProperty('--ember-soft', `color-mix(in oklch, ${hex} 55%, white)`);
      root.style.setProperty('--ember-deep', `color-mix(in oklch, ${hex} 72%, black)`);
    }
    if (a.state.accentStatus) root.style.setProperty('--success', hex);
    else root.style.removeProperty('--success');
  });
  watchEffect(() => applyFontPreset(a.state.font));
  watchEffect(() => {
    root.classList.toggle('fx-parallax', a.state.parallax.enabled);
    root.style.setProperty('--par-max', `${a.state.parallax.enabled ? a.state.parallax.strength : 0}px`);
    if (!a.state.parallax.enabled) {
      root.style.setProperty('--par-x', '0px');
      root.style.setProperty('--par-y', '0px');
    }
    root.classList.toggle('fx-reactive', a.state.reactive);
  });
  watchEffect(() => configureSound({ ...a.state.sound }));
  watch(() => !!vpn.status?.connected, (now, before) => {
    root.classList.toggle('wawity-vpn-on', now);
    root.classList.toggle('wawity-vpn-off', !now);
    if (before === undefined || performance.now() - bootAt < 3000) return;
    if (now && !before) playSound('connect');
    else if (!now && before) playSound('disconnect');
  }, { immediate: true });
  watch(() => vpn.connectError, (err) => { if (err) playSound('error'); });
  window.addEventListener('pointerdown', primeAudio, { once: true, passive: true });
  // Decode on the same context that will play them, and before anything can
  // need them: decoding a buffer mid-gesture is what makes a click land late.
  window.addEventListener('pointerdown', warmSounds, { once: true, passive: true });
  window.addEventListener('click', (e) => {
    if (!a.state.sound.clicks) return;
    const el = (e.target as HTMLElement | null)?.closest?.('button, a, [role="switch"], [role="tab"]');
    if (!el || el.closest('.power-btn')) return;
    playSound(el.getAttribute('role') === 'switch' ? 'toggle' : 'click');
  }, { passive: true, capture: true });
  let raf = 0;
  let tx = 0;
  let ty = 0;
  window.addEventListener('pointermove', (e) => {
    if (!a.state.parallax.enabled) return;
    tx = (e.clientX / Math.max(1, window.innerWidth) - 0.5) * -2;
    ty = (e.clientY / Math.max(1, window.innerHeight) - 0.5) * -2;
    if (raf) return;
    raf = requestAnimationFrame(() => {
      raf = 0;
      if (!a.state.parallax.enabled) return;
      const strength = a.state.parallax.strength;
      root.style.setProperty('--par-x', `${(tx * strength).toFixed(1)}px`);
      root.style.setProperty('--par-y', `${(ty * strength).toFixed(1)}px`);
    });
  }, { passive: true });
}
