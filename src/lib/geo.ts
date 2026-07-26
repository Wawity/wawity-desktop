import type { SubscriptionGroup } from '../types/vpn';

export type ServerEntry = SubscriptionGroup['servers'][number];

export type CountryBucket = {
  code: string;
  name: string;
  servers: ServerEntry[];
  best: number | null;
  lat: number;
  lon: number;
};

const capitals: Record<string, [number, number]> = {
  AD: [42.51, 1.52], AE: [24.45, 54.38], AF: [34.53, 69.17], AG: [17.12, -61.85],
  AL: [41.33, 19.82], AM: [40.18, 44.51], AO: [-8.84, 13.23], AR: [-34.6, -58.38],
  AT: [48.21, 16.37], AU: [-35.28, 149.13], AW: [12.53, -70.03], AZ: [40.41, 49.87],
  BA: [43.85, 18.36], BB: [13.1, -59.62], BD: [23.81, 90.41], BE: [50.85, 4.35],
  BF: [12.37, -1.52], BG: [42.7, 23.32], BH: [26.23, 50.59], BI: [-3.43, 29.93],
  BJ: [6.5, 2.6], BM: [32.29, -64.78], BN: [4.94, 114.95], BO: [-16.5, -68.15],
  BR: [-15.79, -47.88], BS: [25.05, -77.35], BT: [27.47, 89.64], BW: [-24.63, 25.92],
  BY: [53.9, 27.57], BZ: [17.25, -88.77], CA: [45.42, -75.7], CD: [-4.44, 15.27],
  CF: [4.39, 18.56], CG: [-4.26, 15.28], CH: [46.95, 7.45], CI: [6.83, -5.29],
  CL: [-33.45, -70.67], CM: [3.87, 11.52], CN: [39.9, 116.41], CO: [4.71, -74.07],
  CR: [9.93, -84.08], CU: [23.11, -82.37], CV: [14.93, -23.51], CW: [12.11, -68.93],
  CY: [35.19, 33.38], CZ: [50.08, 14.44], DE: [52.52, 13.41], DJ: [11.59, 43.15],
  DK: [55.68, 12.57], DM: [15.31, -61.39], DO: [18.49, -69.93], DZ: [36.75, 3.06],
  EC: [-0.18, -78.47], EE: [59.44, 24.75], EG: [30.04, 31.24], ER: [15.32, 38.93],
  ES: [40.42, -3.7], ET: [9.03, 38.74], FI: [60.17, 24.94], FJ: [-18.14, 178.44],
  FM: [6.92, 158.16], FO: [62.01, -6.77], FR: [48.86, 2.35], GA: [0.42, 9.47],
  GB: [51.51, -0.13], GD: [12.06, -61.75], GE: [41.72, 44.79], GG: [49.46, -2.54],
  GH: [5.6, -0.19], GI: [36.14, -5.35], GL: [64.18, -51.72], GM: [13.45, -16.58],
  GN: [9.64, -13.58], GQ: [3.75, 8.78], GR: [37.98, 23.73], GT: [14.63, -90.51],
  GW: [11.86, -15.6], GY: [6.8, -58.16], HK: [22.32, 114.17], HN: [14.07, -87.19],
  HR: [45.81, 15.98], HT: [18.54, -72.34], HU: [47.5, 19.04], ID: [-6.21, 106.85],
  IE: [53.35, -6.26], IL: [31.77, 35.21], IM: [54.15, -4.48], IN: [28.61, 77.21],
  IQ: [33.31, 44.36], IR: [35.69, 51.39], IS: [64.15, -21.94], IT: [41.9, 12.5],
  JE: [49.19, -2.11], JM: [18.02, -76.8], JO: [31.95, 35.93], JP: [35.68, 139.69],
  KE: [-1.29, 36.82], KG: [42.87, 74.59], KH: [11.56, 104.92], KI: [1.45, 173.03],
  KM: [-11.7, 43.26], KN: [17.3, -62.72], KP: [39.03, 125.75], KR: [37.57, 126.98],
  KW: [29.38, 47.99], KY: [19.29, -81.37], KZ: [51.17, 71.45], LA: [17.98, 102.63],
  LB: [33.89, 35.5], LC: [14.01, -61.0], LI: [47.14, 9.52], LK: [6.93, 79.85],
  LR: [6.29, -10.76], LS: [-29.32, 27.48], LT: [54.69, 25.28], LU: [49.61, 6.13],
  LV: [56.95, 24.11], LY: [32.89, 13.19], MA: [34.02, -6.84], MC: [43.73, 7.42],
  MD: [47.01, 28.86], ME: [42.44, 19.26], MG: [-18.88, 47.51], MH: [7.09, 171.38],
  MK: [41.99, 21.43], ML: [12.64, -8.0], MM: [19.76, 96.08], MN: [47.89, 106.91],
  MO: [22.2, 113.55], MR: [18.08, -15.98], MT: [35.9, 14.51], MU: [-20.16, 57.5],
  MV: [4.18, 73.51], MW: [-13.96, 33.77], MX: [19.43, -99.13], MY: [3.14, 101.69],
  MZ: [-25.97, 32.57], NA: [-22.56, 17.07], NC: [-22.28, 166.46], NE: [13.51, 2.11],
  NG: [9.06, 7.5], NI: [12.11, -86.24], NL: [52.37, 4.9], NO: [59.91, 10.75],
  NP: [27.72, 85.32], NR: [-0.55, 166.92], NZ: [-41.29, 174.78], OM: [23.59, 58.41],
  PA: [8.98, -79.52], PE: [-12.05, -77.04], PF: [-17.55, -149.56], PG: [-9.44, 147.18],
  PH: [14.6, 120.98], PK: [33.68, 73.05], PL: [52.23, 21.01], PR: [18.47, -66.11],
  PT: [38.72, -9.14], PW: [7.5, 134.62], PY: [-25.26, -57.58], QA: [25.29, 51.53],
  RE: [-20.88, 55.45], RO: [44.43, 26.1], RS: [44.79, 20.45], RU: [55.76, 37.62],
  RW: [-1.94, 30.06], SA: [24.71, 46.68], SB: [-9.43, 159.95], SC: [-4.62, 55.45],
  SD: [15.5, 32.56], SE: [59.33, 18.07], SG: [1.35, 103.82], SI: [46.06, 14.51],
  SK: [48.15, 17.11], SL: [8.47, -13.23], SM: [43.94, 12.45], SN: [14.72, -17.47],
  SO: [2.05, 45.32], SR: [5.85, -55.2], SS: [4.86, 31.57], ST: [0.34, 6.73],
  SV: [13.69, -89.19], SY: [33.51, 36.29], SZ: [-26.31, 31.14], TD: [12.13, 15.06],
  TG: [6.13, 1.22], TH: [13.76, 100.5], TJ: [38.56, 68.79], TL: [-8.56, 125.57],
  TM: [37.96, 58.33], TN: [36.81, 10.17], TO: [-21.14, -175.2], TR: [39.93, 32.86],
  TT: [10.65, -61.51], TV: [-8.52, 179.2], TW: [25.03, 121.57], TZ: [-6.16, 35.75],
  UA: [50.45, 30.52], UG: [0.35, 32.58], US: [38.91, -77.04], UY: [-34.9, -56.19],
  UZ: [41.3, 69.24], VA: [41.9, 12.45], VC: [13.16, -61.22], VE: [10.48, -66.9],
  VG: [18.43, -64.62], VN: [21.03, 105.85], VU: [-17.73, 168.32], WS: [-13.83, -171.77],
  XK: [42.66, 21.17], YE: [15.37, 44.19], ZA: [-25.75, 28.19], ZM: [-15.39, 28.32],
  ZW: [-17.83, 31.05],
};

export function countryCoords(code: string): [number, number] {
  const hit = capitals[code.toUpperCase()];
  if (hit) return hit;
  let acc = 0;
  for (let i = 0; i < code.length; i++) {
    acc = (acc * 31 + code.charCodeAt(i)) % 997;
  }
  const lat = ((acc % 140) - 70) * 0.9;
  const lon = ((acc * 7) % 360) - 180;
  return [lat, lon];
}

export function countryName(code: string, lang: string): string {
  try {
    const label = new Intl.DisplayNames([lang], { type: 'region' }).of(code.toUpperCase());
    return label ?? code;
  } catch {
    return code;
  }
}

export function worldMarkers(): Array<{ code: string; lat: number; lon: number }> {
  return Object.entries(capitals).map(([code, [lat, lon]]) => ({ code, lat, lon }));
}

export function groupServers(
  subs: SubscriptionGroup[],
  query: string,
  lang: string,
): CountryBucket[] {
  const needle = query.trim().toLowerCase();
  const byCode = new Map<string, ServerEntry[]>();
  for (const sub of subs) {
    for (const srv of sub.servers) {
      if (
        needle &&
        !srv.name.toLowerCase().includes(needle) &&
        !srv.server.toLowerCase().includes(needle)
      ) {
        continue;
      }
      const code = (srv.countryCode || 'UN').toUpperCase();
      const bucket = byCode.get(code);
      if (bucket) bucket.push(srv);
      else byCode.set(code, [srv]);
    }
  }
  const out: CountryBucket[] = [];
  for (const [code, servers] of byCode) {
    const pings = servers
      .map(s => s.latencyMs)
      .filter((v): v is number => v !== null && v !== undefined);
    const [lat, lon] = countryCoords(code);
    out.push({
      code,
      name: countryName(code, lang),
      servers: [...servers].sort((a, b) => (a.latencyMs ?? 99999) - (b.latencyMs ?? 99999)),
      best: pings.length ? Math.min(...pings) : null,
      lat,
      lon,
    });
  }
  return out.sort((a, b) => a.name.localeCompare(b.name));
}

export function pingTier(ms: number | null | undefined): string {
  if (ms === null || ms === undefined) return '';
  if (ms < 90) return 'tier-good';
  if (ms < 180) return 'tier-ok';
  if (ms < 300) return 'tier-slow';
  return 'tier-bad';
}