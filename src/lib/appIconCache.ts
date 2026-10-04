

import { reactive } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';

export const appIcons = reactive<Record<string, string>>({});

let inFlight = false;
let pending: string[] = [];

export async function fetchAppIcons(paths: string[]): Promise<void> {
  const missing = [...new Set(paths.filter((path) => path && !appIcons[path]))].slice(0, 400);
  if (missing.length === 0) return;

  pending.push(...missing);

  
  if (inFlight) return;
  inFlight = true;
  try {
    while (pending.length > 0) {
      const batch = pending.splice(0, 64);
      try {
        const loaded = await invoke<Array<string | null>>('collect_app_icons', {
          paths: batch,
        });
        loaded.forEach((image, index) => {
          if (image) appIcons[batch[index]] = image;
        });
      } catch {
        break;
      }
    }
  } finally {
    inFlight = false;
  }
}
