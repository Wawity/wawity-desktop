import { computed, onBeforeUnmount, ref } from 'vue';

export function useTilt(maxDegrees = 7) {
  const rotX = ref(0);
  const rotY = ref(0);
  const glowX = ref(50);
  const glowY = ref(30);
  const engaged = ref(false);
  let frame = 0;

  function handleMove(event: PointerEvent) {
    const target = event.currentTarget as HTMLElement | null;
    if (!target) return;
    const rect = target.getBoundingClientRect();
    const relX = (event.clientX - rect.left) / rect.width - 0.5;
    const relY = (event.clientY - rect.top) / rect.height - 0.5;
    if (frame) cancelAnimationFrame(frame);
    frame = requestAnimationFrame(() => {
      rotY.value = relX * maxDegrees * 2;
      rotX.value = -relY * maxDegrees * 2;
      glowX.value = (relX + 0.5) * 100;
      glowY.value = (relY + 0.5) * 100;
      engaged.value = true;
    });
  }

  function handleLeave() {
    if (frame) cancelAnimationFrame(frame);
    frame = 0;
    rotX.value = 0;
    rotY.value = 0;
    glowX.value = 50;
    glowY.value = 30;
    engaged.value = false;
  }

  const tiltStyle = computed(() => ({
    '--rx': `${rotX.value.toFixed(2)}deg`,
    '--ry': `${rotY.value.toFixed(2)}deg`,
    '--gx': `${glowX.value.toFixed(1)}%`,
    '--gy': `${glowY.value.toFixed(1)}%`,
    '--engaged': engaged.value ? '1' : '0',
  }));

  onBeforeUnmount(() => {
    if (frame) cancelAnimationFrame(frame);
  });

  return { tiltStyle, handleMove, handleLeave, engaged };
}
