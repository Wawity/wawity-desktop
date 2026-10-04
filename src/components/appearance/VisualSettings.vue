<template>
  <div class="vs">
    <header class="vs-top">
      <div class="vs-title">
        <span class="vs-kicker">{{ tr('ВИЗУАЛ', 'VISUALS') }}</span>
        <h2>{{ tr('Кастомизация', 'Customization') }}</h2>
      </div>
      <label class="vs-search">
        <svg width="13" height="13" viewBox="0 0 16 16"><circle cx="7" cy="7" r="5" fill="none" stroke="currentColor" stroke-width="1.5" /><path d="M11 11l3.5 3.5" stroke="currentColor" stroke-width="1.5" /></svg>
        <input v-model="q" :placeholder="tr('Найти модуль…', 'Find module…')" spellcheck="false" />
      </label>
      <div class="vs-actions">
        <button class="vs-act" @click="exportTheme">{{ copied ? tr('Скопировано', 'Copied') : tr('Экспорт', 'Export') }}</button>
        <button class="vs-act" @click="importOpen = !importOpen">{{ tr('Импорт', 'Import') }}</button>
        <button class="vs-act vs-act--danger" @click="resetAll">{{ confirmReset ? tr('Точно?', 'Sure?') : tr('Сброс', 'Reset') }}</button>
      </div>
    </header>

    <div v-if="importOpen" class="vs-import">
      <input v-model="importCode" placeholder="wawity-theme:…" spellcheck="false" @keydown.enter="importTheme" />
      <button class="vs-act" @click="importTheme">OK</button>
      <span v-if="importError" class="vs-err">{{ importError }}</span>
    </div>

    <div class="vs-themes">
      <span class="vs-themes-label">{{ tr('Готовые стили', 'Looks') }}</span>
      <button
        v-for="p in THEMES"
        :key="p.id"
        class="vs-theme"
        :class="{ 'vs-theme--on': themeActive(p) }"
        :style="{ '--t': accentHex(p.accent) }"
        @click="applyTheme(p)"
      >
        <span class="vs-theme-dot" />
        <span>{{ tr(p.ru, p.en) }}</span>
      </button>
    </div>

    <div class="vs-grid">
      <!-- ── COLUMN 1: BACKGROUND ── -->
      <section class="vs-col">
        <div class="vs-col-head"><span class="vs-col-tag">01</span>{{ tr('Фон', 'Background') }}</div>

        <ModuleCard v-show="match('фон картинка сцена background image scene wallpaper')" id="bg-source" glyph="BG" :title="tr('Источник фона', 'Background source')" :badge="bgModeLabel">
          <div class="vs-modes">
            <button v-for="m in bgModes" :key="m.id" class="vs-mode" :class="{ 'vs-mode--on': bgMode === m.id }" @click="setBgMode(m.id)">
              <span class="vs-mode-art" :class="`vs-mode-art--${m.id}`" :style="m.id === 'image' && bgSrc ? { backgroundImage: `url('${bgSrc}')` } : undefined" />
              <span class="vs-mode-name">{{ m.label }}</span>
            </button>
          </div>

          <!-- live preview of the whole stage -->
          <div class="pv">
            <div class="pv-bg" :class="{ 'pv-bg--scene': bgMode !== 'image' || !bgSrc, 'pv-bg--flat': bgMode === 'flat' }" :style="pvBgStyle" />
            <div class="pv-dim" :style="{ opacity: bgMode === 'image' ? s.bg_custom_dim / 100 : 0.35 }" />
            <div v-if="a.state.fx.enabled" class="pv-fx">
              <span class="pv-tint" :style="{ opacity: a.state.fx.tint / 100 * 0.85 }" />
              <span class="pv-vig" :style="{ opacity: a.state.fx.vignette / 100 }" />
              <span class="pv-grain" :style="{ opacity: a.state.fx.grain / 100 * 0.6 }" />
            </div>
            <div class="pv-side"><i /><i /><i /><i /></div>
            <div class="pv-btn" :class="{ 'pv-btn--on': previewOn }" @click="previewOn = !previewOn">
              <PowerArt :connected="previewOn" />
              <svg width="16" height="16" viewBox="0 0 24 24"><path d="M12 3v8M6.3 6.3a8 8 0 1 0 11.4 0" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" /></svg>
            </div>
            <span class="pv-cap">{{ tr('Превью · клик по кнопке', 'Preview · click the button') }}</span>
          </div>

          <template v-if="bgMode === 'image'">
            <div class="vs-drop" @click="pickBg">
              <span class="vs-drop-plus">+</span>
              <span>{{ bgSrc ? tr('Заменить картинку', 'Replace picture') : tr('Выбрать картинку с ПК', 'Pick a picture') }}</span>
              <small>PNG · JPG · WEBP · GIF</small>
            </div>
            <div class="vs-url">
              <input v-model="urlDraft" placeholder="https://…/wallpaper.jpg" spellcheck="false" @keydown.enter="applyUrl" />
              <button class="vs-act" @click="applyUrl">URL</button>
              <button v-if="s.bg_custom_url" class="vs-act vs-act--danger" @click="clearBg">✕</button>
            </div>
            <span v-if="imgError" class="vs-err">{{ imgError }}</span>
          </template>

          <template v-else>
            <div class="vs-drop" @click="pickBgVideo">
              <span class="vs-drop-plus">+</span>
              <span>{{ a.bgVideoUrl ? tr('Заменить видео', 'Replace video') : tr('Выбрать видео с ПК', 'Pick a video') }}</span>
              <small>MP4 · WEBM · MOV</small>
            </div>
            <div class="vs-url">
              <span class="vs-act vs-act--wide vs-act--dim">{{ tr('Зацикливается и играет без звука', 'Loops and plays muted') }}</span>
              <button v-if="a.bgVideoUrl" class="vs-act vs-act--danger" @click="dropBgVideo">✕</button>
            </div>
          </template>
        </ModuleCard>

        <ModuleCard v-show="(bgMode === 'image' || bgMode === 'video') && match('обработка затемнение размытие масштаб позиция dim blur zoom position')" id="bg-tune" glyph="IMG" :title="tr('Обработка картинки', 'Picture tuning')">
          <WSlider :model-value="s.bg_custom_dim" :label="tr('Затемнение', 'Dim')" :max="90" unit="%" :default-value="45" @update:model-value="set('bg_custom_dim', $event)" />
          <WSlider :model-value="s.bg_custom_blur" :label="tr('Размытие', 'Blur')" :max="40" unit="px" :default-value="0" @update:model-value="set('bg_custom_blur', $event)" />
          <WSlider :model-value="s.bg_custom_zoom" :label="tr('Масштаб', 'Zoom')" :min="1" :max="2.5" :step="0.05" unit="×" :default-value="1" @update:model-value="set('bg_custom_zoom', $event)" />
          <WSlider :model-value="s.bg_custom_pos_x" :label="tr('Позиция X', 'Position X')" unit="%" :default-value="50" @update:model-value="set('bg_custom_pos_x', $event)" />
          <WSlider :model-value="s.bg_custom_pos_y" :label="tr('Позиция Y', 'Position Y')" unit="%" :default-value="50" @update:model-value="set('bg_custom_pos_y', $event)" />
        </ModuleCard>

        <ModuleCard v-show="match('плёнка пленка зерно виньетка тонирование grain vignette tint film')" id="bg-fx" v-model:enabled="a.state.fx.enabled" glyph="FX" :title="tr('Плёнка', 'Film')" :desc="tr('Зерно, виньетка и тонирование акцентом поверх любого фона.', 'Grain, vignette and accent tint over any background.')">
          <WSlider v-model="a.state.fx.grain" :label="tr('Зерно', 'Grain')" unit="%" :default-value="18" :disabled="!a.state.fx.enabled" />
          <WSlider v-model="a.state.fx.vignette" :label="tr('Виньетка', 'Vignette')" unit="%" :default-value="35" :disabled="!a.state.fx.enabled" />
          <WSlider v-model="a.state.fx.tint" :label="tr('Тон акцента', 'Accent tint')" unit="%" :default-value="0" :disabled="!a.state.fx.enabled" />
        </ModuleCard>

        <ModuleCard v-show="match('параллакс мышь parallax mouse')" id="bg-parallax" v-model:enabled="a.state.parallax.enabled" glyph="PRX" :title="tr('Параллакс', 'Parallax')" :desc="tr('Картинка слегка следует за курсором.', 'The picture gently follows the cursor.')">
          <WSlider v-model="a.state.parallax.strength" :label="tr('Сила', 'Strength')" :min="4" :max="40" unit="px" :default-value="14" :disabled="!a.state.parallax.enabled" />
        </ModuleCard>

        <ModuleCard v-show="match('живой статус реакция reactive status')" id="bg-reactive" v-model:enabled="a.state.reactive" glyph="LIV" :title="tr('Живой статус', 'Live status')" :desc="tr('Без VPN фон тускнеет и теряет цвет, при подключении — оживает.', 'Background desaturates while disconnected and comes alive on connect.')" />
      </section>

      <!-- ── COLUMN 2: BUTTON & COLOR ── -->
      <section class="vs-col">
        <div class="vs-col-head"><span class="vs-col-tag">02</span>{{ tr('Кнопка и цвет', 'Button & color') }}</div>

        <ModuleCard v-show="match('кнопка картинка подключения узор button art pattern power')" id="power-art" :enabled="p.source !== 'none'" glyph="PWR" :title="tr('Картинка на кнопке', 'Button art')" :badge="p.source !== 'none' ? sourceLabel(p.source) : undefined" @update:enabled="togglePowerArt">
          <div class="pw-trio">
            <div v-for="st in powerStates" :key="st.id" class="pw-cell">
              <div class="pw-btn" :class="{ 'pw-btn--on': st.on, 'pw-btn--busy': st.busy }">
                <PowerArt :connected="st.on" :loading="st.busy" />
                <svg width="18" height="18" viewBox="0 0 24 24"><path d="M12 3v8M6.3 6.3a8 8 0 1 0 11.4 0" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" /></svg>
              </div>
              <span>{{ st.label }}</span>
            </div>
          </div>

          <div class="pw-sources">
            <button v-for="src in POWER_SOURCES" :key="src" class="pw-src" :class="{ 'pw-src--on': p.source === src }" :title="sourceLabel(src)" @click="chooseSource(src)">
              <span class="pw-src-art">
                <PowerArt v-if="src !== 'custom' || a.powerUrl" connected :override="{ ...p, source: src, opacityOn: 70, blur: 0, zoom: 1, edgeFade: false, spin: false, blend: 'normal' }" />
                <b v-else>+</b>
              </span>
              <span class="pw-src-name">{{ sourceLabel(src) }}</span>
            </button>
          </div>
          <template v-if="p.source === 'custom'">
            <button class="vs-act vs-act--wide" @click="pickPower">{{ a.powerUrl ? tr('Другая картинка', 'Another picture') : tr('Выбрать картинку', 'Pick a picture') }}</button>
            <button v-if="!a.powerVideoUrl" class="vs-act vs-act--wide" @click="pickPowerVideo">{{ tr('Загрузить видео', 'Upload a video') }}</button>
            <button v-else class="vs-act vs-act--wide sn-own-btn--drop" @click="dropPowerVideo">{{ tr('Убрать видео', 'Remove the video') }}</button>
          </template>

          <WSlider v-model="p.opacityOff" :label="tr('Видность — выкл', 'Opacity — off')" :max="60" unit="%" :default-value="30" />
          <WSlider v-model="p.opacityOn" :label="tr('Видность — вкл', 'Opacity — on')" :max="80" unit="%" :default-value="62" />
          <WSlider v-model="p.blur" :label="tr('Размытие', 'Blur')" :max="12" :step="0.5" unit="px" :default-value="1" />
          <WSlider v-model="p.zoom" :label="tr('Масштаб', 'Zoom')" :min="0.6" :max="2.5" :step="0.05" unit="×" :default-value="1" />
          <template v-if="p.source === 'custom'">
            <WSlider v-model="p.posX" :label="tr('Позиция X', 'Position X')" unit="%" :default-value="50" />
            <WSlider v-model="p.posY" :label="tr('Позиция Y', 'Position Y')" unit="%" :default-value="50" />
          </template>
          <WChips v-model="p.blend" :label="tr('Смешивание', 'Blend')" :options="blendOptions" />
          <div class="vs-switches">
            <WSwitch v-model="p.grayscaleOff" :label="tr('Ч/б, пока отключено', 'Grayscale while off')" />
            <WSwitch v-model="p.pulse" :label="tr('Дышит при подключении', 'Breathe while connecting')" />
            <WSwitch v-model="p.spin" :label="tr('Медленно вращается', 'Slow spin')" />
            <WSwitch v-model="p.edgeFade" :label="tr('Мягкие края', 'Soft edges')" />
          </div>
        </ModuleCard>

        <ModuleCard v-show="match('акцент цвет палитра accent color palette')" id="accent" glyph="CLR" :title="tr('Акцент', 'Accent')" :badge="accentLabel">
          <div class="ac-grid">
            <button v-for="c in ACCENTS" :key="c.id" class="ac" :class="{ 'ac--on': a.state.accent === c.id }" :style="{ '--c': c.hex }" :title="tr(c.ru, c.en)" @click="a.state.accent = c.id">
              <span class="ac-chip" />
              <span class="ac-name">{{ tr(c.ru, c.en) }}</span>
            </button>
            <label class="ac" :class="{ 'ac--on': a.state.accent === 'custom' }" :style="{ '--c': a.state.accentCustom }">
              <span class="ac-chip ac-chip--custom"><input type="color" :value="a.state.accentCustom" @input="onCustomColor" /></span>
              <span class="ac-name">{{ tr('Свой', 'Custom') }}</span>
            </label>
          </div>
          <WSwitch v-model="a.state.accentStatus" :label="tr('Статус «Подключено» тоже в акценте', 'Use accent for connected status')" />
        </ModuleCard>

        <ModuleCard v-show="match('стиль интерфейс стекло анимации скругление material glass motion radius')" id="ui-style" glyph="UI" :title="tr('Стиль интерфейса', 'Interface style')">
          <WChips :model-value="s.ui_style || 'wawity'" :label="tr('Тема', 'Theme')" :options="[{ value: 'wawity', label: 'Wawity' }, { value: 'material', label: 'Material' }]" @update:model-value="set('ui_style', $event)" />
          <WChips :model-value="s.motion_level || 'fancy'" :label="tr('Анимации', 'Motion')" :options="[{ value: 'simple', label: tr('Простые', 'Simple') }, { value: 'fancy', label: tr('Красивые', 'Fancy') }]" @update:model-value="set('motion_level', $event)" />
          <WSwitch :model-value="!!s.liquid_glass" :label="tr('Жидкое стекло', 'Liquid glass')" @update:model-value="set('liquid_glass', $event)" />
          <WSlider v-model="a.state.radius" :label="tr('Скругление', 'Corner radius')" :max="22" unit="px" :default-value="10" />
        </ModuleCard>
      </section>

      <!-- ── COLUMN 3: TYPE, SOUND, LAYOUT ── -->
      <section class="vs-col">
        <div class="vs-col-head"><span class="vs-col-tag">03</span>{{ tr('Шрифт, звук, макет', 'Type, sound, layout') }}</div>

        <ModuleCard v-show="match('шрифт шрифты пресет font fonts preset typography')" id="fonts" glyph="Aa" :title="tr('Пресет шрифтов', 'Font preset')" :badge="tr(fontPreset(a.state.font).ru, fontPreset(a.state.font).en)" :desc="tr('Каждый пресет меняет сразу все 4 шрифта приложения: текст, заголовки, метки и цифры.', 'Each preset swaps all 4 app fonts at once: body, headings, labels and numbers.')">
          <div class="ft-list">
            <button v-for="f in FONT_PRESETS" :key="f.id" class="ft" :class="{ 'ft--on': a.state.font === f.id }" @click="a.state.font = f.id">
              <span class="ft-top">
                <span class="ft-display" :style="{ fontFamily: f.display }">{{ tr(f.ru, f.en) }}</span>
                <span class="ft-label" :style="{ fontFamily: f.label }">{{ tr('ПОДКЛЮЧИТЬ', 'CONNECT') }}</span>
              </span>
              <span class="ft-bottom">
                <span class="ft-sans" :style="{ fontFamily: f.sans }">{{ tr(f.vibe.ru, f.vibe.en) }}</span>
                <span class="ft-mono" :style="{ fontFamily: f.mono }">42 ms · 10.0.0.1</span>
              </span>
            </button>
          </div>
        </ModuleCard>

        <ModuleCard v-show="match('звук звуки громкость sound sounds volume audio')" id="sound" v-model:enabled="a.state.sound.enabled" glyph="SND" :title="tr('Звуки', 'Sounds')" :badge="a.state.sound.enabled ? packLabel : undefined">
          <div class="sn-packs">
            <button v-for="pk in SOUND_PACKS" :key="pk.id" class="sn" :class="{ 'sn--on': a.state.sound.pack === pk.id }" @click="choosePack(pk.id)">
              <span class="sn-wave"><i v-for="n in 7" :key="n" :style="{ animationDelay: `${n * 0.08}s` }" /></span>
              <span class="sn-name">{{ tr(pk.ru, pk.en) }}</span>
              <span class="sn-hint">{{ tr(pk.hint.ru, pk.hint.en) }}</span>
            </button>
          </div>
          <WSlider v-model="a.state.sound.volume" :label="tr('Громкость', 'Volume')" unit="%" :default-value="45" />
          <div class="sn-test">
            <button v-for="ev in testEvents" :key="ev.id" class="vs-act" @click="previewSound(a.state.sound.pack, ev.id, a.state.sound.volume)">▶ {{ ev.label }}</button>
          </div>
          <div class="vs-switches">
            <WSwitch v-model="a.state.sound.connect" :label="tr('При подключении', 'On connect')" />
            <WSwitch v-model="a.state.sound.disconnect" :label="tr('При отключении', 'On disconnect')" />
            <WSwitch v-model="a.state.sound.error" :label="tr('При ошибке', 'On error')" />
            <WSwitch v-model="a.state.sound.clicks" :label="tr('Клики интерфейса', 'UI clicks')" />
          </div>

          <div class="sn-own">
            <div class="sn-own-head">
              <span class="sn-own-title" v-text="tr('Свои звуки', 'Own sounds')" />
              <span class="sn-own-limit" v-text="tr(`до ${MAX_SECONDS} с, длиннее — с затуханием`, `up to ${MAX_SECONDS}s, longer fades out`)" />
            </div>
            <p class="sn-own-note" v-text="tr('Свой звук на событие важнее пака: он играет вместо него.', 'A sound on an event overrides the pack.')" />
            <div v-for="ev in ownEvents" :key="ev.id" class="sn-own-row">
              <span class="sn-own-name" v-text="ev.label" />
              <span class="sn-own-state" v-if="ev.has" v-text="tr('свой', 'custom')" />
              <input
                type="file"
                accept="audio/*"
                class="sn-own-file"
                :id="`own-sound-${ev.id}`"
                @change="pickOwnSound(ev.id, $event)"
              />
              <label class="vs-act sn-own-btn" :for="`own-sound-${ev.id}`" v-text="tr('Загрузить', 'Upload')" />
              <button v-if="ev.has" class="vs-act sn-own-btn sn-own-btn--drop" @click="dropOwnSound(ev.id)" v-text="tr('Убрать', 'Remove')" />
            </div>
          </div>
        </ModuleCard>

        <ModuleCard v-show="match('макет сайдбар статистика сервера глобус layout sidebar stats servers globe')" id="layout" glyph="LAY" :title="tr('Макет', 'Layout')">
          <WChips :model-value="s.sidebar_style || 'basic'" :label="tr('Сайдбар', 'Sidebar')" :options="[{ value: 'basic', label: tr('Обычный', 'Basic') }, { value: 'compact', label: tr('Компакт', 'Compact') }, { value: 'extra', label: tr('Расширенный', 'Extra') }]" @update:model-value="set('sidebar_style', $event)" />
          <WChips :model-value="s.home_stats_style || 'basic'" :label="tr('Статистика на главной', 'Home stats')" :options="[{ value: 'basic', label: tr('Обычная', 'Basic') }, { value: 'pill', label: tr('Пилюли', 'Pills') }, { value: 'oversized', label: tr('Крупная', 'Oversized') }]" @update:model-value="set('home_stats_style', $event)" />
          <WChips :model-value="s.server_view || 'list'" :label="tr('Серверы', 'Servers')" :options="[{ value: 'list', label: tr('Список', 'List') }, { value: 'globe', label: tr('Глобус', 'Globe') }]" @update:model-value="set('server_view', $event)" />
          <WChips :model-value="s.connection_servers || 'off'" :label="tr('Серверы на главной', 'Servers on home')" :options="[{ value: 'off', label: tr('Нет', 'Off') }, { value: 'inline', label: tr('Под кнопкой', 'Inline') }, { value: 'sidebar', label: tr('Сбоку', 'Side') }]" @update:model-value="set('connection_servers', $event)" />
        </ModuleCard>
      </section>
    </div>
    <p class="vs-foot">{{ tr('ЛКМ по модулю — вкл/выкл · ПКМ или стрелка — настройки · двойной клик по ползунку — сброс', 'LMB on a module — on/off · RMB or caret — settings · double-click a slider — reset') }}</p>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { writeText } from '@tauri-apps/api/clipboard';
import { useVpnStore } from '../../stores/vpn';
import { useAppearance, IDB_BG, type PowerSource, type SoundPack } from '../../appearance/store';
import { tr } from '../../appearance/i18n';
import { ACCENTS, accentHex as hexOf } from '../../appearance/accents';
import { FONT_PRESETS, fontPreset, loadAllPresetFonts } from '../../appearance/fonts';
import { SOUND_PACKS, previewSound, type SoundEvent } from '../../appearance/sounds';
import { fileToImageBlob, pickImageFile } from '../../appearance/image';
import { checkVideo, pickVideoFile } from '../../appearance/media';
import { PATTERNS } from '../../appearance/patterns';
import {
  AUDIO_EVENTS,
  EVENT_LABEL,
  SOUND_MAX_SECONDS,
  clearCustom,
  hasCustom,
  setCustom,
  type AudioEvent,
} from '../../appearance/audioslots';
import { sharedAudioContext } from '../../appearance/sounds';
import { pushToast } from '../../composables/useNotifications';
import ModuleCard from './ModuleCard.vue';
import PowerArt from './PowerArt.vue';
import WSlider from '../ui/WSlider.vue';
import WSwitch from '../ui/WSwitch.vue';
import WChips from '../ui/WChips.vue';

const vpn = useVpnStore();
const a = useAppearance();
const s = computed(() => vpn.settings as any);
const p = a.state.power;

function set(key: string, value: unknown) {
  vpn.updateSettings({ [key]: value } as any);
}

// ── search ──
const q = ref('');
function match(keywords: string) {
  const needle = q.value.trim().toLowerCase();
  return !needle || keywords.toLowerCase().includes(needle);
}

onMounted(() => {
  loadAllPresetFonts();
  void a.hydrateImages();
});

// ── background ──
type BgMode = 'scenes' | 'image' | 'video' | 'flat';
const bgModes = computed(() => [
  { id: 'scenes' as BgMode, label: tr('Живые сцены', 'Live scenes') },
  { id: 'image' as BgMode, label: tr('Своя картинка', 'Own picture') },
  { id: 'video' as BgMode, label: tr('Своё видео', 'Own video') },
  { id: 'flat' as BgMode, label: tr('Плоский', 'Flat') },
]);
const bgMode = computed<BgMode>(() => {
  if (s.value.ui_style === 'material') return 'flat';
  if (!s.value.bg_custom_enabled) return 'scenes';
  // A stored clip outranks the still in the same slot, so that switching to
  // the video mode never lands you back on the picture panel.
  if (a.bgVideoUrl) return 'video';
  return 'image';
});
const bgModeLabel = computed(() => bgModes.value.find((m) => m.id === bgMode.value)?.label);
const bgSrc = computed(() => (s.value.bg_custom_url === IDB_BG ? a.bgUrl : s.value.bg_custom_url || ''));
const urlDraft = ref(s.value.bg_custom_url && s.value.bg_custom_url !== IDB_BG ? s.value.bg_custom_url : '');
const imgError = ref('');

function setBgMode(m: BgMode) {
  if (m === 'flat') return set('ui_style', 'material');
  if (s.value.ui_style === 'material') set('ui_style', 'wawity');
  // Leaving the video mode without a clip would leave a stale panel behind,
  // so a clip is dropped as soon as the user asks for something else.
  if (m !== 'video' && a.bgVideoUrl) void a.setVideo('bg', null);
  set('bg_custom_enabled', m === 'image' || m === 'video');
  if (m === 'image' || m === 'video') {
    a.state.openModules = { ...a.state.openModules, 'bg-tune': true };
  }
}
async function pickBg() {
  imgError.value = '';
  const file = await pickImageFile();
  if (!file) return;
  try {
    const blob = await fileToImageBlob(file, 2560);
    await a.setImage('bg', blob);
    vpn.updateSettings({ bg_custom_url: IDB_BG, bg_custom_enabled: true } as any);
    urlDraft.value = '';
  } catch {
    imgError.value = tr('Не получилось прочитать картинку', 'Could not read the picture');
  }
}
function applyUrl() {
  const u = urlDraft.value.trim();
  if (!/^(https?:|data:image\/|asset:|blob:)/i.test(u)) {
    imgError.value = tr('Нужна ссылка http(s)://', 'Need an http(s):// link');
    return;
  }
  imgError.value = '';
  vpn.updateSettings({ bg_custom_url: u, bg_custom_enabled: true } as any);
}
async function clearBg() {
  if (s.value.bg_custom_url === IDB_BG) await a.setImage('bg', null);
  vpn.updateSettings({ bg_custom_url: '', bg_custom_enabled: false } as any);
  urlDraft.value = '';
}

// ── video slots ──
// Background and button art both accept a clip. They live in their own
// IndexedDB store so uploading one never overwrites the still in the same slot.
const vidError = ref('');
async function pickBgVideo() {
  vidError.value = '';
  const file = await pickVideoFile();
  if (!file) return;
  const bad = checkVideo(file);
  if (bad === 'not-a-video') {
    vidError.value = tr('Это не видео', 'That is not a video');
    return;
  }
  if (bad === 'too-big') {
    vidError.value = tr('Файл больше 220 МБ', 'File is over 220 MB');
    return;
  }
  await a.setVideo('bg', file);
  vpn.updateSettings({ bg_custom_enabled: true } as any);
}
async function dropBgVideo() {
  await a.setVideo('bg', null);
  vpn.updateSettings({ bg_custom_enabled: false } as any);
}

async function pickPowerVideo() {
  const file = await pickVideoFile();
  if (!file) return;
  const bad = checkVideo(file);
  if (bad) {
    pushToast(
      'error',
      tr('Не удалось загрузить видео', 'Could not load the video'),
      bad === 'too-big'
        ? tr('Файл больше 220 МБ', 'File is over 220 MB')
        : tr('Это не видео', 'That is not a video'),
    );
    return;
  }
  await a.setVideo('power', file);
  p.source = 'custom';
  lastSource = 'custom';
}
async function dropPowerVideo() {
  await a.setVideo('power', null);
}
const pvBgStyle = computed(() => {
  if (bgMode.value !== 'image' || !bgSrc.value) return undefined;
  return {
    backgroundImage: `url('${bgSrc.value}')`,
    backgroundPosition: `${s.value.bg_custom_pos_x ?? 50}% ${s.value.bg_custom_pos_y ?? 50}%`,
    transform: `scale(${s.value.bg_custom_zoom ?? 1})`,
    filter: `blur(${(s.value.bg_custom_blur ?? 0) / 3}px)`,
  };
});
const previewOn = ref(false);

// ── power art ──
// PATTERNS is already trimmed to the files that exist, at build time. There is
// nothing to probe and nothing to wait for: the list is final on first render.
const POWER_SOURCES = computed<PowerSource[]>(() => ['custom', ...PATTERNS.map((p) => p.id)]);
let lastSource: PowerSource = 'custom';
function sourceLabel(src: PowerSource) {
  if (src === 'custom') return tr('Своя', 'Own');
  if (src === 'none') return tr('Нет', 'None');
  const def = PATTERNS.find((x) => x.id === src);
  return def ? tr(def.ru, def.en) : src;
}
function togglePowerArt(on: boolean) {
  if (on) {
    p.source = lastSource === 'custom' && !a.powerUrl ? 'none' : lastSource;
  } else {
    lastSource = p.source;
    p.source = 'none';
  }
}
async function chooseSource(src: PowerSource) {
  if (src === 'custom' && !a.powerUrl) {
    const ok = await pickPower();
    if (!ok) return;
  }
  p.source = src;
  lastSource = src;
}
async function pickPower(): Promise<boolean> {
  const file = await pickImageFile();
  if (!file) return false;
  try {
    await a.setImage('power', await fileToImageBlob(file, 512));
    p.source = 'custom';
    return true;
  } catch {
    return false;
  }
}
const powerStates = computed(() => [
  { id: 'off', label: tr('Выкл', 'Off'), on: false, busy: false },
  { id: 'busy', label: tr('Соединение', 'Connecting'), on: false, busy: true },
  { id: 'on', label: tr('Вкл', 'On'), on: true, busy: false },
]);
const blendOptions = computed(() => [
  { value: 'screen', label: tr('Свет', 'Screen') },
  { value: 'normal', label: tr('Обычно', 'Normal') },
  { value: 'overlay', label: tr('Наложение', 'Overlay') },
  { value: 'luminosity', label: tr('Яркость', 'Luma') },
]);

// ── accent ──
const accentHex = (id: string) => hexOf(id, a.state.accentCustom);
const accentLabel = computed(() => {
  if (a.state.accent === 'custom') return a.state.accentCustom.toUpperCase();
  const c = ACCENTS.find((x) => x.id === a.state.accent);
  return c ? tr(c.ru, c.en) : '';
});
let colorTimer: number | undefined;
function onCustomColor(e: Event) {
  const v = (e.target as HTMLInputElement).value;
  window.clearTimeout(colorTimer);
  colorTimer = window.setTimeout(() => {
    a.state.accentCustom = v;
    a.state.accent = 'custom';
  }, 30);
}

// ── sound ──
const packLabel = computed(() => {
  const pk = SOUND_PACKS.find((x) => x.id === a.state.sound.pack);
  return pk ? tr(pk.ru, pk.en) : '';
});

// ── own sounds, one slot per event ──
const MAX_SECONDS = SOUND_MAX_SECONDS;
const ownRev = ref(0);
const ownEvents = computed(() => {
  // hasCustom reads a plain Map, so it is not reactive on its own. Reading
  // the revision here is what makes the badge and Remove button appear.
  void ownRev.value;
  return AUDIO_EVENTS.map((id) => ({
    id,
    label: tr(EVENT_LABEL[id][0], EVENT_LABEL[id][1]),
    has: hasCustom(id),
  }));
});

/** Share the player's context: a buffer decoded elsewhere carries that
 *  context's sample rate, and mixing rates is a resample nobody asked for. */
function soundCtx(): AudioContext | null {
  return sharedAudioContext();
}

async function pickOwnSound(ev: AudioEvent, e: Event) {
  const input = e.target as HTMLInputElement;
  const file = input.files?.[0];
  input.value = '';
  if (!file) return;
  const ctx = soundCtx();
  if (!ctx) return;
  try {
    const { trimmed } = await setCustom(ev, file, ctx);
    ownRev.value++;
    pushToast(
      'success',
      tr('Звук загружен', 'Sound loaded'),
      trimmed
        ? tr(`Длиннее ${MAX_SECONDS} с — оставлено начало с затуханием`, `Trimmed to ${MAX_SECONDS}s with a fade`)
        : tr('Готово', 'Ready'),
    );
    previewSound(a.state.sound.pack, ev, a.state.sound.volume);
  } catch {
    pushToast('error', tr('Не удалось загрузить', 'Could not load'), tr('Файл не декодируется', 'File could not be decoded'));
  }
}

async function dropOwnSound(ev: AudioEvent) {
  await clearCustom(ev).catch(() => {});
  ownRev.value++;
}
function choosePack(id: SoundPack) {
  a.state.sound.pack = id;
  previewSound(id, 'connect', a.state.sound.volume);
}
const testEvents = computed<Array<{ id: SoundEvent; label: string }>>(() => [
  { id: 'connect', label: tr('Подкл.', 'Connect') },
  { id: 'disconnect', label: tr('Откл.', 'Disconnect') },
  { id: 'error', label: tr('Ошибка', 'Error') },
  { id: 'click', label: tr('Клик', 'Click') },
]);

// ── ready-made looks (accent + fonts + sound + film at once) ──
interface Theme {
  id: string;
  ru: string;
  en: string;
  accent: string;
  font: string;
  pack: SoundPack;
  fx: { enabled: boolean; grain: number; vignette: number; tint: number };
  power: PowerSource;
}
const THEMES: Theme[] = [
  { id: 'ember', ru: 'Wawity', en: 'Wawity', accent: 'ember', font: 'wawity', pack: 'orbit', fx: { enabled: false, grain: 18, vignette: 35, tint: 0 }, power: 'none' },
  { id: 'nebula', ru: 'Туманность', en: 'Nebula', accent: 'nebula', font: 'grotesk', pack: 'crystal', fx: { enabled: true, grain: 10, vignette: 45, tint: 18 }, power: 'none' },
  { id: 'terminal', ru: 'Терминал', en: 'Terminal', accent: 'aurora', font: 'terminal', pack: 'pulsar', fx: { enabled: true, grain: 25, vignette: 50, tint: 8 }, power: 'none' },
  { id: 'arcade', ru: 'Аркада', en: 'Arcade', accent: 'sakura', font: 'arcade', pack: 'retro', fx: { enabled: true, grain: 35, vignette: 30, tint: 12 }, power: 'none' },
  { id: 'cyber', ru: 'Киберпанк', en: 'Cyberpunk', accent: 'solar', font: 'cyber', pack: 'pulsar', fx: { enabled: true, grain: 15, vignette: 40, tint: 20 }, power: 'none' },
  { id: 'ice', ru: 'Лёд', en: 'Ice', accent: 'ice', font: 'soft', pack: 'crystal', fx: { enabled: true, grain: 6, vignette: 25, tint: 10 }, power: 'none' },
  { id: 'noir', ru: 'Нуар', en: 'Noir', accent: 'mono', font: 'editorial', pack: 'void', fx: { enabled: true, grain: 40, vignette: 65, tint: 0 }, power: 'none' },
];
function applyTheme(t: Theme) {
  a.state.accent = t.accent;
  a.state.font = t.font;
  a.state.sound.pack = t.pack;
  Object.assign(a.state.fx, t.fx);
  // A preset no longer chooses the button art: the patterns are pictures the
  // user supplies, so there is nothing for a built-in theme to select. Clearing
  // it would throw away their picture, so only a non-custom source is reset.
  if (p.source !== 'custom') p.source = t.power;
}
function themeActive(t: Theme) {
  return a.state.accent === t.accent && a.state.font === t.font && a.state.sound.pack === t.pack;
}

// ── export / import / reset ──
const copied = ref(false);
const importOpen = ref(false);
const importCode = ref('');
const importError = ref('');
const confirmReset = ref(false);
const BG_KEYS = ['bg_custom_dim', 'bg_custom_blur', 'bg_custom_zoom', 'bg_custom_pos_x', 'bg_custom_pos_y', 'ui_style', 'liquid_glass', 'motion_level', 'sidebar_style', 'home_stats_style'];

async function exportTheme() {
  const { openModules, ...look } = JSON.parse(JSON.stringify(a.state));
  void openModules;
  const settings: Record<string, unknown> = {};
  for (const k of BG_KEYS) settings[k] = s.value[k];
  if (/^https?:/i.test(s.value.bg_custom_url || '')) {
    settings.bg_custom_url = s.value.bg_custom_url;
    settings.bg_custom_enabled = s.value.bg_custom_enabled;
  }
  const code = 'wawity-theme:' + btoa(unescape(encodeURIComponent(JSON.stringify({ v: 1, a: look, s: settings }))));
  try {
    await writeText(code);
  } catch {
    await navigator.clipboard?.writeText(code).catch(() => {});
  }
  copied.value = true;
  setTimeout(() => (copied.value = false), 1600);
}
function importTheme() {
  importError.value = '';
  try {
    const raw = importCode.value.trim().replace(/^wawity-theme:/, '');
    const data = JSON.parse(decodeURIComponent(escape(atob(raw))));
    if (data?.v !== 1) throw new Error('version');
    a.patch(data.a);
    const patch: Record<string, unknown> = {};
    for (const k of [...BG_KEYS, 'bg_custom_url', 'bg_custom_enabled']) if (data.s?.[k] !== undefined) patch[k] = data.s[k];
    vpn.updateSettings(patch as any);
    importOpen.value = false;
    importCode.value = '';
  } catch {
    importError.value = tr('Неверный код темы', 'Invalid theme code');
  }
}
function resetAll() {
  if (!confirmReset.value) {
    confirmReset.value = true;
    setTimeout(() => (confirmReset.value = false), 2500);
    return;
  }
  confirmReset.value = false;
  a.reset();
  vpn.updateSettings({ bg_custom_dim: 45, bg_custom_blur: 0, bg_custom_zoom: 1, bg_custom_pos_x: 50, bg_custom_pos_y: 50 } as any);
}

</script>

<style scoped>
.vs {
  --vs-w: min(1180px, calc(100vw - 290px));
  width: var(--vs-w);
  margin-left: calc((100% - var(--vs-w)) / 2);
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding-bottom: 24px;
}
@media (max-width: 860px) {
  .vs { --vs-w: 100%; }
}

/* top bar */
.vs-top { display: flex; align-items: flex-end; gap: 14px; flex-wrap: wrap; }
.vs-title { display: flex; flex-direction: column; gap: 2px; margin-right: auto; }
.vs-kicker { font-family: var(--font-label); font-size: 9.5px; letter-spacing: 0.22em; color: var(--ember); }
.vs-title h2 { margin: 0; font-family: var(--font-display); font-size: 24px; font-weight: 700; color: var(--paper); letter-spacing: -0.02em; }
.vs-search {
  display: flex; align-items: center; gap: 7px; height: 32px; padding: 0 10px; width: 220px;
  border: 1px solid var(--border); border-radius: 6px; background: oklch(1 0 0 / 3%); color: var(--muted-foreground);
}
.vs-search:focus-within { border-color: color-mix(in oklch, var(--ember) 60%, transparent); }
.vs-search input { flex: 1; min-width: 0; background: none; border: 0; outline: none; color: var(--foreground); font-size: 12px; }
.vs-actions { display: flex; gap: 6px; }
.vs-act {
  height: 30px; padding: 0 11px; font-size: 11.5px; color: var(--foreground);
  background: oklch(1 0 0 / 4%); border: 1px solid var(--border); border-radius: 6px; cursor: pointer;
  transition: border-color 0.15s, background 0.15s;
}
.vs-act:hover { border-color: oklch(1 0 0 / 25%); background: oklch(1 0 0 / 7%); }
.vs-act--danger:hover { border-color: var(--ember); color: var(--ember-soft); }
.vs-act--wide { width: 100%; }
.vs-import { display: flex; gap: 6px; align-items: center; }
.vs-import input, .vs-url input {
  flex: 1; min-width: 0; height: 30px; padding: 0 10px; font-family: var(--font-mono); font-size: 11px;
  color: var(--foreground); background: oklch(1 0 0 / 3%); border: 1px solid var(--border); border-radius: 6px; outline: none;
}
.vs-err { font-size: 11px; color: var(--ember-soft); }

/* looks */
.vs-themes { display: flex; align-items: center; gap: 6px; flex-wrap: wrap; }
.vs-themes-label { font-family: var(--font-label); font-size: 9.5px; letter-spacing: 0.18em; text-transform: uppercase; color: var(--muted-foreground); margin-right: 4px; }
.vs-theme {
  display: flex; align-items: center; gap: 7px; height: 28px; padding: 0 11px 0 9px; font-size: 11.5px;
  color: var(--muted-foreground); background: oklch(1 0 0 / 3%); border: 1px solid var(--border); border-radius: 999px; cursor: pointer;
  transition: color 0.15s, border-color 0.15s;
}
.vs-theme:hover { color: var(--foreground); }
.vs-theme-dot { width: 8px; height: 8px; border-radius: 50%; background: var(--t); box-shadow: 0 0 8px var(--t); }
.vs-theme--on { color: var(--paper); border-color: color-mix(in srgb, var(--t) 60%, transparent); }

/* the ClickGUI grid: 3 panels across, modules flow down */
.vs-grid { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 12px; align-items: start; }
@media (max-width: 1080px) { .vs-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); } }
@media (max-width: 680px) { .vs-grid { grid-template-columns: 1fr; } }
.vs-col {
  position: relative; display: flex; flex-direction: column; overflow: hidden;
  background: rgba(12, 11, 11, 0.62); backdrop-filter: blur(14px); -webkit-backdrop-filter: blur(14px);
  border: 1px solid var(--border); border-radius: var(--radius);
}
.vs-col::before { content: ''; position: absolute; left: 0; right: 0; top: 0; height: 2px; background: linear-gradient(90deg, var(--ember), transparent 85%); }
.vs-col-head {
  display: flex; align-items: center; gap: 9px; height: 40px; padding: 0 14px;
  font-family: var(--font-label); font-size: 10.5px; letter-spacing: 0.14em; text-transform: uppercase; color: var(--paper);
  border-bottom: 1px solid var(--border); background: oklch(1 0 0 / 2.5%);
}
.vs-col-tag { font-family: var(--font-mono); font-size: 9.5px; color: var(--ember); letter-spacing: 0; }
.vs-foot { margin: 2px 0 0; text-align: center; font-size: 10.5px; color: var(--muted-foreground); opacity: 0.7; }

/* background modes */
.vs-modes { display: grid; grid-template-columns: repeat(3, 1fr); gap: 6px; }
.vs-mode { display: flex; flex-direction: column; gap: 5px; padding: 0; background: none; border: 0; cursor: pointer; color: var(--muted-foreground); font-size: 10.5px; }
.vs-mode-art {
  height: 44px; border-radius: 6px; border: 1px solid var(--border); background-size: cover; background-position: center;
  transition: border-color 0.15s, box-shadow 0.15s;
}
.vs-mode-art--scenes { background: radial-gradient(circle at 50% 60%, #000 18%, color-mix(in oklch, var(--ember) 55%, transparent) 22%, transparent 36%), radial-gradient(ellipse at 50% 60%, #1a0d0a, #050505 70%); }
.vs-mode-art--image { background-color: #111; background-image: linear-gradient(135deg, #222 25%, transparent 25%, transparent 50%, #222 50%, #222 75%, transparent 75%); background-size: 10px 10px; }
.vs-mode-art--flat { background: linear-gradient(#1c1b1f, #141317); }
.vs-mode--on { color: var(--paper); }
.vs-mode--on .vs-mode-art { border-color: var(--ember); box-shadow: 0 0 0 1px var(--ember), 0 0 14px color-mix(in oklch, var(--ember) 40%, transparent); }

/* live preview */
.pv { position: relative; height: 128px; border-radius: 8px; overflow: hidden; border: 1px solid var(--border); isolation: isolate; }
.pv-bg { position: absolute; inset: -6px; background-size: cover; transition: filter 0.3s, transform 0.3s; }
.pv-bg--scene { background: radial-gradient(circle at 60% 55%, #000 10%, color-mix(in oklch, var(--ember) 60%, transparent) 13%, transparent 26%), radial-gradient(ellipse at 60% 55%, #22100b, #050505 70%); }
.pv-bg--flat { background: linear-gradient(#1c1b1f, #141317); }
.pv-dim { position: absolute; inset: 0; background: #090808; }
.pv-fx > span { position: absolute; inset: 0; }
.pv-tint { background: var(--ember); mix-blend-mode: soft-light; }
.pv-vig { background: radial-gradient(ellipse, transparent 38%, #000 100%); }
.pv-grain { background-image: url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='120' height='120'><filter id='n'><feTurbulence type='fractalNoise' baseFrequency='.9' numOctaves='2'/><feColorMatrix values='0 0 0 0 1  0 0 0 0 1  0 0 0 0 1  0 0 0 .55 0'/></filter><rect width='100%' height='100%' filter='url(%23n)'/></svg>"); mix-blend-mode: overlay; }
.pv-side { position: absolute; left: 6px; top: 6px; bottom: 6px; width: 30px; display: flex; flex-direction: column; gap: 5px; padding: 6px 5px; border-radius: 5px; background: rgba(255, 255, 255, 0.05); border: 1px solid rgba(255, 255, 255, 0.07); }
.pv-side i { height: 4px; border-radius: 2px; background: rgba(255, 255, 255, 0.18); }
.pv-side i:first-child { background: var(--ember); }
.pv-btn, .pw-btn {
  position: absolute; left: 58%; top: 50%; translate: -50% -50%; width: 56px; height: 56px; border-radius: 50%;
  display: grid; place-items: center; overflow: hidden; cursor: pointer; color: var(--paper);
  background: radial-gradient(circle at 35% 30%, #2a2828, #121111); border: 1px solid rgba(255, 255, 255, 0.12);
  transition: box-shadow 0.4s, border-color 0.4s;
}
.pv-btn > svg, .pw-btn > svg { position: relative; z-index: 1; }
.pv-btn--on, .pw-btn--on { border-color: var(--success); box-shadow: 0 0 22px color-mix(in oklch, var(--success) 45%, transparent); }
.pw-btn--busy { border-color: var(--ember); }
.pv-cap { position: absolute; right: 8px; bottom: 6px; font-family: var(--font-mono); font-size: 9px; color: rgba(255, 255, 255, 0.45); }

.vs-drop {
  display: flex; flex-direction: column; align-items: center; gap: 3px; padding: 12px; cursor: pointer; text-align: center;
  font-size: 11.5px; color: var(--foreground); border: 1px dashed oklch(1 0 0 / 20%); border-radius: 8px; transition: border-color 0.15s, background 0.15s;
}
.vs-drop:hover { border-color: var(--ember); background: color-mix(in oklch, var(--ember) 7%, transparent); }
.vs-drop small { font-family: var(--font-mono); font-size: 9px; color: var(--muted-foreground); }
.vs-drop-plus { font-size: 18px; line-height: 1; color: var(--ember); }
.vs-url { display: flex; gap: 5px; }
.vs-switches { display: flex; flex-direction: column; }

/* power art */
.pw-trio { display: grid; grid-template-columns: repeat(3, 1fr); gap: 6px; }
.pw-cell { display: flex; flex-direction: column; align-items: center; gap: 6px; font-size: 10px; color: var(--muted-foreground); }
.pw-cell .pw-btn { position: relative; left: auto; top: auto; translate: none; cursor: default; width: 58px; height: 58px; }
.pw-sources { display: grid; grid-template-columns: repeat(4, 1fr); gap: 6px; }
.pw-src { display: flex; flex-direction: column; align-items: center; gap: 4px; padding: 0; background: none; border: 0; cursor: pointer; font-size: 10px; color: var(--muted-foreground); }
.pw-src-art {
  position: relative; width: 38px; height: 38px; border-radius: 50%; overflow: hidden; display: grid; place-items: center;
  background: #141313; border: 1px solid var(--border); transition: border-color 0.15s, box-shadow 0.15s;
}
.pw-src-art b { color: var(--ember); font-weight: 400; font-size: 18px; }
.pw-src--on { color: var(--paper); }
.pw-src--on .pw-src-art { border-color: var(--ember); box-shadow: 0 0 10px color-mix(in oklch, var(--ember) 45%, transparent); }

/* accent */
.ac-grid { display: grid; grid-template-columns: repeat(5, 1fr); gap: 8px 4px; }
.ac { display: flex; flex-direction: column; align-items: center; gap: 4px; padding: 0; background: none; border: 0; cursor: pointer; font-size: 9.5px; color: var(--muted-foreground); }
.ac-chip { position: relative; width: 26px; height: 26px; border-radius: 7px; background: var(--c); box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.15); transition: transform 0.15s, box-shadow 0.2s; }
.ac:hover .ac-chip { transform: translateY(-1px); }
.ac--on { color: var(--paper); }
.ac--on .ac-chip { box-shadow: 0 0 0 2px #0b0b0b, 0 0 0 3px var(--c), 0 0 14px var(--c); }
.ac-chip--custom { background: conic-gradient(#f43, #fb2, #3e9, #3cf, #95f, #f5a, #f43); overflow: hidden; }
.ac-chip--custom input { position: absolute; inset: 0; opacity: 0; cursor: pointer; width: 100%; height: 100%; }
.ac-name { max-width: 100%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

/* fonts */
.ft-list { display: flex; flex-direction: column; gap: 5px; max-height: 380px; overflow-y: auto; padding-right: 2px; }
.ft {
  display: flex; flex-direction: column; gap: 4px; padding: 9px 10px; text-align: left; cursor: pointer; color: var(--foreground);
  background: oklch(1 0 0 / 2.5%); border: 1px solid var(--border); border-radius: 7px; transition: border-color 0.15s, background 0.15s;
}
.ft:hover { border-color: oklch(1 0 0 / 22%); }
.ft--on { border-color: color-mix(in oklch, var(--ember) 70%, transparent); background: color-mix(in oklch, var(--ember) 9%, transparent); }
.ft-top, .ft-bottom { display: flex; align-items: baseline; justify-content: space-between; gap: 8px; }
.ft-display { font-size: 15px; font-weight: 700; color: var(--paper); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.ft-label { font-size: 8.5px; letter-spacing: 0.14em; color: var(--ember-soft); white-space: nowrap; }
.ft-sans { font-size: 11px; color: var(--muted-foreground); }
.ft-mono { font-size: 10px; color: var(--muted-foreground); white-space: nowrap; }

/* sound */
.sn-packs { display: grid; grid-template-columns: repeat(2, 1fr); gap: 5px; }
.sn {
  display: grid; grid-template-columns: auto 1fr; grid-template-rows: auto auto; column-gap: 8px; align-items: center;
  padding: 7px 9px; text-align: left; cursor: pointer; background: oklch(1 0 0 / 2.5%); border: 1px solid var(--border); border-radius: 7px;
}
.sn:hover { border-color: oklch(1 0 0 / 22%); }
.sn--on { border-color: color-mix(in oklch, var(--ember) 70%, transparent); background: color-mix(in oklch, var(--ember) 9%, transparent); }
.sn-wave { grid-row: span 2; display: flex; align-items: center; gap: 1.5px; height: 18px; }
.sn-wave i { width: 2px; height: 30%; border-radius: 1px; background: oklch(1 0 0 / 35%); }
.sn--on .sn-wave i { background: var(--ember); animation: sn-eq 0.9s ease-in-out infinite alternate; }
@keyframes sn-eq { from { height: 20%; } to { height: 100%; } }
.sn-name { font-size: 11.5px; color: var(--paper); }
.sn-hint { font-size: 9.5px; color: var(--muted-foreground); }
.sn-test { display: flex; flex-wrap: wrap; gap: 4px; }
.sn-test .vs-act { height: 26px; padding: 0 8px; font-size: 10.5px; }

.sn-own {
  display: flex;
  flex-direction: column;
  gap: 9px;
  margin-top: 4px;
  padding-top: 14px;
  border-top: 1px solid var(--border);
}
.sn-own-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 10px;
  flex-wrap: wrap;
}
.sn-own-title { font-family: var(--font-display); font-size: 14px; letter-spacing: -.015em; }
.sn-own-limit { font-size: 11px; color: var(--muted-foreground); }
.sn-own-note { margin: 0; font-size: 11.5px; line-height: 1.5; color: var(--muted-foreground); }
.sn-own-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 9px 0;
  border-bottom: 1px solid var(--border);
}
.sn-own-row:last-child { border-bottom: 0; }
.sn-own-name { flex: 1; min-width: 0; font-size: 12.5px; color: var(--foreground); }
.sn-own-state {
  font-family: var(--font-label);
  font-size: 8px;
  letter-spacing: .12em;
  text-transform: uppercase;
  color: var(--ember);
  flex-shrink: 0;
}
.sn-own-file { display: none; }
.sn-own-btn { flex-shrink: 0; }
.sn-own-btn--drop { color: var(--muted-foreground); }
</style>
