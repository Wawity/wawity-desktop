import { ref } from 'vue';
import { gsap } from 'gsap';

export const EASE_OUT_EXPO = 'expo.out';
export const EASE_BACK = 'back.out(1.4)';

let reducedMotion = false;
if (typeof window !== 'undefined') {
  const mq = window.matchMedia('(prefers-reduced-motion: reduce)');
  reducedMotion = mq.matches;
  mq.addEventListener?.('change', (e) => {
    reducedMotion = e.matches;
  });
}

const motionLevel = ref<'simple' | 'fancy'>('fancy');

export function setMotionLevel(level: 'simple' | 'fancy') {
  motionLevel.value = level === 'simple' ? 'simple' : 'fancy';
  if (typeof document !== 'undefined') {
    const root = document.documentElement;
    root.classList.toggle('motion-fancy', level !== 'simple');
    root.classList.toggle('motion-simple', level === 'simple');
  }
}

export function motionLevelRef() {
  return motionLevel;
}

export function isFancy(): boolean {
  return !reducedMotion && motionLevel.value === 'fancy';
}

export function prefersReduced(): boolean {
  return reducedMotion;
}

export function pressPop(el: HTMLElement | null): void {
  if (!isFancy() || !el) return;
  gsap.killTweensOf(el);
  gsap
    .timeline()
    .to(el, { scale: 0.94, duration: 0.08, ease: 'power2.out' })
    .to(el, { scale: 1, duration: 0.34, ease: EASE_BACK });
}

export function staggerChildren(
  parent: Element | null,
  selector = '.card, .setting-row, .stat-card',
  opts?: { per?: number; from?: number },
): void {
  if (!isFancy() || !parent) return;
  const targets = parent.querySelectorAll(selector);
  if (targets.length === 0) return;
  gsap.fromTo(
    targets,
    { opacity: 0, y: 12 },
    {
      opacity: 1,
      y: 0,
      duration: 0.36,
      ease: EASE_OUT_EXPO,
      stagger: opts?.per ?? 0.03,
      delay: opts?.from ?? 0,
      clearProps: 'transform',
    },
  );
}

let warmed = false;

export function warmUpMotion(): void {
  if (warmed) return;
  warmed = true;
  try {
    const ghost = document.createElement('div');
    ghost.style.cssText =
      'position:fixed;top:-100px;left:-100px;width:8px;height:8px;opacity:0;pointer-events:none;';
    document.body.appendChild(ghost);

    const sequence = [
      () => gsap.to(ghost, { opacity: 1, y: 10, duration: 0.01 }),
      () => gsap.fromTo(ghost, { opacity: 0 }, { opacity: 1, duration: 0.01 }),
      () => gsap.to(ghost, { filter: 'blur(4px)', duration: 0.01 }),
      () => gsap.to(ghost, { filter: 'blur(0px)', duration: 0.01 }),
      () => gsap.to(ghost, { scale: 1.2, duration: 0.01, ease: 'expo.out' }),
      () => gsap.to(ghost, { scale: 1, duration: 0.01, ease: 'back.out(1.4)' }),
      () => gsap.to(ghost, { scale: 1, duration: 0.01, ease: 'elastic.out(1,0.45)' }),
      () =>
        gsap.timeline().to(ghost, { x: 5, duration: 0.01 }).to(ghost, { x: 0, duration: 0.01 }),
      () => gsap.fromTo(ghost, {}, { clearProps: 'all' }),
    ];
    
    for (const step of sequence) step();

    window.setTimeout(() => {
      gsap.killTweensOf(ghost);
      ghost.remove();
    }, 80);
  } catch {}
}
