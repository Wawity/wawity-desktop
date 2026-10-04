<template>
  <LookModal
    :open="open"
    :icon="Settings"
    :title="tr('Настройки главной', 'Home settings')"
    :subtitle="tr('Фон, кнопка подключения и звуки — всё в одном месте', 'Background, connect button and sounds in one place')"
    size="md"
    @close="emit('close')"
  >
    <template #tabs>
      <div ref="tabsRef" class="lk-tabs" role="tablist" :aria-label="tr('Настройки главной', 'Home settings')" @keydown="onTabKey">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          type="button"
          role="tab"
          :id="`${panelId}-${tab.id}-tab`"
          :aria-controls="`${panelId}-${tab.id}`"
          :tabindex="activeTab === tab.id ? 0 : -1"
          :aria-selected="activeTab === tab.id"
          :class="['lk-tab', { 'lk-tab--on': activeTab === tab.id }]"
          @click="activeTab = tab.id"
        >
          <component :is="tab.icon" :size="14" aria-hidden="true" />
          <span v-text="tab.label" />
        </button>
      </div>
    </template>

    <div v-if="activeTab === 'bg'" :id="`${panelId}-bg`" :aria-labelledby="`${panelId}-bg-tab`" role="tabpanel" tabindex="0">
      <p v-if="isMaterial" class="lk-note" v-text="tr('В стиле Material свой фон скрыт. Переключи стиль на Wawity, чтобы его увидеть.', 'Material style hides the custom background. Switch to Wawity to see it.')" />

      <LookRow :icon="ImageIcon" :title="t('settings.bgCustom')" :desc="t('settings.bgCustomDesc')">
        <LookSwitch v-model="bgEnabled" :label="t('settings.bgCustom')" />
      </LookRow>


        <div v-if="bgSrc" class="cl-preview">
          <img
            :src="bgSrc"
            alt=""
            draggable="false"
            :style="previewImgStyle"
          />
          <span class="cl-preview-dim" :style="{ opacity: String(bgDim / 100) }" />
        </div>

        <LookRow :icon="Link2" :title="tr('Картинка', 'Picture')" :desc="tr('Ссылка http(s) или файл с компьютера', 'An http(s) link or a file from your computer')" stack>
          <div class="lk-url">
            <input
              v-model="bgUrlDraft"
              :class="['lk-input', { 'lk-input--bad': bgUrlDraft.trim() !== '' && !bgUrlValid }]"
              placeholder="https://…/wallpaper.jpg"
              spellcheck="false"
              :aria-label="t('settings.bgUrl')"
              @keydown.enter.prevent="commitBgUrl"
            />
            <button type="button" class="lk-pill" @click="pasteBgUrl" v-text="tr('Вставить', 'Paste')" />
            <button type="button" class="lk-pill lk-pill--on" :disabled="!bgUrlValid || bgImporting" @click="commitBgUrl" v-text="tr('Применить', 'Apply')" />
          </div>
          <div class="lk-pills">
            <button type="button" class="lk-pill" :disabled="bgImporting" @click="pickBgFile">
              <Upload :size="13"  aria-hidden="true" />
              <span v-text="bgImporting ? tr('Загружаю…', 'Loading…') : tr('Файл с ПК', 'File from PC')" />
            </button>
            <button type="button" class="lk-pill" :disabled="!bgSrc || bgImporting" @click="openFocus">
              <Crosshair :size="13" aria-hidden="true" />
              <span v-text="t('settings.bgFocusPick')" />
            </button>
            <button type="button" class="lk-pill" @click="resetFocus">
              <RotateCcw :size="13" aria-hidden="true" />
              <span v-text="t('settings.bgFocusReset')" />
            </button>
            <button type="button" class="lk-pill lk-pill--danger" :disabled="!bgUrl || bgImporting" @click="clearBgImage">
              <X :size="13" aria-hidden="true" />
              <span v-text="tr('Убрать', 'Remove')" />
            </button>
          </div>
        </LookRow>

        <LookRow :icon="Moon" :title="tr('Затемнение', 'Dim')" :desc="tr('Чтобы текст не терялся на яркой картинке', 'Keeps text readable on bright pictures')">
          <LookSlider v-model="bgDim" :min="0" :max="90" :step="5" unit="%" :default-value="45" :label="tr('Затемнение', 'Dim')" />
        </LookRow>
        <LookRow :icon="Droplets" :title="tr('Размытие', 'Blur')">
          <LookSlider v-model="bgBlur" :min="0" :max="40" :step="1" unit="px" :default-value="0" :label="tr('Размытие', 'Blur')" />
        </LookRow>
        <LookRow :icon="ZoomIn" :title="tr('Масштаб', 'Zoom')">
          <LookSlider v-model="bgZoom" :min="100" :max="250" :step="5" unit="%" :default-value="100" :label="tr('Масштаб', 'Zoom')" />
        </LookRow>
        <LookRow :title="tr('Позиция X', 'Position X')">
          <LookSlider v-model="bgPosX" unit="%" :default-value="50" :label="tr('Позиция X', 'Position X')" />
        </LookRow>
        <LookRow :title="tr('Позиция Y', 'Position Y')">
          <LookSlider v-model="bgPosY" unit="%" :default-value="50" :label="tr('Позиция Y', 'Position Y')" />
        </LookRow>

    </div>

    <div v-else-if="activeTab === 'button'" :id="`${panelId}-button`" :aria-labelledby="`${panelId}-button-tab`" role="tabpanel" tabindex="0">
      <div class="cl-power-preview" :aria-label="tr('Превью кнопки', 'Button preview')">
        <div v-for="st in powerStates" :key="st.id" class="cl-power-cell">
          <span :class="['cl-power-button', { 'cl-power-button--on': st.on, 'cl-power-button--busy': st.busy }]">
            <PowerArt :connected="st.on" :loading="st.busy" />
            <Power :size="24" aria-hidden="true" />
          </span>
          <span class="cl-power-label">{{ st.label }}</span>
        </div>
      </div>
      <LookRow :icon="Sparkles" :title="tr('Картинка на кнопке', 'Button art')" :desc="tr('Узор или своя картинка внутри круглой кнопки', 'A pattern or your own picture inside the round button')">
        <LookSwitch :model-value="powerOn" :label="tr('Картинка на кнопке', 'Button art')" @update:model-value="togglePowerArt" />
      </LookRow>


        <LookRow :title="tr('Узор', 'Pattern')" stack>
          <div class="lk-pills">
            <button
              v-for="src in POWER_SOURCES"
              :key="src.id"
              type="button"
              :class="['lk-pill', { 'lk-pill--on': power.source === src.id }]"
              :disabled="src.id === 'custom' && !look.powerUrl"
              :title="src.id === 'custom' && !look.powerUrl ? tr('Сначала загрузи картинку', 'Upload a picture first') : undefined"
              @click="power.source = src.id"
              v-text="tr(src.ru, src.en)"
            />
          </div>
          <div class="lk-pills">
            <button type="button" class="lk-pill" :disabled="powerBusy" @click="uploadPowerArt">
              <Upload :size="13"  aria-hidden="true" />
              <span v-text="look.powerUrl ? tr('Другая картинка', 'Another picture') : tr('Своя картинка', 'Own picture')" />
            </button>
            <button v-if="look.powerUrl" type="button" class="lk-pill lk-pill--danger" :disabled="powerBusy" @click="removePowerArt">
              <X :size="13" aria-hidden="true" />
              <span v-text="tr('Удалить свою', 'Remove own')" />
            </button>
          </div>
        </LookRow>

        <LookRow :title="tr('Видность — выключено', 'Opacity — off')">
          <LookSlider v-model="power.opacityOff" :label="tr('Видимость без подключения', 'Visibility while disconnected')" :min="0" :max="100" unit="%" :default-value="30" />
        </LookRow>
        <LookRow :title="tr('Видность — подключено', 'Opacity — on')">
          <LookSlider v-model="power.opacityOn" :label="tr('Видимость при подключении', 'Visibility while connected')" :min="0" :max="100" unit="%" :default-value="62" />
        </LookRow>
        <LookRow :title="tr('Размытие', 'Blur')">
          <LookSlider v-model="power.blur" :label="tr('Размытие картинки на кнопке', 'Button art blur')" :min="0" :max="12" :step="0.5" unit="px" :default-value="1" />
        </LookRow>
        <LookRow :title="tr('Масштаб', 'Zoom')">
          <LookSlider v-model="power.zoom" :label="tr('Масштаб картинки на кнопке', 'Button art zoom')" :min="0.6" :max="2.5" :step="0.05" unit="×" :default-value="1" />
        </LookRow>
        <LookRow :title="tr('Позиция X', 'Position X')">
          <LookSlider v-model="power.posX" unit="%" :default-value="50" :label="tr('Позиция картинки на кнопке X', 'Button art position X')" />
        </LookRow>
        <LookRow :title="tr('Позиция Y', 'Position Y')">
          <LookSlider v-model="power.posY" unit="%" :default-value="50" :label="tr('Позиция картинки на кнопке Y', 'Button art position Y')" />
        </LookRow>
        <LookRow :title="tr('Смешивание', 'Blend')" stack>
          <div class="lk-pills">
            <button v-for="b in blendOptions" :key="b.id" type="button" :class="['lk-pill', { 'lk-pill--on': power.blend === b.id }]" :aria-pressed="power.blend === b.id" @click="power.blend = b.id">{{ b.label }}</button>
          </div>
        </LookRow>
        <LookRow :title="tr('Ч/б, пока отключено', 'Grayscale while off')">
          <LookSwitch v-model="power.grayscaleOff" :label="tr('Ч/б, пока отключено', 'Grayscale while off')" />
        </LookRow>
        <LookRow :title="tr('Дышит при подключении', 'Breathe while connecting')">
          <LookSwitch v-model="power.pulse" :label="tr('Дыхание при подключении', 'Breathe while connecting')" />
        </LookRow>
        <LookRow :title="tr('Медленно вращается', 'Slow spin')">
          <LookSwitch v-model="power.spin" :label="tr('Медленное вращение', 'Slow spin')" />
        </LookRow>
        <LookRow :title="tr('Мягкие края', 'Soft edges')">
          <LookSwitch v-model="power.edgeFade" :label="tr('Мягкие края', 'Soft edges')" />
        </LookRow>

    </div>

    <div v-else :id="`${panelId}-sound`" :aria-labelledby="`${panelId}-sound-tab`" role="tabpanel" tabindex="0">
      <LookRow :icon="Bell" :title="tr('Звуки', 'Sounds')" :desc="tr('Короткие сигналы на подключение, отключение и ошибки', 'Short cues on connect, disconnect and errors')">
        <LookSwitch v-model="sound.enabled" :label="tr('Звуки', 'Sounds')" />
      </LookRow>


        <LookRow :title="tr('Набор', 'Pack')" stack>
          <div class="lk-pills">
            <button
              v-for="pk in SOUND_PACKS"
              :key="pk.id"
              type="button"
              :class="['lk-pill', { 'lk-pill--on': sound.pack === pk.id }]"
              :title="tr(pk.hint.ru, pk.hint.en)"
              @click="pickSoundPack(pk.id)"
              v-text="tr(pk.ru, pk.en)"
            />
          </div>
        </LookRow>
        <LookRow :title="tr('Громкость', 'Volume')">
          <LookSlider v-model="sound.volume" :label="tr('Громкость', 'Volume')" :min="0" :max="100" unit="%" :default-value="45" />
        </LookRow>
        <LookRow :title="tr('Прослушать', 'Preview')" stack>
          <div class="lk-pills">
            <button
              v-for="ev in testEvents"
              :key="ev.id"
              type="button"
              class="lk-pill"
              @click="testSound(ev.id)"
              v-text="'▶ ' + ev.label"
            />
          </div>
        </LookRow>
        <LookRow :title="tr('При подключении', 'On connect')">
          <LookSwitch v-model="sound.connect" :label="tr('При подключении', 'On connect')" />
        </LookRow>
        <LookRow :title="tr('При отключении', 'On disconnect')">
          <LookSwitch v-model="sound.disconnect" :label="tr('При отключении', 'On disconnect')" />
        </LookRow>
        <LookRow :title="tr('При ошибке', 'On error')">
          <LookSwitch v-model="sound.error" :label="tr('При ошибке', 'On error')" />
        </LookRow>
        <LookRow :title="tr('Клики интерфейса', 'UI clicks')">
          <LookSwitch v-model="sound.clicks" :label="tr('Клики интерфейса', 'UI clicks')" />
        </LookRow>

    </div>
    <template #footer>
      <button type="button" class="lk-pill lk-pill--on" @click="emit('close')">{{ tr('Готово', 'Done') }}</button>
    </template>
  </LookModal>

  <LookModal
    :open="open && focusOpen && !!bgSrc"
    :icon="Crosshair"
    :title="t('settings.bgFocusPick')"
    :subtitle="t('settings.bgFocusModalDesc')"
    size="lg"
    @close="closeFocus"
  >
    <div
      ref="focusStage"
      :class="['cl-stage', { 'cl-stage--grab': focusGrabbing }]"
      :style="{ aspectRatio: String(focusRatio) }"
      @pointerdown="focusStart"
      @pointermove="focusDrag"
      @pointerup="focusEnd"
      @pointercancel="focusEnd"
      @lostpointercapture="focusEnd"
    >
      <img ref="focusImg" :src="bgSrc" alt="" draggable="false" :style="focusImgStyle" @dragstart.prevent />
      <span class="cl-stage-grid" aria-hidden="true" />
      <span class="cl-stage-hint" v-text="t('settings.bgFocusDrag')" />
    </div>
    <template #footer>
      <span class="cl-readout mono" v-text="`X ${posX}% · Y ${posY}%`" />
      <button type="button" class="lk-pill" @click="resetFocus">
        <RotateCcw :size="13" aria-hidden="true" />
        <span v-text="t('settings.bgFocusReset')" />
      </button>
      <button type="button" class="lk-pill lk-pill--on" @click="closeFocus">
        <Check :size="13" aria-hidden="true" />
        <span v-text="t('settings.bgFocusDone')" />
      </button>
    </template>
  </LookModal>
</template>

<script setup lang="ts">
import { computed, getCurrentInstance, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { open as openDialog } from '@tauri-apps/api/dialog';
import { readText } from '@tauri-apps/api/clipboard';
import { readBinaryFile } from '@tauri-apps/api/fs';
import {
  Bell,
  Check,
  Crosshair,
  Droplets,
  ImageIcon,
  Link2,
  Moon,
  Power,
  RotateCcw,
  Settings,
  Sparkles,
  Upload,
  X,
  ZoomIn,
} from '../../lib/appIcons';
import { useVpnStore } from '../../stores/vpn';
import { useNotifications } from '../../composables/useNotifications';
import { t } from '../../i18n';
import { useAppearance, IDB_BG, type Blend, type PowerSource, type SoundPack } from '../../appearance/store';
import { SOUND_PACKS, previewSound, type SoundEvent } from '../../appearance/sounds';
import { fileToImageBlob } from '../../appearance/image';
import { PATTERNS } from '../../appearance/patterns';
import { tr } from '../../appearance/i18n';
import PowerArt from './PowerArt.vue';
import LookModal from './LookModal.vue';
import LookRow from './LookRow.vue';
import LookSlider from './LookSlider.vue';
import LookSwitch from './LookSwitch.vue';

const props = defineProps<{ open: boolean; initialTab?: TabId }>();
const emit = defineEmits<{ (e: 'close'): void }>();

type TabId = 'bg' | 'button' | 'sound';
const TAB_KEY = 'wawity_look_tab';
const panelId = `home-look-${getCurrentInstance()?.uid ?? 0}`;
const tabsRef = ref<HTMLElement | null>(null);

const vpnStore = useVpnStore();
const look = useAppearance();
const { pushToast } = useNotifications();
const power = look.state.power;
const sound = look.state.sound;

function readTab(): TabId {
  try {
    const saved = localStorage.getItem(TAB_KEY);
    if (saved === 'bg' || saved === 'button' || saved === 'sound') return saved;
  } catch {}
  return 'bg';
}

const activeTab = ref<TabId>(props.initialTab ?? readTab());
async function onTabKey(e: KeyboardEvent) {
  if (!['ArrowLeft', 'ArrowRight', 'Home', 'End'].includes(e.key)) return;
  e.preventDefault();
  const ids: TabId[] = ['bg', 'button', 'sound'];
  const index = ids.indexOf(activeTab.value);
  const next = e.key === 'Home' ? 0 : e.key === 'End' ? 2 : (index + (e.key === 'ArrowLeft' ? 2 : 1)) % 3;
  activeTab.value = ids[next];
  await nextTick();
  tabsRef.value?.querySelector<HTMLElement>(`#${panelId}-${activeTab.value}-tab`)?.focus();
}
watch(() => props.open, (open) => {
  if (open) {
    if (props.initialTab) activeTab.value = props.initialTab;
    void look.hydrateImages();
  } else closeFocus();
});
watch(activeTab, (tab) => {
  try {
    localStorage.setItem(TAB_KEY, tab);
  } catch {}
});

const isMaterial = computed(() => vpnStore.settings.ui_style === 'material');
const bgUrl = computed(() => vpnStore.settings.bg_custom_url || '');
const bgSrc = computed(() => (bgUrl.value === IDB_BG ? look.bgUrl || '' : bgUrl.value));
const powerOn = computed(() => power.source !== 'none');

const tabs = computed(() => [
  { id: 'bg' as TabId, label: tr('Фон', 'Background'), icon: ImageIcon, live: !!vpnStore.settings.bg_custom_enabled && !!bgUrl.value },
  { id: 'button' as TabId, label: tr('Кнопка', 'Button'), icon: Sparkles, live: powerOn.value },
  { id: 'sound' as TabId, label: tr('Звуки', 'Sounds'), icon: Bell, live: sound.enabled },
]);

const bgEnabled = computed<boolean>({
  get: () => !!vpnStore.settings.bg_custom_enabled,
  set: (v) => vpnStore.updateSettings({ bg_custom_enabled: v }),
});

const bgDim = computed<number>({
  get: () => vpnStore.settings.bg_custom_dim ?? 45,
  set: (v) => vpnStore.updateSettings({ bg_custom_dim: Math.round(v) }),
});

const bgBlur = computed<number>({
  get: () => vpnStore.settings.bg_custom_blur ?? 0,
  set: (v) => vpnStore.updateSettings({ bg_custom_blur: Math.round(v) }),
});

const bgZoom = computed<number>({
  get: () => Math.round((vpnStore.settings.bg_custom_zoom || 1) * 100),
  set: (v) => vpnStore.updateSettings({ bg_custom_zoom: Math.min(2.5, Math.max(1, v / 100)) }),
});

const bgPosX = computed<number>({
  get: () => vpnStore.settings.bg_custom_pos_x ?? 50,
  set: (v) => vpnStore.updateSettings({ bg_custom_pos_x: Math.round(v) }),
});
const bgPosY = computed<number>({
  get: () => vpnStore.settings.bg_custom_pos_y ?? 50,
  set: (v) => vpnStore.updateSettings({ bg_custom_pos_y: Math.round(v) }),
});
const posX = computed(() => vpnStore.settings.bg_custom_pos_x ?? 50);
const posY = computed(() => vpnStore.settings.bg_custom_pos_y ?? 50);
const zoomFactor = computed(() => Math.min(2.5, Math.max(1, vpnStore.settings.bg_custom_zoom || 1)));

const previewImgStyle = computed(() => ({
  objectPosition: `${posX.value}% ${posY.value}%`,
  transform: `scale(${zoomFactor.value})`,
  transformOrigin: `${posX.value}% ${posY.value}%`,
  filter: bgBlur.value > 0 ? `blur(${bgBlur.value / 3}px)` : undefined,
}));

const focusImgStyle = computed(() => ({
  objectPosition: `${posX.value}% ${posY.value}%`,
  transform: `scale(${zoomFactor.value})`,
  transformOrigin: `${posX.value}% ${posY.value}%`,
}));

const bgImporting = ref(false);
const bgUrlDraft = ref('');

function draftFromUrl(url: string) {
  return url.startsWith('data:') || url === IDB_BG ? '' : url;
}

watch(bgUrl, (url) => {
  const next = draftFromUrl(url);
  if (bgUrlDraft.value !== next) bgUrlDraft.value = next;
}, { immediate: true });

const bgUrlValid = computed(() => {
  const v = bgUrlDraft.value.trim();
  return v.startsWith('data:image/') || /^https?:\/\/\S+$/i.test(v);
});

async function commitBgUrl() {
  if (!bgUrlValid.value || bgImporting.value) return;
  const next = bgUrlDraft.value.trim();
  vpnStore.updateSettings({ bg_custom_url: next, bg_custom_enabled: true });
  pushToast('success', t('settings.bgApplied'), t('settings.bgAppliedDesc'), 3000);
}

async function pasteBgUrl() {
  try {
    const text = await readText();
    if (text) bgUrlDraft.value = text.trim();
  } catch (e) {
    pushToast('error', tr('Не получилось вставить ссылку', 'Could not paste the link'), String(e), 4000);
  }
}

async function clearBgImage() {
  if (bgImporting.value) return;
  bgImporting.value = true;
  try {
    if (bgUrl.value === IDB_BG) await look.setImage('bg', null);
    closeFocus();
    bgUrlDraft.value = '';
    vpnStore.updateSettings({ bg_custom_url: '', bg_custom_enabled: false });
  } catch (e) {
    pushToast('error', t('settings.bgImportFail'), String(e), 5000);
  } finally {
    bgImporting.value = false;
  }
}

async function readPickedImage(): Promise<File | null> {
  const chosen = await openDialog({
    multiple: false,
    directory: false,
    filters: [{ name: t('settings.bgFileFilter'), extensions: ['png', 'jpg', 'jpeg', 'webp', 'gif', 'bmp', 'avif'] }],
  });
  if (!chosen || typeof chosen !== 'string') return null;
  const bytes = await readBinaryFile(chosen);
  if (bytes.length > 24 * 1024 * 1024) throw new Error(tr('Максимальный размер картинки — 24 МБ', 'Maximum image size is 24 MB'));
  const ext = chosen.split('.').pop()?.toLowerCase() ?? 'png';
  const types: Record<string, string> = { png: 'image/png', jpg: 'image/jpeg', jpeg: 'image/jpeg', webp: 'image/webp', gif: 'image/gif', bmp: 'image/bmp', avif: 'image/avif' };
  const copy = new Uint8Array(bytes.length);
  copy.set(bytes);
  return new File([copy.buffer], chosen.split(/[\\/]/).pop() ?? 'image', { type: types[ext] ?? 'image/png' });
}

async function pickBgFile() {
  if (bgImporting.value) return;
  bgImporting.value = true;
  try {
    const file = await readPickedImage();
    if (!file) return;
    const blob = await fileToImageBlob(file, 2560);
    await look.setImage('bg', blob);
    vpnStore.updateSettings({ bg_custom_url: IDB_BG, bg_custom_enabled: true });
    bgUrlDraft.value = '';
    pushToast('success', t('settings.bgImported'), t('settings.bgImportedDesc'), 3000);
  } catch (e) {
    pushToast('error', t('settings.bgImportFail'), String(e), 5000);
  } finally {
    bgImporting.value = false;
  }
}

const focusOpen = ref(false);
const focusStage = ref<HTMLElement | null>(null);
const focusImg = ref<HTMLImageElement | null>(null);
const focusGrabbing = ref(false);
const focusRatio = ref(1.6);
let focusPointer: number | null = null;
let lastX = 0;
let lastY = 0;

function setFocus(x: number, y: number) {
  vpnStore.updateSettings({
    bg_custom_pos_x: Math.round(Math.min(100, Math.max(0, x))),
    bg_custom_pos_y: Math.round(Math.min(100, Math.max(0, y))),
  });
}

function openFocus() {
  if (!bgSrc.value) return;
  focusRatio.value = window.innerWidth / Math.max(1, window.innerHeight);
  focusOpen.value = true;
}

function focusEnd() {
  if (focusPointer !== null) {
    const id = focusPointer;
    focusPointer = null;
    try {
      if (focusStage.value?.hasPointerCapture(id)) focusStage.value.releasePointerCapture(id);
    } catch {}
  }
  focusGrabbing.value = false;
}

function closeFocus() {
  focusEnd();
  focusOpen.value = false;
}

function focusStart(e: PointerEvent) {
  if (e.button !== 0 || focusPointer !== null) return;
  e.preventDefault();
  focusPointer = e.pointerId;
  lastX = e.clientX;
  lastY = e.clientY;
  focusGrabbing.value = true;
  try {
    focusStage.value?.setPointerCapture(e.pointerId);
  } catch {}
}

function focusDrag(e: PointerEvent) {
  if (focusPointer !== e.pointerId) return;
  const stage = focusStage.value;
  const img = focusImg.value;
  if (!stage || !img || !img.naturalWidth || !img.naturalHeight) return;
  const pw = stage.clientWidth;
  const ph = stage.clientHeight;
  const dx = e.clientX - lastX;
  const dy = e.clientY - lastY;
  lastX = e.clientX;
  lastY = e.clientY;
  if (!pw || !ph || (dx === 0 && dy === 0)) return;
  const cover = Math.max(pw / img.naturalWidth, ph / img.naturalHeight) * zoomFactor.value;
  const overflowX = img.naturalWidth * cover - pw;
  const overflowY = img.naturalHeight * cover - ph;
  let x = posX.value;
  let y = posY.value;
  if (overflowX > 0.5) x -= (dx / overflowX) * 100;
  if (overflowY > 0.5) y -= (dy / overflowY) * 100;
  setFocus(x, y);
}

function resetFocus() {
  setFocus(50, 50);
  vpnStore.updateSettings({ bg_custom_zoom: 1 });
}

watch(bgSrc, (src) => {
  if (!src) closeFocus();
});

/** Pictures only, straight from PATTERNS — which the build already trimmed to
 *  the files that exist. No probing, nothing async, nothing to wait for. */
const POWER_SOURCES = computed<Array<{ id: PowerSource; ru: string; en: string }>>(() => [
  { id: 'custom', ru: 'Своя', en: 'Custom' },
  ...PATTERNS.map((def) => ({ id: def.id as PowerSource, ru: def.ru, en: def.en })),
]);

function sourceLabel(src: PowerSource) {
  if (src === 'custom') return tr('Своя', 'Custom');
  if (src === 'none') return tr('Нет', 'None');
  const def = PATTERNS.find((x) => x.id === src);
  return def ? tr(def.ru, def.en) : src;
}

let lastPowerSource: PowerSource = power.source !== 'none' ? power.source : 'none';
const powerBusy = ref(false);

function togglePowerArt(on: boolean) {
  if (on) {
    power.source = lastPowerSource === 'custom' && !look.powerUrl ? 'none' : lastPowerSource;
  } else {
    if (power.source !== 'none') lastPowerSource = power.source;
    power.source = 'none';
  }
}

async function uploadPowerArt() {
  if (powerBusy.value) return;
  powerBusy.value = true;
  try {
    const file = await readPickedImage();
    if (!file) return;
    const blob = await fileToImageBlob(file, 600);
    await look.setImage('power', blob);
    power.source = 'custom';
    lastPowerSource = 'custom';
  } catch (e) {
    pushToast('error', tr('Не получилось прочитать картинку', 'Could not read the picture'), String(e), 5000);
  } finally {
    powerBusy.value = false;
  }
}

async function removePowerArt() {
  if (powerBusy.value) return;
  powerBusy.value = true;
  try {
    await look.setImage('power', null);
    if (power.source === 'custom') power.source = 'none';
    if (lastPowerSource === 'custom') lastPowerSource = 'none';
  } catch (e) {
    pushToast('error', tr('Не получилось удалить картинку', 'Could not remove the picture'), String(e), 5000);
  } finally {
    powerBusy.value = false;
  }
}

function pickSoundPack(id: SoundPack) {
  sound.pack = id;
  previewSound(id, 'connect', sound.volume);
}

function testSound(ev: SoundEvent) {
  previewSound(sound.pack, ev, sound.volume);
}

const powerStates = computed(() => [
  { id: 'off', label: tr('Выключено', 'Disconnected'), on: false, busy: false },
  { id: 'busy', label: tr('Подключение', 'Connecting'), on: false, busy: true },
  { id: 'on', label: tr('Подключено', 'Connected'), on: true, busy: false },
]);
const blendOptions = computed<Array<{ id: Blend; label: string }>>(() => [
  { id: 'screen', label: tr('Свет', 'Screen') },
  { id: 'normal', label: tr('Обычно', 'Normal') },
  { id: 'overlay', label: tr('Наложение', 'Overlay') },
  { id: 'luminosity', label: tr('Яркость', 'Luminosity') },
]);
onBeforeUnmount(focusEnd);

const testEvents = computed<Array<{ id: SoundEvent; label: string }>>(() => [
  { id: 'connect', label: tr('Подкл.', 'Connect') },
  { id: 'disconnect', label: tr('Откл.', 'Disconnect') },
  { id: 'error', label: tr('Ошибка', 'Error') },
  { id: 'click', label: tr('Клик', 'Click') },
]);
</script>

<style scoped>
.cl-preview {
  position: relative;
  height: 120px;
  margin-top: 12px;
  overflow: hidden;
  border: 1px solid var(--border);
  border-radius: 0;
  background: var(--background);
}

.cl-preview img,
.cl-stage img {
  display: block;
  width: 100%;
  height: 100%;
  object-fit: cover;
  pointer-events: none;
}

.cl-preview-dim {
  position: absolute;
  inset: 0;
  background: var(--background);
  pointer-events: none;
}

.cl-stage {
  position: relative;
  width: 100%;
  max-height: 56vh;
  margin-top: 12px;
  overflow: hidden;
  border: 1px solid var(--border);
  border-radius: 0;
  background: var(--background);
  cursor: grab;
  touch-action: none;
  user-select: none;
}

.cl-stage--grab { cursor: grabbing; }

.cl-stage-grid {
  position: absolute;
  inset: 0;
  pointer-events: none;
  background-image:
    linear-gradient(to right, rgba(255, 255, 255, 0.16) 1px, transparent 1px),
    linear-gradient(to bottom, rgba(255, 255, 255, 0.16) 1px, transparent 1px);
  background-size: 33.333% 33.333%;
}

.cl-stage-hint {
  position: absolute;
  left: 50%;
  bottom: 12px;
  transform: translateX(-50%);
  padding: 6px 14px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 999px;
  background: rgba(5, 6, 10, 0.72);
  backdrop-filter: blur(6px);
  color: rgba(235, 238, 250, 0.78);
  font-size: 11px;
  white-space: nowrap;
  pointer-events: none;
  transition: opacity 0.18s ease;
}

.cl-stage--grab .cl-stage-hint { opacity: 0; }

.cl-readout {
  margin-right: auto;
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--muted-foreground);
}
.cl-power-preview { display: grid; grid-template-columns: repeat(3, 1fr); gap: 16px; padding: 22px 0; border-bottom: 1px solid var(--border); }
.cl-power-cell { display: flex; flex-direction: column; align-items: center; gap: 10px; }
.cl-power-button { position: relative; display: grid; place-items: center; width: 76px; height: 76px; border: 1px solid var(--input); border-radius: 50%; background: color-mix(in oklch, var(--foreground) 4%, transparent); color: var(--muted-foreground); overflow: hidden; }
.cl-power-button > svg { position: relative; z-index: 1; }
.cl-power-button--on { border-color: color-mix(in oklch, var(--success) 55%, transparent); color: var(--success); background: color-mix(in oklch, var(--success) 7%, transparent); }
.cl-power-button--busy { border-color: var(--ember); }
.cl-power-label { font-family: var(--font-label); font-size: 8px; letter-spacing: 0.06em; color: var(--muted-foreground); text-align: center; }
</style>
