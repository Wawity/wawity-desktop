import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import type { VpnStatus, AppSettings, SubscriptionGroup, ServerEntry, SessionRecord, BlockReport } from '../types/vpn.d';
import { useNotifications } from '../composables/useNotifications';
import { setLanguage, t } from '../i18n';

let activeHotkeyKey = '';
let hotkeyTogglePending = false;

import { setTelemetryAllowed, track } from '../lib/telemetry';

const DEFAULT_SETTINGS: AppSettings = {
  kill_switch: true,
  always_on: false,
  auto_connect: false,
  start_on_boot: false,
  telemetry: true,
  lan_access: false,
  block_trackers: true,
  notifications: false,
  multihop_enabled: false,
  quantum_resistant: true,
  black_hole_bg: true,
  black_hole_detail: 'simple',
  liquid_glass: false,
  server_view: 'list',
  protocol: 'auto',
  bypass_apps: [],
  split_mode: 'exclude',
  split_domains: [],
  split_ips: [],
  split_processes: [],
  split_templates: [],
  theme: 'dark',
  language: 'en',
  discord_rpc: false,
  discord_rpc_show_server: true,
  discord_rpc_show_subscription: false,
  hotkeys_enabled: false,
  hotkey_toggle: 'CommandOrControl+Shift+X',
  strict_route: true,
  allow_insecure_tls: false,
  auto_ping_minutes: 0,
  tunnel_own_traffic: true,
  dns_leak_guard: true,
  bootstrap_dns: 'cloudflare',
  online_geolocation: false,
};

const DEFAULT_STATUS: VpnStatus = {
  connected: false,
  pid: null,
  server: null,
  kill_switch: false,
  interface: null,
  server_name: null,
  entry_server_name: null,
  multihop: false,
  bytes_rx: 0,
  bytes_tx: 0,
  speed_rx: 0,
  speed_tx: 0,
  always_on_locked: false,
};

const STORAGE_KEY_SELECTED_SERVER = 'wawity_selected_server';

const KEYWORD_MAP: Record<string, string> = {
  'россия': 'ru', 'российская': 'ru', 'москва': 'ru', 'russia': 'ru', 'russian': 'ru', 'moscow': 'ru', 'moskva': 'ru',
  'германия': 'de', 'germany': 'de', 'german': 'de', 'frankfurt': 'de', 'berlin': 'de', 'nuremberg': 'de',
  'нидерланды': 'nl', 'netherlands': 'nl', 'amsterdam': 'nl', 'holland': 'nl',
  'сша': 'us', 'соединенные штаты': 'us', 'united states': 'us', 'usa': 'us',
  'new york': 'us', 'los angeles': 'us', 'chicago': 'us', 'seattle': 'us', 'dallas': 'us', 'miami': 'us', 'ashburn': 'us',
  'великобритания': 'gb', 'британия': 'gb', 'united kingdom': 'gb', 'london': 'gb', 'england': 'gb', 'britain': 'gb',
  'франция': 'fr', 'france': 'fr', 'paris': 'fr',
  'япония': 'jp', 'japan': 'jp', 'tokyo': 'jp', 'osaka': 'jp',
  'сингапур': 'sg', 'singapore': 'sg',
  'канада': 'ca', 'canada': 'ca', 'toronto': 'ca', 'montreal': 'ca', 'vancouver': 'ca',
  'австралия': 'au', 'australia': 'au', 'sydney': 'au', 'melbourne': 'au',
  'швеция': 'se', 'sweden': 'se', 'stockholm': 'se',
  'финляндия': 'fi', 'finland': 'fi', 'helsinki': 'fi',
  'норвегия': 'no', 'norway': 'no', 'oslo': 'no',
  'швейцария': 'ch', 'switzerland': 'ch', 'zurich': 'ch', 'geneva': 'ch',
  'польша': 'pl', 'poland': 'pl', 'warsaw': 'pl',
  'украина': 'ua', 'ukraine': 'ua', 'kyiv': 'ua', 'kiev': 'ua',
  'турция': 'tr', 'turkey': 'tr', 'istanbul': 'tr', 'ankara': 'tr',
  'бразилия': 'br', 'brazil': 'br', 'sao paulo': 'br',
  'индия': 'in', 'india': 'in', 'mumbai': 'in', 'delhi': 'in',
  'китай': 'cn', 'china': 'cn', 'beijing': 'cn', 'shanghai': 'cn',
  'южная корея': 'kr', 'south korea': 'kr', 'korea': 'kr', 'seoul': 'kr',
  'гонконг': 'hk', 'hong kong': 'hk',
  'тайвань': 'tw', 'taiwan': 'tw', 'taipei': 'tw',
  'испания': 'es', 'spain': 'es', 'madrid': 'es', 'barcelona': 'es',
  'италия': 'it', 'italy': 'it', 'rome': 'it', 'milan': 'it',
  'австрия': 'at', 'austria': 'at', 'vienna': 'at',
  'чехия': 'cz', 'czech': 'cz', 'prague': 'cz',
  'румыния': 'ro', 'romania': 'ro', 'bucharest': 'ro',
  'венгрия': 'hu', 'hungary': 'hu', 'budapest': 'hu',
  'португалия': 'pt', 'portugal': 'pt', 'lisbon': 'pt',
  'аргентина': 'ar', 'argentina': 'ar', 'buenos aires': 'ar',
  'мексика': 'mx', 'mexico': 'mx',
  'израиль': 'il', 'israel': 'il', 'tel aviv': 'il',
  'оаэ': 'ae', 'uae': 'ae', 'dubai': 'ae',
  'саудовская аравия': 'sa', 'saudi': 'sa', 'riyadh': 'sa',
  'южная африка': 'za', 'south africa': 'za', 'johannesburg': 'za',
  'латвия': 'lv', 'latvia': 'lv', 'riga': 'lv',
  'литва': 'lt', 'lithuania': 'lt', 'vilnius': 'lt',
  'эстония': 'ee', 'estonia': 'ee', 'tallinn': 'ee',
  'молдова': 'md', 'moldova': 'md',
  'беларусь': 'by', 'belarus': 'by', 'minsk': 'by',
  'казахстан': 'kz', 'kazakhstan': 'kz', 'almaty': 'kz',
  'грузия': 'ge', 'georgia': 'ge', 'tbilisi': 'ge',
  'армения': 'am', 'armenia': 'am', 'yerevan': 'am',
  'азербайджан': 'az', 'azerbaijan': 'az', 'baku': 'az',
  'индонезия': 'id', 'indonesia': 'id', 'jakarta': 'id',
  'малайзия': 'my', 'malaysia': 'my', 'kuala lumpur': 'my',
  'таиланд': 'th', 'thailand': 'th', 'bangkok': 'th',
  'вьетнам': 'vn', 'vietnam': 'vn', 'hanoi': 'vn',
  'филиппины': 'ph', 'philippines': 'ph', 'manila': 'ph',
  'пакистан': 'pk', 'pakistan': 'pk', 'karachi': 'pk',
  'египет': 'eg', 'egypt': 'eg', 'cairo': 'eg',
  'дания': 'dk', 'denmark': 'dk', 'copenhagen': 'dk',
  'бельгия': 'be', 'belgium': 'be', 'brussels': 'be',
  'словакия': 'sk', 'slovakia': 'sk', 'bratislava': 'sk',
  'болгария': 'bg', 'bulgaria': 'bg', 'sofia': 'bg',
  'сербия': 'rs', 'serbia': 'rs', 'belgrade': 'rs',
  'хорватия': 'hr', 'croatia': 'hr', 'zagreb': 'hr',
  'греция': 'gr', 'greece': 'gr', 'athens': 'gr',
  'ирландия': 'ie', 'ireland': 'ie', 'dublin': 'ie',
  'люксембург': 'lu', 'luxembourg': 'lu',
  'исландия': 'is', 'iceland': 'is', 'reykjavik': 'is',
  'новая зеландия': 'nz', 'new zealand': 'nz', 'auckland': 'nz',
  'чили': 'cl', 'chile': 'cl', 'santiago': 'cl',
  'колумбия': 'co', 'colombia': 'co', 'bogota': 'co',
  'перу': 'pe', 'peru': 'pe', 'lima': 'pe',
};

const ISO2_SET = new Set([
  'ru','de','nl','us','gb','fr','jp','sg','ca','au','se','fi','no','ch','pl',
  'ua','tr','br','in','cn','kr','hk','tw','es','it','at','cz','ro','hu','pt',
  'ar','mx','il','ae','sa','za','lv','lt','ee','md','by','kz','ge','am','az',
  'id','my','th','vn','ph','pk','eg','ng','dk','be','sk','bg','rs','hr','gr',
  'ie','lu','is','nz','cl','co','pe','uk',
]);

const HOST_SUFFIX_BLACKLIST = new Set([
  'com', 'net', 'org', 'io', 'to', 'cc', 'me', 'tv', 'xyz', 'info', 'pro',
  'top', 'site', 'online', 'click', 'link', 'host', 'cloud', 'app', 'dev',
  'tech', 'biz', 'name', 'one', 'vip', 'fun', 'live', 'world', 'store',
  'space', 'website', 'icu', 'vpn', 'best', 'win', 'services', 'solutions',
  'company', 'group', 'network', 'systems', 'digital', 'media', 'life',
]);

function stripRegistrableSuffix(hostLower: string): string[] {
  const parts = hostLower.split('.').filter(Boolean);
  if (parts.length <= 1) return parts;
  const last = parts[parts.length - 1];
  const secondLast = parts.length >= 2 ? parts[parts.length - 2] : '';
  const dropCount = last.length <= 3 && secondLast.length <= 3 && parts.length >= 3 ? 2 : 1;
  return parts.slice(0, Math.max(0, parts.length - dropCount));
}

export function extractCountryCode(serverHost: string, name: string): string {
  const nameLower = name.toLowerCase();
  const hostLower = serverHost.toLowerCase();
  const hay = ' ' + nameLower + ' ' + hostLower + ' ';
  for (const [kw, code] of Object.entries(KEYWORD_MAP)) {
    if (hay.includes(kw)) return code;
  }
  const subdomainLabels = stripRegistrableSuffix(hostLower);
  for (const label of subdomainLabels) {
    for (const seg of label.split('-')) {
      if (seg.length === 2 && !HOST_SUFFIX_BLACKLIST.has(seg) && ISO2_SET.has(seg)) {
        return seg === 'uk' ? 'gb' : seg;
      }
    }
  }
  const nameTokens = nameLower.split(/[\s\-_|[\]()/\\#@.,;:]+/).filter(t => t.length === 2);
  for (const t of nameTokens) {
    if (!HOST_SUFFIX_BLACKLIST.has(t) && ISO2_SET.has(t)) {
      return t === 'uk' ? 'gb' : t;
    }
  }
  return 'UN';
}

function balanceBrackets(s: string): string {
  let open = 0;
  let result = '';
  for (const ch of s) {
    if (ch === '(') { open++; result += ch; }
    else if (ch === ')') { if (open > 0) { open--; result += ch; } }
    else result += ch;
  }
  return result + ')'.repeat(open);
}

function stripServerName(raw: string): string {
  let name = raw;
  name = name.replace(/[\u{1F1E0}-\u{1F1FF}]{2}/gu, '');
  name = name.replace(/[\u{1F300}-\u{1FAFF}]/gu, '');
  name = name.replace(/[\u{2600}-\u{26FF}]/gu, '');
  name = name.replace(/[\u{2700}-\u{27BF}]/gu, '');
  name = name.replace(/\[[A-Z]{2}\]\s*/g, '');
  name = name.replace(/\([A-Z]{2}\)\s*/g, '');
  name = name.replace(/#[A-Z]{2}#\s*/g, '');
  name = name.replace(/\b[A-Z]{2}\s*[-|]\s*/g, '');
  name = name.replace(/\s*[-|]\s*[A-Z]{2}\b/g, '');
  name = name.replace(/^[\s\-|_#@[\]()/\\]+/, '');
  name = name.replace(/[\s\-|_#@[\]()/\\]+$/, '');
  name = name.replace(/\s{2,}/g, ' ').trim();
  name = balanceBrackets(name);
  return name.trim() || raw.trim();
}

function isIp(host: string): boolean {
  return /^(\d{1,3}\.){3}\d{1,3}$/.test(host);
}

function parseServerPort(url: string): number {
  try {
    const u = new URL(url);
    return u.port ? parseInt(u.port, 10) : 443;
  } catch {
    const m = url.match(/@[^:]+:(\d+)/);
    return m ? parseInt(m[1], 10) : 443;
  }
}

async function resolveHostsToCountries(hosts: string[]): Promise<Map<string, string>> {
  const result = new Map<string, string>();
  if (hosts.length === 0) return result;
  if (!useVpnStore().settings.online_geolocation) return result;
  try {
    const codes = await invoke<(string | null)[]>('geolocate_servers', { hosts });
    hosts.forEach((h, i) => {
      const code = codes[i];
      if (code) result.set(h, code);
    });
  } catch {}
  return result;
}

function normalizeWinPath(raw: string): string {
  const trimmed = raw.trim();
  if (!trimmed) return '';
  let p = trimmed;
  if (p.startsWith('\\\\?\\UNC\\')) {
    p = '\\\\' + p.slice(8);
  } else if (p.startsWith('\\\\?\\')) {
    p = p.slice(4);
  } else if (p.startsWith('\\\\.\\')) {
    p = p.slice(4);
  }
  p = p.replace(/\//g, '\\');
  if (p.length >= 3 && p[1] === ':' && p[2] === '\\' && /[a-z]/.test(p[0])) {
    p = p[0].toUpperCase() + p.slice(1);
  }
  return p;
}

const DOMAIN_SHAPE = /^[a-z0-9]([a-z0-9-]*[a-z0-9])?(\.[a-z0-9]([a-z0-9-]*[a-z0-9])?)+$/;

export function cleanDomain(raw: string): string | null {
  let value = raw.trim().toLowerCase();
  if (!value) return null;
  value = value.replace(/^[a-z]+:\/\//, '');
  value = value.split('/')[0].split('?')[0];
  value = value.replace(/^\*\./, '').replace(/^\.+/, '').replace(/\.+$/, '');
  if (!value || value.length > 253) return null;
  return DOMAIN_SHAPE.test(value) ? value : null;
}

export function cleanProcess(raw: string): string | null {
  const value = raw.trim().split(/[\\/]/).pop()?.trim().toLowerCase() ?? '';
  if (!value || value.length > 128) return null;
  if (!/^[a-z0-9][a-z0-9 ._+-]*$/.test(value)) return null;
  return value.endsWith('.exe') ? value : `${value}.exe`;
}

export function cleanCidr(raw: string): string | null {
  const value = raw.trim();
  if (!value) return null;
  const [addr, maskRaw] = value.split('/');
  const octets = addr.split('.');
  if (octets.length !== 4) return null;
  for (const octet of octets) {
    if (!/^\d{1,3}$/.test(octet)) return null;
    if (Number(octet) > 255) return null;
  }
  if (maskRaw === undefined) return addr + '/32';
  if (!/^\d{1,2}$/.test(maskRaw)) return null;
  const mask = Number(maskRaw);
  if (mask > 32) return null;
  return addr + '/' + mask;
}

function normalizePathList(paths: string[]): string[] {
  const seen = new Set<string>();
  const out: string[] = [];
  for (const raw of paths) {
    const norm = normalizeWinPath(raw);
    if (!norm) continue;
    const key = norm.toLowerCase();
    if (!seen.has(key)) {
      seen.add(key);
      out.push(norm);
    }
  }
  return out;
}

export interface RefreshResult {
  added: number;
  removed: number;
  error: string | null;
}

interface SubscriptionInfoRaw {
  expire: number | null;
  total: number | null;
  upload: number | null;
  download: number | null;
}

interface SubscriptionInfoParsed {
  expiresAt: number | null;
  totalBytes: number | null;
  usedBytes: number | null;
}

export const useVpnStore = defineStore('vpn', {
  state: () => ({
    status: { ...DEFAULT_STATUS } as VpnStatus,
    settings: { ...DEFAULT_SETTINGS } as AppSettings,
    subscriptions: [] as SubscriptionGroup[],
    sessions: [] as SessionRecord[],
    selectedServerId: null as string | null,
    selectedEntryServerId: null as string | null,
    selectedSubId: null as string | null,
    loading: false,
    connectError: null as string | null,
    sessionStartedAt: null as number | null,
    sessionSeconds: 0,
    autoSelectLoading: false,
    latencyLoading: false,
    splitDirty: false,
    splitApplying: false,
    detectingBlocks: false,
    currentPingMs: null as number | null,
    refreshingSubIds: new Set<string>(),
    _sessionTimer: null as ReturnType<typeof setInterval> | null,
    _statusTimer: null as ReturnType<typeof setInterval> | null,
    _presenceKey: '' as string,
    _pingTimer: null as ReturnType<typeof setInterval> | null,
    _autoPingTimer: null as ReturnType<typeof setInterval> | null,
    _prevBytesRx: 0,
    _prevBytesTx: 0,
    _prevPollTs: 0,
  }),

  getters: {
    allServers: (state): ServerEntry[] => state.subscriptions.flatMap(g => g.servers),
    availableServers: (state): ServerEntry[] => {
      const now = Date.now();
      return state.subscriptions
        .filter(g => g.expiresAt === null || g.expiresAt > now)
        .flatMap(g => g.servers);
    },
    trayServers: (state): ServerEntry[] => {
      const now = Date.now();
      const live = state.subscriptions.filter(g => g.expiresAt === null || g.expiresAt > now);
      const active = live.find(g => g.id === state.selectedSubId);
      if (active) return active.servers;
      return live.flatMap(g => g.servers);
    },
    isServerExpired: (state) => (serverId: string): boolean => {
      const now = Date.now();
      for (const sub of state.subscriptions) {
        if (sub.servers.some(s => s.id === serverId)) {
          return sub.expiresAt !== null && sub.expiresAt <= now;
        }
      }
      return false;
    },
    selectedServer: (state): ServerEntry | null => {
      if (!state.selectedServerId) return null;
      for (const sub of state.subscriptions) {
        const found = sub.servers.find(s => s.id === state.selectedServerId);
        if (found) return found;
      }
      return null;
    },
    entryServer: (state): ServerEntry | null => {
      if (!state.selectedEntryServerId) return null;
      for (const sub of state.subscriptions) {
        const found = sub.servers.find(s => s.id === state.selectedEntryServerId);
        if (found) return found;
      }
      return null;
    },
    selectedSubscription: (state): SubscriptionGroup | null => {
      if (!state.selectedSubId) return null;
      return state.subscriptions.find(s => s.id === state.selectedSubId) ?? null;
    },
    sessionDuration: (state): string => {
      const s = state.sessionSeconds;
      const h = String(Math.floor(s / 3600)).padStart(2, '0');
      const m = String(Math.floor((s % 3600) / 60)).padStart(2, '0');
      const sec = String(s % 60).padStart(2, '0');
      return `${h}:${m}:${sec}`;
    },
    speedRxFormatted: (state): string => formatSpeed(state.status.speed_rx),
    speedTxFormatted: (state): string => formatSpeed(state.status.speed_tx),
    totalRxFormatted: (state): string => formatBytes(state.status.bytes_rx),
    totalTxFormatted: (state): string => formatBytes(state.status.bytes_tx),
    currentPingDisplay: (state): string => {
      if (!state.status.connected) return '—';
      if (state.currentPingMs === null || state.currentPingMs === undefined) return t('connection.measuring');
      return `${state.currentPingMs} ms`;
    },
  },

  actions: {
    async boot() {
      this.loadSettings();
      track('app_started');
      this.syncDiscordPresence();
      this.syncHotkeys();
      this.loadSelectedServer();
      this.loadSubscriptions();
      this.loadEntrySelection();
      listen('wawity-tray-sync', () => {
        this.loadSelectedServer();
        this.loadSubscriptions();
        this.refreshStatus().catch(() => {});
        this.syncTrayState();
      }).catch(() => {});
      listen('wawity-hotkey-toggle', () => {
        this.refreshStatus().catch(() => {});
        this.syncTrayState();
      }).catch(() => {});
      listen('wawity-hotkey-error', (e) => {
        const { pushToast } = useNotifications();
        pushToast('error', t('toast.hotkeyFailed'), String(e.payload), 5000);
      }).catch(() => {});
      await this.refreshStatus();
      await this.reconcileStartOnBoot();
      if (this.settings.always_on && !this.status.connected) {
        try { await invoke('set_always_on', { enabled: true }); } catch {}
        await this.refreshStatus();
      }
      if (this.settings.auto_connect && !this.status.connected && this.selectedServerId) {
        this.bootAutoConnect();
      }
      this.syncTrayState();
      this.startPolling();
      this._refreshAllSubInfoStale();
    },

    async bootAutoConnect() {
      for (let attempt = 1; attempt <= 4; attempt++) {
        await this.refreshStatus().catch(() => {});
        if (this.status.connected) return;
        try {
          await this.connect();
          return;
        } catch {
          await new Promise((resolve) => setTimeout(resolve, 4000 * attempt));
        }
      }
    },

    async reconcileStartOnBoot() {
      try {
        const actual = await invoke<boolean>('get_start_on_boot');
        if (actual !== this.settings.start_on_boot) {
          this.settings.start_on_boot = actual;
          this.persistSettings();
        }
      } catch {}
    },

    async setStartOnBoot(enabled: boolean): Promise<boolean> {
      try {
        await invoke('set_start_on_boot', { enabled });
        this.settings.start_on_boot = enabled;
        this.persistSettings();
        return true;
      } catch (err) {
        const { pushToast } = useNotifications();
        pushToast('error', 'Startup setting failed', String(err), 6000);
        return false;
      }
    },

    syncTrayState() {
      invoke('sync_tray_state', {
        servers: this.trayServers.map(s => ({
          id: s.id,
          name: s.name,
          url: s.url,
          countryCode: s.countryCode,
        })),
        selectedId: this.selectedServerId,
        killSwitch: this.settings.kill_switch,
        quantumResistant: this.settings.quantum_resistant,
        bypassApps: this.settings.bypass_apps,
      }).catch(() => {});
    },

    loadEntrySelection() {
      try {
        const id = localStorage.getItem('wawity_entry_server');
        if (id) this.selectedEntryServerId = id;
      } catch {}
    },

    selectEntryServer(serverId: string | null) {
      if (serverId && this.isServerExpired(serverId)) return;
      this.selectedEntryServerId = serverId;
      try {
        if (serverId) localStorage.setItem('wawity_entry_server', serverId);
        else localStorage.removeItem('wawity_entry_server');
      } catch {}
    },

    persistSelectedServer() {
      try {
        if (this.selectedServerId) {
          localStorage.setItem(STORAGE_KEY_SELECTED_SERVER, JSON.stringify({
            serverId: this.selectedServerId,
            subId: this.selectedSubId,
          }));
        } else {
          localStorage.removeItem(STORAGE_KEY_SELECTED_SERVER);
        }
      } catch {}
    },

    loadSelectedServer() {
      try {
        const raw = localStorage.getItem(STORAGE_KEY_SELECTED_SERVER);
        if (!raw) return;
        const parsed = JSON.parse(raw) as { serverId?: string; subId?: string | null };
        if (parsed.serverId) {
          this.selectedServerId = parsed.serverId;
          this.selectedSubId = parsed.subId ?? null;
        }
      } catch {}
    },

    startPolling() {
      if (this._statusTimer) return;
      this._statusTimer = setInterval(async () => { await this.refreshStatus(); }, 2000);
    },

    stopPolling() {
      if (this._statusTimer) { clearInterval(this._statusTimer); this._statusTimer = null; }
    },

    startAutoPing() {
      this.stopAutoPing();
      const minutes = this.settings.auto_ping_minutes;
      if (!minutes || minutes <= 0) return;
      this._autoPingTimer = setInterval(() => {
        if (this.latencyLoading || this.autoSelectLoading) return;
        if (this.allServers.length === 0) return;
        void this.measureLatencies();
      }, minutes * 60_000);
    },

    stopAutoPing() {
      if (this._autoPingTimer) { clearInterval(this._autoPingTimer); this._autoPingTimer = null; }
    },

    _startSessionTimer() {
      this.sessionSeconds = 0;
      this.sessionStartedAt = Date.now();
      if (this._sessionTimer) clearInterval(this._sessionTimer);
      this._sessionTimer = setInterval(() => { this.sessionSeconds++; }, 1000);
      this.measureCurrentPing();
      if (this._pingTimer) clearInterval(this._pingTimer);
      this._pingTimer = setInterval(() => { this.measureCurrentPing(); }, 7000);
    },

    _stopSessionTimer() {
      if (this._sessionTimer) { clearInterval(this._sessionTimer); this._sessionTimer = null; }
      if (this._pingTimer) { clearInterval(this._pingTimer); this._pingTimer = null; }
      this.sessionSeconds = 0;
      this.sessionStartedAt = null;
      this.currentPingMs = null;
    },

    async measureCurrentPing() {
      if (!this.status.connected) { this.currentPingMs = null; return; }
      try {
        const ms = await invoke<number | null>('measure_tunnel_latency');
        this.currentPingMs = ms;
      } catch {
        this.currentPingMs = null;
      }
    },

    async connect(serverId?: string) {
      const target = serverId
        ? this.allServers.find(s => s.id === serverId)
        : this.allServers.find(s => s.id === this.selectedServerId);
      if (!target) { this.connectError = 'No server selected'; return; }
      if (this.isServerExpired(target.id)) {
        this.connectError = t('servers.subscriptionExpired');
        return;
      }
      let entry: ServerEntry | null = null;
      if (this.settings.multihop_enabled) {
        entry = this.entryServer;
        if (!entry) {
          this.connectError = 'Multi-hop is enabled — pick an entry server on the Servers page';
          return;
        }
        if (entry.id === target.id || entry.server === target.server) {
          this.connectError = 'Entry and exit servers must be different';
          return;
        }
      }
      this.loading = true;
      this.connectError = null;
      this.stopPolling();
      try {
        await invoke('connect_vpn', {
          subUrl: target.url,
          entrySubUrl: entry ? entry.url : null,
          serverName: target.name,
          entryServerName: entry ? entry.name : null,
          killSwitch: this.settings.kill_switch,
          bypassApps: this.settings.bypass_apps,
          quantumResistant: this.settings.quantum_resistant,
          privacy: {
            strict_route: this.settings.strict_route,
            allow_insecure_tls: this.settings.allow_insecure_tls,
            tunnel_own_traffic: this.settings.tunnel_own_traffic,
            dns_leak_guard: this.settings.dns_leak_guard,
            bootstrap_dns: this.settings.bootstrap_dns,
          },
        });
        this.selectServer(target.id);
        await this.refreshStatus();
        this._startSessionTimer();
        track('vpn_connected', { multihop: this.settings.multihop_enabled, protocol: this.settings.protocol });
      } catch (err) {
        this.connectError = String(err);
        track('vpn_connect_failed', { reason: String(err).slice(0, 120) });
      } finally {
        this.loading = false;
        this.startPolling();
      }
    },

    async disconnect() {
      this.loading = true;
      this.connectError = null;
      this.stopPolling();
      try {
        await invoke('disconnect_vpn');
        await this.refreshStatus();
        this._stopSessionTimer();
        this._prevBytesRx = 0;
        this._prevBytesTx = 0;
      } catch (err) {
        this.connectError = String(err);
      } finally {
        this.loading = false;
        this.startPolling();
      }
    },

    async switchServer(serverId: string) {
      if (serverId === this.selectedServerId) return;
      const target = this.allServers.find(s => s.id === serverId);
      if (!target) { this.connectError = 'Server not found'; return; }
      if (this.isServerExpired(target.id)) {
        this.connectError = t('servers.subscriptionExpired');
        return;
      }
      if (!this.status.connected) {
        this.selectServer(serverId);
        return;
      }
      let entry: ServerEntry | null = null;
      if (this.settings.multihop_enabled) {
        entry = this.entryServer;
        if (!entry) {
          this.connectError = 'Multi-hop is enabled — pick an entry server on the Servers page';
          return;
        }
        if (entry.id === target.id || entry.server === target.server) {
          this.connectError = 'Entry and exit servers must be different';
          return;
        }
      }
      this.loading = true;
      this.connectError = null;
      this.stopPolling();
      try {
        await invoke('switch_vpn_server', {
          subUrl: target.url,
          entrySubUrl: entry ? entry.url : null,
          serverName: target.name,
          entryServerName: entry ? entry.name : null,
          bypassApps: this.settings.bypass_apps,
          quantumResistant: this.settings.quantum_resistant,
        });
        this.selectServer(serverId);
        await this.refreshStatus();
      } catch (err) {
        this.connectError = String(err);
        await this.refreshStatus();
        if (!this.status.connected) this._stopSessionTimer();
      } finally {
        this.loading = false;
        this.startPolling();
      }
    },

    async setAlwaysOn(enabled: boolean): Promise<boolean> {
      try {
        await invoke('set_always_on', { enabled });
        this.settings.always_on = enabled;
        this.persistSettings();
        await this.refreshStatus();
        return true;
      } catch (err) {
        const { pushToast } = useNotifications();
        pushToast('error', 'Always-On lockdown failed', String(err), 6000);
        return false;
      }
    },

    async refreshStatus() {
      try {
        const now = Date.now();
        const raw = await invoke<VpnStatus>('get_vpn_status');
        if (this._prevPollTs > 0 && raw.connected) {
          const dt = (now - this._prevPollTs) / 1000;
          raw.speed_rx = Math.max(0, (raw.bytes_rx - this._prevBytesRx) / dt);
          raw.speed_tx = Math.max(0, (raw.bytes_tx - this._prevBytesTx) / dt);
        } else {
          raw.speed_rx = 0;
          raw.speed_tx = 0;
        }
        this._prevBytesRx = raw.bytes_rx;
        this._prevBytesTx = raw.bytes_tx;
        this._prevPollTs = now;
        const wasConnected = this.status.connected;
        this.status = raw;
        this.syncDiscordPresence();
        if (wasConnected && !raw.connected) this._stopSessionTimer();
        else if (!wasConnected && raw.connected) this._startSessionTimer();
      } catch {}
    },

    async fetchSubscriptionPreview(url: string): Promise<{ name: string; servers: ServerEntry[] }> {
      const raw = await invoke<{ name: string; url: string; protocol: string; server: string }[]>(
        'fetch_subscription_raw',
        { url: url.trim() }
      );
      const servers: ServerEntry[] = raw.map((s, i) => ({
        id: `preview-${i}`,
        name: stripServerName(s.name),
        url: s.url,
        protocol: s.protocol,
        server: s.server,
        countryCode: extractCountryCode(s.server, s.name),
        latencyMs: null,
      }));
      const hostname = (() => {
        try { return new URL(url).hostname; } catch { return url.slice(0, 30); }
      })();
      return { name: hostname, servers };
    },

    async fetchSubscriptionInfo(url: string): Promise<SubscriptionInfoParsed> {
      try {
        const raw = await invoke<SubscriptionInfoRaw>('get_subscription_info', { url: url.trim() });
        const expiresAt = raw.expire !== null && raw.expire !== undefined ? raw.expire * 1000 : null;
        const totalBytes = raw.total ?? null;
        const hasUsage = (raw.upload !== null && raw.upload !== undefined) || (raw.download !== null && raw.download !== undefined);
        const usedBytes = hasUsage ? (raw.upload ?? 0) + (raw.download ?? 0) : null;
        return { expiresAt, totalBytes, usedBytes };
      } catch {
        return { expiresAt: null, totalBytes: null, usedBytes: null };
      }
    },

    _refreshSubInfoInBackground(subId: string) {
      const sub = this.subscriptions.find(s => s.id === subId);
      if (!sub) return;
      this.fetchSubscriptionInfo(sub.url).then(info => {
        const target = this.subscriptions.find(s => s.id === subId);
        if (!target) return;
        target.expiresAt = info.expiresAt;
        target.trafficTotalBytes = info.totalBytes;
        target.trafficUsedBytes = info.usedBytes;
        target.infoCheckedAt = Date.now();
        this.persistSubscriptions();
      }).catch(() => {});
    },

    _refreshAllSubInfoStale() {
      const staleThreshold = Date.now() - 6 * 60 * 60 * 1000;
      for (const sub of this.subscriptions) {
        if (!sub.infoCheckedAt || sub.infoCheckedAt < staleThreshold) {
          this._refreshSubInfoInBackground(sub.id);
        }
      }
    },

    async refreshSubscription(subId: string): Promise<RefreshResult> {
      const sub = this.subscriptions.find(s => s.id === subId);
      if (!sub) return { added: 0, removed: 0, error: 'Subscription not found' };
      if (this.refreshingSubIds.has(subId)) {
        return { added: 0, removed: 0, error: 'Already refreshing' };
      }
      this.refreshingSubIds.add(subId);
      try {
        const fresh = await this.fetchSubscriptionPreview(sub.url);
        if (fresh.servers.length === 0) {
          return { added: 0, removed: 0, error: 'Subscription returned no servers' };
        }
        const oldServers = sub.servers;
        const oldByUrl = new Map(oldServers.map(s => [s.url, s]));
        const newUrls = new Set(fresh.servers.map(s => s.url));
        let addedCount = 0;
        const mergedServers: ServerEntry[] = fresh.servers.map((freshSrv, i) => {
          const existing = oldByUrl.get(freshSrv.url);
          if (existing) {
            return {
              ...existing,
              name: freshSrv.name,
              protocol: freshSrv.protocol,
              server: freshSrv.server,
              countryCode: existing.countryCode !== 'UN' ? existing.countryCode : freshSrv.countryCode,
            };
          }
          addedCount++;
          return { ...freshSrv, id: `${subId}-srv-${Date.now()}-${i}` };
        });
        const removedCount = oldServers.filter(s => !newUrls.has(s.url)).length;
        const removedIds = new Set(oldServers.filter(s => !newUrls.has(s.url)).map(s => s.id));
        sub.servers = mergedServers;
        const info = await this.fetchSubscriptionInfo(sub.url);
        sub.expiresAt = info.expiresAt;
        sub.trafficTotalBytes = info.totalBytes;
        sub.trafficUsedBytes = info.usedBytes;
        sub.infoCheckedAt = Date.now();
        this.persistSubscriptions();
        if (this.selectedServerId && removedIds.has(this.selectedServerId)) {
          const fallback = mergedServers[0];
          this.selectedServerId = fallback ? fallback.id : null;
          this.selectedSubId = fallback ? subId : null;
          this.persistSelectedServer();
        }
        if (this.selectedEntryServerId && removedIds.has(this.selectedEntryServerId)) {
          this.selectEntryServer(null);
        }
        const unresolvedNew = mergedServers.filter(s => s.countryCode === 'UN');
        this._resolveCountriesInBackground(unresolvedNew);
        return { added: addedCount, removed: removedCount, error: null };
      } catch (e) {
        return { added: 0, removed: 0, error: String(e) };
      } finally {
        this.refreshingSubIds.delete(subId);
      }
    },

    async measureLatencies(subId?: string) {
      const pool = subId
        ? (this.subscriptions.find(s => s.id === subId)?.servers ?? [])
        : this.allServers;
      if (pool.length === 0) return;
      this.latencyLoading = true;
      try {
        const targets = pool.map(s => ({
          host: s.server,
          port: parseServerPort(s.url),
        }));
        const results = await invoke<{ host: string; port: number; latency_ms: number | null }[]>(
          'ping_servers',
          { targets }
        );
        const latencyMap = new Map(results.map(r => [r.host, r.latency_ms]));
        for (const sub of this.subscriptions) {
          let dirty = false;
          for (const srv of sub.servers) {
            if (!pool.find(p => p.id === srv.id)) continue;
            const lat = latencyMap.get(srv.server) ?? null;
            if (srv.latencyMs !== lat) {
              srv.latencyMs = lat;
              dirty = true;
            }
          }
          if (dirty) this.persistSubscriptions();
        }
      } finally {
        this.latencyLoading = false;
      }
    },

    async autoSelectFastest(subId?: string) {
      const pool = subId
        ? (this.subscriptions.find(s => s.id === subId)?.servers ?? [])
        : this.allServers;
      if (pool.length === 0) return;
      this.autoSelectLoading = true;
      try {
        const targets = pool.map(s => ({
          host: s.server,
          port: parseServerPort(s.url),
        }));
        const results = await invoke<{ host: string; port: number; latency_ms: number | null }[]>(
          'ping_servers',
          { targets }
        );
        const latencyMap = new Map(results.map(r => [r.host, r.latency_ms]));
        for (const sub of this.subscriptions) {
          for (const srv of sub.servers) {
            const lat = latencyMap.get(srv.server) ?? null;
            if (srv.latencyMs !== lat) srv.latencyMs = lat;
          }
        }
        this.persistSubscriptions();
        let bestMs = Infinity;
        let bestId: string | null = null;
        for (const r of results) {
          if (r.latency_ms === null) continue;
          const srv = pool.find(s => s.server === r.host);
          if (!srv) continue;
          if (this.isServerExpired(srv.id)) continue;
          if (r.latency_ms < bestMs) {
            bestMs = r.latency_ms;
            bestId = srv.id;
          }
        }
        if (bestId) this.selectServer(bestId);
      } finally {
        this.autoSelectLoading = false;
      }
    },

    _resolveCountriesInBackground(servers: ServerEntry[]) {
      const hosts = [...new Set(
        servers.filter(s => s.countryCode === 'UN').map(s => s.server)
      )];
      if (hosts.length === 0) return;
      resolveHostsToCountries(hosts).then(codeMap => {
        for (const sub of this.subscriptions) {
          let dirty = false;
          for (const srv of sub.servers) {
            if (srv.countryCode !== 'UN') continue;
            const code = codeMap.get(srv.server);
            if (code && code !== srv.countryCode) {
              srv.countryCode = code;
              dirty = true;
            }
          }
          if (dirty) this.persistSubscriptions();
        }
      }).catch(() => {});
    },

    addSubscription(subUrl: string, name: string, servers: ServerEntry[]) {
      const subId = `sub-${Date.now()}`;
      const finalServers = servers.map((s, i) => ({ ...s, id: `${subId}-srv-${i}` }));
      this.subscriptions.push({
        id: subId,
        name,
        url: subUrl,
        addedAt: Date.now(),
        servers: finalServers,
        expiresAt: null,
        trafficTotalBytes: null,
        trafficUsedBytes: null,
        infoCheckedAt: null,
      });
      if (!this.selectedServerId && finalServers.length > 0) {
        this.selectedServerId = finalServers[0].id;
        this.selectedSubId = subId;
        this.persistSelectedServer();
      }
      this.persistSubscriptions();
      this._resolveCountriesInBackground(finalServers);
      this._refreshSubInfoInBackground(subId);
      void this.measureLatencies(subId);
    },

    removeSubscription(subId: string) {
      const sub = this.subscriptions.find(s => s.id === subId);
      if (sub) {
        const ids = new Set(sub.servers.map(s => s.id));
        if (this.selectedServerId && ids.has(this.selectedServerId)) {
          this.selectedServerId = null;
          this.selectedSubId = null;
          this.persistSelectedServer();
        }
      }
      this.subscriptions = this.subscriptions.filter(s => s.id !== subId);
      this.persistSubscriptions();
    },

    selectSubscription(subId: string) {
      const now = Date.now();
      const sub = this.subscriptions.find(s => s.id === subId);
      if (!sub || (sub.expiresAt !== null && sub.expiresAt <= now)) return;
      this.selectedSubId = subId;
      const inSub = sub.servers.some(s => s.id === this.selectedServerId);
      if (!inSub) {
        this.selectedServerId = sub.servers.length > 0 ? sub.servers[0].id : null;
      }
      this.persistSelectedServer();
      this.syncTrayState();
    },

    selectServer(serverId: string) {
      if (this.isServerExpired(serverId)) return;
      this.selectedServerId = serverId;
      const sub = this.subscriptions.find(s => s.servers.some(srv => srv.id === serverId));
      this.selectedSubId = sub?.id ?? null;
      this.persistSelectedServer();
      this.syncTrayState();
    },

    persistSubscriptions() {
      try { localStorage.setItem('wawity_subscriptions', JSON.stringify(this.subscriptions)); } catch {}
      this.syncTrayState();
    },

    loadSubscriptions() {
      try {
        const raw = localStorage.getItem('wawity_subscriptions');
        if (raw) {
          const parsed = JSON.parse(raw) as SubscriptionGroup[];
          this.subscriptions = parsed.map(sub => ({
            ...sub,
            expiresAt: sub.expiresAt ?? null,
            trafficTotalBytes: sub.trafficTotalBytes ?? null,
            trafficUsedBytes: sub.trafficUsedBytes ?? null,
            infoCheckedAt: sub.infoCheckedAt ?? null,
          }));
        }
      } catch {}
      const knownIds = new Set(this.subscriptions.flatMap(s => s.servers.map(srv => srv.id)));
      if (this.selectedServerId && !knownIds.has(this.selectedServerId)) {
        this.selectedServerId = null;
        this.selectedSubId = null;
      }
      if (!this.selectedServerId && this.subscriptions.length > 0) {
        const now = Date.now();
        const firstSub = this.subscriptions.find(
          s => (s.expiresAt === null || s.expiresAt > now) && s.servers.length > 0,
        );
        if (firstSub) {
          this.selectedServerId = firstSub.servers[0].id;
          this.selectedSubId = firstSub.id;
        }
      }
      this.persistSelectedServer();
    },

    async applySplitRules() {
      this.persistSettings();
      this.splitDirty = false;
      if (!this.status.connected) return;
      this.splitApplying = true;
      try {
        await invoke('set_split_rules', {
          mode: this.settings.split_mode,
          processes: this.settings.split_processes,
          domains: this.settings.split_domains,
          ips: this.settings.split_ips,
        });
      } catch (err) {
        const { pushToast } = useNotifications();
        pushToast('error', 'Split tunneling failed', String(err), 6000);
        await this.refreshStatus();
      } finally {
        this.splitApplying = false;
      }
    },

    stageSplitChange() {
      this.persistSettings();
      if (this.status.connected) {
        this.splitDirty = true;
      } else {
        this.splitDirty = false;
      }
    },

    setSplitMode(mode: AppSettings['split_mode']) {
      if (this.settings.split_mode === mode) return;
      this.settings.split_mode = mode;
      this.stageSplitChange();
    },

    addSplitDomain(raw: string): boolean {
      const domain = cleanDomain(raw);
      if (!domain) return false;
      if (this.settings.split_domains.includes(domain)) return false;
      this.settings.split_domains = [...this.settings.split_domains, domain];
      this.stageSplitChange();
      return true;
    },

    addSplitProcess(raw: string): boolean {
      const name = cleanProcess(raw);
      if (!name) return false;
      if (this.settings.split_processes.includes(name)) return false;
      this.settings.split_processes = [...this.settings.split_processes, name];
      this.stageSplitChange();
      return true;
    },

    removeSplitProcess(name: string) {
      this.settings.split_processes = this.settings.split_processes.filter((item) => item !== name);
      this.stageSplitChange();
    },

    addSplitIp(raw: string): boolean {
      const cidr = cleanCidr(raw);
      if (!cidr) return false;
      if (this.settings.split_ips.includes(cidr)) return false;
      this.settings.split_ips = [...this.settings.split_ips, cidr];
      this.stageSplitChange();
      return true;
    },

    removeSplitDomain(domain: string) {
      this.settings.split_domains = this.settings.split_domains.filter((d) => d !== domain);
      this.stageSplitChange();
    },

    removeSplitIp(cidr: string) {
      this.settings.split_ips = this.settings.split_ips.filter((v) => v !== cidr);
      this.stageSplitChange();
    },

    mergeSplitDomains(domains: string[]) {
      const merged = new Set(this.settings.split_domains);
      for (const item of domains) {
        const clean = cleanDomain(item);
        if (clean) merged.add(clean);
      }
      this.settings.split_domains = [...merged];
    },

    dropSplitDomains(domains: string[]) {
      const doomed = new Set(domains.map((d) => cleanDomain(d)).filter(Boolean) as string[]);
      if (doomed.size === 0) return;
      this.settings.split_domains = this.settings.split_domains.filter((d) => !doomed.has(d));
    },

    enableSplitTemplate(id: string, domains: string[]) {
      if (this.settings.split_templates.includes(id)) return;
      this.settings.split_templates = [...this.settings.split_templates, id];
      this.mergeSplitDomains(domains);
      this.stageSplitChange();
    },

    disableSplitTemplate(id: string, domains: string[]) {
      if (!this.settings.split_templates.includes(id)) return;
      this.settings.split_templates = this.settings.split_templates.filter((t) => t !== id);
      this.dropSplitDomains(domains);
      this.stageSplitChange();
    },

    async detectBlockedServices(): Promise<BlockReport[]> {
      this.detectingBlocks = true;
      try {
        return await invoke<BlockReport[]>('detect_blocked_services');
      } catch (err) {
        const { pushToast } = useNotifications();
        pushToast('error', 'Detection failed', String(err), 6000);
        return [];
      } finally {
        this.detectingBlocks = false;
      }
    },

    clearSplitRules() {
      this.settings.split_domains = [];
      this.settings.split_ips = [];
      this.settings.split_templates = [];
      this.stageSplitChange();
    },

    async addBypassApp(path: string) {
      const normalized = normalizeWinPath(path);
      if (!normalized) return;
      const current = normalizePathList(this.settings.bypass_apps);
      if (current.some(p => p.toLowerCase() === normalized.toLowerCase())) return;
      const previous = this.settings.bypass_apps;
      this.settings.bypass_apps = [...current, normalized];
      if (this.status.connected) {
        try {
          await invoke('update_bypass_apps', { paths: this.settings.bypass_apps });
          this.persistSettings();
        } catch (err) {
          this.settings.bypass_apps = previous;
          const { pushToast } = useNotifications();
          pushToast('error', 'Split tunneling failed', String(err), 6000);
          await this.refreshStatus();
        }
      } else {
        this.persistSettings();
      }
    },

    async removeBypassApp(path: string) {
      const normalized = normalizeWinPath(path);
      if (!normalized) return;
      const current = normalizePathList(this.settings.bypass_apps);
      const filtered = current.filter(p => p.toLowerCase() !== normalized.toLowerCase());
      if (filtered.length === current.length) return;
      const previous = this.settings.bypass_apps;
      this.settings.bypass_apps = filtered;
      if (this.status.connected) {
        try {
          await invoke('update_bypass_apps', { paths: this.settings.bypass_apps });
          this.persistSettings();
        } catch (err) {
          this.settings.bypass_apps = previous;
          const { pushToast } = useNotifications();
          pushToast('error', 'Split tunneling failed', String(err), 6000);
          await this.refreshStatus();
        }
      } else {
        this.persistSettings();
      }
    },

    async addBypassApps(paths: string[]): Promise<number> {
      const normalizedNew = normalizePathList(paths);
      if (normalizedNew.length === 0) return 0;
      const current = normalizePathList(this.settings.bypass_apps);
      const currentLower = new Set(current.map(p => p.toLowerCase()));
      const toAdd = normalizedNew.filter(p => !currentLower.has(p.toLowerCase()));
      if (toAdd.length === 0) return 0;
      const previous = this.settings.bypass_apps;
      this.settings.bypass_apps = [...current, ...toAdd];
      if (this.status.connected) {
        try {
          await invoke('update_bypass_apps', { paths: this.settings.bypass_apps });
          this.persistSettings();
        } catch (err) {
          this.settings.bypass_apps = previous;
          const { pushToast } = useNotifications();
          pushToast('error', 'Split tunneling failed', String(err), 6000);
          await this.refreshStatus();
          return 0;
        }
      } else {
        this.persistSettings();
      }
      return toAdd.length;
    },

    updateSettings(patch: Partial<AppSettings>) {
      this.settings = { ...this.settings, ...patch };
      this.persistSettings();
      setLanguage(this.settings.language);
      invoke('set_app_language', { language: this.settings.language }).catch(() => {});
      this.syncDiscordPresence();
      this.syncHotkeys();
    },

    syncDiscordPresence() {
      const s = this.settings;
      const connected = !!this.status.connected;
      let subName: string | null = null;
      if (connected && this.selectedServerId) {
        const owner = this.subscriptions.find(g =>
          g.servers.some(sv => sv.id === this.selectedServerId),
        );
        subName = owner?.name ?? null;
      }
      const payload = {
        enabled: !!s.discord_rpc,
        showServer: !!s.discord_rpc_show_server,
        showSubscription: !!s.discord_rpc_show_subscription,
        connected,
        serverName: this.status.server_name ?? null,
        subscriptionName: subName,
      };
      const key = JSON.stringify(payload);
      if (key === this._presenceKey) return;
      this._presenceKey = key;
      invoke('sync_discord_presence', { payload }).catch(() => {
        this._presenceKey = '';
      });
    },

    async syncHotkeys() {
      const combo = this.settings.hotkeys_enabled ? (this.settings.hotkey_toggle || '').trim() : '';
      const key = combo || 'off';
      if (key === activeHotkeyKey) return;
      activeHotkeyKey = key;
      try {
        await invoke('sync_hotkeys', { combo: combo || null });
      } catch (err) {
        activeHotkeyKey = '';
        const { pushToast } = useNotifications();
        pushToast('error', t('toast.hotkeyFailed'), String(err), 5000);
      }
    },

    toggleTelemetry() {
      this.settings.telemetry = !this.settings.telemetry;
      setTelemetryAllowed(this.settings.telemetry);
      invoke('set_telemetry_enabled', { enabled: this.settings.telemetry }).catch(() => {});
      this.persistSettings();
    },

    persistSettings() {
      try { localStorage.setItem('wawity_settings', JSON.stringify(this.settings)); } catch {}
      this.syncTrayState();
      this.startAutoPing();
    },

    loadSettings() {
      try {
        const raw = localStorage.getItem('wawity_settings');
        if (raw) this.settings = { ...DEFAULT_SETTINGS, ...JSON.parse(raw) };
      } catch {}
      setTelemetryAllowed(this.settings.telemetry);
      invoke('set_telemetry_enabled', { enabled: this.settings.telemetry }).catch(() => {});
      setLanguage(this.settings.language);
      invoke('set_app_language', { language: this.settings.language }).catch(() => {});
      this.startAutoPing();
    },

    resetSettings() {
      this.settings = { ...DEFAULT_SETTINGS };
      this.persistSettings();
      setLanguage(this.settings.language);
      invoke('set_app_language', { language: this.settings.language }).catch(() => {});
      this.syncDiscordPresence();
      this.syncHotkeys();
    },
  },
});

export function formatSpeed(bps: number): string {
  if (bps >= 1_000_000) return `${(bps / 1_000_000).toFixed(1)} MB/s`;
  if (bps >= 1_000) return `${(bps / 1_000).toFixed(0)} KB/s`;
  return `${bps.toFixed(0)} B/s`;
}

export function formatBytes(b: number): string {
  if (b >= 1_073_741_824) return `${(b / 1_073_741_824).toFixed(2)} GB`;
  if (b >= 1_048_576) return `${(b / 1_048_576).toFixed(1)} MB`;
  if (b >= 1_024) return `${(b / 1_024).toFixed(0)} KB`;
  return `${b} B`;
}

