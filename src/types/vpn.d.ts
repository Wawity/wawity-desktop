export interface VpnStatus {
  connected: boolean;
  pid: number | null;
  server: string | null;
  kill_switch: boolean;
  interface: string | null;
  server_name: string | null;
  entry_server_name: string | null;
  multihop: boolean;
  bytes_rx: number;
  bytes_tx: number;
  speed_rx: number;
  speed_tx: number;
  always_on_locked: boolean;
}

export interface ServerEntry {
  id: string;
  name: string;
  url: string;
  protocol: string;
  server: string;
  countryCode: string;
  latencyMs: number | null;
}

export interface SubscriptionGroup {
  id: string;
  name: string;
  url: string;
  addedAt: number;
  servers: ServerEntry[];
  expiresAt: number | null;
  trafficTotalBytes: number | null;
  trafficUsedBytes: number | null;
  infoCheckedAt: number | null;
}

export interface ParsedServer {
  name: string;
  url: string;
  protocol: string;
  server: string;
}

export interface AppSettings {
  kill_switch: boolean;
  always_on: boolean;
  auto_connect: boolean;
  start_on_boot: boolean;
  lan_access: boolean;
  block_trackers: boolean;
  notifications: boolean;
  multihop_enabled: boolean;
  quantum_resistant: boolean;
  black_hole_bg: boolean;
  black_hole_detail: 'simple' | 'detailed' | 'new';
  liquid_glass: boolean;
  telemetry: boolean;
  server_view: 'list' | 'globe';
  protocol: 'auto' | 'vless' | 'shadowsocks';
  bypass_apps: string[];
  split_mode: SplitMode;
  split_domains: string[];
  split_ips: string[];
  split_processes: string[];
  split_templates: string[];
  theme: 'dark' | 'light';
  language: 'en' | 'ru';
  discord_rpc: boolean;
  discord_rpc_show_server: boolean;
  discord_rpc_show_subscription: boolean;
  hotkeys_enabled: boolean;
  hotkey_toggle: string;
  strict_route: boolean;
  allow_insecure_tls: boolean;
  auto_ping_minutes: number;
  tunnel_own_traffic: boolean;
  dns_leak_guard: boolean;
  bootstrap_dns: 'cloudflare' | 'quad9' | 'google';
  online_geolocation: boolean;
}

export interface SessionRecord {
  server_id: string;
  server_name: string;
  location: string;
  duration_seconds: number;
  bytes_total: number;
  started_at: string;
}
export type SplitMode = 'off' | 'exclude' | 'include' | 'smart';

export type BlockVerdict =
  | 'reachable'
  | 'dnspoisoned'
  | 'unresolved'
  | 'refused'
  | 'unreachable'
  | 'snireset'
  | 'snitimeout';

export interface BlockReport {
  domain: string;
  label: string;
  blocked: boolean;
  verdict: BlockVerdict;
  elapsedMs: number;
}

export interface DetectedGame {
  key: string;
  displayName: string;
  exePaths: string[];
  recommended: boolean;
  launcher: string;
  installDir: string;
}

export interface SplitTemplate {
  id: string;
  mode: SplitMode;
  domains: string[];
  ips: string[];
}

