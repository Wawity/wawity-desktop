import { defineConfig, type Plugin } from 'vite';
import vue from '@vitejs/plugin-vue';
import { resolve } from 'path';
import { readdirSync, existsSync, statSync } from 'fs';

const ART_DIR = resolve(__dirname, 'public/art');
const SOUND_DIR = resolve(__dirname, 'public/sounds');
const MODULE = 'virtual:wawity-assets';
const RESOLVED = '\0' + MODULE;

const MEDIA = new Set(['.png', '.jpg', '.jpeg', '.webp', '.avif', '.gif', '.mp4', '.webm', '.ogv', '.mov']);

/**
 * The list of button-art files and file-backed sound packs, decided while the
 * bundle is built.
 *
 * This used to be probed at runtime: the app created an Image for each name and
 * only offered the ones that loaded. Under the tauri:// protocol that probe
 * never passed, so the picker came up empty and the pictures looked like they
 * had not shipped — even though the files were sitting in public/. Asking the
 * filesystem at build time cannot fail like that, and the app does no image
 * loading just to decide what to draw in a menu.
 */
function assetManifest(): Plugin {
  const art: string[] = [];
  const sounds: Record<string, string[]> = {};

  if (existsSync(ART_DIR)) {
    for (const name of readdirSync(ART_DIR).sort()) {
      const full = resolve(ART_DIR, name);
      if (statSync(full).isFile() && MEDIA.has(name.slice(name.lastIndexOf('.')))) art.push(name);
    }
  }
  if (existsSync(SOUND_DIR)) {
    for (const entry of readdirSync(SOUND_DIR).sort()) {
      const dir = resolve(SOUND_DIR, entry);
      if (!statSync(dir).isDirectory()) continue;
      const files = readdirSync(dir)
        .filter((f) => MEDIA.has(f.slice(f.lastIndexOf('.'))))
        .sort();
      if (files.length) sounds[entry] = files;
    }
  }

  const payload = { art, sounds };
  return {
    name: 'wawity-asset-manifest',
    resolveId: (id) => (id === MODULE ? RESOLVED : null),
    load(id) {
      if (id !== RESOLVED) return null;
      // A normal function, not an arrow: Rollup binds `this` to the plugin
      // context and an arrow would lose it.
      const counts = `art: ${art.length} файл(ов), звуковых паков: ${Object.keys(sounds).length}`;
      (this as { info?: (m: string) => void }).info?.(counts);
      return `export default ${JSON.stringify(payload)};`;
    },
    handleHotUpdate: (ctx) => {
      const f = ctx.file.replace(/\\/g, '/');
      if (f.includes('/public/art/') || f.includes('/public/sounds/')) {
        ctx.server.restart();
      }
    },
  };
}

export default defineConfig({
  plugins: [vue(), assetManifest()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: true,
    watch: {
      ignored: ['**/src-tauri/**', '**/node_modules/**'],
    },
  },
  envPrefix: ['VITE_', 'TAURI_'],
  build: {
    target: ['es2021', 'chrome100', 'safari13'],
    minify: 'esbuild',
    sourcemap: false,
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'index.html'),
        splash: resolve(__dirname, 'splash.html'),
      },
    },
  },
});

declare module 'virtual:wawity-assets' {
  const data: { art: string[]; sounds: Record<string, string[]> };
  export default data;
}
