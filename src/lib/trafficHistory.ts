

export interface TrafficDay {
  d: string;
  rx: number;
  tx: number;
}

export interface SubTrafficHistory {
  days: TrafficDay[];
  
  liveRx: number;
  liveTx: number;
}

const STORAGE_KEY = 'wawity_traffic_history';
const MAX_DAYS = 60;

function dayKey(ts = Date.now()): string {
  const dt = new Date(ts);
  const m = String(dt.getMonth() + 1).padStart(2, '0');
  const day = String(dt.getDate()).padStart(2, '0');
  return `${dt.getFullYear()}-${m}-${day}`;
}

export function loadHistory(): Record<string, SubTrafficHistory> {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return {};
    const parsed = JSON.parse(raw) as Record<string, SubTrafficHistory>;
    for (const key of Object.keys(parsed)) {
      const entry = parsed[key];
      if (!entry || !Array.isArray(entry.days)) {
        delete parsed[key];
        continue;
      }
      
      entry.days = entry.days
        .filter((d) => d && typeof d.d === 'string' && Number.isFinite(d.rx) && Number.isFinite(d.tx))
        .slice(-MAX_DAYS);
      if (!Number.isFinite(entry.liveRx)) entry.liveRx = 0;
      if (!Number.isFinite(entry.liveTx)) entry.liveTx = 0;
      if (entry.days.length === 0 && entry.liveRx === 0 && entry.liveTx === 0) {
        delete parsed[key];
      }
    }
    return parsed;
  } catch {
    return {};
  }
}

export function saveHistory(history: Record<string, SubTrafficHistory>): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(history));
  } catch {}
}

export function accumulate(
  history: Record<string, SubTrafficHistory>,
  subId: string,
  deltaRx: number,
  deltaTx: number,
): void {
  if (deltaRx <= 0 && deltaTx <= 0) return;
  let entry = history[subId];
  if (!entry) {
    entry = { days: [], liveRx: 0, liveTx: 0 };
    history[subId] = entry;
  }
  const today = dayKey();
  let bucket = entry.days[entry.days.length - 1];
  if (!bucket || bucket.d !== today) {
    
    while (entry.days.length >= MAX_DAYS) entry.days.shift();
    bucket = { d: today, rx: 0, tx: 0 };
    entry.days.push(bucket);
  }
  bucket.rx += Math.max(0, deltaRx);
  bucket.tx += Math.max(0, deltaTx);
}

export function sumDays(entry: SubTrafficHistory | undefined, days: number): { rx: number; tx: number; total: number } {
  if (!entry) return { rx: 0, tx: 0, total: 0 };
  const cutoffKeys = new Set<string>();
  const now = Date.now();
  for (let i = 0; i < days; i++) cutoffKeys.add(dayKey(now - i * 86_400_000));

  let rx = 0;
  let tx = 0;
  for (const day of entry.days) {
    if (cutoffKeys.has(day.d)) {
      rx += day.rx;
      tx += day.tx;
    }
  }
  
  if (cutoffKeys.has(dayKey())) {
    rx += entry.liveRx;
    tx += entry.liveTx;
  }
  return { rx, tx, total: rx + tx };
}
