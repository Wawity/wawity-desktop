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
  badgeIcon: string;
  badgeColor: string;
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
  ui_style: 'wawity' | 'material';
  motion_level: 'simple' | 'fancy';
  language: 'en' | 'ru';
  discord_rpc: boolean;
  discord_rpc_show_server: boolean;
  discord_rpc_show_subscription: boolean;
  hotkeys_enabled: boolean;
  hotkey_panic: string;
  strict_route: boolean;
  allow_insecure_tls: boolean;
  auto_ping_minutes: number;
  tunnel_own_traffic: boolean;
  dns_leak_guard: boolean;
  bootstrap_dns:
    | 'cloudflare'
    | 'quad9'
    | 'google'
    | 'mullvad'
    | 'dns_sbi'
    | 'dns_sby'
    | 'digitale'
    | 'yandex';
  dns_remote:
    | 'cloudflare'
    | 'google'
    | 'quad9'
    | 'adguard'
    | 'mullvad'
    | 'dns_sbi'
    | 'dns_sby'
    | 'digitale'
    | 'yandex';
  dns_custom_doh: string;
  dns_block_ads: boolean;
  dns_block_trackers: boolean;
  auto_failover: boolean;
  steam_fix: boolean;
  streamer_mode: boolean;
  online_geolocation: boolean;
  dpi_profile: DpiProfile;
  smart_connect: boolean;
  failover_enabled: boolean;
  failover_chain: string[];
  failover_retries: number;
  auto_off_default_minutes: number;
  hwid_enabled: boolean;
  bg_custom_enabled: boolean;
  bg_custom_url: string;
  bg_custom_dim: number;
  bg_custom_blur: number;
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
  category: string;
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

export type DpiProfile = 'off' | 'soft' | 'medium' | 'hard';

export type AutoOffMode = 'off' | 'timer' | 'process';

export interface AutoOffPlan {
  mode: AutoOffMode;
  endsAt: number | null;
  process: string;
}

export interface SpeedTick {
  phase: string;
  mbps: number;
  progress: number;
  transferred: number;
  elapsedMs: number;
}

export interface SpeedResult {
  downloadMbps: number;
  uploadMbps: number;
  pingMs: number;
  jitterMs: number;
  loss: number;
  colo: string;
  exitIp: string;
  carrier: string;
  country: string;
  downBytes: number;
  upBytes: number;
  tookMs: number;
  aborted: boolean;
}

export interface ResolverHop {
  ip: string;
  country: string;
  carrier: string;
}

export interface LeakAudit {
  exitIp: string;
  exitCountry: string;
  carrier: string;
  colo: string;
  ipv6: string;
  ipv6Exposed: boolean;
  resolvers: ResolverHop[];
  resolverCountries: string[];
  dnsOutsideTunnel: boolean;
  resolverCount: number;
  tookMs: number;
}

export interface DeepSample {
  id: string;
  reachable: boolean;
  connectMs: number;
  bestMs: number;
  jitterMs: number;
  loss: number;
  handshakeMs: number;
  score: number;
}

export interface ServerStat {
  id: string;
  ewmaMs: number;
  jitterMs: number;
  attempts: number;
  drops: number;
  lastOkAt: number;
  score: number;
}

export interface WatchState {
  armed: boolean;
  process: string;
  running: boolean;
}

export type RuleAction = 'proxy' | 'direct' | 'block';

export type RuleMatchType = 'domain' | 'domainSuffix' | 'domainKeyword' | 'ip' | 'process';

export interface RoutingRule {
  id: string;
  type: RuleMatchType;
  value: string;
  action: RuleAction;
}

export type ProviderKind = 'domain' | 'ip';

export interface RuleProvider {
  id: string;
  name: string;
  url: string;
  kind: ProviderKind;
  action: RuleAction;
  enabled: boolean;
  updatedAt: number | null;
  count: number;
  entries: string[];
}

export interface RoleOverrides {
  dpi_profile: DpiProfile | null;
  bootstrap_dns: AppSettings['bootstrap_dns'] | null;
  tunnel_own_traffic: boolean | null;
  route_all: boolean;
}

export interface Role {
  id: string;
  name: string;
  icon: string;
  color: string;
  builtin: boolean;
  rules: RoutingRule[];
  providers: string[];
  overrides: RoleOverrides;
}

export interface RouteRuleSpec {
  domain?: string[];
  domain_suffix?: string[];
  domain_keyword?: string[];
  ip_cidr?: string[];
  process_name?: string[];
  action: RuleAction;
}

export interface ServerGroup {
  id: string;
  name: string;
  serverId: string;
}
