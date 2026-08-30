import { defineStore } from 'pinia';
import { badgeForIndex } from '../lib/subicons';
import type { AutoOffPlan, DeepSample, ServerStat } from '../types/vpn';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import type {
  VpnStatus,
  AppSettings,
  SubscriptionGroup,
  ServerEntry,
  SessionRecord,
  BlockReport,
  Role,
  RoleOverrides,
  RoutingRule,
  RuleProvider,
  RuleAction,
  RuleMatchType,
  ProviderKind,
  RouteRuleSpec,
  ServerGroup,
} from '../types/vpn.d';
import { useNotifications } from '../composables/useNotifications';
import { setLanguage, t } from '../i18n';

let activeHotkeyKey = '';
let hotkeyTogglePending = false;

import { setTelemetryAllowed, track } from '../lib/telemetry';
import { setMotionLevel } from '../lib/motion';
import {
  accumulate as accumulateTraffic,
  loadHistory as loadTrafficHistory,
  saveHistory as saveTrafficHistory,
  sumDays,
  type SubTrafficHistory,
} from '../lib/trafficHistory';

const STREAMING_DOMAINS = [
  'netflix.com',
  'nflxvideo.net',
  'nflximg.net',
  'youtube.com',
  'googlevideo.com',
  'ytimg.com',
  'youtu.be',
  'twitch.tv',
  'ttvnw.net',
  'jtvnw.net',
  'disneyplus.com',
  'disney-plus.net',
  'dssott.com',
  'primevideo.com',
  'aiv-cdn.net',
  'media-amazon.com',
  'spotify.com',
  'scdn.co',
  'spotifycdn.com',
  'hbomax.com',
  'max.com',
];

const TRACKER_DOMAINS = [
  'doubleclick.net',
  'google-analytics.com',
  'googletagmanager.com',
  'googlesyndication.com',
  'adservice.google.com',
  'scorecardresearch.com',
  'app-measurement.com',
  'crashlytics.com',
  'branch.io',
  'appsflyer.com',
  'adjust.com',
  'amplitude.com',
  'mixpanel.com',
  'analytics.tiktok.com',
  'ads.yahoo.com',
];

function normalizeCidr(raw: string): string {
  const value = raw.trim();
  if (!value) return value;
  if (value.includes('/')) return value;
  if (value.includes(':')) return `${value}/128`;
  return `${value}/32`;
}

function presetRule(
  type: RuleMatchType,
  value: string,
  action: RuleAction,
  tag: string,
): RoutingRule {
  return { id: `preset-${tag}`, type, value, action };
}

const ROLE_PRESETS: Record<string, RoutingRule[]> = {
  streaming: STREAMING_DOMAINS.map((d, i) => presetRule('domainSuffix', d, 'proxy', `stream-${i}`)),
  privacy: TRACKER_DOMAINS.map((d, i) => presetRule('domainSuffix', d, 'block', `priv-${i}`)),
};

function builtinRoles(): Role[] {
  return [
    {
      id: 'standard',
      name: '',
      icon: 'Shield',
      color: '#5ee69a',
      builtin: true,
      rules: [],
      providers: [],
      overrides: {
        dpi_profile: null,
        bootstrap_dns: null,
        tunnel_own_traffic: null,
        route_all: true,
      },
    },
    {
      id: 'work',
      name: '',
      icon: 'Anchor',
      color: '#8fb6ff',
      builtin: true,
      rules: [],
      providers: [],
      overrides: {
        dpi_profile: 'medium',
        bootstrap_dns: 'quad9',
        tunnel_own_traffic: null,
        route_all: true,
      },
    },
    {
      id: 'gaming',
      name: '',
      icon: 'Zap',
      color: '#f0d36a',
      builtin: true,
      rules: [],
      providers: [],
      overrides: {
        dpi_profile: 'off',
        bootstrap_dns: null,
        tunnel_own_traffic: true,
        route_all: true,
      },
    },
    {
      id: 'streaming',
      name: '',
      icon: 'Signal',
      color: '#ff9f6b',
      builtin: true,
      rules: [],
      providers: [],
      overrides: {
        dpi_profile: 'soft',
        bootstrap_dns: null,
        tunnel_own_traffic: null,
        route_all: true,
      },
    },
    {
      id: 'privacy',
      name: '',
      icon: 'ShieldCheck',
      color: '#a78bfa',
      builtin: true,
      rules: [],
      providers: [],
      overrides: {
        dpi_profile: 'hard',
        bootstrap_dns: 'quad9',
        tunnel_own_traffic: true,
        route_all: true,
      },
    },
  ];
}

const DEFAULT_SETTINGS: AppSettings = {
  kill_switch: true,
  always_on: false,
  auto_connect: false,
  start_on_boot: false,
  telemetry: false,
  lan_access: false,
  block_trackers: true,
  notifications: false,
  multihop_enabled: false,
  quantum_resistant: true,
  ui_style: 'wawity',
      black_hole_bg: true,
  black_hole_detail: 'simple',
  liquid_glass: false,
  motion_level: 'fancy',
  server_view: 'list',
  protocol: 'auto',
  dns_remote: 'cloudflare' as const,
  dns_custom_doh: '',
  dns_block_ads: true,
  dns_block_trackers: true,
  auto_failover: false,
  streamer_mode: false,
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
  hotkey_panic: 'Ctrl+Alt+Z',
  strict_route: true,
  allow_insecure_tls: false,
  auto_ping_minutes: 0,
  tunnel_own_traffic: true,
  dns_leak_guard: true,
  bootstrap_dns: 'cloudflare',
  online_geolocation: false,
  dpi_profile: 'off',
  smart_connect: true,
  failover_enabled: false,
  failover_chain: [],
  failover_retries: 2,
  auto_off_default_minutes: 30,
  hwid_enabled: true,
  bg_custom_enabled: false,
  bg_custom_url: '',
  bg_custom_dim: 45,
  bg_custom_blur: 0,
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

const STORAGE_KEY_STATS = 'wawity_server_stats';
const STORAGE_KEY_AUTO_OFF = 'wawity_auto_off';
const EWMA_ALPHA = 0.3;

const LIVE_POLL_MS = 2000;
const HIDDEN_POLL_MS = 10000;

const KEYWORD_MAP: Record<string, string> = {
  россия: 'ru',
  российская: 'ru',
  москва: 'ru',
  russia: 'ru',
  russian: 'ru',
  moscow: 'ru',
  moskva: 'ru',
  германия: 'de',
  germany: 'de',
  german: 'de',
  frankfurt: 'de',
  berlin: 'de',
  nuremberg: 'de',
  нидерланды: 'nl',
  netherlands: 'nl',
  amsterdam: 'nl',
  holland: 'nl',
  сша: 'us',
  'соединенные штаты': 'us',
  'united states': 'us',
  usa: 'us',
  'new york': 'us',
  'los angeles': 'us',
  chicago: 'us',
  seattle: 'us',
  dallas: 'us',
  miami: 'us',
  ashburn: 'us',
  великобритания: 'gb',
  британия: 'gb',
  'united kingdom': 'gb',
  london: 'gb',
  england: 'gb',
  britain: 'gb',
  франция: 'fr',
  france: 'fr',
  paris: 'fr',
  япония: 'jp',
  japan: 'jp',
  tokyo: 'jp',
  osaka: 'jp',
  сингапур: 'sg',
  singapore: 'sg',
  канада: 'ca',
  canada: 'ca',
  toronto: 'ca',
  montreal: 'ca',
  vancouver: 'ca',
  австралия: 'au',
  australia: 'au',
  sydney: 'au',
  melbourne: 'au',
  швеция: 'se',
  sweden: 'se',
  stockholm: 'se',
  финляндия: 'fi',
  finland: 'fi',
  helsinki: 'fi',
  норвегия: 'no',
  norway: 'no',
  oslo: 'no',
  швейцария: 'ch',
  switzerland: 'ch',
  zurich: 'ch',
  geneva: 'ch',
  польша: 'pl',
  poland: 'pl',
  warsaw: 'pl',
  украина: 'ua',
  ukraine: 'ua',
  kyiv: 'ua',
  kiev: 'ua',
  турция: 'tr',
  turkey: 'tr',
  istanbul: 'tr',
  ankara: 'tr',
  бразилия: 'br',
  brazil: 'br',
  'sao paulo': 'br',
  индия: 'in',
  india: 'in',
  mumbai: 'in',
  delhi: 'in',
  китай: 'cn',
  china: 'cn',
  beijing: 'cn',
  shanghai: 'cn',
  'южная корея': 'kr',
  'south korea': 'kr',
  korea: 'kr',
  seoul: 'kr',
  гонконг: 'hk',
  'hong kong': 'hk',
  тайвань: 'tw',
  taiwan: 'tw',
  taipei: 'tw',
  испания: 'es',
  spain: 'es',
  madrid: 'es',
  barcelona: 'es',
  италия: 'it',
  italy: 'it',
  rome: 'it',
  milan: 'it',
  австрия: 'at',
  austria: 'at',
  vienna: 'at',
  чехия: 'cz',
  czech: 'cz',
  prague: 'cz',
  румыния: 'ro',
  romania: 'ro',
  bucharest: 'ro',
  венгрия: 'hu',
  hungary: 'hu',
  budapest: 'hu',
  португалия: 'pt',
  portugal: 'pt',
  lisbon: 'pt',
  аргентина: 'ar',
  argentina: 'ar',
  'buenos aires': 'ar',
  мексика: 'mx',
  mexico: 'mx',
  израиль: 'il',
  israel: 'il',
  'tel aviv': 'il',
  оаэ: 'ae',
  uae: 'ae',
  dubai: 'ae',
  'саудовская аравия': 'sa',
  saudi: 'sa',
  riyadh: 'sa',
  'южная африка': 'za',
  'south africa': 'za',
  johannesburg: 'za',
  латвия: 'lv',
  latvia: 'lv',
  riga: 'lv',
  литва: 'lt',
  lithuania: 'lt',
  vilnius: 'lt',
  эстония: 'ee',
  estonia: 'ee',
  tallinn: 'ee',
  молдова: 'md',
  moldova: 'md',
  беларусь: 'by',
  belarus: 'by',
  minsk: 'by',
  казахстан: 'kz',
  kazakhstan: 'kz',
  almaty: 'kz',
  грузия: 'ge',
  georgia: 'ge',
  tbilisi: 'ge',
  армения: 'am',
  armenia: 'am',
  yerevan: 'am',
  азербайджан: 'az',
  azerbaijan: 'az',
  baku: 'az',
  индонезия: 'id',
  indonesia: 'id',
  jakarta: 'id',
  малайзия: 'my',
  malaysia: 'my',
  'kuala lumpur': 'my',
  таиланд: 'th',
  thailand: 'th',
  bangkok: 'th',
  вьетнам: 'vn',
  vietnam: 'vn',
  hanoi: 'vn',
  филиппины: 'ph',
  philippines: 'ph',
  manila: 'ph',
  пакистан: 'pk',
  pakistan: 'pk',
  karachi: 'pk',
  египет: 'eg',
  egypt: 'eg',
  cairo: 'eg',
  дания: 'dk',
  denmark: 'dk',
  copenhagen: 'dk',
  бельгия: 'be',
  belgium: 'be',
  brussels: 'be',
  словакия: 'sk',
  slovakia: 'sk',
  bratislava: 'sk',
  болгария: 'bg',
  bulgaria: 'bg',
  sofia: 'bg',
  сербия: 'rs',
  serbia: 'rs',
  belgrade: 'rs',
  хорватия: 'hr',
  croatia: 'hr',
  zagreb: 'hr',
  греция: 'gr',
  greece: 'gr',
  athens: 'gr',
  ирландия: 'ie',
  ireland: 'ie',
  dublin: 'ie',
  люксембург: 'lu',
  luxembourg: 'lu',
  исландия: 'is',
  iceland: 'is',
  reykjavik: 'is',
  'новая зеландия': 'nz',
  'new zealand': 'nz',
  auckland: 'nz',
  чили: 'cl',
  chile: 'cl',
  santiago: 'cl',
  колумбия: 'co',
  colombia: 'co',
  bogota: 'co',
  перу: 'pe',
  peru: 'pe',
  lima: 'pe',
};

const ISO2_SET = new Set([
  'ru',
  'de',
  'nl',
  'us',
  'gb',
  'fr',
  'jp',
  'sg',
  'ca',
  'au',
  'se',
  'fi',
  'no',
  'ch',
  'pl',
  'ua',
  'tr',
  'br',
  'in',
  'cn',
  'kr',
  'hk',
  'tw',
  'es',
  'it',
  'at',
  'cz',
  'ro',
  'hu',
  'pt',
  'ar',
  'mx',
  'il',
  'ae',
  'sa',
  'za',
  'lv',
  'lt',
  'ee',
  'md',
  'by',
  'kz',
  'ge',
  'am',
  'az',
  'id',
  'my',
  'th',
  'vn',
  'ph',
  'pk',
  'eg',
  'ng',
  'dk',
  'be',
  'sk',
  'bg',
  'rs',
  'hr',
  'gr',
  'ie',
  'lu',
  'is',
  'nz',
  'cl',
  'co',
  'pe',
  'uk',
]);

const HOST_SUFFIX_BLACKLIST = new Set([
  'com',
  'net',
  'org',
  'io',
  'to',
  'cc',
  'me',
  'tv',
  'xyz',
  'info',
  'pro',
  'top',
  'site',
  'online',
  'click',
  'link',
  'host',
  'cloud',
  'app',
  'dev',
  'tech',
  'biz',
  'name',
  'one',
  'vip',
  'fun',
  'live',
  'world',
  'store',
  'space',
  'website',
  'icu',
  'vpn',
  'best',
  'win',
  'services',
  'solutions',
  'company',
  'group',
  'network',
  'systems',
  'digital',
  'media',
  'life',
]);

function stripRegistrableSuffix(hostLower: string): string[] {
  const parts = hostLower.split('.').filter(Boolean);
  if (parts.length <= 1) return parts;
  const last = parts[parts.length - 1];
  const secondLast = parts.length >= 2 ? parts[parts.length - 2] : '';
  const dropCount = last.length <= 3 && secondLast.length <= 3 && parts.length >= 3 ? 2 : 1;
  return parts.slice(0, Math.max(0, parts.length - dropCount));
}

const KEYWORD_PAIRS: Array<[string, string]> = Object.entries(KEYWORD_MAP);

export function extractCountryCode(serverHost: string, name: string): string {
  const nameLower = name.toLowerCase();
  const hostLower = serverHost.toLowerCase();
  const hay = ' ' + nameLower + ' ' + hostLower + ' ';
  for (let i = 0; i < KEYWORD_PAIRS.length; i++) {
    if (hay.includes(KEYWORD_PAIRS[i][0])) return KEYWORD_PAIRS[i][1];
  }
  const subdomainLabels = stripRegistrableSuffix(hostLower);
  for (const label of subdomainLabels) {
    for (const seg of label.split('-')) {
      if (seg.length === 2 && !HOST_SUFFIX_BLACKLIST.has(seg) && ISO2_SET.has(seg)) {
        return seg === 'uk' ? 'gb' : seg;
      }
    }
  }
  const nameTokens = nameLower.split(/[\s\-_|[\]()/\\#@.,;:]+/).filter((t) => t.length === 2);
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
    if (ch === '(') {
      open++;
      result += ch;
    } else if (ch === ')') {
      if (open > 0) {
        open--;
        result += ch;
      }
    } else result += ch;
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
    const codes = await invoke<(string | null)[]>('geolocate_servers', {
      hosts,
    });
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



function readVisualSettings(): Partial<AppSettings> {
  try {
    const raw = localStorage.getItem('wawity_settings');
    if (!raw) return {};
    const saved = JSON.parse(raw) as Partial<AppSettings>;
    const out: Partial<AppSettings> = {};
    if (saved.ui_style === 'wawity' || saved.ui_style === 'material') out.ui_style = saved.ui_style;
    if (typeof saved.liquid_glass === 'boolean') out.liquid_glass = saved.liquid_glass;
    if (saved.motion_level === 'simple' || saved.motion_level === 'fancy')
      out.motion_level = saved.motion_level;
    return out;
  } catch {
    return {};
  }
}

export const useVpnStore = defineStore('vpn', {
  state: () => ({
    status: { ...DEFAULT_STATUS } as VpnStatus,
    settings: { ...DEFAULT_SETTINGS, ...readVisualSettings() } as AppSettings,
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
    _pollHidden: false,
    _pollVisibilityBound: false,
    _presenceKey: '' as string,
    _trayKey: '' as string,
    _pingTimer: null as ReturnType<typeof setInterval> | null,
    _autoPingTimer: null as ReturnType<typeof setInterval> | null,
    _autoPingMinutes: -1 as number,
    _prevBytesRx: 0,
    _prevBytesTx: 0,
    _prevPollTs: 0,
    _reconnectTimer: null as ReturnType<typeof setTimeout> | null,
    _autoReconnectArmed: false,
    _netListenerBound: false,
    _wasConnectedBeforeSleep: false,
    _disconnectIntent: false,
    _trafficNotifiedKey: '' as string,
    serverStats: {} as Record<string, ServerStat>,
    autoOff: { mode: 'off', endsAt: null, process: '' } as AutoOffPlan,
    autoOffLeft: 0,
    smartPicking: false,
    calibrating: false,
    _autoOffTimer: null as ReturnType<typeof setInterval> | null,
    _autoOffBound: false,
    roles: [] as Role[],
    activeRoleId: 'standard' as string,
    ruleProviders: [] as RuleProvider[],
    hwid: '' as string,
    serverGroups: [] as ServerGroup[],
    hiddenServers: [] as string[],
    favorites: [] as string[],
    _failoverTries: 0,
    trafficHistory: {} as Record<string, SubTrafficHistory>,
    _trafficPersistCounter: 0,
    captureActive: false,
    _streamPoll: null as ReturnType<typeof setInterval> | null,
  }),

  getters: {
    allServers: (state): ServerEntry[] => state.subscriptions.flatMap((g) => g.servers),
    availableServers: (state): ServerEntry[] => {
      const now = Date.now();
      return state.subscriptions
        .filter((g) => g.expiresAt === null || g.expiresAt > now)
        .flatMap((g) => g.servers);
    },
    trayServers: (state): ServerEntry[] => {
      const now = Date.now();
      const live = state.subscriptions.filter((g) => g.expiresAt === null || g.expiresAt > now);
      const active = live.find((g) => g.id === state.selectedSubId);
      if (active) return active.servers;
      return live.flatMap((g) => g.servers);
    },
    serverExpiryIndex: (state): Map<string, number | null> => {
      const index = new Map<string, number | null>();
      for (const sub of state.subscriptions) {
        for (const srv of sub.servers) {
          index.set(srv.id, sub.expiresAt ?? null);
        }
      }
      return index;
    },
    isServerExpired(state): (serverId: string) => boolean {
      void state;
      const index = this.serverExpiryIndex;
      return (serverId: string): boolean => {
        const expiresAt = index.get(serverId);
        return expiresAt !== undefined && expiresAt !== null && expiresAt <= Date.now();
      };
    },
    selectedServer: (state): ServerEntry | null => {
      if (!state.selectedServerId) return null;
      for (const sub of state.subscriptions) {
        const found = sub.servers.find((s) => s.id === state.selectedServerId);
        if (found) return found;
      }
      return null;
    },

    
    serverGroupsResolved(state): { id: string; name: string; server: ServerEntry | null }[] {
      return state.serverGroups.map((group) => {
        for (const sub of state.subscriptions) {
          const found = sub.servers.find((s) => s.id === group.serverId);
          if (found) return { id: group.id, name: group.name, server: found };
        }
        return { id: group.id, name: group.name, server: null };
      });
    },

    
    favoriteServers(state): ServerEntry[] {
      const out: ServerEntry[] = [];
      for (const id of state.favorites) {
        for (const sub of state.subscriptions) {
          const found = sub.servers.find((s) => s.id === id);
          if (found) {
            out.push(found);
            break;
          }
        }
      }
      return out;
    },
    entryServer: (state): ServerEntry | null => {
      if (!state.selectedEntryServerId) return null;
      for (const sub of state.subscriptions) {
        const found = sub.servers.find((s) => s.id === state.selectedEntryServerId);
        if (found) return found;
      }
      return null;
    },
    selectedSubscription: (state): SubscriptionGroup | null => {
      if (!state.selectedSubId) return null;
      return state.subscriptions.find((s) => s.id === state.selectedSubId) ?? null;
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

    
    trafficToday(state): { name: string; rx: number; tx: number; total: number }[] {
      return state.subscriptions
        .map((sub) => {
          const s = sumDays(state.trafficHistory[sub.id], 1);
          return { name: sub.name, rx: s.rx, tx: s.tx, total: s.total };
        })
        .filter((row) => row.total > 0 || state.trafficHistory[row.name] !== undefined)
        .sort((a, b) => b.total - a.total);
    },

    trafficWeekTotal(state): { rx: number; tx: number; total: number } {
      let rx = 0;
      let tx = 0;
      for (const sub of state.subscriptions) {
        const s = sumDays(state.trafficHistory[sub.id], 7);
        rx += s.rx;
        tx += s.tx;
      }
      
      const known = new Set(state.subscriptions.map((s) => s.id));
      for (const [subId, entry] of Object.entries(state.trafficHistory)) {
        if (known.has(subId)) continue;
        const s = sumDays(entry, 7);
        rx += s.rx;
        tx += s.tx;
      }
      return { rx, tx, total: rx + tx };
    },

    
    trafficQuota(state): { used: number; total: number; left: number } | null {
      const candidates = [
        this.selectedSubscription,
        ...state.subscriptions.filter((s) => s.id !== state.selectedSubId),
      ];
      for (const sub of candidates) {
        if (!sub) continue;
        if (sub.trafficTotalBytes && sub.trafficTotalBytes > 0) {
          const used = sub.trafficUsedBytes ?? 0;
          return {
            used,
            total: sub.trafficTotalBytes,
            left: Math.max(0, sub.trafficTotalBytes - used),
          };
        }
      }
      return null;
    },
    currentPingDisplay: (state): string => {
      if (!state.status.connected) return '—';
      if (state.currentPingMs === null || state.currentPingMs === undefined)
        return t('connection.measuring');
      return `${state.currentPingMs} ms`;
    },
    badgeByServerId(state): Record<string, { icon: string; color: string }> {
      const map: Record<string, { icon: string; color: string }> = {};
      state.subscriptions.forEach((sub, idx) => {
        const fallback = badgeForIndex(idx);
        const badge = {
          icon: sub.badgeIcon || fallback.icon,
          color: sub.badgeColor || fallback.color,
        };
        sub.servers.forEach((srv) => {
          map[srv.id] = badge;
        });
      });
      return map;
    },

    badgeBySubId(state): Record<string, { icon: string; color: string }> {
      const map: Record<string, { icon: string; color: string }> = {};
      state.subscriptions.forEach((sub, idx) => {
        const fallback = badgeForIndex(idx);
        map[sub.id] = {
          icon: sub.badgeIcon || fallback.icon,
          color: sub.badgeColor || fallback.color,
        };
      });
      return map;
    },

    failoverServers(state) {
      const pool = state.subscriptions.flatMap((sub) => sub.servers);
      const picked: typeof pool = [];
      for (const id of state.settings.failover_chain) {
        const hit = pool.find((srv) => srv.id === id);
        if (hit) picked.push(hit);
      }
      return picked;
    },

    autoOffArmed(state): boolean {
      return state.autoOff.mode !== 'off';
    },

    autoOffLabel(state): string {
      if (state.autoOff.mode === 'process') return state.autoOff.process;
      if (state.autoOff.mode !== 'timer') return '';
      const left = Math.max(0, state.autoOffLeft);
      const h = Math.floor(left / 3600);
      const m = Math.floor((left % 3600) / 60);
      const sec = left % 60;
      if (h > 0) return `${h}:${String(m).padStart(2, '0')}:${String(sec).padStart(2, '0')}`;
      return `${m}:${String(sec).padStart(2, '0')}`;
    },

    activeRoleObject(state): Role | null {
      return state.roles.find((r) => r.id === state.activeRoleId) ?? null;
    },
  },
  actions: {
    async boot() {
      this.loadSettings();
      this.loadRoles();
      this.loadServerStats();
      this.initHwid();
      this.restoreAutoOff();
      this.trafficHistory = loadTrafficHistory();
      this.bindConnectivityWatchers();
      track('app_started');
      this.syncDiscordPresence();
      this.syncHotkeys();
      this.loadSelectedServer();
      this.loadSubscriptions();
      this.loadEntrySelection();
      this.loadServerGroups();
      this.loadFavorites();
      this.loadHiddenServers();
      this.applyStreamerWatch();
      listen('wawity-tray-sync', () => {
        this.loadSelectedServer();
        this.loadSubscriptions();
        this.refreshStatus().catch(() => {});
        this.syncTrayState();
      }).catch(() => {});
      listen('wawity-tray-disconnect', () => {
        this._disconnectIntent = true;
        this._cancelReconnect();
      }).catch(() => {});
      listen('wawity-hotkey-toggle', (e) => {
        if (e.payload === false) {
          this._disconnectIntent = true;
          this._cancelReconnect();
        } else {
          this._disconnectIntent = false;
        }
        this.refreshStatus().catch(() => {});
        this.syncTrayState();
      }).catch(() => {});
      listen('wawity-hotkey-panic', () => {
        this.clearDisconnectIntent();
        void this.panicNow();
      }).catch(() => {});
      listen('wawity-hotkey-error', (e) => {
        const { pushToast } = useNotifications();
        pushToast('error', t('toast.hotkeyFailed'), String(e.payload), 5000);
      }).catch(() => {});
      await this.refreshStatus();
      await this.reconcileStartOnBoot();
      if (this.settings.always_on && !this.status.connected) {
        try {
          await invoke('set_always_on', { enabled: true });
        } catch {}
        await this.refreshStatus();
      }
      if (this.settings.auto_connect && !this.status.connected && this.selectedServerId) {
        this.bootAutoConnect();
      }
      this.syncTrayState();
      this.startPolling();
      this._refreshAllSubInfoStale();
      this._autoReconnectArmed = true;
    },

    async bootAutoConnect() {
      for (let attempt = 1; attempt <= 4; attempt++) {
        if (this._disconnectIntent) return;
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
      const payload = {
        servers: this.trayServers.map((s) => ({
          id: s.id,
          name: s.name,
          url: s.url,
          countryCode: s.countryCode,
        })),
        selectedId: this.selectedServerId,
        killSwitch: this.settings.kill_switch,
        quantumResistant: this.settings.quantum_resistant,
        bypassApps: this.settings.bypass_apps,
      };
      const key = JSON.stringify(payload);
      if (key === this._trayKey) return;
      this._trayKey = key;
      invoke('sync_tray_state', payload).catch(() => {
        this._trayKey = '';
      });
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
          localStorage.setItem(
            STORAGE_KEY_SELECTED_SERVER,
            JSON.stringify({
              serverId: this.selectedServerId,
              subId: this.selectedSubId,
            }),
          );
        } else {
          localStorage.removeItem(STORAGE_KEY_SELECTED_SERVER);
        }
      } catch {}
    },

    loadSelectedServer() {
      try {
        const raw = localStorage.getItem(STORAGE_KEY_SELECTED_SERVER);
        if (!raw) return;
        const parsed = JSON.parse(raw) as {
          serverId?: string;
          subId?: string | null;
        };
        if (parsed.serverId) {
          this.selectedServerId = parsed.serverId;
          this.selectedSubId = parsed.subId ?? null;
        }
      } catch {}
    },

    startPolling() {
      if (this._statusTimer) return;
      this._pollHidden = document.hidden;
      this._statusTimer = setInterval(
        () => {
          void this.refreshStatus();
        },
        this._pollHidden ? HIDDEN_POLL_MS : LIVE_POLL_MS,
      );
      if (!this._pollVisibilityBound) {
        this._pollVisibilityBound = true;
        document.addEventListener('visibilitychange', () => {
          if (!this._statusTimer) return;
          if (document.hidden === this._pollHidden) return;
          this.stopPolling();
          this.startPolling();
        });
      }
    },

    stopPolling() {
      if (this._statusTimer) {
        clearInterval(this._statusTimer);
        this._statusTimer = null;
      }
    },

    startAutoPing() {
      const minutes = this.settings.auto_ping_minutes;
      if (this._autoPingTimer && this._autoPingMinutes === minutes) return;
      this.stopAutoPing();
      this._autoPingMinutes = minutes;
      if (!minutes || minutes <= 0) return;
      this._autoPingTimer = setInterval(() => {
        if (this.latencyLoading || this.autoSelectLoading) return;
        if (this.allServers.length === 0) return;
        void this.measureLatencies();
      }, minutes * 60_000);
    },

    stopAutoPing() {
      if (this._autoPingTimer) {
        clearInterval(this._autoPingTimer);
        this._autoPingTimer = null;
      }
      this._autoPingMinutes = -1;
    },

    _startSessionTimer() {
      this.sessionSeconds = 0;
      this.sessionStartedAt = Date.now();
      if (this._sessionTimer) clearInterval(this._sessionTimer);
      this._sessionTimer = setInterval(() => {
        this.sessionSeconds++;
      }, 1000);
      this.measureCurrentPing();
      if (this._pingTimer) clearInterval(this._pingTimer);
      this._pingTimer = setInterval(() => {
        this.measureCurrentPing();
      }, 7000);
    },

    _stopSessionTimer() {
      if (this._sessionTimer) {
        clearInterval(this._sessionTimer);
        this._sessionTimer = null;
      }
      if (this._pingTimer) {
        clearInterval(this._pingTimer);
        this._pingTimer = null;
      }
      this.sessionSeconds = 0;
      this.sessionStartedAt = null;
      this.currentPingMs = null;
    },

    async measureCurrentPing() {
      if (!this.status.connected) {
        this.currentPingMs = null;
        return;
      }
      try {
        const ms = await invoke<number | null>('measure_tunnel_latency');
        this.currentPingMs = ms;
      } catch {
        this.currentPingMs = null;
      }
    },

    async connect(serverId?: string) {
      
      
      if (this._disconnectIntent) {
        console.warn('[wawity] connect suppressed: user disconnect intent is active');
        return;
      }
      const target = serverId
        ? this.allServers.find((s) => s.id === serverId)
        : this.allServers.find((s) => s.id === this.selectedServerId);
      if (!target) {
        this.connectError = 'No server selected';
        return;
      }
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
            dpi_profile: this.settings.dpi_profile,
            strict_route: this.settings.strict_route,
            allow_insecure_tls: this.settings.allow_insecure_tls,
            tunnel_own_traffic: this.settings.tunnel_own_traffic,
            dns_leak_guard: this.settings.dns_leak_guard,
            bootstrap_dns: this.settings.bootstrap_dns,
            dns_remote: this.settings.dns_remote ?? 'cloudflare',
            dns_custom_doh:
              this.settings.dns_custom_doh && this.settings.dns_custom_doh.trim()
                ? this.settings.dns_custom_doh.trim()
                : null,
            dns_block_ads: this.settings.dns_block_ads !== false,
            dns_block_trackers: this.settings.dns_block_trackers !== false,
            route_rules: this.compileRouteRules(),
            route_all: this.activeRoleObject ? this.activeRoleObject.overrides.route_all : true,
          },
        });
        this.selectServer(target.id);
        await this.refreshStatus();
        this._startSessionTimer();
        track('vpn_connected', {
          multihop: this.settings.multihop_enabled,
          protocol: this.settings.protocol,
        });
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
      
      this._disconnectIntent = true;
      this._cancelReconnect();
      this._wasConnectedBeforeSleep = false;
      try {
        await invoke('disconnect_vpn');
        await this.refreshStatus();
        saveTrafficHistory(this.trafficHistory);
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
      const target = this.allServers.find((s) => s.id === serverId);
      if (!target) {
        this.connectError = 'Server not found';
        return;
      }
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
        this._failoverTries = 0;
        this._disconnectIntent = false;
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
        
        if (raw.connected && this._prevPollTs > 0 && this.selectedSubId) {
          const dRx = raw.bytes_rx - this._prevBytesRx;
          const dTx = raw.bytes_tx - this._prevBytesTx;
          if (dRx > 0 || dTx > 0) {
            accumulateTraffic(this.trafficHistory, this.selectedSubId, Math.max(0, dRx), Math.max(0, dTx));
            if (++this._trafficPersistCounter % 20 === 0) {
              saveTrafficHistory(this.trafficHistory);
            }
          }
        }
        this._prevBytesRx = raw.bytes_rx;
        this._prevBytesTx = raw.bytes_tx;
        this._prevPollTs = now;
        const wasConnected = this.status.connected;
        const current = this.status as unknown as Record<string, unknown>;
        const next = raw as unknown as Record<string, unknown>;
        for (const key in next) {
          if (current[key] !== next[key]) current[key] = next[key];
        }
        this.syncDiscordPresence();
        if (wasConnected && !raw.connected) {
          saveTrafficHistory(this.trafficHistory);
          this._stopSessionTimer();
          this._scheduleReconnect();
        } else if (!wasConnected && raw.connected) {
          
          this._disconnectIntent = false;
          this._cancelReconnect();
          if (this._wasConnectedBeforeSleep) {
            this._wasConnectedBeforeSleep = false;
            const { pushToast } = useNotifications();
            const name = raw.server_name ?? t('toast.unknownServer');
            pushToast('success', t('toast.reconnected'), name, 4000);
            track('vpn_reconnected');
          }
          this._startSessionTimer();
        }
        this._checkTrafficQuota();
      } catch {}
    },

    

    bindConnectivityWatchers() {
      if (this._netListenerBound) return;
      this._netListenerBound = true;

      document.addEventListener('visibilitychange', () => {
        if (!document.hidden) {
          
          void this._probeAfterWake();
        }
      });

      window.addEventListener('online', () => {
        void this._probeAfterWake();
      });

      window.addEventListener('offline', () => {
        if (navigator.onLine === false && this.status.connected) {
          
          this._scheduleReconnect();
        }
      });

      window.addEventListener('beforeunload', () => {
        saveTrafficHistory(this.trafficHistory);
      });
    },

    async _probeAfterWake() {
      await this.refreshStatus().catch(() => {});
      if (this.status.connected) return;
      
      
      window.setTimeout(() => {
        void this.refreshStatus().then(() => {
          if (!this.status.connected && this.selectedServerId) {
            this._scheduleReconnect();
          }
        });
      }, 2500);
    },

    _scheduleReconnect() {
      if (!this._autoReconnectArmed) return;
      if (!this.settings.auto_connect || !this.selectedServerId) return;
      
      if (this._disconnectIntent) return;
      if (this._reconnectTimer || this.loading || this.status.connected) return;
      if (this.settings.always_on !== true && navigator.onLine === false) return;
      this._wasConnectedBeforeSleep = true;
      this._reconnectTimer = setTimeout(() => {
        this._reconnectTimer = null;
        void this._attemptReconnect();
      }, 5000);
    },

    _cancelReconnect() {
      if (this._reconnectTimer) {
        clearTimeout(this._reconnectTimer);
        this._reconnectTimer = null;
      }
    },

    
    clearDisconnectIntent() {
      this._disconnectIntent = false;
    },

    /* ---------- streamer mode ---------- */

    applyStreamerWatch() {
      const enabled = this.settings.streamer_mode === true;
      if (!enabled) {
        if (this._streamPoll) {
          clearInterval(this._streamPoll);
          this._streamPoll = null;
        }
        this.captureActive = false;
        document.documentElement.classList.remove('streamer-on');
        return;
      }
      if (this._streamPoll) return;
      const tick = async () => {
        if (document.hidden) return;
        try {
          const on = await invoke<boolean>('stream_capture_running');
          this.captureActive = on;
          document.documentElement.classList.toggle('streamer-on', on);
        } catch {}
      };
      void tick();
      this._streamPoll = setInterval(tick, 4000);
    },

    async _attemptReconnect() {
      if (this._disconnectIntent) return;
      if (this.status.connected || this.loading || !this.selectedServerId) return;

      
      
      this._failoverTries += 1;
      if (
        this.settings.auto_failover &&
        this._failoverTries >= 2 &&
        this.favorites.length > 0
      ) {
        try {
          await this.autoSelectFastest();
          const best = this.selectedServer;
          if (best && this.favorites.includes(best.id)) {
            this._failoverTries = 0;
        this._disconnectIntent = false;
            const { pushToast } = useNotifications();
            pushToast('info', t('toast.failover'), best.name, 4000);
          }
        } catch {}
        if (this.status.connected) return;
      }

      try {
        await this.connect(this.selectedServerId);
      } catch {}
    },

    

    _checkTrafficQuota() {
      const quota = this.trafficQuota;
      if (!quota || quota.total <= 0) return;
      const leftRatio = quota.left / quota.total;
      
      let bucket: 'none' | 'low' | 'critical' | 'empty' = 'none';
      if (quota.left === 0) bucket = 'empty';
      else if (leftRatio <= 0.05) bucket = 'critical';
      else if (leftRatio <= 0.2) bucket = 'low';
      else return;

      const subName = this.selectedSubscription?.name ?? this.subscriptions[0]?.name ?? '';
      const key = `${subName}:${bucket}`;
      if (this._trafficNotifiedKey.startsWith(subName + ':')) {
        const prevLevel = this._trafficNotifiedKey.split(':')[1];
        if (
          prevLevel === 'empty' ||
          (prevLevel === 'critical' && bucket !== 'empty') ||
          (prevLevel === 'low' && bucket === 'low')
        ) {
          return; 
        }
      }
      this._trafficNotifiedKey = key;

      const { pushToast } = useNotifications();
      if (bucket === 'empty') {
        pushToast('error', t('toast.quotaEmpty'), t('toast.quotaEmptyDesc', { name: subName }), 8000);
      } else {
        const leftStr = formatBytes(quota.left);
        pushToast(
          'warning',
          t('toast.quotaLow'),
          t('toast.quotaLowDesc', { left: leftStr, name: subName }),
          7000,
        );
      }
    },

    async fetchSubscriptionPreview(url: string): Promise<{ name: string; servers: ServerEntry[] }> {
      const raw = await invoke<{ name: string; url: string; protocol: string; server: string }[]>(
        'fetch_subscription_raw',
        { url: url.trim() },
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
        try {
          return new URL(url).hostname;
        } catch {
          return url.slice(0, 30);
        }
      })();
      return { name: hostname, servers };
    },

    async fetchSubscriptionInfo(url: string): Promise<SubscriptionInfoParsed> {
      try {
        const raw = await invoke<SubscriptionInfoRaw>('get_subscription_info', {
          url: url.trim(),
        });
        const expiresAt =
          raw.expire !== null && raw.expire !== undefined ? raw.expire * 1000 : null;
        const totalBytes = raw.total ?? null;
        const hasUsage =
          (raw.upload !== null && raw.upload !== undefined) ||
          (raw.download !== null && raw.download !== undefined);
        const usedBytes = hasUsage ? (raw.upload ?? 0) + (raw.download ?? 0) : null;
        return { expiresAt, totalBytes, usedBytes };
      } catch {
        return { expiresAt: null, totalBytes: null, usedBytes: null };
      }
    },

    _refreshSubInfoInBackground(subId: string) {
      const sub = this.subscriptions.find((s) => s.id === subId);
      if (!sub) return;
      this.fetchSubscriptionInfo(sub.url)
        .then((info) => {
          const target = this.subscriptions.find((s) => s.id === subId);
          if (!target) return;
          target.expiresAt = info.expiresAt;
          target.trafficTotalBytes = info.totalBytes;
          target.trafficUsedBytes = info.usedBytes;
          target.infoCheckedAt = Date.now();
          this.persistSubscriptions();
        })
        .catch(() => {});
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
      const sub = this.subscriptions.find((s) => s.id === subId);
      if (!sub) return { added: 0, removed: 0, error: 'Subscription not found' };
      if (this.refreshingSubIds.has(subId)) {
        return { added: 0, removed: 0, error: 'Already refreshing' };
      }
      this.refreshingSubIds.add(subId);
      try {
        const fresh = await this.fetchSubscriptionPreview(sub.url);
        if (fresh.servers.length === 0) {
          return {
            added: 0,
            removed: 0,
            error: 'Subscription returned no servers',
          };
        }
        const oldServers = sub.servers;
        const oldByUrl = new Map(oldServers.map((s) => [s.url, s]));
        const newUrls = new Set(fresh.servers.map((s) => s.url));
        let addedCount = 0;
        const mergedServers: ServerEntry[] = fresh.servers.map((freshSrv, i) => {
          const existing = oldByUrl.get(freshSrv.url);
          if (existing) {
            return {
              ...existing,
              name: freshSrv.name,
              protocol: freshSrv.protocol,
              server: freshSrv.server,
              countryCode:
                existing.countryCode !== 'UN' ? existing.countryCode : freshSrv.countryCode,
            };
          }
          addedCount++;
          return { ...freshSrv, id: `${subId}-srv-${Date.now()}-${i}` };
        });
        const removedCount = oldServers.filter((s) => !newUrls.has(s.url)).length;
        const removedIds = new Set(oldServers.filter((s) => !newUrls.has(s.url)).map((s) => s.id));
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
        const unresolvedNew = mergedServers.filter((s) => s.countryCode === 'UN');
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
        ? (this.subscriptions.find((s) => s.id === subId)?.servers ?? [])
        : this.allServers;
      if (pool.length === 0) return;
      this.latencyLoading = true;
      try {
        const targets = pool.map((s) => ({
          host: s.server,
          port: parseServerPort(s.url),
        }));
        const results = await invoke<{ host: string; port: number; latency_ms: number | null }[]>(
          'ping_servers',
          { targets },
        );
        const latencyMap = new Map(results.map((r) => [r.host, r.latency_ms]));
        const poolIds = new Set(pool.map((p) => p.id));
        let dirty = false;
        for (const sub of this.subscriptions) {
          for (const srv of sub.servers) {
            if (!poolIds.has(srv.id)) continue;
            const lat = latencyMap.get(srv.server) ?? null;
            if (srv.latencyMs !== lat) {
              srv.latencyMs = lat;
              dirty = true;
            }
          }
        }
        if (dirty) this.persistSubscriptions();
      } finally {
        this.latencyLoading = false;
      }
    },

    async autoSelectFastest(subId?: string) {
      const pool = subId
        ? (this.subscriptions.find((s) => s.id === subId)?.servers ?? [])
        : this.allServers;
      if (pool.length === 0) return;
      this.autoSelectLoading = true;
      try {
        const targets = pool.map((s) => ({
          host: s.server,
          port: parseServerPort(s.url),
        }));
        const results = await invoke<{ host: string; port: number; latency_ms: number | null }[]>(
          'ping_servers',
          { targets },
        );
        const latencyMap = new Map(results.map((r) => [r.host, r.latency_ms]));
        for (const sub of this.subscriptions) {
          for (const srv of sub.servers) {
            const lat = latencyMap.get(srv.server) ?? null;
            if (srv.latencyMs !== lat) srv.latencyMs = lat;
          }
        }
        this.persistSubscriptions();
        const poolByHost = new Map<string, ServerEntry>();
        for (const srv of pool) {
          if (!poolByHost.has(srv.server)) poolByHost.set(srv.server, srv);
        }
        const expired = this.isServerExpired;
        let bestMs = Infinity;
        let bestId: string | null = null;
        for (const r of results) {
          if (r.latency_ms === null) continue;
          const srv = poolByHost.get(r.host);
          if (!srv) continue;
          if (expired(srv.id)) continue;
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
      const hosts = [
        ...new Set(servers.filter((s) => s.countryCode === 'UN').map((s) => s.server)),
      ];
      if (hosts.length === 0) return;
      resolveHostsToCountries(hosts)
        .then((codeMap) => {
          let dirty = false;
          for (const sub of this.subscriptions) {
            for (const srv of sub.servers) {
              if (srv.countryCode !== 'UN') continue;
              const code = codeMap.get(srv.server);
              if (code && code !== srv.countryCode) {
                srv.countryCode = code;
                dirty = true;
              }
            }
          }
          if (dirty) this.persistSubscriptions();
        })
        .catch(() => {});
    },

    addSubscription(subUrl: string, name: string, servers: ServerEntry[]) {
      const subId = `sub-${Date.now()}`;
      const finalServers = servers.map((s, i) => ({
        ...s,
        id: `${subId}-srv-${i}`,
      }));
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
      const sub = this.subscriptions.find((s) => s.id === subId);
      if (sub) {
        const ids = new Set(sub.servers.map((s) => s.id));
        if (this.selectedServerId && ids.has(this.selectedServerId)) {
          this.selectedServerId = null;
          this.selectedSubId = null;
          this.persistSelectedServer();
        }
      }
      this.subscriptions = this.subscriptions.filter((s) => s.id !== subId);
      this.persistSubscriptions();
    },

    selectSubscription(subId: string) {
      const now = Date.now();
      const sub = this.subscriptions.find((s) => s.id === subId);
      if (!sub || (sub.expiresAt !== null && sub.expiresAt <= now)) return;
      this.selectedSubId = subId;
      const inSub = sub.servers.some((s) => s.id === this.selectedServerId);
      if (!inSub) {
        this.selectedServerId = sub.servers.length > 0 ? sub.servers[0].id : null;
      }
      this.persistSelectedServer();
      this.syncTrayState();
    },

    selectServer(serverId: string) {
      if (this.isServerExpired(serverId)) return;
      this.selectedServerId = serverId;
      const sub = this.subscriptions.find((s) => s.servers.some((srv) => srv.id === serverId));
      this.selectedSubId = sub?.id ?? null;
      this.persistSelectedServer();
      this.syncTrayState();
    },

    persistSubscriptions() {
      try {
        localStorage.setItem('wawity_subscriptions', JSON.stringify(this.subscriptions));
      } catch {}
      this.syncTrayState();
    },

    loadSubscriptions() {
      try {
        const raw = localStorage.getItem('wawity_subscriptions');
        if (raw) {
          const parsed = JSON.parse(raw) as SubscriptionGroup[];
          this.subscriptions = parsed.map((sub) => ({
            ...sub,
            expiresAt: sub.expiresAt ?? null,
            trafficTotalBytes: sub.trafficTotalBytes ?? null,
            trafficUsedBytes: sub.trafficUsedBytes ?? null,
            infoCheckedAt: sub.infoCheckedAt ?? null,
          }));
        }
      } catch {}
      const knownIds = new Set(this.subscriptions.flatMap((s) => s.servers.map((srv) => srv.id)));
      if (this.selectedServerId && !knownIds.has(this.selectedServerId)) {
        this.selectedServerId = null;
        this.selectedSubId = null;
      }
      if (!this.selectedServerId && this.subscriptions.length > 0) {
        const now = Date.now();
        const firstSub = this.subscriptions.find(
          (s) => (s.expiresAt === null || s.expiresAt > now) && s.servers.length > 0,
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
      if (current.some((p) => p.toLowerCase() === normalized.toLowerCase())) return;
      const previous = this.settings.bypass_apps;
      this.settings.bypass_apps = [...current, normalized];
      if (this.status.connected) {
        try {
          await invoke('update_bypass_apps', {
            paths: this.settings.bypass_apps,
          });
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
      const filtered = current.filter((p) => p.toLowerCase() !== normalized.toLowerCase());
      if (filtered.length === current.length) return;
      const previous = this.settings.bypass_apps;
      this.settings.bypass_apps = filtered;
      if (this.status.connected) {
        try {
          await invoke('update_bypass_apps', {
            paths: this.settings.bypass_apps,
          });
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
      const currentLower = new Set(current.map((p) => p.toLowerCase()));
      const toAdd = normalizedNew.filter((p) => !currentLower.has(p.toLowerCase()));
      if (toAdd.length === 0) return 0;
      const previous = this.settings.bypass_apps;
      this.settings.bypass_apps = [...current, ...toAdd];
      if (this.status.connected) {
        try {
          await invoke('update_bypass_apps', {
            paths: this.settings.bypass_apps,
          });
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
      if (Object.prototype.hasOwnProperty.call(patch, 'telemetry')) {
        setTelemetryAllowed(this.settings.telemetry);
        invoke('set_telemetry_enabled', {
          enabled: this.settings.telemetry,
        }).catch(() => {});
      }
      if (Object.prototype.hasOwnProperty.call(patch, 'motion_level')) {
        setMotionLevel(this.settings.motion_level);
      }
      if (Object.prototype.hasOwnProperty.call(patch, 'streamer_mode')) {
        this.applyStreamerWatch();
      }
      if (Object.prototype.hasOwnProperty.call(patch, 'hwid_enabled')) {
        invoke('set_hwid_enabled', { enabled: this.settings.hwid_enabled }).catch(() => {});
      }
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
        const owner = this.subscriptions.find((g) =>
          g.servers.some((sv) => sv.id === this.selectedServerId),
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
        countryCode: this.selectedServer?.countryCode ?? null,
        sessionStart:
          connected && this.sessionStartedAt
            ? Math.floor(this.sessionStartedAt / 1000)
            : null,
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
      const panicCombo = this.settings.hotkeys_enabled
        ? (this.settings.hotkey_panic || '').trim()
        : '';
      const key = (combo || 'off') + '|' + (panicCombo || 'off');
      if (key === activeHotkeyKey) return;
      activeHotkeyKey = key;
      try {
        await invoke('sync_hotkeys', { combo: combo || null, panicCombo: panicCombo || null });
      } catch (err) {
        activeHotkeyKey = '';
        const { pushToast } = useNotifications();
        pushToast('error', t('toast.hotkeyFailed'), String(err), 5000);
      }
    },

    async panicNow() {
      const { pushToast } = useNotifications();
      this.updateSettings({ kill_switch: true });
      try {
        await this.disconnect();
      } catch {}
      pushToast('warning', t('toast.panicTitle'), t('toast.panicDesc'), 5000);
      try {
        const { appWindow } = await import('@tauri-apps/api/window');
        await appWindow.hide();
      } catch {}
    },

    toggleTelemetry() {
      this.settings.telemetry = !this.settings.telemetry;
      setTelemetryAllowed(this.settings.telemetry);
      invoke('set_telemetry_enabled', {
        enabled: this.settings.telemetry,
      }).catch(() => {});
      this.persistSettings();
    },

    async initHwid() {
      await invoke('set_hwid_enabled', { enabled: this.settings.hwid_enabled }).catch(() => {});
      await this.loadHwid();
    },

    async loadHwid(): Promise<string> {
      try {
        const id = await invoke<string>('get_hwid');
        this.hwid = id;
        return id;
      } catch {
        return this.hwid;
      }
    },

    async resetHwid(): Promise<string> {
      try {
        const id = await invoke<string>('reset_hwid');
        this.hwid = id;
        return id;
      } catch {
        return this.hwid;
      }
    },

    loadRoles() {
      const builtins = builtinRoles();
      let saved: {
        roles?: Role[];
        activeRoleId?: string;
        providers?: RuleProvider[];
      } | null = null;
      try {
        const raw = localStorage.getItem('wawity_roles');
        if (raw) saved = JSON.parse(raw);
      } catch {}
      if (saved && Array.isArray(saved.roles)) {
        const savedBuiltins = new Map<string, Role>(
          saved.roles.filter((r) => r.builtin).map((r) => [r.id, r]),
        );
        const merged = builtins.map((base) => {
          const sb = savedBuiltins.get(base.id);
          if (!sb) return base;
          return {
            ...base,
            rules: Array.isArray(sb.rules) ? sb.rules : base.rules,
            providers: Array.isArray(sb.providers) ? sb.providers : base.providers,
            overrides: { ...base.overrides, ...(sb.overrides || {}) },
          };
        });
        const customs = saved.roles.filter((r) => !r.builtin);
        this.roles = [...merged, ...customs];
        this.ruleProviders = Array.isArray(saved.providers) ? saved.providers : [];
        this.activeRoleId = saved.activeRoleId || 'standard';
      } else {
        this.roles = builtins;
        this.ruleProviders = [];
        this.activeRoleId = 'standard';
      }
      if (!this.roles.some((r) => r.id === this.activeRoleId)) {
        this.activeRoleId = 'standard';
      }
    },

    persistRoles() {
      try {
        localStorage.setItem(
          'wawity_roles',
          JSON.stringify({
            roles: this.roles,
            activeRoleId: this.activeRoleId,
            providers: this.ruleProviders,
          }),
        );
      } catch {}
    },

    rolePresets(roleId: string): RoutingRule[] {
      return ROLE_PRESETS[roleId] ? [...ROLE_PRESETS[roleId]] : [];
    },

    createRole(name: string): string {
      const id = `role-${Date.now()}`;
      const role: Role = {
        id,
        name: name.trim() || t('roles.untitled'),
        icon: 'Sparkles',
        color: '#a78bfa',
        builtin: false,
        rules: [],
        providers: [],
        overrides: {
          dpi_profile: null,
          bootstrap_dns: null,
          tunnel_own_traffic: null,
          route_all: true,
        },
      };
      this.roles.push(role);
      this.persistRoles();
      return id;
    },

    renameRole(id: string, name: string) {
      const role = this.roles.find((r) => r.id === id);
      if (!role || role.builtin) return;
      role.name = name.trim() || role.name;
      this.persistRoles();
    },

    deleteRole(id: string) {
      const role = this.roles.find((r) => r.id === id);
      if (!role || role.builtin) return;
      this.roles = this.roles.filter((r) => r.id !== id);
      if (this.activeRoleId === id) this.activeRoleId = 'standard';
      this.persistRoles();
    },

    setRoleOverride<K extends keyof RoleOverrides>(id: string, key: K, value: RoleOverrides[K]) {
      const role = this.roles.find((r) => r.id === id);
      if (!role) return;
      role.overrides[key] = value;
      this.persistRoles();
      if (this.activeRoleId === id) this.applyRole(id);
    },

    addRule(id: string, rule: Omit<RoutingRule, 'id'>) {
      const role = this.roles.find((r) => r.id === id);
      if (!role) return;
      const value = rule.value.trim();
      if (!value) return;
      role.rules.push({ ...rule, value, id: `rule-${Date.now()}-${role.rules.length}` });
      this.persistRoles();
      if (this.activeRoleId === id && this.status.connected) this.applyRole(id);
    },

    removeRule(id: string, ruleId: string) {
      const role = this.roles.find((r) => r.id === id);
      if (!role) return;
      role.rules = role.rules.filter((r) => r.id !== ruleId);
      this.persistRoles();
      if (this.activeRoleId === id && this.status.connected) this.applyRole(id);
    },

    async applyRole(id: string) {
      const role = this.roles.find((r) => r.id === id);
      if (!role) return;
      this.activeRoleId = id;
      const ov = role.overrides;
      if (ov.dpi_profile) this.settings.dpi_profile = ov.dpi_profile;
      if (ov.bootstrap_dns) this.settings.bootstrap_dns = ov.bootstrap_dns;
      if (ov.tunnel_own_traffic !== null && ov.tunnel_own_traffic !== undefined) {
        this.settings.tunnel_own_traffic = ov.tunnel_own_traffic;
      }
      this.persistSettings();
      this.persistRoles();
      track('role_applied', { role: role.builtin ? role.id : 'custom' });
      if (this.status.connected && this.selectedServerId) {
        await this.connect(this.selectedServerId);
      }
    },

    addProvider(input: {
      name: string;
      url: string;
      kind: ProviderKind;
      action: RuleAction;
    }): string {
      const id = `prov-${Date.now()}`;
      this.ruleProviders.push({
        id,
        name: input.name.trim() || input.url,
        url: input.url.trim(),
        kind: input.kind,
        action: input.action,
        enabled: true,
        updatedAt: null,
        count: 0,
        entries: [],
      });
      this.persistRoles();
      return id;
    },

    removeProvider(id: string) {
      this.ruleProviders = this.ruleProviders.filter((p) => p.id !== id);
      for (const role of this.roles) {
        role.providers = role.providers.filter((p) => p !== id);
      }
      this.persistRoles();
    },

    toggleProviderForRole(roleId: string, providerId: string) {
      const role = this.roles.find((r) => r.id === roleId);
      if (!role) return;
      if (role.providers.includes(providerId)) {
        role.providers = role.providers.filter((p) => p !== providerId);
      } else {
        role.providers.push(providerId);
      }
      this.persistRoles();
      if (this.activeRoleId === roleId && this.status.connected) this.applyRole(roleId);
    },

    async refreshProvider(id: string): Promise<{ ok: boolean; error?: string }> {
      const provider = this.ruleProviders.find((p) => p.id === id);
      if (!provider) return { ok: false, error: 'not found' };
      try {
        const entries = await invoke<string[]>('fetch_rule_list', { url: provider.url });
        provider.entries = entries;
        provider.count = entries.length;
        provider.updatedAt = Date.now();
        this.persistRoles();
        if (this.status.connected && this.activeRoleObject?.providers.includes(id)) {
          await this.applyRole(this.activeRoleId);
        }
        return { ok: true };
      } catch (e) {
        return { ok: false, error: String(e) };
      }
    },

    compileRouteRules(): RouteRuleSpec[] {
      const role = this.roles.find((r) => r.id === this.activeRoleId);
      if (!role) return [];
      const groups = new Map<string, { field: string; action: RuleAction; values: string[] }>();
      const add = (field: string, action: RuleAction, values: string[]) => {
        const cleaned = values.map((v) => v.trim()).filter(Boolean);
        if (!cleaned.length) return;
        const key = `${action}|${field}`;
        const existing = groups.get(key) || { field, action, values: [] };
        existing.values.push(...cleaned);
        groups.set(key, existing);
      };
      const fieldFor = (type: RuleMatchType): string => {
        if (type === 'domain') return 'domain';
        if (type === 'domainSuffix') return 'domain_suffix';
        if (type === 'domainKeyword') return 'domain_keyword';
        if (type === 'ip') return 'ip_cidr';
        return 'process_name';
      };
      const allRules = [...this.rolePresets(role.id), ...role.rules];
      for (const rule of allRules) {
        if (rule.type === 'process') continue;
        const value = rule.type === 'ip' ? normalizeCidr(rule.value) : rule.value.trim();
        add(fieldFor(rule.type), rule.action, [value]);
      }
      for (const providerId of role.providers) {
        const provider = this.ruleProviders.find((p) => p.id === providerId);
        if (!provider || !provider.enabled || !provider.entries.length) continue;
        if (provider.kind === 'ip') {
          add('ip_cidr', provider.action, provider.entries.map(normalizeCidr));
        } else {
          add('domain_suffix', provider.action, provider.entries);
        }
      }
      const order: RuleAction[] = ['block', 'direct', 'proxy'];
      const specs: RouteRuleSpec[] = [];
      for (const action of order) {
        for (const group of groups.values()) {
          if (group.action !== action) continue;
          const spec: RouteRuleSpec = { action };
          (spec as Record<string, unknown>)[group.field] = Array.from(new Set(group.values));
          specs.push(spec);
        }
      }
      return specs;
    },

    persistSettings() {
      try {
        localStorage.setItem('wawity_settings', JSON.stringify(this.settings));
      } catch {}
      this.syncTrayState();
      this.startAutoPing();
    },

    loadSettings() {
      try {
        const raw = localStorage.getItem('wawity_settings');
        if (raw) this.settings = { ...DEFAULT_SETTINGS, ...JSON.parse(raw) };
      } catch {}
      setTelemetryAllowed(this.settings.telemetry);
      invoke('set_telemetry_enabled', {
        enabled: this.settings.telemetry,
      }).catch(() => {});
      setLanguage(this.settings.language);
      invoke('set_app_language', { language: this.settings.language }).catch(() => {});
      this.startAutoPing();
      setMotionLevel(this.settings.motion_level ?? 'fancy');
    },

    resetSettings() {
      this.settings = { ...DEFAULT_SETTINGS };
      this.persistSettings();
      setLanguage(this.settings.language);
      invoke('set_app_language', { language: this.settings.language }).catch(() => {});
      this.syncDiscordPresence();
      this.syncHotkeys();
    },

    loadServerStats() {
      try {
        const raw = localStorage.getItem(STORAGE_KEY_STATS);
        this.serverStats = raw ? JSON.parse(raw) : {};
      } catch {
        this.serverStats = {};
      }
    },

    

    loadServerGroups() {
      try {
        const raw = localStorage.getItem('wawity_server_groups');
        if (raw) {
          const parsed = JSON.parse(raw) as ServerGroup[];
          this.serverGroups = Array.isArray(parsed)
            ? parsed.filter((g) => g && typeof g.id === 'string' && typeof g.serverId === 'string')
            : [];
        }
      } catch {}
    },

    persistServerGroups() {
      try {
        localStorage.setItem('wawity_server_groups', JSON.stringify(this.serverGroups));
      } catch {}
    },

    addServerGroup(name: string, serverId: string): boolean {
      const clean = name.trim().slice(0, 24);
      if (!clean || !serverId) return false;
      if (this.serverGroups.length >= 8) return false;
      if (this.serverGroups.some((g) => g.name.toLowerCase() === clean.toLowerCase())) return false;
      this.serverGroups.push({ id: `grp-${Date.now()}`, name: clean, serverId });
      this.persistServerGroups();
      return true;
    },

    renameServerGroup(id: string, name: string) {
      const group = this.serverGroups.find((g) => g.id === id);
      const clean = name.trim().slice(0, 24);
      if (!group || !clean) return;
      group.name = clean;
      this.persistServerGroups();
    },

    setGroupTarget(id: string, serverId: string) {
      const group = this.serverGroups.find((g) => g.id === id);
      if (!group || !serverId) return;
      group.serverId = serverId;
      this.persistServerGroups();
    },

    removeServerGroup(id: string) {
      this.serverGroups = this.serverGroups.filter((g) => g.id !== id);
      this.persistServerGroups();
    },

    
    async activateServerGroup(groupId: string) {
      const group = this.serverGroups.find((g) => g.id === groupId);
      if (!group) return;
      if (group.serverId === this.selectedServerId) return;
      if (this.status.connected) {
        await this.switchServer(group.serverId);
      } else {
        this.selectServer(group.serverId);
      }
    },

    

    loadFavorites() {
      try {
        const raw = localStorage.getItem('wawity_favorites');
        if (raw) {
          const parsed = JSON.parse(raw) as string[];
          this.favorites = Array.isArray(parsed) ? parsed.filter((id) => typeof id === 'string') : [];
        }
      } catch {}
    },

    loadHiddenServers() {
      try {
        const raw = localStorage.getItem('wawity_hidden_servers');
        if (raw) {
          const parsed = JSON.parse(raw) as string[];
          this.hiddenServers = Array.isArray(parsed) ? parsed.filter((id) => typeof id === 'string') : [];
        }
      } catch {}
    },

    toggleHideServer(serverId: string) {
      if (this.hiddenServers.includes(serverId)) {
        this.hiddenServers = this.hiddenServers.filter((id) => id !== serverId);
      } else {
        this.hiddenServers = [...this.hiddenServers, serverId];
      }
      try {
        localStorage.setItem('wawity_hidden_servers', JSON.stringify(this.hiddenServers));
      } catch {}
    },

    persistFavorites() {
      try {
        localStorage.setItem('wawity_favorites', JSON.stringify(this.favorites));
      } catch {}
    },

    isFavorite(serverId: string): boolean {
      return this.favorites.includes(serverId);
    },

    toggleFavorite(serverId: string) {
      if (!serverId) return;
      if (this.favorites.includes(serverId)) {
        this.favorites = this.favorites.filter((id) => id !== serverId);
      } else {
        this.favorites = [...this.favorites, serverId];
      }
      this.persistFavorites();
    },

    async activateFavorite(serverId: string) {
      if (!serverId) return;
      if (this.status.connected) {
        await this.switchServer(serverId);
      } else {
        this.selectServer(serverId);
      }
    },

    persistServerStats() {
      try {
        localStorage.setItem(STORAGE_KEY_STATS, JSON.stringify(this.serverStats));
      } catch {}
    },

    statFor(id: string): ServerStat {
      const found = this.serverStats[id];
      if (found) return found;
      const fresh: ServerStat = {
        id,
        ewmaMs: 0,
        jitterMs: 0,
        attempts: 0,
        drops: 0,
        lastOkAt: 0,
        score: 0,
      };
      this.serverStats[id] = fresh;
      return fresh;
    },

    noteLatency(id: string, ms: number) {
      if (!id || ms <= 0) return;
      const stat = this.statFor(id);
      const prev = stat.ewmaMs;
      stat.ewmaMs = prev ? prev * (1 - EWMA_ALPHA) + ms * EWMA_ALPHA : ms;
      stat.jitterMs = prev ? stat.jitterMs * 0.7 + Math.abs(ms - prev) * 0.3 : 0;
      stat.lastOkAt = Date.now();
      this.persistServerStats();
    },

    noteAttempt(id: string) {
      if (!id) return;
      this.statFor(id).attempts += 1;
      this.persistServerStats();
    },

    noteDrop(id: string) {
      if (!id) return;
      this.statFor(id).drops += 1;
      this.persistServerStats();
    },

    scoreServer(srv: { id: string; latencyMs?: number | null }): number {
      const stat = this.serverStats[srv.id];
      const live = typeof srv.latencyMs === 'number' && srv.latencyMs > 0 ? srv.latencyMs : 0;
      const base =
        stat && stat.ewmaMs > 0 ? stat.ewmaMs * 0.6 + (live || stat.ewmaMs) * 0.4 : live || 850;
      let score = 100 - Math.min(52, base / 7);
      if (stat) {
        score -= Math.min(14, stat.jitterMs / 3);
        const rate = stat.attempts ? stat.drops / stat.attempts : 0;
        score -= rate * 34;
        if (stat.lastOkAt && Date.now() - stat.lastOkAt < 3_600_000) score += 4;
      }
      return Math.max(0, Math.round(score * 10) / 10);
    },

    rankServers<T extends { id: string; latencyMs?: number | null }>(list: T[]): T[] {
      return [...list].sort((a, b) => this.scoreServer(b) - this.scoreServer(a));
    },

    async deepProbe(list: Array<{ id: string; server: string }>): Promise<DeepSample[]> {
      const targets = list
        .map((srv) => {
          const raw = (srv.server || '').trim();
          const cut = raw.lastIndexOf(':');
          const host = cut > 0 ? raw.slice(0, cut) : raw;
          const port = cut > 0 ? Number(raw.slice(cut + 1)) || 443 : 443;
          return { id: srv.id, host, port, sni: host };
        })
        .filter((target) => target.host.length > 2);
      if (!targets.length) return [];
      return await invoke<DeepSample[]>('probe_servers_deep', { targets });
    },

    async smartSelect() {
      if (this.smartPicking) return;
      this.smartPicking = true;
      this.autoSelectLoading = true;
      try {
        const pool = this.availableServers as Array<any>;
        if (!pool.length) return;
        const shortlist = this.rankServers(pool).slice(0, 14);
        let samples: DeepSample[] = [];
        try {
          samples = await this.deepProbe(shortlist);
        } catch {
          samples = [];
        }
        const live = new Map(samples.map((row) => [row.id, row]));
        let best = shortlist[0];
        let bestScore = -999;
        for (const srv of shortlist) {
          const probe = live.get(srv.id);
          let score = this.scoreServer(srv);
          if (probe) {
            if (!probe.reachable) {
              score -= 60;
            } else {
              score = score * 0.45 + probe.score * 0.55;
              score -= Math.min(12, probe.jitterMs / 3);
              score -= probe.loss * 25;
              const stat = this.statFor(srv.id);
              stat.ewmaMs = stat.ewmaMs
                ? stat.ewmaMs * (1 - EWMA_ALPHA) + probe.bestMs * EWMA_ALPHA
                : probe.bestMs;
              stat.jitterMs = probe.jitterMs;
              stat.lastOkAt = Date.now();
            }
          }
          this.statFor(srv.id).score = Math.round(score * 10) / 10;
          if (score > bestScore) {
            bestScore = score;
            best = srv;
          }
        }
        this.persistServerStats();
        if (!best) return;
        this.selectedServerId = best.id;
        try {
          localStorage.setItem(STORAGE_KEY_SELECTED_SERVER, best.id);
        } catch {}
        if (this.status.connected) {
          try {
            await this.switchServer(best.id);
          } catch {}
        }
      } finally {
        this.smartPicking = false;
        this.autoSelectLoading = false;
      }
    },

    async autoSelectSmartOrFast() {
      if (this.settings.smart_connect) {
        await this.smartSelect();
        return;
      }
      await this.autoSelectFastest();
    },

    async calibrateFailover() {
      if (this.calibrating) return;
      this.calibrating = true;
      try {
        const pool = this.availableServers as Array<any>;
        if (!pool.length) return;
        const shortlist = this.rankServers(pool).slice(0, 18);
        let samples: DeepSample[] = [];
        try {
          samples = await this.deepProbe(shortlist);
        } catch {
          samples = [];
        }
        const live = new Map(samples.map((row) => [row.id, row]));
        const ranked = shortlist
          .map((srv) => {
            const probe = live.get(srv.id);
            const own = this.scoreServer(srv);
            const score = probe && probe.reachable ? probe.score * 0.6 + own * 0.4 : own - 40;
            return { id: srv.id, score, land: String(srv.countryCode || '') };
          })
          .sort((a, b) => b.score - a.score);
        const chain: string[] = [];
        const lands = new Set<string>();
        for (const row of ranked) {
          if (chain.length >= 4) break;
          if (row.land && lands.has(row.land) && chain.length < 3) continue;
          if (row.land) lands.add(row.land);
          chain.push(row.id);
        }
        this.settings.failover_chain = chain;
        this.settings.failover_enabled = chain.length > 1;
        this.persistSettings();
      } finally {
        this.calibrating = false;
      }
    },

    addFailoverEntry(id: string) {
      if (!id || this.settings.failover_chain.includes(id)) return;
      this.settings.failover_chain = [...this.settings.failover_chain, id];
      this.persistSettings();
    },

    removeFailoverEntry(id: string) {
      this.settings.failover_chain = this.settings.failover_chain.filter((row) => row !== id);
      if (this.settings.failover_chain.length < 2) this.settings.failover_enabled = false;
      this.persistSettings();
    },

    moveFailoverEntry(id: string, delta: number) {
      const chain = [...this.settings.failover_chain];
      const at = chain.indexOf(id);
      const to = at + delta;
      if (at < 0 || to < 0 || to >= chain.length) return;
      chain.splice(to, 0, chain.splice(at, 1)[0]);
      this.settings.failover_chain = chain;
      this.persistSettings();
    },

    async connectWithChain() {
      const chain = [this.selectedServerId, ...this.settings.failover_chain].filter(
        (id): id is string => Boolean(id),
      );
      if (!this.settings.failover_enabled || chain.length < 2) {
        if (this.selectedServerId) this.noteAttempt(this.selectedServerId);
        await this.connect();
        return;
      }
      const seen = new Set<string>();
      const limit = Math.max(1, this.settings.failover_retries) + 1;
      let tries = 0;
      for (const id of chain) {
        
        if (this._disconnectIntent || this.loading) return;
        if (seen.has(id)) continue;
        seen.add(id);
        if (tries >= limit) break;
        tries += 1;
        this.selectedServerId = id;
        try {
          localStorage.setItem(STORAGE_KEY_SELECTED_SERVER, id);
        } catch {}
        this.noteAttempt(id);
        await this.connect();
        if (this.status.connected) return;
        this.noteDrop(id);
        await new Promise((done) => setTimeout(done, 450));
        if (this._disconnectIntent) return;
      }
    },

    setSubscriptionBadge(subId: string, icon: string, color: string) {
      const sub = this.subscriptions.find((row) => row.id === subId);
      if (!sub) return;
      sub.badgeIcon = icon;
      sub.badgeColor = color;
      try {
        localStorage.setItem('wawity_subscriptions', JSON.stringify(this.subscriptions));
      } catch {}
    },

    restoreAutoOff() {
      try {
        const raw = localStorage.getItem(STORAGE_KEY_AUTO_OFF);
        if (!raw) return;
        const saved = JSON.parse(raw) as AutoOffPlan;
        if (!saved || saved.mode === 'off') return;
        if (saved.mode === 'timer' && (!saved.endsAt || saved.endsAt <= Date.now())) {
          localStorage.removeItem(STORAGE_KEY_AUTO_OFF);
          return;
        }
        this.autoOff = saved;
        if (saved.mode === 'process' && saved.process) {
          invoke('arm_app_watch', { process: saved.process }).catch(() => {});
        }
        this.bindAutoOff();
      } catch {}
    },

    persistAutoOff() {
      try {
        if (this.autoOff.mode === 'off') localStorage.removeItem(STORAGE_KEY_AUTO_OFF);
        else localStorage.setItem(STORAGE_KEY_AUTO_OFF, JSON.stringify(this.autoOff));
      } catch {}
    },

    armAutoOffTimer(minutes: number) {
      const span = Math.max(1, Math.round(minutes));
      this.autoOff = {
        mode: 'timer',
        endsAt: Date.now() + span * 60_000,
        process: '',
      };
      this.autoOffLeft = span * 60;
      this.persistAutoOff();
      this.bindAutoOff();
    },

    async armAutoOffProcess(target: string) {
      const leaf = target.trim();
      if (!leaf) return;
      await invoke('arm_app_watch', { process: leaf });
      this.autoOff = { mode: 'process', endsAt: null, process: leaf };
      this.autoOffLeft = 0;
      this.persistAutoOff();
      this.bindAutoOff();
    },

    async disarmAutoOff() {
      if (this.autoOff.mode === 'process') {
        await invoke('disarm_app_watch').catch(() => {});
      }
      this.autoOff = { mode: 'off', endsAt: null, process: '' };
      this.autoOffLeft = 0;
      if (this._autoOffTimer) {
        clearInterval(this._autoOffTimer);
        this._autoOffTimer = null;
      }
      this.persistAutoOff();
    },

    tickAutoOff() {
      if (this.autoOff.mode !== 'timer' || !this.autoOff.endsAt) return;
      const left = Math.max(0, Math.round((this.autoOff.endsAt - Date.now()) / 1000));
      this.autoOffLeft = left;
      if (left > 0) return;
      this.disarmAutoOff();
      if (this.status.connected) this.disconnect();
    },

    bindAutoOff() {
      if (!this._autoOffTimer) {
        this._autoOffTimer = setInterval(() => this.tickAutoOff(), 1000);
      }
      if (this._autoOffBound) return;
      this._autoOffBound = true;
      listen('wawity-watched-app-closed', () => {
        this.disarmAutoOff();
        if (this.status.connected) this.disconnect();
      }).catch(() => {
        this._autoOffBound = false;
      });
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
