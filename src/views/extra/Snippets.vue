<template>
  <ExtraPage :title="t('snippets.title')" :subtitle="t('snippets.subtitle')">
    <div v-if="help" class="glass help-block">
      <p v-for="(line, i) in helpLines" :key="i" v-text="line" />
    </div>

    <div class="glass snip-list">
      <div v-for="snip in SNIPPETS" :key="snip.key" class="snip-row">
        <div class="snip-info">
          <p class="snip-name">
            <span v-text="t('snippets.name_' + snip.key)" />
            <span v-if="ADMIN_SNIPPETS.has(snip.key)" class="snip-admin mono" v-text="t('snippets.adminReq')" />
          </p>
          <p class="snip-desc" v-text="t('snippets.desc_' + snip.key)" />
        </div>
        <button
          type="button"
          class="tool-action snip-copy"
          @click="copy(snip)"
        >
          <span v-text="copiedKey === snip.key ? t('snippets.copied') : t('snippets.copy')" />
        </button>
      </div>
    </div>

    <p class="tool-hint" v-text="t('snippets.hint')" />
  </ExtraPage>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { writeText } from '@tauri-apps/api/clipboard';
import ExtraPage from '../../components/ExtraPage.vue';
import { showCopyHint } from '../../composables/useCopyHint';
import { t } from '../../i18n';

interface Snippet {
  key: string;
  cmd: string;
  admin?: boolean;
}

const ADMIN_SNIPPETS = new Set(['winsock', 'resetAdapter', 'resetFirewall', 'routeClean']);
const SNIPPETS: Snippet[] = [
  { key: 'flushdns', cmd: 'ipconfig /flushdns' },
  { key: 'winsock', cmd: 'netsh winsock reset' },
  { key: 'renew', cmd: 'ipconfig /release && ipconfig /renew' },
  { key: 'resetAdapter', cmd: 'netsh interface set interface "Ethernet" disable && netsh interface set interface "Ethernet" enable' },
  { key: 'resetFirewall', cmd: 'netsh advfirewall reset' },
  { key: 'routeClean', cmd: 'route -f' },
];

const copiedKey = ref<string | null>(null);
  const help = ref(true);
  const helpLines = computed(() => [1, 2, 3].map((n) => t('snippets.help' + n)));
let copyTimer: ReturnType<typeof setTimeout> | null = null;

async function copy(snip: Snippet) {
  try {
    await writeText(snip.cmd);
    copiedKey.value = snip.key;
    showCopyHint(t('snippets.copied'));
    if (copyTimer) clearTimeout(copyTimer);
    copyTimer = setTimeout(() => {
      copiedKey.value = null;
    }, 1600);
  } catch {}
}
</script>

<style scoped>
.snip-list {
  padding: 4px 0;
}

.snip-row {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 12px 18px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
}

.snip-row:last-child { border-bottom: none; }

.snip-info {
  flex: 1;
  min-width: 0;
}

.snip-name {
  margin: 0;
  font-size: 12.5px;
  font-weight: 500;
}

.snip-desc {
  margin: 2px 0 0;
  font-size: 11px;
  color: var(--muted-foreground);
  line-height: 1.45;
}

.snip-copy {
  flex-shrink: 0;
  padding: 6px 14px;
  font-size: 11px;
}

.tool-hint {
  font-size: 11px;
  line-height: 1.5;
  color: var(--muted-foreground);
}

/* ---------- shared extra-tool styles ---------- */
.tool-action {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 7px 14px;
  border-radius: 10px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  background: rgba(255, 255, 255, 0.05);
  color: var(--foreground);
  font-size: 12px;
  cursor: pointer;
  transition:
    background 160ms ease,
    border-color 160ms ease,
    color 160ms ease;
}

.tool-action:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.1);
  border-color: rgba(255, 255, 255, 0.2);
}

.tool-action:disabled {
  opacity: 0.5;
  cursor: default;
}

.tool-action--primary {
  border-color: rgba(167, 139, 250, 0.45);
  background: linear-gradient(180deg, rgba(167, 139, 250, 0.3), rgba(124, 92, 255, 0.18));
  color: #efe9ff;
}

.tool-action--primary:hover:not(:disabled) {
  background: linear-gradient(180deg, rgba(167, 139, 250, 0.42), rgba(124, 92, 255, 0.26));
}

.tool-action--active {
  border-color: rgba(167, 139, 250, 0.5);
  background: rgba(167, 139, 250, 0.14);
  color: #d9ccff;
}

.tool-error {
  margin: 0 0 12px;
  padding: 10px 14px;
  border-radius: 10px;
  border: 1px solid rgba(255, 108, 120, 0.3);
  background: rgba(255, 108, 120, 0.08);
  color: #ff9aa2;
  font-size: 12px;
}

.tool-hint {
  margin: 14px 0 0;
  font-size: 11px;
  line-height: 1.5;
  color: var(--muted-foreground);
}

.rc-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 14px;
  margin-bottom: 12px;
  font-size: 12px;
  color: var(--muted-foreground);
}

.rc-meta b {
  color: var(--foreground);
}

.help-block {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 14px;
  padding: 12px 16px;
  border-radius: 12px;
  border: 1px solid rgba(167, 139, 250, 0.25);
  background: rgba(124, 92, 255, 0.07);
}

.help-block p {
  margin: 0;
  font-size: 12px;
  line-height: 1.55;
  color: rgba(235, 238, 250, 0.78);
}
</style>
