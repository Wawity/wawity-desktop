<template>
  <div class="page">
    <div class="page-header">
      <h1 class="page-title" v-text="t('settings.title')" />
      <p class="page-sub" v-text="t('settings.subtitle')" />
    </div>

    <nav class="seg">
      <button
        v-for="tab in tabs"
        :key="tab.key"
        type="button"
        class="seg-btn"
        :class="{ 'seg-btn--active': activeTab === tab.key }"
        :title="t(tab.label)"
        @click="activeTab = tab.key"
      >
        <component :is="tab.icon" :size="15" />
        <span class="seg-label-wrap">
          <span class="seg-label" v-text="t(tab.label)" />
        </span>
      </button>
    </nav>

    <div ref="paneWrapRef" class="pane-wrap">
      <Transition name="pane" mode="out-in">
        <div :key="activeTab" class="pane">
          <div v-if="activeTab === 'security'" class="card">
            <div class="setting-row">
              <div class="row-left">
                <Network :size="16" class="row-icon" />
                <div class="row-text">
                  <p class="row-title" v-text="t('settings.dpiTitle')" />
                  <p class="row-desc" v-text="dpiHint" />
                </div>
              </div>
              <div class="seg">
                <button
                  v-for="mode in DPI_MODES"
                  :key="mode"
                  type="button"
                  :class="[
                    'seg-btn',
                    vpnStore.settings.dpi_profile === mode ? 'seg-btn--active' : '',
                  ]"
                  @click="setDpi(mode)"
                  v-text="dpiLabel(mode)"
                />
              </div>
            </div>

            <div
              ref="killSwitchRowRef"
              class="setting-row setting-row--danger"
              :class="{ 'setting-row--flash': flashTarget === 'killswitch' }"
            >
              <div class="row-left">
                <ShieldAlert :size="16" class="row-icon row-icon--danger" />
                <div class="row-text">
                  <p class="row-title" v-text="t('settings.killSwitch')" />
                  <p class="row-desc" v-text="t('settings.killSwitchDesc')" />
                </div>
              </div>
              <button
                type="button"
                role="switch"
                :aria-checked="vpnStore.settings.kill_switch"
                :aria-label="t('settings.killSwitch')"
                :class="['toggle', vpnStore.settings.kill_switch ? 'toggle--accent' : '']"
                @click="toggleKillSwitch"
              >
                <span
                  :class="['toggle-thumb', vpnStore.settings.kill_switch ? 'toggle-thumb--on' : '']"
                />
              </button>
            </div>

            <div
              ref="alwaysOnRowRef"
              class="setting-row setting-row--danger"
              :class="{ 'setting-row--flash': flashTarget === 'alwayson' }"
            >
              <div class="row-left">
                <Lock :size="16" class="row-icon row-icon--danger" />
                <div class="row-text">
                  <p class="row-title" v-text="t('settings.alwaysOn')" />
                  <p class="row-desc" v-text="t('settings.alwaysOnDesc')" />
                </div>
              </div>
              <button
                type="button"
                role="switch"
                :aria-checked="vpnStore.settings.always_on"
                :aria-label="t('settings.alwaysOn')"
                :disabled="alwaysOnPending"
                :class="['toggle', vpnStore.settings.always_on ? 'toggle--accent' : '']"
                @click="toggleAlwaysOn"
              >
                <span
                  :class="['toggle-thumb', vpnStore.settings.always_on ? 'toggle-thumb--on' : '']"
                />
              </button>
            </div>

            <div class="setting-row">
              <div class="row-left">
                <Atom :size="16" class="row-icon" />
                <div class="row-text">
                  <p class="row-title" v-text="t('settings.quantum')" />
                  <p class="row-desc" v-text="t('settings.quantumDesc')" />
                </div>
              </div>
              <button
                type="button"
                role="switch"
                :aria-checked="quantumResistant"
                :aria-label="t('settings.quantum')"
                :class="['toggle', quantumResistant ? 'toggle--on' : '']"
                @click="quantumResistant = !quantumResistant"
              >
                <span :class="['toggle-thumb', quantumResistant ? 'toggle-thumb--on' : '']" />
              </button>
            </div>

            <div class="setting-row">
              <div class="row-left">
                <Lock :size="16" class="row-icon" />
                <div class="row-text">
                  <p class="row-title" v-text="t('settings.strictRoute')" />
                  <p class="row-desc" v-text="t('settings.strictRouteDesc')" />
                </div>
              </div>
              <button
                type="button"
                role="switch"
                :aria-checked="strictRoute"
                :aria-label="t('settings.strictRoute')"
                :class="['toggle', strictRoute ? 'toggle--on' : '']"
                @click="strictRoute = !strictRoute"
              >
                <span :class="['toggle-thumb', strictRoute ? 'toggle-thumb--on' : '']" />
              </button>
            </div>

            <div class="setting-row">
              <div class="row-left">
                <Wifi :size="16" class="row-icon" />
                <div class="row-text">
                  <p class="row-title" v-text="t('settings.dnsLeakGuard')" />
                  <p class="row-desc" v-text="t('settings.dnsLeakGuardDesc')" />
                </div>
              </div>
              <button
                type="button"
                role="switch"
                :aria-checked="dnsLeakGuard"
                :aria-label="t('settings.dnsLeakGuard')"
                :class="['toggle', dnsLeakGuard ? 'toggle--on' : '']"
                @click="dnsLeakGuard = !dnsLeakGuard"
              >
                <span :class="['toggle-thumb', dnsLeakGuard ? 'toggle-thumb--on' : '']" />
              </button>
            </div>

            <div class="setting-row">
              <div class="row-left">
                <RefreshCw :size="16" class="row-icon" />
                <div class="row-text">
                  <p class="row-title" v-text="t('settings.tunnelOwnTraffic')" />
                  <p class="row-desc" v-text="t('settings.tunnelOwnTrafficDesc')" />
                </div>
              </div>
              <button
                type="button"
                role="switch"
                :aria-checked="tunnelOwnTraffic"
                :aria-label="t('settings.tunnelOwnTraffic')"
                :class="['toggle', tunnelOwnTraffic ? 'toggle--on' : '']"
                @click="tunnelOwnTraffic = !tunnelOwnTraffic"
              >
                <span :class="['toggle-thumb', tunnelOwnTraffic ? 'toggle-thumb--on' : '']" />
              </button>
            </div>

            <div class="setting-row">
              <div class="row-left">
                <ShieldAlert :size="16" class="row-icon" />
                <div class="row-text">
                  <p class="row-title" v-text="t('settings.insecureTls')" />
                  <p class="row-desc" v-text="t('settings.insecureTlsDesc')" />
                </div>
              </div>
              <button
                type="button"
                role="switch"
                :aria-checked="allowInsecureTls"
                :aria-label="t('settings.insecureTls')"
                :class="['toggle', allowInsecureTls ? 'toggle--on' : '']"
                @click="allowInsecureTls = !allowInsecureTls"
              >
                <span :class="['toggle-thumb', allowInsecureTls ? 'toggle-thumb--on' : '']" />
              </button>
            </div>

            <div class="setting-row">
              <div class="row-left">
                <Globe2 :size="16" class="row-icon" />
                <div class="row-text">
                  <p class="row-title" v-text="t('settings.bootstrapDns')" />
                  <p class="row-desc" v-text="t('settings.bootstrapDnsDesc')" />
                </div>
              </div>
              <div style="display: flex; gap: 6px">
                <button
                  v-for="opt in bootstrapOptions"
                  :key="opt"
                  type="button"
                  :class="[
                    'seg-btn',
                    vpnStore.settings.bootstrap_dns === opt ? 'seg-btn--active' : '',
                  ]"
                  @click="setBootstrapDns(opt)"
                  v-text="opt"
                />
              </div>
            </div>
          </div>

          <div v-else-if="activeTab === 'connection'" class="card">
            <div class="setting-row">
              <div class="row-left">
                <Wand2 :size="16" class="row-icon" />
                <div class="row-text">
                  <p class="row-title" v-text="t('settings.smartConnect')" />
                  <p class="row-desc" v-text="t('settings.smartConnectDesc')" />
                </div>
              </div>
              <button
                type="button"
                role="switch"
                :aria-checked="vpnStore.settings.smart_connect"
                :aria-label="t('settings.smartConnect')"
                :class="['toggle', vpnStore.settings.smart_connect ? 'toggle--on' : '']"
                @click="
                  vpnStore.updateSettings({ smart_connect: !vpnStore.settings.smart_connect })
                "
              >
                <span
                  :class="[
                    'toggle-thumb',
                    vpnStore.settings.smart_connect ? 'toggle-thumb--on' : '',
                  ]"
                />
              </button>
            </div>

            <div class="setting-row">
              <div class="row-left">
                <Network :size="16" class="row-icon" />
                <div class="row-text">
                  <p class="row-title" v-text="t('settings.failover')" />
                  <p class="row-desc" v-text="t('settings.failoverDesc')" />
                </div>
              </div>
              <button
                type="button"
                role="switch"
                :aria-checked="vpnStore.settings.failover_enabled"
                :aria-label="t('settings.failover')"
                :class="['toggle', vpnStore.settings.failover_enabled ? 'toggle--on' : '']"
                @click="
                  vpnStore.updateSettings({ failover_enabled: !vpnStore.settings.failover_enabled })
                "
              >
                <span
                  :class="[
                    'toggle-thumb',
                    vpnStore.settings.failover_enabled ? 'toggle-thumb--on' : '',
                  ]"
                />
              </button>
            </div>

            <Transition name="nested-reveal">
              <div v-if="vpnStore.settings.failover_enabled" class="chain-box">
                <div class="chain-head">
                  <button
                    type="button"
                    class="chain-cal"
                    :disabled="vpnStore.calibrating"
                    @click="vpnStore.calibrateFailover()"
                  >
                    <Wand2 :size="13" />
                    <span
                      v-text="
                        vpnStore.calibrating
                          ? t('settings.failoverCalibrating')
                          : t('settings.failoverCalibrate')
                      "
                    />
                  </button>
                  <label class="chain-retry">
                    <span v-text="t('settings.failoverRetries')" />
                    <input
                      type="number"
                      min="0"
                      max="5"
                      :value="vpnStore.settings.failover_retries"
                      @change="onRetries"
                    />
                  </label>
                </div>

                <p
                  v-if="!vpnStore.failoverServers.length"
                  class="chain-empty"
                  v-text="t('settings.failoverEmpty')"
                />

                <div v-for="(srv, idx) in vpnStore.failoverServers" :key="srv.id" class="chain-row">
                  <span
                    class="chain-idx"
                    v-text="
                      idx === 0
                        ? t('settings.failoverPrimary')
                        : t('settings.failoverBackup') + ' ' + idx
                    "
                  />
                  <span class="chain-name" v-text="srv.name" />
                  <button
                    type="button"
                    class="chain-btn"
                    @click="vpnStore.moveFailoverEntry(srv.id, -1)"
                  >
                    &#8593;
                  </button>
                  <button
                    type="button"
                    class="chain-btn"
                    @click="vpnStore.moveFailoverEntry(srv.id, 1)"
                  >
                    &#8595;
                  </button>
                  <button
                    type="button"
                    class="chain-btn chain-btn--kill"
                    @click="vpnStore.removeFailoverEntry(srv.id)"
                  >
                    &#215;
                  </button>
                </div>
              </div>
            </Transition>

            <div class="setting-row">
              <div class="row-left">
                <Power :size="16" class="row-icon" />
                <div class="row-text">
                  <p class="row-title" v-text="t('settings.startOnBoot')" />
                  <p class="row-desc" v-text="t('settings.startOnBootDesc')" />
                </div>
              </div>
              <button
                type="button"
                role="switch"
                :aria-checked="vpnStore.settings.start_on_boot"
                :aria-label="t('settings.startOnBoot')"
                :disabled="startOnBootPending"
                :class="['toggle', vpnStore.settings.start_on_boot ? 'toggle--on' : '']"
                @click="toggleStartOnBoot"
              >
                <span
                  :class="[
                    'toggle-thumb',
                    vpnStore.settings.start_on_boot ? 'toggle-thumb--on' : '',
                  ]"
                />
              </button>
            </div>

            <Transition name="nested-reveal">
              <div v-if="vpnStore.settings.start_on_boot" class="nested-setting-row">
                <div class="row-left">
                  <Rocket :size="15" class="row-icon row-icon--nested" />
                  <div class="row-text">
                    <p class="row-title" v-text="t('settings.autoConnect')" />
                    <p class="row-desc" v-text="t('settings.autoConnectDesc')" />
                  </div>
                </div>
                <button
                  type="button"
                  role="switch"
                  :aria-checked="autoConnect"
                  :aria-label="t('settings.autoConnect')"
                  :class="['toggle', autoConnect ? 'toggle--on' : '']"
                  @click="autoConnect = !autoConnect"
                >
                  <span :class="['toggle-thumb', autoConnect ? 'toggle-thumb--on' : '']" />
                </button>
              </div>
            </Transition>

            <div class="setting-row">
              <div class="row-left">
                <Wifi :size="16" class="row-icon" />
                <div class="row-text">
                  <p class="row-title" v-text="t('settings.lanAccess')" />
                  <p class="row-desc" v-text="t('settings.lanAccessDesc')" />
                </div>
              </div>
              <button
                type="button"
                role="switch"
                :aria-checked="lanAccess"
                :aria-label="t('settings.lanAccess')"
                :class="['toggle', lanAccess ? 'toggle--on' : '']"
                @click="lanAccess = !lanAccess"
              >
                <span :class="['toggle-thumb', lanAccess ? 'toggle-thumb--on' : '']" />
              </button>
            </div>

            <div class="setting-row">
              <div class="row-left">
                <Timer :size="16" class="row-icon" />
                <div class="row-text">
                  <p class="row-title" v-text="t('settings.autoPing')" />
                  <p class="row-desc" v-text="t('settings.autoPingDesc')" />
                </div>
              </div>
              <select
                v-model.number="autoPingMinutes"
                class="row-select"
                :aria-label="t('settings.autoPing')"
              >
                <option :value="0" v-text="t('settings.autoPingOff')" />
                <option
                  v-for="span in AUTO_PING_CHOICES"
                  :key="span"
                  :value="span"
                  v-text="t('settings.autoPingEvery', { count: span })"
                />
              </select>
            </div>

            <div class="setting-row">
              <div class="row-left">
                <Timer :size="16" class="row-icon" />
                <div class="row-text">
                  <p class="row-title" v-text="t('autoOff.title')" />
                  <p
                    class="row-desc"
                    v-text="vpnStore.autoOffArmed ? autoOffStatus : t('autoOff.subtitle')"
                  />
                </div>
              </div>
              <div class="autooff-modes">
                <button
                  v-for="mode in AUTO_OFF_MODES"
                  :key="mode"
                  type="button"
                  class="autooff-mode-btn"
                  :class="{ 'autooff-mode-btn--active': autoOffMode === mode }"
                  @click="chooseAutoOffMode(mode)"
                  v-text="t(`autoOff.${mode === 'off' ? 'none' : mode}`)"
                />
              </div>
            </div>

            <Transition name="nested-reveal" mode="out-in">
              <div v-if="autoOffMode === 'timer'" key="autooff-timer" class="autooff-panel">
                <div class="autooff-presets">
                  <button
                    v-for="span in AUTO_OFF_CHOICES"
                    :key="span"
                    type="button"
                    class="autooff-preset"
                    :class="{
                      'autooff-preset--active':
                        autoOffMinutes === span && vpnStore.autoOff.mode === 'timer',
                    }"
                    @click="armAutoOffMinutes(span)"
                    v-text="autoOffPresetLabel(span)"
                  />
                </div>
                <div class="autooff-line">
                  <input
                    v-model.number="autoOffMinutes"
                    class="autooff-input mono"
                    type="number"
                    min="1"
                    max="10080"
                    :placeholder="t('autoOff.custom')"
                  />
                  <button
                    type="button"
                    class="autooff-apply"
                    @click="armCustomAutoOff"
                    v-text="t('autoOff.apply')"
                  />
                </div>
              </div>

              <div
                v-else-if="autoOffMode === 'process'"
                key="autooff-process"
                class="autooff-panel"
              >
                <div class="autooff-line">
                  <select
                    v-model="autoOffProcess"
                    class="row-select autooff-app-select"
                    :disabled="loadingProcs"
                  >
                    <option
                      value=""
                      disabled
                      v-text="loadingProcs ? t('autoOff.loadingApps') : t('autoOff.chooseApp')"
                    />
                    <option
                      v-if="autoOffProcess && !processes.some((app) => app.path === autoOffProcess)"
                      :value="autoOffProcess"
                      v-text="appDisplayName(autoOffProcess)"
                    />
                    <option
                      v-for="app in processes"
                      :key="app.path"
                      :value="app.path"
                      v-text="app.name || appDisplayName(app.path)"
                    />
                  </select>
                  <button
                    type="button"
                    class="autooff-refresh"
                    :disabled="loadingProcs"
                    :title="t('autoOff.refreshApps')"
                    @click="loadProcesses"
                  >
                    <Loader2 v-if="loadingProcs" :size="13" class="spin" />
                    <RefreshCw v-else :size="13" />
                  </button>
                  <button
                    type="button"
                    class="autooff-apply"
                    :disabled="!autoOffProcess || loadingProcs"
                    @click="armProcessAutoOff"
                    v-text="t('autoOff.apply')"
                  />
                </div>
              </div>
            </Transition>

            <Transition name="nested-reveal">
              <div v-if="vpnStore.autoOffArmed" class="autooff-active">
                <span class="autooff-active-dot" aria-hidden="true" />
                <span class="autooff-active-text" v-text="autoOffStatus" />
                <button
                  type="button"
                  class="autooff-cancel"
                  @click="disableAutoOff"
                  v-text="t('autoOff.disarm')"
                />
              </div>
            </Transition>

            <div
              ref="multihopRowRef"
              class="setting-row"
              :class="{ 'setting-row--flash': flashTarget === 'multihop' }"
            >
              <div class="row-left">
                <Shuffle :size="16" class="row-icon" />
                <div class="row-text">
                  <p class="row-title" v-text="t('settings.multihop')" />
                  <p class="row-desc" v-text="t('settings.multihopDesc')" />
                </div>
              </div>
              <button
                type="button"
                role="switch"
                :aria-checked="multihopEnabled"
                :aria-label="t('settings.multihop')"
                :class="['toggle', multihopEnabled ? 'toggle--on' : '']"
                @click="multihopEnabled = !multihopEnabled"
              >
                <span :class="['toggle-thumb', multihopEnabled ? 'toggle-thumb--on' : '']" />
              </button>
            </div>

            <div class="setting-row">
              <div class="row-left">
                <Keyboard :size="16" class="row-icon" />
                <div class="row-text">
                  <p class="row-title" v-text="t('settings.hotkeys')" />
                  <p class="row-desc" v-text="t('settings.hotkeysDesc')" />
                </div>
              </div>
              <button
                type="button"
                role="switch"
                :aria-checked="hotkeysEnabled"
                :aria-label="t('settings.hotkeys')"
                :class="['toggle', hotkeysEnabled ? 'toggle--on' : '']"
                @click="hotkeysEnabled = !hotkeysEnabled"
              >
                <span :class="['toggle-thumb', hotkeysEnabled ? 'toggle-thumb--on' : '']" />
              </button>
            </div>

            <Transition name="nested-reveal">
              <div v-if="hotkeysEnabled" class="nested-setting-row">
                <div class="row-left">
                  <Zap :size="15" class="row-icon row-icon--nested" />
                  <div class="row-text">
                    <p class="row-title" v-text="t('settings.hotkeyToggle')" />
                    <p class="row-desc" v-text="t('settings.hotkeyToggleDesc')" />
                  </div>
                </div>
                <button
                  type="button"
                  class="hotkey-btn"
                  :class="{ 'hotkey-btn--recording': recordingTarget === 'toggle' }"
                  @click="startHotkeyCapture($event, 'toggle')"
                  @keydown="captureHotkey"
                  @blur="stopHotkeyCapture"
                >
                  <span
                    class="mono"
                    v-text="recordingTarget === 'toggle' ? t('settings.hotkeyPress') : hotkeyLabel"
                  />
                </button>
              </div>
            </Transition>

            <Transition name="nested-reveal">
              <div v-if="hotkeysEnabled" class="nested-setting-row">
                <div class="row-left">
                  <ShieldAlert :size="15" class="row-icon row-icon--nested row-icon--danger" />
                  <div class="row-text">
                    <p class="row-title" v-text="t('settings.hotkeyPanic')" />
                    <p class="row-desc" v-text="t('settings.hotkeyPanicDesc')" />
                  </div>
                </div>
                <button
                  type="button"
                  class="hotkey-btn"
                  :class="{ 'hotkey-btn--recording': recordingTarget === 'panic' }"
                  @click="startHotkeyCapture($event, 'panic')"
                  @keydown="captureHotkey"
                  @blur="stopHotkeyCapture"
                >
                  <span
                    class="mono"
                    v-text="recordingTarget === 'panic' ? t('settings.hotkeyPress') : panicLabel"
                  />
                </button>
              </div>
            </Transition>
          </div>

          <div v-else-if="activeTab === 'privacy'" class="card">
            <div class="setting-row" :class="{ 'setting-row--flash': vpnStore.captureActive }">
              <div class="row-left">
                <Radar :size="16" class="row-icon" />
                <div class="row-text">
                  <p class="row-title">
                    <span v-text="t('settings.streamerTitle')" />
                    <span
                      v-if="vpnStore.captureActive"
                      class="tl-badge tl-badge--on"
                      v-text="t('settings.streamerActive')"
                    />
                  </p>
                  <p class="row-desc" v-text="t('settings.streamerDesc')" />
                </div>
              </div>
              <button
                type="button"
                role="switch"
                :aria-checked="streamerMode"
                :aria-label="t('settings.streamerTitle')"
                :class="['toggle', streamerMode ? 'toggle--on' : '']"
                @click="streamerMode = !streamerMode"
              >
                <span :class="['toggle-thumb', streamerMode ? 'toggle-thumb--on' : '']" />
              </button>
            </div>

            <div class="setting-row">
              <div class="row-left">
                <Globe2 :size="16" class="row-icon" />
                <div class="row-text">
                  <p class="row-title" v-text="t('settings.blockTrackers')" />
                  <p class="row-desc" v-text="t('settings.blockTrackersDesc')" />
                </div>
              </div>
              <button
                type="button"
                role="switch"
                :aria-checked="blockTrackers"
                :aria-label="t('settings.blockTrackers')"
                :class="['toggle', blockTrackers ? 'toggle--on' : '']"
                @click="blockTrackers = !blockTrackers"
              >
                <span :class="['toggle-thumb', blockTrackers ? 'toggle-thumb--on' : '']" />
              </button>
            </div>

            <div class="setting-row">
              <div class="row-left">
                <MapPin :size="16" class="row-icon" />
                <div class="row-text">
                  <p class="row-title" v-text="t('settings.onlineGeo')" />
                  <p class="row-desc" v-text="t('settings.onlineGeoDesc')" />
                </div>
              </div>
              <button
                type="button"
                role="switch"
                :aria-checked="onlineGeo"
                :aria-label="t('settings.onlineGeo')"
                :class="['toggle', onlineGeo ? 'toggle--on' : '']"
                @click="onlineGeo = !onlineGeo"
              >
                <span :class="['toggle-thumb', onlineGeo ? 'toggle-thumb--on' : '']" />
              </button>
            </div>

            <div class="setting-row">
              <div class="row-left">
                <Bell :size="16" class="row-icon" />
                <div class="row-text">
                  <p class="row-title" v-text="t('settings.notifications')" />
                  <p class="row-desc" v-text="t('settings.notificationsDesc')" />
                </div>
              </div>
              <button
                type="button"
                role="switch"
                :aria-checked="notificationsEnabled"
                :aria-label="t('settings.notifications')"
                :class="['toggle', notificationsEnabled ? 'toggle--on' : '']"
                @click="notificationsEnabled = !notificationsEnabled"
              >
                <span :class="['toggle-thumb', notificationsEnabled ? 'toggle-thumb--on' : '']" />
              </button>
            </div>

            <div
              ref="telemetryRowRef"
              class="setting-row"
              :class="{ 'setting-row--danger': vpnStore.settings.telemetry }"
            >
              <div class="row-left">
                <Activity :size="16" class="row-icon" />
                <div class="row-text">
                  <p class="row-title">
                    <span v-text="t('settings.telemetry')" />
                    <span
                      v-if="vpnStore.settings.telemetry"
                      class="tl-badge tl-badge--on"
                      v-text="t('settings.telemetryBadgeOn')"
                    />
                    <span
                      v-else
                      class="tl-badge"
                      v-text="t('settings.telemetryBadgeOff')"
                    />
                  </p>
                  <p class="row-desc" v-text="t('settings.telemetryDesc')" />
                </div>
              </div>
              <button
                type="button"
                role="switch"
                :aria-checked="vpnStore.settings.telemetry"
                :aria-label="t('settings.telemetry')"
                :class="['toggle', vpnStore.settings.telemetry ? 'toggle--on' : '']"
                @click="vpnStore.toggleTelemetry()"
              >
                <span
                  :class="['toggle-thumb', vpnStore.settings.telemetry ? 'toggle-thumb--on' : '']"
                />
              </button>
            </div>

            <Transition name="nested-reveal">
              <div v-if="telemetryDetailsOpen" class="nested-setting-row telemetry-details">
                <ul class="tm-list">
                  <li v-text="t('settings.tmYes1')" />
                  <li v-text="t('settings.tmYes2')" />
                  <li v-text="t('settings.tmYes3')" />
                </ul>
                <ul class="tm-list tm-list--never">
                  <li v-for="item in TM_NEVER" :key="item" v-text="t(item)" />
                </ul>
                <pre class="tm-payload mono">{{ TELEMETRY_SAMPLE }}</pre>
                <p class="row-desc tm-note" v-text="t('settings.tmNote')" />
              </div>
            </Transition>

            <button type="button" class="telemetry-details-btn" @click="telemetryDetailsOpen = !telemetryDetailsOpen">
              <component :is="telemetryDetailsOpen ? ArrowUp : ArrowDown" :size="13" />
              <span v-text="telemetryDetailsOpen ? t('settings.hideTmDetails') : t('settings.showTmDetails')" />
            </button>

            <div class="setting-row">
              <div class="row-left">
                <Cpu :size="16" class="row-icon" />
                <div class="row-text">
                  <p class="row-title" v-text="t('settings.deviceId')" />
                  <p class="row-desc" v-text="t('settings.deviceIdDesc')" />
                </div>
              </div>
              <button
                type="button"
                role="switch"
                :aria-checked="hwidEnabled"
                :aria-label="t('settings.deviceId')"
                :class="['toggle', hwidEnabled ? 'toggle--on' : '']"
                @click="hwidEnabled = !hwidEnabled"
              >
                <span :class="['toggle-thumb', hwidEnabled ? 'toggle-thumb--on' : '']" />
              </button>
            </div>

            <div v-if="hwidEnabled" data-sensitive class="hwid-panel">
              <code class="hwid-value" :title="hwidValue">{{ hwidShort }}</code>
              <div class="hwid-actions">
                <button type="button" class="hwid-btn" :disabled="!hwidValue" @click="copyHwid">
                  <Check v-if="hwidCopied" :size="14" />
                  <span
                    v-text="hwidCopied ? t('settings.deviceIdCopied') : t('settings.deviceIdCopy')"
                  />
                </button>
                <button type="button" class="hwid-btn" @click="resetHwid">
                  <RotateCcw :size="14" />
                  <span v-text="t('settings.deviceIdReset')" />
                </button>
              </div>
            </div>

            <div class="setting-row">
              <div class="row-left">
                <Gamepad2 :size="16" class="row-icon" />
                <div class="row-text">
                  <p class="row-title" v-text="t('settings.discordRpc')" />
                  <p class="row-desc" v-text="t('settings.discordRpcDesc')" />
                </div>
              </div>
              <button
                type="button"
                role="switch"
                :aria-checked="discordRpc"
                :aria-label="t('settings.discordRpc')"
                :class="['toggle', discordRpc ? 'toggle--on' : '']"
                @click="discordRpc = !discordRpc"
              >
                <span :class="['toggle-thumb', discordRpc ? 'toggle-thumb--on' : '']" />
              </button>
            </div>

            <Transition name="rpc-fold">
              <div v-if="discordRpc" class="rpc-sub">
                <div class="setting-row">
                  <div class="row-left">
                    <MapPin :size="16" class="row-icon" />
                    <div class="row-text">
                      <p class="row-title" v-text="t('settings.discordRpcServer')" />
                      <p class="row-desc" v-text="t('settings.discordRpcServerDesc')" />
                    </div>
                  </div>
                  <button
                    type="button"
                    role="switch"
                    :aria-checked="discordRpcShowServer"
                    :aria-label="t('settings.discordRpcServer')"
                    :class="['toggle', discordRpcShowServer ? 'toggle--on' : '']"
                    @click="discordRpcShowServer = !discordRpcShowServer"
                  >
                    <span
                      :class="['toggle-thumb', discordRpcShowServer ? 'toggle-thumb--on' : '']"
                    />
                  </button>
                </div>

                <div class="setting-row">
                  <div class="row-left">
                    <Layers :size="16" class="row-icon" />
                    <div class="row-text">
                      <p class="row-title" v-text="t('settings.discordRpcSub')" />
                      <p class="row-desc" v-text="t('settings.discordRpcSubDesc')" />
                    </div>
                  </div>
                  <button
                    type="button"
                    role="switch"
                    :aria-checked="discordRpcShowSub"
                    :aria-label="t('settings.discordRpcSub')"
                    :class="['toggle', discordRpcShowSub ? 'toggle--on' : '']"
                    @click="discordRpcShowSub = !discordRpcShowSub"
                  >
                    <span :class="['toggle-thumb', discordRpcShowSub ? 'toggle-thumb--on' : '']" />
                  </button>
                </div>
              </div>
            </Transition>
          </div>

          <div v-else-if="activeTab === 'appearance'" class="card">
            

            <div class="setting-row">
              <div class="row-left">
                <Sparkles :size="16" class="row-icon" />
                <div class="row-text">
                  <p class="row-title" v-text="t('settings.uiStyle')" />
                  <p class="row-desc" v-text="t('settings.uiStyleDesc')" />
                </div>
              </div>
              <div class="pill-group">
                <button
                  type="button"
                  :class="['pill-btn', { 'pill-btn--active': vpnStore.settings.ui_style === 'wawity' }]"
                  @click="setUiStyle('wawity')"
                  v-text="t('settings.uiStyleWawity')"
                ></button>
                <button
                  type="button"
                  :class="['pill-btn', { 'pill-btn--active': vpnStore.settings.ui_style === 'material' }]"
                  @click="setUiStyle('material')"
                  v-text="t('settings.uiStyleMaterial')"
                ></button>
              </div>
            </div>

            <div class="setting-row">
              <div class="row-left">
                <Zap :size="16" class="row-icon" />
                <div class="row-text">
                  <p class="row-title" v-text="t('settings.motionLevel')" />
                  <p class="row-desc" v-text="t('settings.motionLevelDesc')" />
                </div>
              </div>
              <div class="pill-group">
                <button
                  type="button"
                  :class="['pill-btn', { 'pill-btn--active': !motionFancy }]"
                  @click="setMotionMode('simple')"
                  v-text="t('settings.motionSimple')"
                ></button>
                <button
                  type="button"
                  :class="['pill-btn', { 'pill-btn--active': motionFancy }]"
                  @click="setMotionMode('fancy')"
                  v-text="t('settings.motionFancy')"
                ></button>
              </div>
            </div>

            <div class="setting-row">
              <div class="row-left">
                <MapIcon :size="16" class="row-icon" />
                <div class="row-text">
                  <p class="row-title" v-text="t('settings.serverView')" />
                  <p class="row-desc" v-text="t('settings.serverViewDesc')" />
                </div>
              </div>
              <div class="pill-switch">
                <button
                  type="button"
                  class="pill-btn"
                  :class="{ 'pill-btn--active': vpnStore.settings.server_view !== 'globe' }"
                  @click="setServerView('list')"
                  v-text="t('settings.serverViewList')"
                />
                <button
                  type="button"
                  class="pill-btn"
                  :class="{ 'pill-btn--active': vpnStore.settings.server_view === 'globe' }"
                  @click="setServerView('globe')"
                  v-text="t('settings.serverViewGlobe')"
                />
              </div>
            </div>

            <div class="setting-row">
              <div class="row-left">
                <ImageIcon :size="16" class="row-icon" />
                <div class="row-text">
                  <p class="row-title" v-text="t('settings.bgCustom')" />
                  <p class="row-desc" v-text="t('settings.bgCustomDesc')" />
                </div>
              </div>
              <button
                type="button"
                role="switch"
                :aria-checked="vpnStore.settings.bg_custom_enabled"
                :aria-label="t('settings.bgCustom')"
                :class="['toggle', vpnStore.settings.bg_custom_enabled ? 'toggle--on' : '']"
                @click="
                  vpnStore.updateSettings({
                    bg_custom_enabled: !vpnStore.settings.bg_custom_enabled,
                  })
                "
              >
                <span
                  :class="['toggle-thumb', vpnStore.settings.bg_custom_enabled ? 'toggle-thumb--on' : '']"
                />
              </button>
            </div>

            <template v-if="vpnStore.settings.bg_custom_enabled">
              <div class="setting-row">
                <div class="row-left">
                  <Link2 :size="16" class="row-icon" />
                  <div class="row-text">
                    <p class="row-title" v-text="t('settings.bgUrl')" />
                    <p class="row-desc" v-text="t('settings.bgUrlDesc')" />
                  </div>
                </div>
                <div class="bg-url-group">
                  <input
                    v-model="bgUrlModel"
                    type="text"
                    placeholder="https://i.pinimg.com/…"
                    spellcheck="false"
                    class="bg-url-input mono"
                    :aria-label="t('settings.bgUrl')"
                    @keydown.enter="commitBgUrl"
                  />
                  <button
                    type="button"
                    class="bg-mini-btn"
                    :title="t('settings.bgPaste')"
                    @click="pasteBgUrl"
                  >
                    <Clipboard :size="13" />
                  </button>
                  <button
                    type="button"
                    class="bg-mini-btn"
                    :title="t('settings.bgApply')"
                    :disabled="!bgUrlValid"
                    @click="commitBgUrl"
                  >
                    <Check :size="13" />
                  </button>
                </div>
              </div>

              <div class="setting-row">
                <div class="row-left">
                  <Upload :size="16" class="row-icon" />
                  <div class="row-text">
                    <p class="row-title" v-text="t('settings.bgUpload')" />
                    <p class="row-desc" v-text="t('settings.bgUploadDesc')" />
                  </div>
                </div>
                <div class="pill-group">
                  <button
                    type="button"
                    class="pill-btn"
                    :disabled="bgImporting"
                    @click="pickBgFile"
                  >
                    <Loader2 v-if="bgImporting" :size="12" class="spin" />
                    <Upload v-else :size="13" />
                    <span
                      style="margin-left: 6px"
                      v-text="bgImporting ? t('settings.bgImporting') : t('settings.bgPickFile')"
                    />
                  </button>
                  <button
                    v-if="vpnStore.settings.bg_custom_url"
                    type="button"
                    class="pill-btn bg-remove-btn"
                    @click="clearBgImage"
                    v-text="t('settings.bgRemove')"
                  />
                </div>
              </div>

              <div class="setting-row">
                <div class="row-left">
                  <Moon :size="16" class="row-icon" />
                  <div class="row-text">
                    <p class="row-title" v-text="t('settings.bgDim')" />
                    <p class="row-desc" v-text="t('settings.bgDimDesc')" />
                  </div>
                </div>
                <div class="bg-slider-wrap">
                  <input
                    v-model.number="bgDimModel"
                    type="range"
                    min="0"
                    max="90"
                    step="5"
                    class="bg-range"
                    :aria-label="t('settings.bgDim')"
                  />
                  <span class="bg-slider-val mono" v-text="vpnStore.settings.bg_custom_dim + '%'" />
                </div>
              </div>

              <div class="setting-row">
                <div class="row-left">
                  <Droplets :size="16" class="row-icon" />
                  <div class="row-text">
                    <p class="row-title" v-text="t('settings.bgBlur')" />
                    <p class="row-desc" v-text="t('settings.bgBlurDesc')" />
                  </div>
                </div>
                <div class="bg-slider-wrap">
                  <input
                    v-model.number="bgBlurModel"
                    type="range"
                    min="0"
                    max="30"
                    step="2"
                    class="bg-range"
                    :aria-label="t('settings.bgBlur')"
                  />
                  <span class="bg-slider-val mono" v-text="vpnStore.settings.bg_custom_blur + 'px'" />
                </div>
              </div>

              <div v-if="vpnStore.settings.bg_custom_url" class="bg-preview-wrap">
                <img :src="vpnStore.settings.bg_custom_url" alt="" class="bg-preview" />
                <span class="bg-preview-label" v-text="t('settings.bgPreview')" />
              </div>
            </template>

            <div class="setting-row">
              <div class="row-left">
                <Languages :size="16" class="row-icon" />
                <div class="row-text">
                  <p class="row-title" v-text="t('settings.language')" />
                  <p class="row-desc" v-text="t('settings.languageDesc')" />
                </div>
              </div>
              <div class="pill-switch">
                <button
                  type="button"
                  class="pill-btn"
                  :class="{ 'pill-btn--active': vpnStore.settings.language === 'en' }"
                  @click="changeLanguage('en')"
                >
                  English
                </button>
                <button
                  type="button"
                  class="pill-btn"
                  :class="{ 'pill-btn--active': vpnStore.settings.language === 'ru' }"
                  @click="changeLanguage('ru')"
                >
                  Русский
                </button>
              </div>
            </div>
          </div>

          <div v-else-if="activeTab === 'split'" class="card split-card">
            <p class="split-desc" v-text="t('settings.splitDesc')" />

            
            <div class="dns-block">
              <div class="block-head">
                <span class="split-mode-title" v-text="t('settings.dnsCenterTitle')" />
                <button
                  type="button"
                  class="tpl-help"
                  :class="{ 'tpl-help--on': openDnsHelp }"
                  :title="t('settings.dnsWhatsThis')"
                  @click="openDnsHelp = !openDnsHelp"
                >
                  <HelpCircle :size="13" />
                </button>
              </div>

              <div v-if="openDnsHelp" class="tpl-detail sg-help">
                <p class="tpl-detail-text" v-html="t('settings.dnsHelpBody')" />
              </div>

              <div class="setting-row dns-row">
                <div class="row-left">
                  <Globe2 :size="16" class="row-icon" />
                  <div class="row-text">
                    <p class="row-title" v-text="t('settings.dnsRemote')" />
                    <p class="row-desc" v-text="t('settings.dnsRemoteDesc')" />
                  </div>
                </div>
                <div class="seg dns-seg">
                  <button
                    v-for="preset in DNS_PRESETS"
                    :key="preset.key"
                    type="button"
                    class="seg-btn dns-pill-btn"
                    :class="{ 'seg-btn--active': activeDnsKeyRef === preset.key }"
                    @click="setDnsPreset(preset.key)"
                    v-text="t(preset.labelKey)"
                  />
                </div>
              </div>

              <div v-if="showCustomDoh" class="setting-row dns-row">
                <div class="row-left">
                  <Link2 :size="16" class="row-icon" />
                  <div class="row-text">
                    <p class="row-title" v-text="t('settings.dnsCustom')" />
                    <p class="row-desc" v-text="t('settings.dnsCustomDesc')" />
                  </div>
                </div>
                <input
                  :value="vpnStore.settings.dns_custom_doh"
                  type="text"
                  placeholder="https://…/dns-query"
                  spellcheck="false"
                  class="app-input dns-custom-input mono"
                  :aria-label="t('settings.dnsCustom')"
                  @change="commitCustomDoh(($event.target as HTMLInputElement).value)"
                />
              </div>

              <div class="setting-row dns-row">
                <div class="row-left">
                  <ShieldAlert :size="16" class="row-icon" />
                  <div class="row-text">
                    <p class="row-title" v-text="t('settings.dnsBlockLists')" />
                    <p class="row-desc" v-text="t('settings.dnsBlockListsDesc')" />
                  </div>
                </div>
                <button
                  type="button"
                  role="switch"
                  :aria-checked="dnsBlockOn"
                  :class="['toggle', dnsBlockOn ? 'toggle--on' : '']"
                  @click="toggleDnsBlock"
                >
                  <span :class="['toggle-thumb', dnsBlockOn ? 'toggle-thumb--on' : '']" />
                </button>
              </div>

              <p v-if="vpnStore.splitDirty" class="dns-apply-hint" v-text="t('settings.dnsApplyHint')" />
            </div>

            <div
              class="sg-block steam-fix"
              :class="{ 'steam-fix--on': vpnStore.settings.steam_fix }"
            >
              <div class="block-head">
                <div class="row-left">
                  <Zap :size="16" class="row-icon" />
                  <div class="row-text">
                    <p class="row-title" v-text="t('settings.steamFixTitle')" />
                    <p class="row-desc" v-text="t('settings.steamFixDesc')" />
                  </div>
                </div>
                <button
                  type="button"
                  role="switch"
                  :aria-checked="vpnStore.settings.steam_fix"
                  :aria-label="t('settings.steamFixTitle')"
                  :class="['toggle', vpnStore.settings.steam_fix ? 'toggle--accent' : '']"
                  @click="toggleSteamFix"
                >
                  <span
                    :class="['toggle-thumb', vpnStore.settings.steam_fix ? 'toggle-thumb--on' : '']"
                  />
                </button>
              </div>
            </div>

            <div class="split-mode-block">
              <div class="mode-pill">
                <button
                  v-for="option in SPLIT_MODES"
                  :key="option"
                  type="button"
                  class="mode-pill-btn"
                  :class="{ 'mode-pill-btn--active': vpnStore.settings.split_mode === option }"
                  :title="t(SPLIT_MODE_LABELS[option])"
                  @click="chooseSplitMode(option)"
                >
                  <component :is="SPLIT_MODE_ICONS[option]" :size="15" />
                </button>
              </div>
              <div class="mode-explain">
                <span
                  class="mode-explain-name"
                  v-text="t(SPLIT_MODE_LABELS[vpnStore.settings.split_mode])"
                />
                <span
                  class="mode-explain-desc"
                  v-text="t(SPLIT_MODE_HINTS[vpnStore.settings.split_mode])"
                />
              </div>
            </div>

            <div v-if="vpnStore.splitDirty" class="apply-bar">
              <span class="apply-text" v-text="t('settings.splitPending')" />
              <button
                type="button"
                class="apply-btn"
                :disabled="vpnStore.splitApplying"
                @click="vpnStore.applySplitRules()"
              >
                <Loader2 v-if="vpnStore.splitApplying" :size="13" class="spin" />
                <span
                  v-text="
                    vpnStore.splitApplying ? t('settings.splitApplying') : t('settings.splitApply')
                  "
                />
              </button>
            </div>

            <div v-if="vpnStore.settings.split_mode === 'smart'" class="smart-box">
              <div class="smart-head">
                <p class="smart-desc" v-text="t('settings.smartDesc')" />
                <button
                  type="button"
                  class="smart-btn"
                  :disabled="vpnStore.detectingBlocks"
                  @click="runDetect"
                >
                  <Loader2 v-if="vpnStore.detectingBlocks" :size="13" class="spin" />
                  <Radar v-else :size="13" />
                  <span
                    v-text="
                      vpnStore.detectingBlocks
                        ? t('settings.smartScanning')
                        : t('settings.smartScan')
                    "
                  />
                </button>
              </div>
              <ul v-if="blockReports.length > 0" class="rule-list">
                <li v-for="report in blockReports" :key="report.domain" class="rule-item">
                  <span
                    class="verdict-dot"
                    :class="report.blocked ? 'verdict-dot--blocked' : 'verdict-dot--ok'"
                  />
                  <span class="rule-value" v-text="report.label" />
                  <span class="verdict-tag" v-text="t('settings.verdict_' + report.verdict)" />
                </li>
              </ul>
            </div>

            <template v-if="splitOn">
              <div class="split-templates">
                <span class="split-mode-title" v-text="t('settings.splitTemplates')" />
                <div v-for="tpl in SPLIT_TEMPLATES" :key="tpl.id" class="tpl-row-wrap">
                  <div class="tpl-row" :class="{ 'tpl-row--on': isTemplateOn(tpl.id) }">
                    <button
                      type="button"
                      class="tpl-check"
                      :class="{ 'tpl-check--on': isTemplateOn(tpl.id) }"
                      @click="toggleTemplate(tpl)"
                    >
                      <Check v-if="isTemplateOn(tpl.id)" :size="12" />
                    </button>
                    <span class="tpl-label" @click="toggleTemplate(tpl)" v-text="t(tpl.labelKey)" />
                    <button
                      type="button"
                      class="tpl-help"
                      :class="{ 'tpl-help--on': openTemplate === tpl.id }"
                      :title="t('settings.tplWhatsInside')"
                      @click="toggleTemplateHelp(tpl.id)"
                    >
                      <HelpCircle :size="13" />
                    </button>
                  </div>
                  <div v-if="openTemplate === tpl.id" class="tpl-detail">
                    <p class="tpl-detail-text" v-text="t(tpl.detailKey)" />
                    <p
                      class="tpl-detail-count"
                      v-text="t('settings.tplEntries', { count: templateItems(tpl).length })"
                    />
                    <div class="tpl-chips">
                      <span
                        v-for="item in templateItems(tpl)"
                        :key="item"
                        class="tpl-chip mono"
                        v-text="item"
                      />
                    </div>
                  </div>
                </div>
              </div>

              <div class="split-tabs">
                <button
                  type="button"
                  class="split-tab"
                  :class="{ 'split-tab--active': splitTab === 'file' }"
                  @click="splitTab = 'file'"
                >
                  <FolderOpen :size="13" />
                  <span v-text="t('settings.fromFile')" />
                </button>
                <button
                  type="button"
                  class="split-tab"
                  :class="{ 'split-tab--active': splitTab === 'process' }"
                  @click="switchToProcess"
                >
                  <AppWindow :size="13" />
                  <span v-text="t('settings.runningApps')" />
                </button>
                <button
                  type="button"
                  class="split-tab"
                  :class="{ 'split-tab--active': splitTab === 'games' }"
                  @click="splitTab = 'games'"
                >
                  <Gamepad2 :size="13" />
                  <span v-text="t('settings.detectGames')" />
                </button>
                <button
                  type="button"
                  class="split-tab"
                  :class="{ 'split-tab--active': splitTab === 'domains' }"
                  @click="splitTab = 'domains'"
                >
                  <Globe :size="13" />
                  <span v-text="t('settings.tabDomains')" />
                </button>
                <button
                  type="button"
                  class="split-tab"
                  :class="{ 'split-tab--active': splitTab === 'ips' }"
                  @click="splitTab = 'ips'"
                >
                  <Network :size="13" />
                  <span v-text="t('settings.tabIps')" />
                </button>
              </div>

              <div v-if="splitTab === 'file'" class="split-panel">
                <div class="add-app-row">
                  <input
                    v-model="newApp"
                    type="text"
                    :placeholder="t('settings.appPathPlaceholder')"
                    class="app-input"
                    @keydown.enter="addAppManual"
                  />
                  <button
                    type="button"
                    class="app-browse-btn"
                    :title="t('settings.browseTitle')"
                    @click="browseFile"
                  >
                    <FolderOpen :size="14" />
                  </button>
                  <button
                    type="button"
                    class="app-add-btn"
                    :disabled="!newApp.trim()"
                    @click="addAppManual"
                    v-text="t('settings.addApp')"
                  />
                </div>
              </div>

              <div v-else-if="splitTab === 'process'" class="split-panel">
                <div class="add-app-row">
                  <input
                    v-model="newProcess"
                    type="text"
                    :placeholder="t('settings.processPlaceholder')"
                    class="app-input"
                    @keydown.enter="submitProcess"
                  />
                  <button
                    type="button"
                    class="app-add-btn"
                    :disabled="!newProcess.trim()"
                    @click="submitProcess"
                    v-text="t('settings.addProcess')"
                  />
                </div>
                <ul v-if="vpnStore.settings.split_processes.length > 0" class="rule-list">
                  <li
                    v-for="name in vpnStore.settings.split_processes"
                    :key="name"
                    class="rule-item"
                  >
                    <Cpu :size="13" class="rule-icon" />
                    <span class="rule-value mono" v-text="name" />
                    <button
                      type="button"
                      class="bypass-remove"
                      :title="t('settings.removeTitle')"
                      @click="vpnStore.removeSplitProcess(name)"
                    >
                      <X :size="13" />
                    </button>
                  </li>
                </ul>
                <div class="process-search">
                  <Search :size="13" class="proc-search-icon" />
                  <input
                    v-model="procQuery"
                    type="text"
                    :placeholder="t('settings.searchRunningApps')"
                    class="proc-search-input"
                  />
                  <button
                    type="button"
                    class="proc-refresh-btn"
                    :disabled="loadingProcs"
                    :title="t('settings.refreshTitle')"
                    @click="loadProcesses"
                  >
                    <Loader2 v-if="loadingProcs" :size="13" class="spin" />
                    <RefreshCw v-else :size="13" />
                  </button>
                </div>

                <div class="proc-list-wrap">
                  <p
                    v-if="loadingProcs && filteredProcs.length === 0"
                    class="proc-hint"
                    v-text="t('settings.loadingProcesses')"
                  />
                  <p
                    v-else-if="filteredProcs.length === 0"
                    class="proc-hint"
                    v-text="t('settings.noProcessesFound')"
                  />
                  <ul v-else class="proc-list">
                    <li
                      v-for="proc in filteredProcs"
                      :key="proc.path"
                      class="proc-row"
                      :class="{ 'proc-row--added': isBypassed(proc.path) }"
                      @click="toggleProcess(proc.path)"
                    >
                      <span class="proc-ico">
                        <img
                          v-if="appIcons[proc.path]"
                          :src="appIcons[proc.path]"
                          alt=""
                          loading="lazy"
                          decoding="async"
                        />
                        <span
                          v-else
                          class="proc-ico-letter"
                          v-text="proc.name.slice(0, 1).toUpperCase()"
                        />
                      </span>
                      <div class="proc-info">
                        <span class="proc-name" v-text="proc.name" />
                        <span class="proc-path mono" v-text="proc.path" />
                      </div>
                      <div class="proc-check" :class="{ 'proc-check--on': isBypassed(proc.path) }">
                        <Check v-if="isBypassed(proc.path)" :size="12" />
                        <Plus v-else :size="12" />
                      </div>
                    </li>
                  </ul>
                </div>
              </div>

              <div v-else-if="splitTab === 'games'" class="split-panel">
                <div class="games-scan-row">
                  <p class="games-scan-desc" v-text="t('settings.gamesScanDesc')" />
                  <button
                    type="button"
                    class="games-scan-btn"
                    :disabled="scanningGames"
                    @click="scanGames"
                  >
                    <Loader2 v-if="scanningGames" :size="14" class="spin" />
                    <Gamepad2 v-else :size="14" />
                    <span
                      v-text="scanningGames ? t('settings.scanning') : t('settings.scanForGames')"
                    />
                  </button>
                </div>

                <div v-if="detectedGames.length > 0" class="games-results">
                  <div class="process-search">
                    <Search :size="13" class="proc-search-icon" />
                    <input
                      v-model="gameQuery"
                      type="text"
                      :placeholder="t('settings.gamesSearch')"
                      class="proc-input"
                    />
                  </div>
                  <div class="games-bulk-row">
                    <button
                      type="button"
                      class="games-bulk-btn"
                      @click="selectAllGames"
                      v-text="t('settings.gamesSelectAll')"
                    />
                    <button
                      type="button"
                      class="games-bulk-btn"
                      @click="selectedGameKeys.clear()"
                      v-text="t('settings.gamesClearSel')"
                    />
                  </div>
                  <p
                    v-if="filteredGames.length === 0"
                    class="split-empty"
                    v-text="t('settings.gamesNothingFound')"
                  />
                  <ul v-else class="games-list">
                    <li
                      v-for="game in filteredGames"
                      :key="game.key"
                      class="game-row"
                      :class="{ 'game-row--selected': selectedGameKeys.has(game.key) }"
                      @click="toggleGameSelection(game.key)"
                    >
                      <div
                        class="game-check"
                        :class="{ 'game-check--on': selectedGameKeys.has(game.key) }"
                      >
                        <Check v-if="selectedGameKeys.has(game.key)" :size="12" />
                      </div>
                      <span class="proc-ico">
                        <img
                          v-if="appIcons[game.exePaths[0]]"
                          :src="appIcons[game.exePaths[0]]"
                          alt=""
                          loading="lazy"
                          decoding="async"
                        />
                        <Gamepad2 v-else :size="13" />
                      </span>
                      <div class="game-info">
                        <span class="game-name" v-text="game.displayName" />
                        <span class="game-count mono">
                          <span class="game-launcher" v-text="game.launcher" />
                          {{ t('settings.executables', { count: game.exePaths.length }) }}
                        </span>
                      </div>
                      <span
                        v-if="game.recommended"
                        class="game-badge"
                        v-text="t('settings.gamesRecommended')"
                      />
                    </li>
                  </ul>
                  <button
                    type="button"
                    class="games-add-btn"
                    :disabled="selectedGameKeys.size === 0"
                    @click="addSelectedGames"
                    v-text="t('settings.addSelectedToBypass')"
                  />
                </div>
              </div>

              <div v-else-if="splitTab === 'domains'" class="split-panel">
                <div class="add-app-row">
                  <input
                    v-model="newDomain"
                    type="text"
                    :placeholder="t('settings.domainPlaceholder')"
                    class="app-input"
                    @keydown.enter="submitDomain"
                  />
                  <button
                    type="button"
                    class="app-add-btn"
                    :disabled="!newDomain.trim()"
                    @click="submitDomain"
                    v-text="t('settings.addDomain')"
                  />
                </div>
                <p
                  v-if="vpnStore.settings.split_domains.length === 0"
                  class="split-empty"
                  v-text="t('settings.noDomains')"
                />
                <ul v-else class="rule-list">
                  <li
                    v-for="domain in vpnStore.settings.split_domains"
                    :key="domain"
                    class="rule-item"
                  >
                    <Globe :size="13" class="rule-icon" />
                    <span class="rule-value mono" v-text="domain" />
                    <button
                      type="button"
                      class="bypass-remove"
                      :title="t('settings.removeTitle')"
                      @click="vpnStore.removeSplitDomain(domain)"
                    >
                      <X :size="13" />
                    </button>
                  </li>
                </ul>
              </div>

              <div v-else-if="splitTab === 'ips'" class="split-panel">
                <div class="add-app-row">
                  <input
                    v-model="newIp"
                    type="text"
                    :placeholder="t('settings.ipPlaceholder')"
                    class="app-input"
                    @keydown.enter="submitIp"
                  />
                  <button
                    type="button"
                    class="app-add-btn"
                    :disabled="!newIp.trim()"
                    @click="submitIp"
                    v-text="t('settings.addIp')"
                  />
                </div>
                <p
                  v-if="vpnStore.settings.split_ips.length === 0"
                  class="split-empty"
                  v-text="t('settings.noIps')"
                />
                <ul v-else class="rule-list">
                  <li v-for="cidr in vpnStore.settings.split_ips" :key="cidr" class="rule-item">
                    <Network :size="13" class="rule-icon" />
                    <span class="rule-value mono" v-text="cidr" />
                    <button
                      type="button"
                      class="bypass-remove"
                      :title="t('settings.removeTitle')"
                      @click="vpnStore.removeSplitIp(cidr)"
                    >
                      <X :size="13" />
                    </button>
                  </li>
                </ul>
              </div>

              <div v-if="vpnStore.settings.bypass_apps.length > 0" class="bypass-list-wrap">
                <p
                  class="bypass-list-title"
                  v-text="
                    t('settings.bypassedApps', { count: vpnStore.settings.bypass_apps.length })
                  "
                />
                <ul class="bypass-list">
                  <li v-for="app in vpnStore.settings.bypass_apps" :key="app" class="bypass-item">
                    <span class="bypass-icon-wrap">
                      <img v-if="appIcons[app]" :src="appIcons[app]" alt="" />
                      <AppWindow v-else :size="13" />
                    </span>
                    <span class="bypass-path mono" v-text="appDisplayName(app)" />
                    <span class="bypass-full-path mono" v-text="app" />
                    <button
                      type="button"
                      class="bypass-remove"
                      :title="t('settings.removeTitle')"
                      @click="removeApp(app)"
                    >
                      <X :size="12" />
                    </button>
                  </li>
                </ul>
              </div>

              <p v-else class="split-empty" v-text="t('settings.noAppsConfigured')" />
            </template>
          </div>

          <div v-else-if="activeTab === 'about'" class="about-stack">
            <div class="card">
              <div class="about-row">
                <span class="about-label" v-text="t('settings.version')" />
                <span class="about-value mono">v0.2.1</span>
              </div>

              <div class="about-row" :title="coreInfo?.path || ''">
                <span class="about-label" v-text="t('settings.coreVersion')" />
                <span
                  class="about-value mono"
                  v-text="coreInfo && coreInfo.found ? 'sing-box ' + coreInfo.version : '—'"
                />
              </div>

              <button
                type="button"
                class="about-row about-row--clickable"
                v-if="coreInfo && coreInfo.found"
                @click="copyCoreHash"
              >
                <span class="about-label" v-text="t('settings.coreHash')" />
                <span
                  class="about-value mono"
                  v-text="copiedHash ? t('settings.copied') : coreShortHash"
                />
              </button>

              <div class="setting-row">
                <div class="row-left">
                  <Wrench :size="16" class="row-icon row-icon--danger" />
                  <div class="row-text">
                    <p class="row-title" v-text="t('settings.emergencyRepair')" />
                    <p class="row-desc" v-text="t('settings.emergencyRepairDesc')" />
                  </div>
                </div>
                <button
                  type="button"
                  class="repair-btn"
                  :disabled="repairing"
                  @click="runRepair"
                >
                  <Loader2 v-if="repairing" :size="13" class="spin" />
                  <span v-text="t('settings.repair')" />
                </button>
              </div>
            </div>

            <div class="card">
              <div class="setting-row">
                <div class="row-left">
                  <Clipboard :size="16" class="row-icon" />
                  <div class="row-text">
                    <p class="row-title" v-text="t('settings.cfgTitle')" />
                    <p class="row-desc" v-text="t('settings.cfgDesc')" />
                  </div>
                </div>
                <div class="games-bulk-row">
                  <button
                    type="button"
                    class="games-bulk-btn"
                    @click="exportConfig"
                    v-text="t('settings.cfgExport')"
                  />
                  <button
                    type="button"
                    class="games-bulk-btn"
                    @click="importConfigFile"
                    v-text="t('settings.cfgImport')"
                  />
                </div>
              </div>

              <div class="setting-row">
                <button type="button" style="flex: 1" class="reset-btn" @click="reset">
                  <RotateCcw :size="14" />
                  <span v-text="t('settings.resetDefaults')" />
                </button>
              </div>
            </div>
          </div>
        </div>
      </Transition>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Wand2 } from '../lib/appIcons';
import { ref, computed, onMounted, onUnmounted, nextTick, watch } from 'vue';
import type { Component } from 'vue';
import { useRoute } from 'vue-router';
import {
  Activity,
  ArrowDown,
  ArrowUp,
  ShieldAlert,
  Power,
  Wifi,
  Globe2,
  Bell,
  RotateCcw,
  FolderOpen,
  Search,
  RefreshCw,
  Loader2,
  Check,
  Clipboard,
  Plus,
  X,
  AppWindow,
  Atom,
  Shuffle,
  Lock,
  Rocket,
  Gamepad2,
  Languages,
  Globe,
  Network,
  Map as MapIcon,
  Info,
  Sparkles,
  MapPin,
  Layers,
  Keyboard,
  Zap,
  Timer,
  Shield,
  ShieldOff,
  Crosshair,
  Filter,
  Radar,
  HelpCircle,
  Cpu,
  Wrench,
  ImageIcon,
  Upload,
  Moon,
  Droplets,
  Link2,
} from '../lib/appIcons';
import { open, save } from '@tauri-apps/api/dialog';
import { writeText, readText } from '@tauri-apps/api/clipboard';
import { writeTextFile, readTextFile, readBinaryFile } from '@tauri-apps/api/fs';
import { invoke } from '@tauri-apps/api/tauri';
import { useVpnStore } from '../stores/vpn';
import type { AppSettings, AutoOffMode, BlockReport, DetectedGame } from '../types/vpn.d';
import { useNotifications } from '../composables/useNotifications';
import { askConfirm } from '../composables/useConfirm';
import { t } from '../i18n';
import { appIcons, fetchAppIcons } from '../lib/appIconCache';
import { gsap } from 'gsap';
import { motionLevelRef, setMotionLevel } from '../lib/motion';

interface InstalledApp {
  name: string;
  path: string;
}

type BooleanSettingKey = {
  [K in keyof AppSettings]: AppSettings[K] extends boolean ? K : never;
}[keyof AppSettings];

type TabKey = 'security' | 'connection' | 'privacy' | 'appearance' | 'split' | 'about';

const tabs: Array<{ key: TabKey; label: string; icon: Component }> = [
  { key: 'security', label: 'settings.security', icon: ShieldAlert },
  { key: 'connection', label: 'settings.connectionSection', icon: Power },
  { key: 'privacy', label: 'settings.privacy', icon: Globe2 },
  { key: 'appearance', label: 'settings.appearance', icon: Sparkles },
  { key: 'split', label: 'settings.splitTunneling', icon: AppWindow },
  { key: 'about', label: 'settings.about', icon: Info },
];

const activeTab = ref<TabKey>('security');

const vpnStore = useVpnStore();
const route = useRoute();
const { pushToast } = useNotifications();

type SplitModeKey = AppSettings['split_mode'];

interface SplitTemplateDef {
  id: string;
  labelKey: string;
  detailKey: string;
  domains: string[];
  matchNames?: string[];
}

const SPLIT_MODES: SplitModeKey[] = ['off', 'exclude', 'include', 'smart'];
const SPLIT_MODE_LABELS: Record<SplitModeKey, string> = {
  off: 'settings.splitModeOff',
  exclude: 'settings.splitModeExclude',
  include: 'settings.splitModeInclude',
  smart: 'settings.splitModeSmart',
};
const SPLIT_MODE_HINTS: Record<SplitModeKey, string> = {
  off: 'settings.splitModeOffDesc',
  exclude: 'settings.splitModeExcludeDesc',
  include: 'settings.splitModeIncludeDesc',
  smart: 'settings.splitModeSmartDesc',
};
const SPLIT_MODE_ICONS: Record<SplitModeKey, Component> = {
  off: ShieldOff,
  exclude: Filter,
  include: Crosshair,
  smart: Radar,
};

const SPLIT_TEMPLATES: SplitTemplateDef[] = [
  {
    id: 'torrent-direct',
    labelKey: 'settings.tplTorrentDirect',
    detailKey: 'settings.tplTorrentDirectDetail',
    domains: [],
    matchNames: [
      'qbittorrent.exe',
      'utorrent.exe',
      'bittorrent.exe',
      'deluge.exe',
      'transmission.exe',
      'transmission-qt.exe',
      'tixati.exe',
      'biglybt.exe',
      'vuze.exe',
      'webtorrent.exe',
      'folx.exe',
    ],
  },
  {
    id: 'banking-direct',
    labelKey: 'settings.tplBankingDirect',
    detailKey: 'settings.tplBankingDirectDetail',
    domains: [
      'sberbank.ru',
      'sbrf.ru',
      'online.sberbank.ru',
      'tinkoff.ru',
      'tbank.ru',
      'alfabank.ru',
      'vtb.ru',
      'gazprombank.ru',
      'psbank.ru',
      'rencredit.ru',
      'gosuslugi.ru',
      'nalog.gov.ru',
      'nalog.ru',
      'sfr.gov.ru',
      'max.ru',
      'vk.com',
      'vk.ru',
    ],
  },
];

const splitTab = ref<'file' | 'process' | 'games' | 'domains' | 'ips'>('file');
const splitOn = computed(() => vpnStore.settings.split_mode !== 'off');



interface DnsPreset {
  key: string;
  labelKey: string;
}

const DNS_PRESETS: DnsPreset[] = [
  { key: 'cloudflare', labelKey: 'settings.dnsPresetCloudflare' },
  { key: 'google', labelKey: 'settings.dnsPresetGoogle' },
  { key: 'quad9', labelKey: 'settings.dnsPresetQuad9' },
  { key: 'adguard', labelKey: 'settings.dnsPresetAdGuard' },
];


const openDnsHelp = ref(false);

function activeDnsKey(): string {
  const s = vpnStore.settings;
  return s.dns_custom_doh && s.dns_custom_doh.trim() ? 'custom' : s.dns_remote ?? 'cloudflare';
}

const activeDnsKeyRef = computed(() => activeDnsKey());

function setDnsPreset(key: string) {
  if (key === 'custom') return;
  if (vpnStore.settings.dns_remote === (key as never) && !(vpnStore.settings.dns_custom_doh ?? '').trim())
    return;
  vpnStore.updateSettings({ dns_remote: key as never, dns_custom_doh: '' });
  vpnStore.stageSplitChange();
}

const showCustomDoh = computed(
  () => activeDnsKeyRef.value === 'custom' || !!(vpnStore.settings.dns_custom_doh ?? '').trim(),
);

function commitCustomDoh(value: string) {
  const trimmed = value.trim();
  if (!trimmed) {
    vpnStore.updateSettings({ dns_custom_doh: '' });
    vpnStore.stageSplitChange();
    return;
  }
  if (!/^https:\/\//.test(trimmed)) {
    pushToast('error', t('settings.dnsCustomInvalid'), t('settings.dnsCustomHint'), 4500);
    return;
  }
  vpnStore.updateSettings({ dns_custom_doh: trimmed });
  vpnStore.stageSplitChange();
}

const dnsBlockOn = computed(
  () => vpnStore.settings.dns_block_ads !== false || vpnStore.settings.dns_block_trackers !== false,
);

function toggleDnsBlock() {
  const next = !dnsBlockOn.value;
  vpnStore.updateSettings({ dns_block_ads: next, dns_block_trackers: next });
  vpnStore.stageSplitChange();
}

async function toggleSteamFix() {
  const on = !vpnStore.settings.steam_fix;
  if (on) {
    vpnStore.addSplitProcess('steam.exe');
    vpnStore.addSplitProcess('steamwebhelper.exe');
    if (vpnStore.settings.split_mode === 'off') {
      vpnStore.setSplitMode('exclude');
    }
    vpnStore.updateSettings({
      steam_fix: true,
      dns_block_ads: false,
      dns_block_trackers: false,
    });
  } else {
    vpnStore.removeSplitProcess('steam.exe');
    vpnStore.removeSplitProcess('steamwebhelper.exe');
    vpnStore.updateSettings({
      steam_fix: false,
      dns_block_ads: true,
      dns_block_trackers: true,
    });
  }
  vpnStore.stageSplitChange();
  pushToast(
    'success',
    t('settings.steamFixTitle'),
    on ? t('settings.steamFixOn') : t('settings.steamFixOff'),
    4500,
  );
}

const processes = ref<InstalledApp[]>([]);
const loadingProcs = ref(false);
const procQuery = ref('');
const newApp = ref('');
const newProcess = ref('');
const newDomain = ref('');
const newIp = ref('');

const scanningGames = ref(false);
const detectedGames = ref<DetectedGame[]>([]);
const gameQuery = ref('');
const selectedGameKeys = ref(new Set<string>());

const openTemplate = ref('');
const blockReports = ref<BlockReport[]>([]);

const alwaysOnPending = ref(false);
const startOnBootPending = ref(false);
const repairing = ref(false);

interface CoreInfo {
  found: boolean;
  version: string;
  sha256: string;
  path: string;
}

const coreInfo = ref<CoreInfo | null>(null);
const copiedHash = ref(false);
let copyHashTimer: ReturnType<typeof setTimeout> | null = null;

onMounted(() => {
  invoke<CoreInfo>('core_info')
    .then((info) => {
      coreInfo.value = info;
    })
    .catch(() => {});
});

const coreShortHash = computed(() => {
  const hash = coreInfo.value?.sha256 ?? '';
  return hash ? `${hash.slice(0, 16)}…${hash.slice(-8)}` : '';
});

const telemetryDetailsOpen = ref(false);
const telemetryRowRef = ref<HTMLElement | null>(null);
const TM_NEVER = ['settings.tmNo1', 'settings.tmNo2', 'settings.tmNo3'];
const TELEMETRY_SAMPLE = [
  '{',
  '  "event": "connect_clicked",',
  '  "sessionId": "1737750000000-12345",   // random per launch',
  '  "appVersion": "0.2.1",',
  '  "os": "windows",',
  '  "props": {}',
  '}',
].join('\n');

async function copyCoreHash() {
  const hash = coreInfo.value?.sha256;
  if (!hash || copiedHash.value) return;
  try {
    await writeText(hash);
    copiedHash.value = true;
    if (copyHashTimer) clearTimeout(copyHashTimer);
    copyHashTimer = setTimeout(() => {
      copiedHash.value = false;
    }, 1600);
  } catch {}
}






const CFG_KEYS = ['wawity_settings', 'wawity_roles'] as const;

function buildConfigPayload(): Record<string, unknown> {
  const data: Record<string, unknown> = {
    app: 'wawity',
    schema: 2,
    exportedAt: new Date().toISOString(),
  };
  for (const key of CFG_KEYS) {
    const raw = localStorage.getItem(key);
    if (raw) data[key] = JSON.parse(raw);
  }
  return data;
}

async function exportConfig() {
  const path = await save({
    defaultPath: `wawity-config-${new Date().toISOString().slice(0, 10)}.wawity`,
    filters: [{ name: 'Wawity configuration', extensions: ['wawity'] }],
  });
  if (!path) return;
  try {
    await writeTextFile(path, JSON.stringify(buildConfigPayload(), null, 2));
    pushToast('success', t('settings.cfgSaved'), String(path), 4500);
  } catch (err) {
    pushToast('error', t('settings.cfgInvalid'), String(err), 5000);
  }
}

async function importConfigFile() {
  const selected = await open({
    multiple: false,
    filters: [{ name: 'Wawity configuration', extensions: ['wawity', 'json'] }],
  });
  if (typeof selected !== 'string' || !selected.trim()) return;
  try {
    const text = await readTextFile(selected);
    applyConfigPayload(text);
  } catch (err) {
    pushToast('error', t('settings.cfgInvalid'), String(err), 5000);
  }
}

function applyConfigPayload(text: string) {
  try {
    const data = JSON.parse(text) as Record<string, unknown>;
    if (data.app !== 'wawity') throw new Error('wrong file');
    let restored = false;
    for (const key of CFG_KEYS) {
      if (data[key]) {
        localStorage.setItem(key, JSON.stringify(data[key]));
        restored = true;
      }
    }
    if (!restored) throw new Error('empty config');
    window.location.reload();
  } catch (err) {
    pushToast('error', t('settings.cfgInvalid'), String(err), 5000);
  }
}

function setUiStyle(style: 'wawity' | 'material') {
  if (vpnStore.settings.ui_style === style) return;
  vpnStore.updateSettings({ ui_style: style });
}

const motionFancy = computed(() => motionLevelRef().value === 'fancy');

function setMotionMode(level: 'simple' | 'fancy') {
  if (motionLevelRef().value === level) return;
  setMotionLevel(level);
  vpnStore.updateSettings({ motion_level: level });
}
function boolSetting(key: BooleanSettingKey) {
  return computed<boolean>({
    get: () => vpnStore.settings[key] as boolean,
    set: (value: boolean) => vpnStore.updateSettings({ [key]: value } as Partial<AppSettings>),
  });
}

const AUTO_PING_CHOICES = [5, 15, 30, 60];
const AUTO_OFF_CHOICES = [15, 30, 60, 120];
const AUTO_OFF_MODES: AutoOffMode[] = ['off', 'timer', 'process'];

const autoPingMinutes = computed<number>({
  get: () => vpnStore.settings.auto_ping_minutes ?? 0,
  set: (value: number) => vpnStore.updateSettings({ auto_ping_minutes: value }),
});

const autoOffMode = ref<AutoOffMode>(vpnStore.autoOff.mode);
const autoOffMinutes = ref<number>(vpnStore.settings.auto_off_default_minutes || 30);
const autoOffProcess = ref(vpnStore.autoOff.process || '');

const autoOffStatus = computed(() => {
  if (vpnStore.autoOff.mode === 'timer') {
    return t('autoOff.armedTimer', { left: vpnStore.autoOffLabel });
  }
  if (vpnStore.autoOff.mode === 'process') {
    return t('autoOff.armedProcess', { app: appDisplayName(vpnStore.autoOff.process) });
  }
  return t('autoOff.none');
});

watch(
  () => vpnStore.autoOff.mode,
  (mode) => {
    autoOffMode.value = mode;
    if (mode === 'process') autoOffProcess.value = vpnStore.autoOff.process;
  },
);

const autoConnect = boolSetting('auto_connect');
const lanAccess = boolSetting('lan_access');
const multihopEnabled = boolSetting('multihop_enabled');
const streamerMode = boolSetting('streamer_mode');
const blockTrackers = boolSetting('block_trackers');
const notificationsEnabled = boolSetting('notifications');
const quantumResistant = boolSetting('quantum_resistant');
const liquidGlass = boolSetting('liquid_glass');
const discordRpc = boolSetting('discord_rpc');
const discordRpcShowServer = boolSetting('discord_rpc_show_server');
const discordRpcShowSub = boolSetting('discord_rpc_show_subscription');
const hotkeysEnabled = boolSetting('hotkeys_enabled');
const strictRoute = boolSetting('strict_route');
const dnsLeakGuard = boolSetting('dns_leak_guard');
const tunnelOwnTraffic = boolSetting('tunnel_own_traffic');
const allowInsecureTls = boolSetting('allow_insecure_tls');
const onlineGeo = boolSetting('online_geolocation');
const hwidEnabled = boolSetting('hwid_enabled');
const hwidCopied = ref(false);
let hwidCopyTimer: ReturnType<typeof setTimeout> | null = null;

const hwidValue = computed(() => vpnStore.hwid);
const hwidShort = computed(() =>
  vpnStore.hwid
    ? `${vpnStore.hwid.slice(0, 10)}…${vpnStore.hwid.slice(-8)}`
    : t('settings.deviceIdPending'),
);

async function copyHwid() {
  if (!vpnStore.hwid) return;
  try {
    await writeText(vpnStore.hwid);
    hwidCopied.value = true;
    if (hwidCopyTimer) clearTimeout(hwidCopyTimer);
    hwidCopyTimer = setTimeout(() => {
      hwidCopied.value = false;
    }, 1600);
  } catch {
    hwidCopied.value = false;
  }
}

async function resetHwid() {
  await vpnStore.resetHwid();
}

const bootstrapOptions: Array<AppSettings['bootstrap_dns']> = ['cloudflare', 'quad9', 'google'];

function setBootstrapDns(value: AppSettings['bootstrap_dns']) {
  vpnStore.updateSettings({ bootstrap_dns: value });
}

const recordingTarget = ref<'toggle' | 'panic' | null>(null);

const hotkeyLabel = computed(() => {
  const combo = vpnStore.settings.hotkey_toggle;
  if (!combo) return t('settings.hotkeyNotSet');
  return combo.replace('CommandOrControl', 'Ctrl');
});

const panicLabel = computed(() => {
  const combo = vpnStore.settings.hotkey_panic;
  if (!combo) return t('settings.hotkeyNotSet');
  return combo.replace('CommandOrControl', 'Ctrl');
});

function startHotkeyCapture(e: MouseEvent, target: 'toggle' | 'panic') {
  recordingTarget.value = target;
  (e.currentTarget as HTMLElement).focus();
}

function stopHotkeyCapture() {
  recordingTarget.value = null;
}

function normalizeHotkeyCode(code: string): string | null {
  if (code.startsWith('Key')) return code.slice(3);
  if (code.startsWith('Digit')) return code.slice(5);
  if (/^F\d{1,2}$/.test(code)) return code;
  const map: Record<string, string> = {
    Space: 'Space',
    Home: 'Home',
    End: 'End',
    PageUp: 'PageUp',
    PageDown: 'PageDown',
    Insert: 'Insert',
    Delete: 'Delete',
    ArrowUp: 'Up',
    ArrowDown: 'Down',
    ArrowLeft: 'Left',
    ArrowRight: 'Right',
    Backquote: '`',
    Minus: '-',
    Equal: '=',
    BracketLeft: '[',
    BracketRight: ']',
    Backslash: '\\',
    Semicolon: ';',
    Quote: "'",
    Comma: ',',
    Period: '.',
    Slash: '/',
  };
  return map[code] ?? null;
}

function captureHotkey(e: KeyboardEvent) {
  if (!recordingTarget.value) return;
  e.preventDefault();
  e.stopPropagation();
  if (e.key === 'Escape') {
    recordingTarget.value = null;
    (e.currentTarget as HTMLElement).blur();
    return;
  }
  const mainKey = normalizeHotkeyCode(e.code);
  if (!mainKey) return;
  const parts: string[] = [];
  if (e.ctrlKey || e.metaKey) parts.push('CommandOrControl');
  if (e.altKey) parts.push('Alt');
  if (e.shiftKey) parts.push('Shift');
  if (parts.length === 0 && !/^F\d{1,2}$/.test(mainKey)) return;
  parts.push(mainKey);
  if (recordingTarget.value === 'panic') {
    vpnStore.updateSettings({ hotkey_panic: parts.join('+') });
  } else {
    vpnStore.updateSettings({ hotkey_toggle: parts.join('+') });
  }
  recordingTarget.value = null;
  (e.currentTarget as HTMLElement).blur();
}

function setServerView(view: 'list' | 'globe') {
  if (vpnStore.settings.server_view === view) return;
  vpnStore.updateSettings({ server_view: view });
}

const bgImporting = ref(false);
const bgUrlModel = ref('');

const bgUrlValid = computed(() => {
  const v = bgUrlModel.value.trim();
  return v.startsWith('data:image/') || /^https?:\/\/\S+$/i.test(v);
});

watch(
  () => vpnStore.settings.bg_custom_url,
  (url) => {
    if (bgUrlModel.value !== url) bgUrlModel.value = url.startsWith('data:') ? '' : url;
  },
  { immediate: true },
);

function commitBgUrl() {
  if (!bgUrlValid.value) return;
  vpnStore.updateSettings({ bg_custom_url: bgUrlModel.value.trim() });
  pushToast('success', t('settings.bgApplied'), t('settings.bgAppliedDesc'), 3000);
}

function pasteBgUrl() {
  readText()
    .then((text) => {
      if (text) bgUrlModel.value = text.trim();
    })
    .catch(() => {});
}

function clearBgImage() {
  bgUrlModel.value = '';
  vpnStore.updateSettings({ bg_custom_url: '', bg_custom_enabled: false });
}

async function pickBgFile() {
  if (bgImporting.value) return;
  try {
    const chosen = await open({
      multiple: false,
      directory: false,
      filters: [{ name: t('settings.bgFileFilter'), extensions: ['png', 'jpg', 'jpeg', 'webp', 'gif', 'bmp'] }],
    });
    if (!chosen || typeof chosen !== 'string') return;
    bgImporting.value = true;
    const bytes = await readBinaryFile(chosen);
    const dataUrl = await imageFileToDataUrl(new Uint8Array(bytes), chosen);
    vpnStore.updateSettings({ bg_custom_url: dataUrl });
    pushToast('success', t('settings.bgImported'), t('settings.bgImportedDesc'), 3000);
  } catch (e) {
    pushToast('error', t('settings.bgImportFail'), String(e), 5000);
  } finally {
    bgImporting.value = false;
  }
}

function imageFileToDataUrl(bytes: Uint8Array, path: string): Promise<string> {
  return new Promise((resolve, reject) => {
    const ext = (path.split('.').pop() || 'png').toLowerCase();
    const mime =
      ext === 'png' ? 'image/png' : ext === 'webp' ? 'image/webp' : ext === 'gif' ? 'image/gif' : ext === 'bmp' ? 'image/bmp' : 'image/jpeg';
    let binary = '';
    const chunk = 0x8000;
    for (let i = 0; i < bytes.length; i += chunk) {
      binary += String.fromCharCode(...bytes.subarray(i, i + chunk));
    }
    const raw = `data:${mime};base64,${btoa(binary)}`;
    if (ext === 'gif' || ext === 'bmp') {
      resolve(raw);
      return;
    }
    const img = new Image();
    img.onload = () => {
      try {
        const maxW = 1920;
        const scale = Math.min(1, maxW / img.naturalWidth);
        if (scale >= 1 && raw.length < 1_800_000) {
          resolve(raw);
          return;
        }
        const canvas = document.createElement('canvas');
        canvas.width = Math.max(1, Math.round(img.naturalWidth * scale));
        canvas.height = Math.max(1, img.naturalHeight * scale);
        const ctx = canvas.getContext('2d');
        if (!ctx) {
          resolve(raw);
          return;
        }
        ctx.drawImage(img, 0, 0, canvas.width, canvas.height);
        const out = canvas.toDataURL('image/webp', 0.85);
        resolve(out.length < raw.length ? out : raw);
      } catch {
        resolve(raw);
      }
    };
    img.onerror = () => resolve(raw);
    img.src = raw;
  });
}

const bgDimModel = computed<number>({
  get: () => vpnStore.settings.bg_custom_dim,
  set: (v: number) => vpnStore.updateSettings({ bg_custom_dim: Math.round(v) }),
});

const bgBlurModel = computed<number>({
  get: () => vpnStore.settings.bg_custom_blur,
  set: (v: number) => vpnStore.updateSettings({ bg_custom_blur: Math.round(v) }),
});


watch(
  () => vpnStore.settings.bypass_apps,
  (apps) => {
    fetchAppIcons([...apps]);
  },
  { immediate: true },
);

const killSwitchRowRef = ref<HTMLElement | null>(null);
const multihopRowRef = ref<HTMLElement | null>(null);
const alwaysOnRowRef = ref<HTMLElement | null>(null);
const flashTarget = ref<string | null>(null);
let flashTimer = 0;

onMounted(() => {
  const tabParam = route.query.tab;
  if (typeof tabParam === 'string' && tabs.some((tb) => tb.key === tabParam)) {
    activeTab.value = tabParam as TabKey;
  }
  const highlight = route.query.highlight;
  if (highlight === 'killswitch' || highlight === 'alwayson') {
    activeTab.value = 'security';
  } else if (highlight === 'multihop') {
    activeTab.value = 'connection';
  } else {
    return;
  }
  flashTarget.value = highlight;
  nextTick(() => {
    const el =
      highlight === 'killswitch'
        ? killSwitchRowRef.value
        : highlight === 'alwayson'
          ? alwaysOnRowRef.value
          : multihopRowRef.value;
    el?.scrollIntoView({ behavior: 'smooth', block: 'center' });
  });
  flashTimer = window.setTimeout(() => {
    flashTarget.value = null;
  }, 1800);
});

onUnmounted(() => {
  if (flashTimer) window.clearTimeout(flashTimer);
});

const paneWrapRef = ref<HTMLElement | null>(null);
let paneResetTimer = 0;

function lockPaneHeight() {
  const wrap = paneWrapRef.value;
  if (!wrap) return;
  if (paneResetTimer) {
    window.clearTimeout(paneResetTimer);
    paneResetTimer = 0;
  }
  wrap.style.height = `${wrap.offsetHeight}px`;
  wrap.classList.add('pane-wrap--animating');
}

function growPaneHeight(el: Element) {
  const wrap = paneWrapRef.value;
  if (!wrap) return;
  const from = wrap.offsetHeight;
  const to = (el as HTMLElement).offsetHeight;
  if (to === from) {
    finishPaneResize();
    return;
  }
  wrap.style.height = `${from}px`;
  requestAnimationFrame(() => {
    wrap.style.height = `${to}px`;
    paneResetTimer = window.setTimeout(finishPaneResize, 380);
  });
}

function finishPaneResize() {
  const wrap = paneWrapRef.value;
  if (!wrap) return;
  wrap.style.height = '';
  wrap.classList.remove('pane-wrap--animating');
  paneResetTimer = 0;
}

watch(
  () => route.query.tab,
  (tabParam) => {
    if (typeof tabParam === 'string' && tabs.some((tb) => tb.key === tabParam)) {
      activeTab.value = tabParam as TabKey;
    }
  },
);

watch(
  () => route.query.highlight,
  (highlight) => {
    if (highlight !== 'killswitch' && highlight !== 'alwayson' && highlight !== 'multihop') return;
    activeTab.value = highlight === 'multihop' ? 'connection' : 'security';
    flashTarget.value = highlight;
    nextTick(() => {
      const el =
        highlight === 'killswitch'
          ? killSwitchRowRef.value
          : highlight === 'alwayson'
            ? alwaysOnRowRef.value
            : multihopRowRef.value;
      el?.scrollIntoView({ behavior: 'smooth', block: 'center' });
    });
    if (flashTimer) window.clearTimeout(flashTimer);
    flashTimer = window.setTimeout(() => {
      flashTarget.value = null;
    }, 1800);
  },
);

const filteredProcs = computed(() => {
  const q = procQuery.value.trim().toLowerCase();
  if (!q) return processes.value;
  return processes.value.filter(
    (p) => p.name.toLowerCase().includes(q) || p.path.toLowerCase().includes(q),
  );
});

const bypassSet = computed(() => new Set(vpnStore.settings.bypass_apps));
function isBypassed(path: string): boolean {
  return bypassSet.value.has(path);
}

function appDisplayName(path: string): string {
  return path.split(/[\\/]/).pop() ?? path;
}

function autoOffPresetLabel(minutes: number): string {
  if (minutes === 15) return t('autoOff.min15');
  if (minutes === 30) return t('autoOff.min30');
  if (minutes === 60) return t('autoOff.min60');
  return t('autoOff.min120');
}

async function chooseAutoOffMode(mode: AutoOffMode) {
  autoOffMode.value = mode;
  if (mode === 'off') {
    await vpnStore.disarmAutoOff();
    return;
  }
  if (vpnStore.autoOffArmed && vpnStore.autoOff.mode !== mode) {
    await vpnStore.disarmAutoOff();
  }
  if (mode === 'process' && processes.value.length === 0) {
    await loadProcesses();
  }
}

function armAutoOffMinutes(minutes: number) {
  const safe = Math.max(1, Math.min(10080, Math.round(Number(minutes) || 0)));
  autoOffMinutes.value = safe;
  vpnStore.updateSettings({ auto_off_default_minutes: safe });
  vpnStore.armAutoOffTimer(safe);
}

function armCustomAutoOff() {
  const raw = Number(autoOffMinutes.value);
  if (!Number.isFinite(raw) || raw < 1) return;
  armAutoOffMinutes(raw);
}

async function armProcessAutoOff() {
  const target = autoOffProcess.value.trim();
  if (!target) return;
  await vpnStore.armAutoOffProcess(target);
}

async function disableAutoOff() {
  await vpnStore.disarmAutoOff();
  autoOffMode.value = 'off';
}

async function browseFile() {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'Executable', extensions: ['exe'] }],
    });
    if (typeof selected === 'string' && selected.trim()) {
      await vpnStore.addBypassApp(selected.trim());
    }
  } catch {}
}

async function addAppManual() {
  const path = newApp.value.trim();
  if (!path) return;
  await vpnStore.addBypassApp(path);
  newApp.value = '';
}

async function removeApp(path: string) {
  await vpnStore.removeBypassApp(path);
}

async function toggleProcess(path: string) {
  if (bypassSet.value.has(path)) {
    await vpnStore.removeBypassApp(path);
  } else {
    await vpnStore.addBypassApp(path);
  }
}

async function loadProcesses() {
  loadingProcs.value = true;
  try {
    const found = await invoke<InstalledApp[]>('list_installed_apps');
    processes.value = found;
    fetchAppIcons(found.map((app) => app.path));
  } catch {
    processes.value = [];
  } finally {
    loadingProcs.value = false;
  }
}

async function switchToProcess() {
  splitTab.value = 'process';
  if (processes.value.length === 0) {
    await loadProcesses();
  }
}

async function scanGames() {
  if (scanningGames.value) return;
  scanningGames.value = true;
  try {
    const games = await invoke<DetectedGame[]>('scan_installed_games');
    detectedGames.value = games;
    fetchAppIcons(games.map((g) => g.exePaths[0]).filter(Boolean));
    selectedGameKeys.value.clear();
    for (const g of games) {
      if (g.recommended) selectedGameKeys.value.add(g.key);
    }
    if (games.length === 0) {
      pushToast('info', t('toast.noGamesFound'), t('toast.noGamesFoundDesc'));
    } else {
      pushToast(
        'success',
        t('toast.scanComplete'),
        t('toast.scanCompleteDesc', { count: games.length }),
      );
    }
  } catch (e) {
    pushToast('error', t('toast.scanFailed'), String(e), 6000);
  } finally {
    scanningGames.value = false;
  }
}

function toggleGameSelection(key: string) {
  if (selectedGameKeys.value.has(key)) {
    selectedGameKeys.value.delete(key);
  } else {
    selectedGameKeys.value.add(key);
  }
}

const filteredGames = computed(() => {
  const q = gameQuery.value.trim().toLowerCase();
  if (!q) return detectedGames.value;
  return detectedGames.value.filter(
    (game) =>
      game.displayName.toLowerCase().includes(q) ||
      game.launcher.toLowerCase().includes(q) ||
      game.exePaths.some((path) => path.toLowerCase().includes(q)),
  );
});

function selectAllGames() {
  for (const game of filteredGames.value) {
    selectedGameKeys.value.add(game.key);
  }
}

function chooseSplitMode(mode: SplitModeKey) {
  vpnStore.setSplitMode(mode);
}

function isTemplateOn(id: string) {
  return vpnStore.settings.split_templates.includes(id);
}

function templateItems(tpl: SplitTemplateDef) {
  return tpl.matchNames ?? tpl.domains;
}

function toggleTemplateHelp(id: string) {
  openTemplate.value = openTemplate.value === id ? '' : id;
}

async function runDetect() {
  const reports = await vpnStore.detectBlockedServices();
  blockReports.value = reports;
  const blocked = reports.filter((report) => report.blocked).map((report) => report.domain);
  if (blocked.length === 0) {
    pushToast('info', t('settings.smartClean'), t('settings.smartCleanDesc'));
    return;
  }
  vpnStore.mergeSplitDomains(blocked);
  vpnStore.stageSplitChange();
  pushToast(
    'success',
    t('settings.smartFound'),
    t('settings.smartFoundDesc', { count: blocked.length }),
  );
}

function submitProcess() {
  const raw = newProcess.value;
  if (!raw.trim()) return;
  if (vpnStore.addSplitProcess(raw)) {
    newProcess.value = '';
    return;
  }
  pushToast('error', t('settings.invalidProcess'), t('settings.invalidProcessDesc'));
}

async function submitDomain() {
  const raw = newDomain.value;
  if (!raw.trim()) return;
  const ok = await vpnStore.addSplitDomain(raw);
  if (ok) {
    newDomain.value = '';
  } else {
    pushToast('error', t('settings.invalidDomain'), raw, 4000);
  }
}

async function submitIp() {
  const raw = newIp.value;
  if (!raw.trim()) return;
  const ok = await vpnStore.addSplitIp(raw);
  if (ok) {
    newIp.value = '';
  } else {
    pushToast('error', t('settings.invalidIp'), raw, 4000);
  }
}

async function addMatchingApps(names: string[]) {
  if (processes.value.length === 0) {
    await loadProcesses();
  }
  const wanted = new Set(names.map((name) => name.toLowerCase()));
  const paths = processes.value
    .filter((app) => wanted.has(appDisplayName(app.path).toLowerCase()))
    .map((app) => app.path);
  if (paths.length > 0) {
    await vpnStore.addBypassApps(paths);
  }
  return paths.length;
}

async function dropMatchingApps(names: string[]) {
  const wanted = new Set(names.map((name) => name.toLowerCase()));
  const doomed = vpnStore.settings.bypass_apps.filter((path) =>
    wanted.has(appDisplayName(path).toLowerCase()),
  );
  for (const path of doomed) {
    await removeApp(path);
  }
}

async function toggleTemplate(tpl: SplitTemplateDef) {
  if (isTemplateOn(tpl.id)) {
    vpnStore.disableSplitTemplate(tpl.id, tpl.domains);
    if (tpl.matchNames) {
      await dropMatchingApps(tpl.matchNames);
    }
    return;
  }
  vpnStore.enableSplitTemplate(tpl.id, tpl.domains);
  if (tpl.matchNames) {
    const added = await addMatchingApps(tpl.matchNames);
    if (added === 0) {
      pushToast('info', t('settings.tplNoClients'), t('settings.tplNoClientsDesc'));
    }
  }
}

async function addSelectedGames() {
  const paths: string[] = [];
  for (const game of detectedGames.value) {
    if (!selectedGameKeys.value.has(game.key)) continue;
    paths.push(...game.exePaths);
  }

  const addedCount = await vpnStore.addBypassApps(paths);
  if (addedCount > 0) {
    pushToast('success', t('toast.appsAdded'), t('toast.appsAddedDesc', { count: addedCount }));
  } else {
    pushToast('info', t('toast.nothingToAdd'), t('toast.nothingToAddDesc'));
  }

  detectedGames.value = [];
  selectedGameKeys.value.clear();
}

async function toggleKillSwitch() {
  const turningOff = vpnStore.settings.kill_switch;
  if (turningOff) {
    const ok = await askConfirm({
      title: t('settings.confirmKillSwitchOffTitle'),
      description: t('settings.confirmKillSwitchOffDesc'),
      confirmLabel: t('settings.confirmKillSwitchOffAction'),
      cancelLabel: t('settings.confirmCancel'),
      danger: true,
    });
    if (!ok) return;
  }
  vpnStore.updateSettings({ kill_switch: !turningOff });
}

async function toggleAlwaysOn() {
  if (alwaysOnPending.value) return;
  const next = !vpnStore.settings.always_on;

  const ok = await askConfirm({
    title: next
      ? t('settings.confirmAlwaysOnEnableTitle')
      : t('settings.confirmAlwaysOnDisableTitle'),
    description: next
      ? t('settings.confirmAlwaysOnEnableDesc')
      : t('settings.confirmAlwaysOnDisableDesc'),
    confirmLabel: next
      ? t('settings.confirmAlwaysOnEnableAction')
      : t('settings.confirmAlwaysOnDisableAction'),
    cancelLabel: t('settings.confirmCancel'),
    danger: true,
  });
  if (!ok) return;

  alwaysOnPending.value = true;
  const success = await vpnStore.setAlwaysOn(next);
  alwaysOnPending.value = false;

  if (success) {
    pushToast(
      next ? 'warning' : 'info',
      next ? t('toast.alwaysOnEnabled') : t('toast.alwaysOnDisabled'),
      next ? t('toast.alwaysOnEnabledDesc') : t('toast.alwaysOnDisabledDesc'),
    );
  }
}

async function runRepair() {
  if (repairing.value) return;

  const ok = await askConfirm({
    title: t('settings.confirmRepairTitle'),
    description: t('settings.confirmRepairDesc'),
    confirmLabel: t('settings.confirmRepairAction'),
    cancelLabel: t('settings.confirmCancel'),
    danger: true,
  });
  if (!ok) return;

  repairing.value = true;
  try {
    await invoke('repair_network');
    await vpnStore.refreshStatus();
  } finally {
    repairing.value = false;
  }
}

async function toggleStartOnBoot() {
  if (startOnBootPending.value) return;
  const nextValue = !vpnStore.settings.start_on_boot;
  startOnBootPending.value = true;
  const ok = await vpnStore.setStartOnBoot(nextValue);
  startOnBootPending.value = false;

  if (ok) {
    pushToast(
      nextValue ? 'success' : 'info',
      nextValue ? t('toast.startupEnabled') : t('toast.startupDisabled'),
      nextValue ? t('toast.startupEnabledDesc') : t('toast.startupDisabledDesc'),
    );
  }
}

function changeLanguage(lang: 'en' | 'ru') {
  if (vpnStore.settings.language === lang) return;
  vpnStore.updateSettings({ language: lang });
}

async function reset() {
  const ok = await askConfirm({
    title: t('settings.confirmResetTitle'),
    description: t('settings.confirmResetDesc'),
    confirmLabel: t('settings.confirmResetAction'),
    cancelLabel: t('settings.confirmCancel'),
    danger: true,
  });
  if (!ok) return;

  if (vpnStore.settings.start_on_boot) {
    await vpnStore.setStartOnBoot(false);
  }
  vpnStore.resetSettings();
}

const DPI_MODES = ['off', 'soft', 'medium', 'hard'];

const dpiHint = computed(() => {
  const mode = vpnStore.settings.dpi_profile || 'off';
  return t('settings.dpiHint' + mode.charAt(0).toUpperCase() + mode.slice(1));
});

function dpiLabel(mode: string) {
  return t('settings.dpi' + mode.charAt(0).toUpperCase() + mode.slice(1));
}

function setDpi(mode: string) {
  vpnStore.updateSettings({ dpi_profile: mode as never });
}

function onRetries(event: Event) {
  const raw = Number((event.target as HTMLInputElement).value);
  const safe = Number.isFinite(raw) ? Math.max(0, Math.min(5, Math.round(raw))) : 2;
  vpnStore.updateSettings({ failover_retries: safe });
}
</script>

<style scoped>
.page {
  display: flex;
  flex-direction: column;
  gap: 16px;
  max-width: 640px;
  margin: 0 auto;
}

.page-header {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.page-title {
  font-size: 22px;
  font-weight: 600;
  letter-spacing: -0.02em;
}
.page-sub {
  font-size: 13px;
  color: rgba(235, 238, 250, 0.55);
}
.seg {
  display: flex;
  gap: 4px;
  padding: 4px;
  border-radius: 999px;
  align-self: center;
  border: 1px solid rgba(255, 255, 255, 0.09);
  background: rgba(255, 255, 255, 0.045);
  backdrop-filter: blur(12px);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.07),
    0 10px 28px rgba(0, 0, 0, 0.35);
}

.hwid-panel {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 10px 12px;
  margin-top: 10px;
  padding: 12px 14px;
  border-radius: 14px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  background: rgba(255, 255, 255, 0.04);
}

.hwid-value {
  flex: 1 1 190px;
  min-width: 0;
  padding: 6px 10px;
  border-radius: 9px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(255, 255, 255, 0.05);
  font-family: 'JetBrains Mono', ui-monospace, monospace;
  font-size: 12px;
  letter-spacing: 0.06em;
  color: rgba(235, 238, 250, 0.85);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  user-select: all;
}

.hwid-actions {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
  flex-wrap: wrap;
  justify-content: flex-end;
}

.hwid-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 30px;
  padding: 0 12px;
  border-radius: 999px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  background: rgba(255, 255, 255, 0.05);
  color: rgba(235, 238, 250, 0.78);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition:
    background 220ms ease,
    color 220ms ease,
    border-color 220ms ease,
    transform 220ms cubic-bezier(0.34, 1.56, 0.64, 1);
}

.hwid-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: #fff;
  transform: translateY(-1px);
}

.hwid-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  transform: none;
}

.seg-btn {
  display: flex;
  align-items: center;
  height: 34px;
  padding: 0 10px;
  border-radius: 999px;
  border: none;
  background: transparent;
  color: rgba(235, 238, 250, 0.5);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  white-space: nowrap;
  transition:
    background 260ms ease,
    color 260ms ease,
    box-shadow 260ms ease,
    padding 340ms cubic-bezier(0.34, 1.2, 0.64, 1);
}

.seg-btn svg {
  flex-shrink: 0;
}

.seg-btn:hover {
  color: rgba(255, 255, 255, 0.85);
}

.seg-btn--active {
  padding: 0 14px;
  background: rgba(255, 255, 255, 0.11);
  color: #fff;
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.14),
    0 4px 14px rgba(0, 0, 0, 0.3);
}

.seg-label-wrap {
  display: grid;
  grid-template-columns: 0fr;
  transition: grid-template-columns 340ms cubic-bezier(0.34, 1.2, 0.64, 1);
}

.seg-btn--active .seg-label-wrap {
  grid-template-columns: 1fr;
}

.seg-label {
  min-width: 0;
  overflow: hidden;
  white-space: nowrap;
  padding-left: 7px;
  opacity: 0;
  transform: translateX(-6px);
  transition:
    opacity 220ms ease,
    transform 340ms cubic-bezier(0.34, 1.2, 0.64, 1);
}

.seg-btn--active .seg-label {
  opacity: 1;
  transform: translateX(0);
}
.pane-wrap {
  transition: height 320ms cubic-bezier(0.22, 1, 0.36, 1);
}

.pane-wrap--animating {
  overflow: hidden;
}

@media (prefers-reduced-motion: reduce) {
  .pane-wrap {
    transition: none;
  }
}

.pane-enter-active {
  animation: paneInFancy 240ms cubic-bezier(0.34, 1.3, 0.64, 1) both;
}

.pane-leave-active {
  animation: paneOutFancy 110ms ease both;
}

@keyframes paneInFancy {
  from { opacity: 0; transform: translateY(12px) scale(0.99); }
  to { opacity: 1; transform: none; }
}

@keyframes paneOutFancy {
  from { opacity: 1; }
  to { opacity: 0; transform: translateY(-6px) scale(0.995); }
}

html.motion-fancy .pane-enter-active {
  animation-name: paneInBlur;
}

html.motion-fancy .pane-leave-active {
  animation-duration: 130ms;
}

@keyframes paneInBlur {
  from { opacity: 0; transform: translateY(14px); filter: blur(5px); }
  60% { filter: blur(2px); }
  to { opacity: 1; transform: none; filter: blur(0); }
}
.card {
  border-radius: 20px;
  border: 1px solid rgba(255, 255, 255, 0.09);
  background: rgba(255, 255, 255, 0.045);
  backdrop-filter: blur(13px);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.08),
    inset 0 -1px 0 rgba(0, 0, 0, 0.25),
    0 18px 44px rgba(0, 0, 0, 0.4);
  overflow: hidden;
}

.about-stack {
  display: flex;
  flex-direction: column;
  gap: 14px;
}
.about-link {
  font-size: 11.5px;
  color: #a78bfa;
  text-decoration: none;
}

.about-link:hover {
  text-decoration: underline;
}
.about-row--clickable {
  width: 100%;
  background: none;
  border: none;
  text-align: left;
  font-family: inherit;
  cursor: pointer;
}
.about-row--clickable:hover .about-value {
  color: #d9ccff;
}

.tl-badge {
  display: inline-block;
  margin-left: 8px;
  padding: 1px 7px;
  border-radius: 999px;
  border: 1px solid rgba(255, 255, 255, 0.14);
  background: rgba(0, 0, 0, 0.25);
  color: rgba(235, 238, 250, 0.45);
  font-size: 9px;
  font-weight: 600;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  vertical-align: middle;
}

.tl-badge--on {
  border-color: rgba(255, 176, 90, 0.4);
  background: rgba(255, 176, 90, 0.12);
  color: #ffc07a;
}

.telemetry-details-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  width: 100%;
  padding: 9px 18px;
  border: none;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  background: rgba(0, 0, 0, 0.16);
  color: rgba(235, 238, 250, 0.5);
  font-size: 11px;
  font-family: inherit;
  cursor: pointer;
  transition:
    color 160ms,
    background 160ms;
}

.telemetry-details-btn:hover {
  color: #d9ccff;
  background: rgba(0, 0, 0, 0.26);
}

.telemetry-details {
  display: flex;
  flex-direction: column;
  gap: 10px;
  align-items: stretch;
  justify-content: flex-start;
  padding: 14px 18px;
}

.tm-list {
  margin: 0;
  padding: 0 0 0 16px;
  font-size: 11px;
  line-height: 1.55;
  color: rgba(235, 238, 250, 0.62);
}

.tm-list--never {
  color: rgba(235, 238, 250, 0.38);
}

.tm-payload {
  margin: 0;
  padding: 10px 12px;
  border-radius: 9px;
  border: 1px solid rgba(255, 255, 255, 0.07);
  background: rgba(0, 0, 0, 0.3);
  color: rgba(206, 190, 255, 0.75);
  font-size: 10.5px;
  line-height: 1.5;
  overflow-x: auto;
  white-space: pre;
}

.tm-note {
  font-size: 10.5px;
}
.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 15px 18px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  
  content-visibility: auto;
  contain-intrinsic-size: auto 54px;
}

.setting-row:last-child {
  border-bottom: none;
}
.setting-row--danger {
  background: rgba(255, 108, 120, 0.045);
}
.setting-row--flash {
  animation: settingFlash 1.8s ease;
}

@keyframes settingFlash {
  0%,
  100% {
    background: transparent;
  }
  20%,
  60% {
    background: rgba(167, 139, 250, 0.14);
  }
}

.nested-setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 12px 18px 12px 38px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  background: rgba(0, 0, 0, 0.14);
}

.nested-reveal-enter-active {
  transition: all 220ms ease;
}
.nested-reveal-leave-active {
  transition: all 150ms ease;
}
.nested-reveal-enter-from,
.nested-reveal-leave-to {
  opacity: 0;
  transform: translateY(-6px);
}

.row-left {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  min-width: 0;
}
.row-icon {
  color: rgba(235, 238, 250, 0.55);
  flex-shrink: 0;
  margin-top: 2px;
}
.row-icon--danger {
  color: #ff8a92;
}
.row-icon--nested {
  color: rgba(235, 238, 250, 0.4);
}
.row-text {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}
.row-title {
  font-size: 13px;
  font-weight: 500;
}
.row-desc {
  font-size: 11.5px;
  line-height: 1.45;
  color: rgba(235, 238, 250, 0.45);
}
.row-select {
  flex-shrink: 0;
  min-width: 132px;
  padding: 7px 10px;
  border-radius: 9px;
  border: 1px solid rgba(255, 255, 255, 0.09);
  background: rgba(255, 255, 255, 0.04);
  color: rgba(235, 238, 250, 0.9);
  font-size: 12.5px;
  font-family: inherit;
  outline: none;
  cursor: pointer;
  transition:
    border-color 0.16s ease,
    background 0.16s ease;
}
.row-select:hover {
  background: rgba(255, 255, 255, 0.07);
}
.row-select:focus-visible {
  border-color: rgba(167, 139, 250, 0.55);
}
.row-select option {
  background: #14161f;
  color: rgba(235, 238, 250, 0.9);
}

.autooff-modes {
  display: flex;
  flex-shrink: 0;
  gap: 3px;
  padding: 3px;
  border-radius: 10px;
  border: 1px solid rgba(255, 255, 255, 0.09);
  background: rgba(255, 255, 255, 0.04);
}

.autooff-mode-btn {
  height: 28px;
  padding: 0 9px;
  border-radius: 7px;
  border: none;
  background: transparent;
  color: rgba(235, 238, 250, 0.48);
  font: inherit;
  font-size: 11px;
  font-weight: 500;
  white-space: nowrap;
  cursor: pointer;
  transition:
    color 150ms,
    background 150ms;
}

.autooff-mode-btn:hover {
  color: rgba(235, 238, 250, 0.85);
}

.autooff-mode-btn--active {
  color: var(--foreground);
  background: rgba(255, 255, 255, 0.1);
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.09);
}

.autooff-panel {
  display: flex;
  flex-direction: column;
  gap: 9px;
  padding: 12px 18px 12px 46px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  background: rgba(0, 0, 0, 0.14);
}

.autooff-presets {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 5px;
}

.autooff-preset {
  height: 30px;
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.09);
  background: rgba(255, 255, 255, 0.035);
  color: rgba(235, 238, 250, 0.62);
  font: inherit;
  font-size: 11.5px;
  cursor: pointer;
  transition:
    color 150ms,
    background 150ms,
    border-color 150ms;
}

.autooff-preset:hover,
.autooff-preset--active {
  color: #e6dcff;
  border-color: rgba(167, 139, 250, 0.35);
  background: rgba(167, 139, 250, 0.12);
}

.autooff-line {
  display: flex;
  align-items: center;
  gap: 6px;
}

.autooff-input {
  flex: 1;
  min-width: 0;
  height: 32px;
  padding: 0 10px;
  border-radius: 9px;
  border: 1px solid rgba(255, 255, 255, 0.09);
  background: rgba(255, 255, 255, 0.04);
  color: var(--foreground);
  font: inherit;
  font-size: 12px;
  outline: none;
}

.autooff-input:focus {
  border-color: rgba(167, 139, 250, 0.5);
}

.autooff-app-select {
  flex: 1;
  min-width: 0;
}

.autooff-apply,
.autooff-refresh,
.autooff-cancel {
  flex-shrink: 0;
  height: 32px;
  border-radius: 9px;
  border: 1px solid rgba(255, 255, 255, 0.09);
  background: rgba(255, 255, 255, 0.055);
  color: rgba(235, 238, 250, 0.78);
  font: inherit;
  font-size: 11.5px;
  font-weight: 600;
  cursor: pointer;
  transition:
    color 150ms,
    background 150ms,
    border-color 150ms;
}

.autooff-apply {
  padding: 0 13px;
  border-color: rgba(167, 139, 250, 0.3);
  background: rgba(167, 139, 250, 0.12);
  color: #e6dcff;
}

.autooff-refresh {
  display: grid;
  place-items: center;
  width: 32px;
}

.autooff-apply:hover,
.autooff-refresh:hover {
  background: rgba(255, 255, 255, 0.1);
}

.autooff-apply:disabled,
.autooff-refresh:disabled {
  opacity: 0.45;
  cursor: default;
}

.autooff-active {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 9px 18px 9px 46px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  background: rgba(94, 230, 154, 0.045);
}

.autooff-active-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #5ee69a;
  box-shadow: 0 0 10px rgba(94, 230, 154, 0.55);
}

.autooff-active-text {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 11.5px;
  color: rgba(235, 238, 250, 0.7);
}

.autooff-cancel {
  height: 27px;
  padding: 0 9px;
  color: #ff9da5;
  border-color: rgba(255, 138, 146, 0.22);
  background: rgba(255, 138, 146, 0.06);
}

.autooff-cancel:hover {
  background: rgba(255, 138, 146, 0.12);
}

.toggle {
  position: relative;
  width: 44px;
  height: 24px;
  border-radius: 999px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  background: rgba(255, 255, 255, 0.08);
  cursor: pointer;
  flex-shrink: 0;
  box-shadow: inset 0 1px 3px rgba(0, 0, 0, 0.3);
  transition:
    background 220ms,
    border-color 220ms;
}

.toggle:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.toggle--on {
  background: linear-gradient(180deg, rgba(167, 139, 250, 0.85), rgba(124, 92, 255, 0.75));
  border-color: rgba(167, 139, 250, 0.6);
}

.toggle--accent {
  background: linear-gradient(180deg, rgba(255, 138, 146, 0.8), rgba(220, 70, 90, 0.7));
  border-color: rgba(255, 138, 146, 0.55);
}

.toggle-thumb {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: #fff;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.4);
  transition: transform 260ms cubic-bezier(0.34, 1.4, 0.64, 1);
}

.toggle-thumb--on {
  transform: translateX(20px);
}

.pill-switch {
  display: flex;
  gap: 3px;
  padding: 3px;
  border-radius: 999px;
  border: 1px solid rgba(255, 255, 255, 0.09);
  background: rgba(0, 0, 0, 0.2);
  flex-shrink: 0;
}

.pill-group {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.pill-group .pill-btn {
  min-width: 92px;
  justify-content: center;
  text-align: center;
}

.row-title,
.row-desc {
  overflow-wrap: anywhere;
}

.pill-btn {
  display: inline-flex;
  align-items: center;
  height: 26px;
  padding: 0 12px;
  border-radius: 999px;
  border: none;
  background: transparent;
  color: rgba(235, 238, 250, 0.5);
  font-size: 11.5px;
  font-weight: 500;
  cursor: pointer;
  white-space: nowrap;
  transition:
    background 180ms,
    color 180ms;
}

.pill-btn--active {
  background: rgba(255, 255, 255, 0.12);
  color: #fff;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.12);
}

.pill-btn:disabled {
  opacity: 0.35;
  cursor: not-allowed;
}

.bg-url-group {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}

.bg-url-input {
  width: 250px;
  height: 32px;
  padding: 0 12px;
  border-radius: 10px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  background: rgba(0, 0, 0, 0.25);
  color: var(--foreground);
  font-size: 11px;
  outline: none;
  transition: border-color 160ms ease;
}

.bg-url-input:focus {
  border-color: rgba(167, 139, 250, 0.55);
}

.bg-mini-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 32px;
  border-radius: 9px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  background: rgba(255, 255, 255, 0.05);
  color: rgba(235, 238, 250, 0.65);
  cursor: pointer;
  flex-shrink: 0;
  transition:
    background 150ms ease,
    color 150ms ease,
    transform 150ms ease;
}

.bg-mini-btn:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.1);
  color: #fff;
}

.bg-mini-btn:active:not(:disabled) {
  transform: scale(0.93);
}

.bg-mini-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.bg-remove-btn {
  color: #ff8a92;
}

.bg-slider-wrap {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-shrink: 0;
}

.bg-slider-val {
  font-size: 11px;
  min-width: 38px;
  text-align: right;
  color: rgba(235, 238, 250, 0.6);
}

.bg-range {
  -webkit-appearance: none;
  appearance: none;
  width: 160px;
  height: 4px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.14);
  outline: none;
  cursor: pointer;
}

.bg-range::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  border: 1px solid rgba(255, 255, 255, 0.4);
  background: linear-gradient(180deg, #c4b5fd, #8b5cf6);
  box-shadow: 0 2px 8px rgba(124, 92, 255, 0.4);
  cursor: pointer;
  transition: transform 140ms ease;
}

.bg-range::-webkit-slider-thumb:hover {
  transform: scale(1.15);
}

.bg-preview-wrap {
  position: relative;
  margin: 2px 0 6px;
  border-radius: 12px;
  overflow: hidden;
  border: 1px solid rgba(255, 255, 255, 0.1);
  height: 84px;
}

.bg-preview {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}

.bg-preview-label {
  position: absolute;
  right: 8px;
  bottom: 6px;
  padding: 2px 8px;
  border-radius: 999px;
  background: rgba(5, 6, 10, 0.6);
  backdrop-filter: blur(6px);
  font-size: 10px;
  color: rgba(235, 238, 250, 0.7);
}

.split-card {
  padding: 16px 18px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}
.split-desc {
  font-size: 12px;
  line-height: 1.5;
  color: rgba(235, 238, 250, 0.5);
}



.block-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.sg-help {
  margin: -2px 0 0;
}

.sg-help-flow {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 9px;
  padding: 8px 11px;
  border-radius: 10px;
  border: 1px solid rgba(167, 139, 250, 0.25);
  background: rgba(124, 92, 255, 0.08);
  font-size: 10px;
  color: rgba(206, 190, 255, 0.85);
}

.sg-help-flow > span {
  white-space: nowrap;
}

.sg-help-arrow {
  color: rgba(235, 238, 250, 0.35);
}

.sg-help-node {
  padding: 2px 8px;
  border-radius: 999px;
  border: 1px solid color-mix(in oklab, var(--success) 40%, transparent);
  background: color-mix(in oklab, var(--success) 12%, transparent);
  color: var(--success);
}

.dns-block .block-head {
  margin-bottom: 4px;
}

.sg-block {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 12px 14px;
  border-radius: 13px;
  border: 1px solid rgba(167, 139, 250, 0.22);
  background: linear-gradient(180deg, rgba(124, 92, 255, 0.09), rgba(124, 92, 255, 0.03));
}

.sg-sub {
  margin: -4px 0 0;
}

.sg-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.sg-chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  max-width: 100%;
  padding: 7px 12px;
  border-radius: 999px;
  border: 1px solid rgba(255, 255, 255, 0.11);
  background: rgba(255, 255, 255, 0.05);
  color: var(--muted-foreground);
  cursor: pointer;
  overflow: hidden;
  transition:
    border-color 200ms ease,
    color 200ms ease,
    background 220ms ease,
    transform 160ms cubic-bezier(0.34, 1.56, 0.64, 1);
}

.sg-chip:hover {
  transform: translateY(-1px);
  border-color: rgba(255, 255, 255, 0.22);
  color: var(--foreground);
}

.sg-chip--on {
  border-color: color-mix(in oklab, var(--success) 45%, transparent);
  background: linear-gradient(
    180deg,
    color-mix(in oklab, var(--success) 16%, transparent),
    color-mix(in oklab, var(--success) 5%, transparent)
  );
  color: var(--success);
}

.sg-chip--missing {
  opacity: 0.55;
}

.sg-chip-name {
  font-weight: 600;
  white-space: nowrap;
}

.sg-chip-target {
  font-size: 10px;
  opacity: 0.75;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 110px;
}

.sg-empty {
  margin: 2px 0 0;
}

.sg-rows {
  margin: 0;
}

.sg-row {
  gap: 8px;
}

.sg-row-server {
  flex: 1;
  min-width: 0;
  text-align: right;
  font-size: 10.5px;
  color: rgba(235, 238, 250, 0.45);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.sg-name-input {
  width: 150px;
  height: 28px;
  padding: 0 10px;
  border-radius: 9px;
  border: 1px solid rgba(167, 139, 250, 0.4);
  background: rgba(0, 0, 0, 0.3);
  color: var(--foreground);
  font-size: 11.5px;
  outline: none;
}

.sg-select {
  flex: 1;
  min-width: 0;
  height: 28px;
}

.sg-add {
  align-self: flex-start;
  display: inline-flex;
  align-items: center;
  gap: 6px;
}



.dns-block {
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin: 0 -18px;
  padding: 14px 18px 4px;
  border-top: 1px solid rgba(255, 255, 255, 0.07);
  border-bottom: 1px solid rgba(255, 255, 255, 0.07);
  background: rgba(0, 0, 0, 0.16);
}

.dns-block > .split-mode-title {
  margin-bottom: 6px;
}

.dns-row {
  border-bottom: none;
  padding: 9px 0;
}

.dns-seg {
  flex-shrink: 0;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.07);
}

.dns-seg .seg-btn--active {
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.14);
}

.dns-pill-btn {
  padding: 0 11px;
}

.dns-custom-input {
  width: 240px;
  height: 32px;
  padding: 0 12px;
  border-radius: 10px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  background: rgba(0, 0, 0, 0.25);
  color: var(--foreground);
  font-size: 11px;
  outline: none;
  transition: border-color 160ms ease;
}

.dns-custom-input:focus {
  border-color: rgba(167, 139, 250, 0.55);
}

.dns-apply-hint {
  margin: 2px 0 10px;
  font-size: 11px;
  color: #ffc07a;
}

.steam-fix {
  border-color: rgba(255, 176, 90, 0.28);
  background: linear-gradient(180deg, rgba(255, 176, 90, 0.06), rgba(255, 176, 90, 0.02));
}

.steam-fix--on {
  border-color: rgba(255, 176, 90, 0.5);
  background: linear-gradient(180deg, rgba(255, 176, 90, 0.12), rgba(255, 176, 90, 0.04));
}

.split-tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 3px;
  padding: 3px;
  border-radius: 14px;
  border: 1px solid rgba(255, 255, 255, 0.09);
  background: rgba(0, 0, 0, 0.2);
}

.split-tab {
  display: flex;
  align-items: center;
  gap: 5px;
  height: 28px;
  padding: 0 10px;
  border-radius: 11px;
  border: none;
  background: transparent;
  color: rgba(235, 238, 250, 0.5);
  font-size: 11.5px;
  font-weight: 500;
  cursor: pointer;
  white-space: nowrap;
  transition:
    background 180ms,
    color 180ms;
}

.split-tab--active {
  background: rgba(255, 255, 255, 0.12);
  color: #fff;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.12);
}

.split-panel {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.add-app-row {
  display: flex;
  gap: 6px;
}

.app-input {
  flex: 1;
  padding: 9px 12px;
  border-radius: 11px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  background: rgba(0, 0, 0, 0.18);
  color: #eef1fb;
  font-size: 12px;
  font-family: var(--font-mono, monospace);
  outline: none;
  transition: border-color 160ms;
}

.app-input:focus {
  border-color: rgba(167, 139, 250, 0.5);
}
.app-input::placeholder {
  color: rgba(235, 238, 250, 0.3);
}

.app-browse-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  border-radius: 11px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  background: rgba(255, 255, 255, 0.06);
  color: rgba(235, 238, 250, 0.6);
  cursor: pointer;
  transition:
    background 160ms,
    color 160ms;
}

.app-browse-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: #fff;
}

.app-add-btn {
  padding: 0 16px;
  border-radius: 11px;
  border: 1px solid rgba(167, 139, 250, 0.45);
  background: linear-gradient(180deg, rgba(167, 139, 250, 0.35), rgba(124, 92, 255, 0.25));
  color: #efeaff;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.16);
  transition: opacity 160ms;
}

.app-add-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.process-search {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 10px;
  height: 34px;
  border-radius: 11px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  background: rgba(0, 0, 0, 0.18);
}

.proc-search-icon {
  color: rgba(235, 238, 250, 0.4);
  flex-shrink: 0;
}

.proc-search-input {
  flex: 1;
  border: none;
  background: transparent;
  outline: none;
  color: #eef1fb;
  font-size: 12px;
  font-family: var(--font-sans);
}

.proc-search-input::placeholder {
  color: rgba(235, 238, 250, 0.3);
}

.proc-refresh-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border-radius: 8px;
  border: none;
  background: transparent;
  color: rgba(235, 238, 250, 0.5);
  cursor: pointer;
  transition: color 160ms;
}

.proc-refresh-btn:hover:not(:disabled) {
  color: #fff;
}
.proc-refresh-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.proc-list-wrap {
  max-height: 240px;
  overflow-y: auto;
  border-radius: 12px;
  border: 1px solid rgba(255, 255, 255, 0.07);
  background: rgba(0, 0, 0, 0.14);
}

.proc-hint {
  padding: 20px;
  text-align: center;
  font-size: 12px;
  color: rgba(235, 238, 250, 0.4);
}
.proc-list {
  list-style: none;
}

.proc-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 8px 12px;
  cursor: pointer;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  transition: background 140ms;
  content-visibility: auto;
  contain-intrinsic-size: auto 41px;
}

.proc-row:last-child {
  border-bottom: none;
}
.proc-row:hover {
  background: rgba(255, 255, 255, 0.05);
}
.proc-row--added {
  background: rgba(167, 139, 250, 0.08);
}

.proc-info {
  display: flex;
  flex-direction: column;
  gap: 1px;
  min-width: 0;
  flex: 1;
}
.proc-name {
  font-size: 12px;
  font-weight: 500;
}
.proc-path {
  font-size: 10px;
  color: rgba(235, 238, 250, 0.35);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.proc-check {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  border-radius: 50%;
  border: 1px solid rgba(255, 255, 255, 0.15);
  color: rgba(235, 238, 250, 0.5);
  flex-shrink: 0;
  transition: all 160ms;
}

.proc-check--on {
  border-color: rgba(94, 230, 154, 0.5);
  background: rgba(94, 230, 154, 0.14);
  color: #5ee69a;
}

.games-scan-row {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.games-scan-desc {
  font-size: 11.5px;
  line-height: 1.5;
  color: rgba(235, 238, 250, 0.45);
}

.games-scan-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 7px;
  padding: 10px 0;
  border-radius: 12px;
  border: 1px solid rgba(167, 139, 250, 0.45);
  background: linear-gradient(180deg, rgba(167, 139, 250, 0.35), rgba(124, 92, 255, 0.25));
  color: #efeaff;
  font-size: 12.5px;
  font-weight: 500;
  cursor: pointer;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.16);
  transition: opacity 160ms;
}

.games-scan-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.games-results {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.games-list {
  list-style: none;
  max-height: 220px;
  overflow-y: auto;
  border-radius: 12px;
  border: 1px solid rgba(255, 255, 255, 0.07);
  background: rgba(0, 0, 0, 0.14);
}

.game-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 9px 12px;
  cursor: pointer;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  transition: background 140ms;
  content-visibility: auto;
  contain-intrinsic-size: auto 43px;
}

.game-row:last-child {
  border-bottom: none;
}
.game-row:hover {
  background: rgba(255, 255, 255, 0.05);
}
.game-row--selected {
  background: rgba(167, 139, 250, 0.08);
}

.game-check {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  border-radius: 6px;
  border: 1px solid rgba(255, 255, 255, 0.2);
  color: transparent;
  flex-shrink: 0;
  transition: all 160ms;
}

.game-check--on {
  border-color: rgba(167, 139, 250, 0.6);
  background: rgba(167, 139, 250, 0.3);
  color: #fff;
}

.game-info {
  display: flex;
  flex-direction: column;
  gap: 1px;
  min-width: 0;
}
.game-name {
  font-size: 12px;
  font-weight: 500;
}
.game-count {
  font-size: 10px;
  color: rgba(235, 238, 250, 0.35);
}

.games-add-btn {
  padding: 10px 0;
  border-radius: 12px;
  border: 1px solid rgba(94, 230, 154, 0.4);
  background: rgba(94, 230, 154, 0.12);
  color: #5ee69a;
  font-size: 12.5px;
  font-weight: 500;
  cursor: pointer;
  transition:
    opacity 160ms,
    background 160ms;
}

.games-add-btn:hover:not(:disabled) {
  background: rgba(94, 230, 154, 0.18);
}
.games-add-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.bypass-list-wrap {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.bypass-list-title {
  font-size: 10.5px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.07em;
  color: rgba(235, 238, 250, 0.4);
}

.bypass-list {
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.bypass-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 10px;
  border-radius: 10px;
  border: 1px solid rgba(255, 255, 255, 0.07);
  background: rgba(0, 0, 0, 0.14);
  font-size: 11px;
}

.bypass-icon-wrap {
  display: flex;
  color: rgba(235, 238, 250, 0.4);
  flex-shrink: 0;
}
.bypass-path {
  font-weight: 500;
  flex-shrink: 0;
}
.bypass-full-path {
  flex: 1;
  color: rgba(235, 238, 250, 0.3);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 10px;
}

.bypass-remove {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  border-radius: 6px;
  border: none;
  background: transparent;
  color: rgba(235, 238, 250, 0.4);
  cursor: pointer;
  flex-shrink: 0;
  transition:
    color 140ms,
    background 140ms;
}

.bypass-remove:hover {
  color: #ff8a92;
  background: rgba(255, 138, 146, 0.1);
}

.split-empty {
  font-size: 11.5px;
  color: rgba(235, 238, 250, 0.35);
  text-align: center;
  padding: 4px 0;
}

.split-mode-block {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.mode-pill {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 3px;
  padding: 3px;
  border-radius: 14px;
  border: 1px solid rgba(255, 255, 255, 0.09);
  background: rgba(0, 0, 0, 0.2);
}

.mode-pill-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 34px;
  border-radius: 11px;
  border: none;
  background: transparent;
  color: rgba(235, 238, 250, 0.45);
  cursor: pointer;
  transition:
    background 180ms,
    color 180ms;
}

.mode-pill-btn:hover {
  color: rgba(235, 238, 250, 0.8);
}

.mode-pill-btn--active {
  background: rgba(167, 139, 250, 0.22);
  color: #efeaff;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.14);
}

.mode-explain {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 0 2px;
}

.mode-explain-name {
  font-size: 12.5px;
  font-weight: 500;
  color: #eef1fb;
}
.mode-explain-desc {
  font-size: 11px;
  line-height: 1.5;
  color: rgba(235, 238, 250, 0.45);
}

.apply-bar {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 9px 12px;
  border-radius: 12px;
  border: 1px solid rgba(255, 193, 112, 0.32);
  background: rgba(255, 193, 112, 0.09);
}

.apply-text {
  flex: 1;
  font-size: 11px;
  line-height: 1.45;
  color: rgba(255, 214, 160, 0.9);
}

.apply-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 7px 14px;
  border-radius: 10px;
  border: 1px solid rgba(167, 139, 250, 0.45);
  background: linear-gradient(180deg, rgba(167, 139, 250, 0.35), rgba(124, 92, 255, 0.25));
  color: #efeaff;
  font-size: 11.5px;
  font-weight: 500;
  cursor: pointer;
  white-space: nowrap;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.16);
}

.apply-btn:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

.smart-box {
  display: flex;
  flex-direction: column;
  gap: 9px;
  padding: 11px 12px;
  border-radius: 12px;
  border: 1px solid rgba(255, 255, 255, 0.07);
  background: rgba(0, 0, 0, 0.14);
}

.smart-head {
  display: flex;
  align-items: center;
  gap: 10px;
}
.smart-desc {
  flex: 1;
  font-size: 11px;
  line-height: 1.5;
  color: rgba(235, 238, 250, 0.45);
}

.smart-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 7px 13px;
  border-radius: 10px;
  border: 1px solid rgba(167, 139, 250, 0.45);
  background: linear-gradient(180deg, rgba(167, 139, 250, 0.35), rgba(124, 92, 255, 0.25));
  color: #efeaff;
  font-size: 11.5px;
  font-weight: 500;
  cursor: pointer;
  white-space: nowrap;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.16);
}

.smart-btn:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

.verdict-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  flex-shrink: 0;
}

.verdict-dot--blocked {
  background: #ff8a92;
  box-shadow: 0 0 7px rgba(255, 138, 146, 0.6);
}
.verdict-dot--ok {
  background: #6ee7a8;
  box-shadow: 0 0 7px rgba(110, 231, 168, 0.5);
}

.verdict-tag {
  flex-shrink: 0;
  font-size: 9.5px;
  font-weight: 500;
  color: rgba(235, 238, 250, 0.4);
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.split-templates {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.split-mode-title {
  font-size: 10.5px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.07em;
  color: rgba(235, 238, 250, 0.4);
}

.tpl-row-wrap {
  display: flex;
  flex-direction: column;
}

.tpl-row {
  display: flex;
  align-items: center;
  gap: 9px;
  padding: 9px 11px;
  border-radius: 11px;
  border: 1px solid rgba(255, 255, 255, 0.07);
  background: rgba(0, 0, 0, 0.14);
  transition:
    border-color 160ms,
    background 160ms;
}

.tpl-row--on {
  border-color: rgba(167, 139, 250, 0.4);
  background: rgba(167, 139, 250, 0.09);
}

.tpl-check {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  border-radius: 6px;
  border: 1px solid rgba(255, 255, 255, 0.2);
  background: transparent;
  color: transparent;
  cursor: pointer;
  flex-shrink: 0;
  transition: all 160ms;
}

.tpl-check--on {
  border-color: rgba(167, 139, 250, 0.6);
  background: rgba(167, 139, 250, 0.35);
  color: #fff;
}

.tpl-label {
  flex: 1;
  font-size: 12px;
  color: #eef1fb;
  cursor: pointer;
}

.tpl-help {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  border-radius: 7px;
  border: none;
  background: transparent;
  color: rgba(235, 238, 250, 0.35);
  cursor: pointer;
  flex-shrink: 0;
  transition:
    color 140ms,
    background 140ms;
}

.tpl-help:hover {
  color: #fff;
  background: rgba(255, 255, 255, 0.08);
}
.tpl-help--on {
  color: rgb(196, 181, 253);
  background: rgba(167, 139, 250, 0.16);
}

.tpl-detail {
  display: flex;
  flex-direction: column;
  gap: 7px;
  margin: 4px 0 2px;
  padding: 11px 12px;
  border-radius: 11px;
  border: 1px solid rgba(255, 255, 255, 0.06);
  background: rgba(0, 0, 0, 0.22);
}

.tpl-detail-text {
  font-size: 11px;
  line-height: 1.55;
  color: rgba(235, 238, 250, 0.6);
}

.tpl-detail-count {
  font-size: 9.5px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: rgba(235, 238, 250, 0.35);
}

.tpl-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  max-height: 148px;
  overflow-y: auto;
}

.tpl-chip {
  padding: 3px 7px;
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.06);
  color: rgba(235, 238, 250, 0.55);
  font-size: 10px;
  white-space: nowrap;
}

.rule-list {
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.rule-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 10px;
  border-radius: 10px;
  border: 1px solid rgba(255, 255, 255, 0.07);
  background: rgba(0, 0, 0, 0.14);
}

.rule-icon {
  color: rgba(235, 238, 250, 0.4);
  flex-shrink: 0;
}

.rule-value {
  flex: 1;
  font-size: 11px;
  color: rgba(235, 238, 250, 0.75);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.games-bulk-row {
  display: flex;
  gap: 6px;
}

.games-bulk-btn {
  padding: 6px 12px;
  border-radius: 999px;
  border: 1px solid rgba(255, 255, 255, 0.09);
  background: rgba(0, 0, 0, 0.2);
  color: rgba(235, 238, 250, 0.55);
  font-size: 11px;
  font-weight: 500;
  cursor: pointer;
  transition:
    background 160ms,
    color 160ms;
}

.games-bulk-btn:hover {
  background: rgba(255, 255, 255, 0.08);
  color: #fff;
}

.game-launcher {
  display: inline-block;
  margin-right: 6px;
  padding: 1px 6px;
  border-radius: 5px;
  background: rgba(255, 255, 255, 0.07);
  color: rgba(235, 238, 250, 0.5);
  font-size: 9.5px;
  font-weight: 500;
}

.game-badge {
  margin-left: auto;
  flex-shrink: 0;
  padding: 2px 8px;
  border-radius: 999px;
  border: 1px solid rgba(167, 139, 250, 0.3);
  background: rgba(167, 139, 250, 0.12);
  color: rgba(206, 190, 255, 0.9);
  font-size: 9.5px;
  font-weight: 500;
  white-space: nowrap;
}

.about-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 18px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
}

.about-label {
  font-size: 12.5px;
  color: rgba(235, 238, 250, 0.55);
}
.about-value {
  font-size: 12px;
  color: rgba(235, 238, 250, 0.75);
}

.repair-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  min-width: 82px;
  padding: 9px 16px;
  border-radius: 11px;
  border: 1px solid rgba(255, 138, 146, 0.4);
  background: rgba(255, 138, 146, 0.1);
  color: #ff8a92;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  flex-shrink: 0;
  transition:
    background 160ms,
    opacity 160ms;
}

.repair-btn:hover:not(:disabled) {
  background: rgba(255, 138, 146, 0.16);
}
.repair-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.reset-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 7px;
  padding: 11px 0;
  border-radius: 14px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  background: rgba(255, 255, 255, 0.04);
  backdrop-filter: blur(10px);
  color: rgba(235, 238, 250, 0.6);
  font-size: 12.5px;
  font-weight: 500;
  cursor: pointer;
  transition:
    color 160ms,
    background 160ms;
}

.reset-btn:hover {
  color: #ff8a92;
  background: rgba(255, 138, 146, 0.06);
}

.spin {
  animation: rotate 0.8s linear infinite;
}
@keyframes rotate {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.hotkey-btn {
  min-width: 132px;
  padding: 8px 14px;
  border-radius: 10px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  background: rgba(0, 0, 0, 0.2);
  color: #eef1fb;
  font-size: 12px;
  cursor: pointer;
  flex-shrink: 0;
  text-align: center;
  transition:
    border-color 160ms,
    background 160ms,
    color 160ms;
}

.hotkey-btn:hover {
  border-color: rgba(255, 255, 255, 0.22);
}

.hotkey-btn--recording {
  border-color: rgba(167, 139, 250, 0.65);
  background: rgba(167, 139, 250, 0.12);
  color: #d9ccff;
  animation: hotkeyPulse 1.1s ease infinite;
}

@keyframes hotkeyPulse {
  50% {
    border-color: rgba(167, 139, 250, 0.25);
  }
}

.rpc-sub {
  display: flex;
  flex-direction: column;
  margin-left: 26px;
  padding-left: 6px;
  border-left: 1px solid rgba(255, 255, 255, 0.08);
}

.rpc-fold-enter-active,
.rpc-fold-leave-active {
  transition:
    max-height 280ms cubic-bezier(0.4, 0, 0.2, 1),
    opacity 200ms ease,
    transform 280ms cubic-bezier(0.4, 0, 0.2, 1);
  overflow: hidden;
}
.rpc-fold-enter-from,
.rpc-fold-leave-to {
  max-height: 0;
  opacity: 0;
  transform: translateY(-10px);
}
.rpc-fold-enter-to,
.rpc-fold-leave-from {
  max-height: 240px;
  opacity: 1;
  transform: translateY(0);
}
.proc-ico {
  width: 26px;
  height: 26px;
  border-radius: 7px;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  background: rgba(255, 255, 255, 0.06);
}

.proc-ico img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}
.proc-ico-letter {
  font-size: 12px;
  font-weight: 700;
  color: rgba(235, 238, 250, 0.55);
}

.bypass-icon-wrap img {
  width: 16px;
  height: 16px;
  border-radius: 4px;
  object-fit: cover;
}
.chain-box {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px 14px;
  margin: 2px 0 6px;
  border-radius: 14px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(255, 255, 255, 0.03);
}

.chain-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  flex-wrap: wrap;
}

.chain-cal {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  height: 30px;
  padding: 0 12px;
  border-radius: 9px;
  border: 1px solid rgba(167, 139, 250, 0.35);
  background: rgba(167, 139, 250, 0.14);
  color: #e6dcff;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
}

.chain-cal:disabled {
  opacity: 0.6;
  cursor: default;
}

.chain-retry {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  font-size: 11.5px;
  color: rgba(235, 238, 250, 0.55);
}

.chain-retry input {
  width: 54px;
  height: 28px;
  padding: 0 8px;
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  background: rgba(0, 0, 0, 0.25);
  color: inherit;
  font-size: 12px;
  outline: none;
}

.chain-empty {
  margin: 0;
  font-size: 11.5px;
  color: rgba(235, 238, 250, 0.45);
}

.chain-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.04);
}

.chain-idx {
  font-size: 9.5px;
  font-weight: 800;
  letter-spacing: 0.07em;
  text-transform: uppercase;
  color: #a78bfa;
  flex-shrink: 0;
}

.chain-name {
  flex: 1;
  min-width: 0;
  font-size: 12.5px;
  font-weight: 600;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.chain-btn {
  width: 24px;
  height: 24px;
  border-radius: 7px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  background: transparent;
  color: rgba(235, 238, 250, 0.6);
  font-size: 13px;
  line-height: 1;
  cursor: pointer;
  flex-shrink: 0;
}

.chain-btn:hover {
  color: #e6dcff;
  border-color: rgba(167, 139, 250, 0.5);
}

.chain-btn--kill:hover {
  color: #ff8a92;
  border-color: rgba(255, 138, 146, 0.5);
}
</style>
