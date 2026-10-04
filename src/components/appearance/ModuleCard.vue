<template>
  <div
    class="mc"
    :class="{ 'mc--on': enabled === true, 'mc--open': isOpen, 'mc--toggle': enabled !== undefined }"
  >
    <div class="mc-head" :title="enabled !== undefined ? headHint : undefined" @click="onHead" @contextmenu.prevent="toggleOpen">
      <span class="mc-bar" />
      <span class="mc-glyph">{{ glyph }}</span>
      <span class="mc-title">{{ title }}</span>
      <span v-if="badge" class="mc-badge">{{ badge }}</span>
      <button type="button" class="mc-caret" :aria-expanded="isOpen" @click.stop="toggleOpen">
        <svg width="10" height="10" viewBox="0 0 10 10"><path d="M2 3.5 5 6.5 8 3.5" fill="none" stroke="currentColor" stroke-width="1.4" /></svg>
      </button>
    </div>
    <div class="mc-wrap">
      <div class="mc-inner">
        <div class="mc-body">
          <p v-if="desc" class="mc-desc">{{ desc }}</p>
          <slot />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
// ClickGUI-style module: click the name = on/off (for toggle modules),
// right-click or the caret = expand settings.
import { computed } from 'vue';
import { useAppearance } from '../../appearance/store';

const props = defineProps<{ id: string; title: string; glyph: string; desc?: string; badge?: string; enabled?: boolean }>();
const emit = defineEmits<{ (e: 'update:enabled', v: boolean): void }>();
const a = useAppearance();
const headHint = 'ЛКМ — вкл/выкл · ПКМ — настройки';

const isOpen = computed(() => !!a.state.openModules[props.id]);
function toggleOpen() {
  a.state.openModules = { ...a.state.openModules, [props.id]: !isOpen.value };
}
function onHead() {
  if (props.enabled === undefined) toggleOpen();
  else emit('update:enabled', !props.enabled);
}
</script>

<style scoped>
.mc {
  position: relative;
  border-top: 1px solid var(--border);
  background: transparent;
  transition: background 0.2s;
}
.mc:first-of-type { border-top: 0; }
.mc-head {
  position: relative;
  display: flex;
  align-items: center;
  gap: 9px;
  height: 38px;
  padding: 0 8px 0 14px;
  cursor: pointer;
  user-select: none;
  transition: background 0.15s;
}
.mc-head:hover { background: oklch(1 0 0 / 4%); }
.mc-bar {
  position: absolute;
  left: 0;
  top: 8px;
  bottom: 8px;
  width: 2px;
  border-radius: 2px;
  background: oklch(1 0 0 / 12%);
  transition: background 0.2s, box-shadow 0.2s, top 0.2s, bottom 0.2s;
}
.mc--on .mc-bar { top: 4px; bottom: 4px; background: var(--ember); box-shadow: 0 0 10px var(--ember); }
.mc:not(.mc--toggle) .mc-bar { background: transparent; }
.mc-glyph {
  flex: none;
  width: 22px;
  font-family: var(--font-mono);
  font-size: 9.5px;
  letter-spacing: 0.04em;
  color: var(--muted-foreground);
}
.mc--on .mc-glyph { color: var(--ember-soft); }
.mc-title { flex: 1; min-width: 0; font-size: 12.5px; font-weight: 500; color: var(--muted-foreground); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; transition: color 0.15s; }
.mc--on .mc-title, .mc:not(.mc--toggle) .mc-title { color: var(--paper); }
.mc-badge {
  font-family: var(--font-mono);
  font-size: 9.5px;
  padding: 1px 5px;
  border-radius: 3px;
  color: var(--muted-foreground);
  border: 1px solid var(--border);
  max-width: 90px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.mc-caret {
  flex: none;
  display: grid;
  place-items: center;
  width: 22px;
  height: 22px;
  border: 0;
  border-radius: 4px;
  background: none;
  color: var(--muted-foreground);
  cursor: pointer;
  transition: transform 0.25s cubic-bezier(0.22, 1, 0.36, 1), background 0.15s;
}
.mc-caret:hover { background: oklch(1 0 0 / 7%); color: var(--foreground); }
.mc--open .mc-caret { transform: rotate(180deg); }

/* height animation without JS */
.mc-wrap { display: grid; grid-template-rows: 0fr; transition: grid-template-rows 0.28s cubic-bezier(0.22, 1, 0.36, 1); }
.mc--open .mc-wrap { grid-template-rows: 1fr; }
.mc-inner { overflow: hidden; min-height: 0; }
.mc-body { display: flex; flex-direction: column; gap: 12px; padding: 4px 14px 14px; }
.mc--open { background: oklch(1 0 0 / 2%); }
.mc-desc { margin: 0; font-size: 11.5px; line-height: 1.45; color: var(--muted-foreground); }
</style>
