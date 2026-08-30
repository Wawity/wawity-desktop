<template>
  <div class="ob-shell" :class="{ 'ob-shell--booted': booted }">
    <NeutronStar :active="true" aria-hidden="true" />
    <div class="ob-vignette" aria-hidden="true"></div>

    <header v-if="step > 0" class="ob-top">
      <div class="ob-wordmark">
        <img :src="iconSrc" class="ob-logo" alt="" aria-hidden="true" />
        <span class="ob-name">wawity</span>
      </div>
      <span v-if="step < 10" class="ob-step mono" v-text="ot('stepLabel', { n: step })"></span>
    </header>

    <main class="ob-stage">
      <Transition name="pane" mode="out-in">
        <section v-if="step === 0" key="intro" class="ob-intro">
          <div class="ob-intro-halo rise" style="--d: 0.15s">
            <img :src="iconSrc" class="ob-intro-logo" alt="" aria-hidden="true" />
          </div>
          <h1 class="ob-hero-title rise" style="--d: 0.18s">Приступим к настройке!</h1>
          <p class="ob-hero-sub rise" style="--d: 0.22s">Let's get you set up!</p>
          <button
            type="button"
            class="ob-btn ob-btn--primary ob-intro-btn rise"
            style="--d: 0.28s"
            @click="step = 1"
          >
            <span>Поехали · Let's go</span>
            <ArrowRight :size="15" aria-hidden="true" />
          </button>
        </section>

        <section v-else-if="step === 1" key="lang" class="ob-lang">
          <h1 class="ob-lang-title rise" style="--d: 0.05s">На каком языке<br />разговариваем?</h1>
          <div class="ob-lang-opts rise" style="--d: 0.15s">
            <button type="button" class="ob-lang-btn" @click="chooseLanguage('ru')">
              <span class="ob-lang-label">Русский</span>
              <span class="ob-lang-hint">продолжить на русском</span>
            </button>
            <button type="button" class="ob-lang-btn" @click="chooseLanguage('en')">
              <span class="ob-lang-label">English</span>
              <span class="ob-lang-hint">continue in English</span>
            </button>
          </div>
        </section>

        
        <section v-else-if="step === 2" key="migrate" class="ob-card">
          <div v-if="migState === 'scanning'" class="ob-mig-scan">
            <span class="ob-mig-orb" aria-hidden="true"></span>
            <p class="ob-mig-status mono" v-text="ot('migScanning')"></p>
            <Transition name="pane" mode="out-in">
              <p :key="migClientLabel" class="ob-mig-client" v-text="migClientLabel"></p>
            </Transition>
          </div>

          <template v-else-if="migState === 'found'">
            <div class="ob-head">
              <div class="ob-badge ob-badge--success rise" style="--d: 0.05s">
                <HardDriveDownload :size="22" aria-hidden="true" />
              </div>
              <h2 class="ob-title rise" style="--d: 0.06s" v-text="ot('migFoundTitle', { n: migPicked })"></h2>
              <p class="ob-sub rise" style="--d: 0.11s" v-text="ot('migFoundSub')"></p>
            </div>
            <div class="ob-list ob-mig-list rise" style="--d: 0.13s">
              <button
                v-for="(sub, i) in migFound"
                :key="sub.name + i"
                type="button"
                class="ob-row"
                :class="{ 'ob-row--picked': !migSkipped.has(sub.name + i) }"
                @click="toggleMigRow(i)"
              >
                <span class="ob-row-icon"><AppWindow :size="14" aria-hidden="true" /></span>
                <span class="ob-row-text">
                  <span class="ob-row-name" v-text="sub.name"></span>
                  <span
                    class="ob-row-meta mono"
                    v-text="
                      sub.url ? shortUrl(sub.url) : ot('migNodes', { n: sub.inlineLinks.length })
                    "
                  ></span>
                </span>
                <span
                  class="ob-check"
                  :class="{ 'ob-check--on': !migSkipped.has(sub.name + i) }"
                >
                  <Check :size="12" aria-hidden="true" />
                </span>
              </button>
            </div>
            <div class="ob-actions rise" style="--d: 0.16s">
              <button
                type="button"
                class="ob-btn ob-btn--primary"
                :disabled="migImporting || migPicked === 0"
                @click="commitMigration"
              >
                <Loader2 v-if="migImporting" :size="14" class="spin" aria-hidden="true" />
                <span
                  v-text="migImporting ? migImportLabel : ot('migImport', { n: migPicked })"
                ></span>
              </button>
              <button
                type="button"
                class="ob-btn ob-btn--ghost"
                :disabled="migImporting"
                @click="skipMigration"
                v-text="ot('migSkip')"
              ></button>
            </div>
          </template>

          <template v-else>
            <div class="ob-head">
              <div class="ob-badge rise" style="--d: 0.05s">
                <HardDriveDownload :size="22" aria-hidden="true" />
              </div>
              <h2 class="ob-title rise" style="--d: 0.06s" v-text="ot('migEmptyTitle')"></h2>
              <p class="ob-sub rise" style="--d: 0.11s" v-text="ot('migEmptySub')"></p>
            </div>
            <div class="ob-actions rise" style="--d: 0.06s">
              <button
                type="button"
                class="ob-btn ob-btn--primary"
                @click="step = 3"
                v-text="ot('migContinue')"
              ></button>
            </div>
          </template>
        </section>

        <section v-else-if="step === 3" key="games" class="ob-card">
          <div class="ob-head">
            <div class="ob-badge rise" style="--d: 0.05s">
              <Gamepad2 :size="22" aria-hidden="true" />
            </div>
            <h2 class="ob-title rise" style="--d: 0.06s" v-text="ot('gamesTitle')"></h2>
            <p class="ob-sub rise" style="--d: 0.11s" v-text="ot('gamesSub')"></p>
          </div>
          <template v-if="!scanStarted">
            <div class="ob-actions rise" style="--d: 0.06s">
              <button
                type="button"
                class="ob-btn ob-btn--primary"
                @click="runGameScan"
                v-text="ot('gamesScan')"
              ></button>
              <button
                type="button"
                class="ob-btn ob-btn--ghost"
@click="step = 4"
                v-text="ot('gamesSkip')"
              ></button>
            </div>
          </template>
          <template v-else-if="scanning">
            <div class="ob-wait">
              <span class="ob-arc" aria-hidden="true"></span>
              <span class="ob-wait-text" v-text="ot('scanning')"></span>
            </div>
          </template>
          <template v-else>
            <p
              v-if="games.length"
              class="ob-note rise"
              style="--d: 0.05s"
              v-text="ot('gamesFound', { n: games.length })"
            ></p>
            <p v-else class="ob-note rise" style="--d: 0.05s" v-text="ot('gamesEmpty')"></p>
            <div v-if="games.length" class="ob-list">
              <button
                v-for="(game, index) in games"
                :key="game.key"
                type="button"
                class="ob-row rise"
                :style="{ '--d': `${Math.min(0.1 + index * 0.05, 0.55)}s` }"
                :class="{ 'ob-row--picked': chosenGames.has(game.key) }"
                @click="toggleGame(game.key)"
              >
                <span class="ob-row-icon">
                  <img v-if="icons[game.exePaths[0]]" :src="icons[game.exePaths[0]]" alt="" />
                  <span
                    v-else
                    class="ob-row-letter"
                    v-text="game.displayName.slice(0, 1).toUpperCase()"
                  ></span>
                </span>
                <span class="ob-row-text">
                  <span class="ob-row-name" v-text="game.displayName"></span>
                </span>
                <span v-if="game.recommended" class="ob-tag" v-text="ot('recommended')"></span>
                <span class="ob-check" :class="{ 'ob-check--on': chosenGames.has(game.key) }">
                  <Check :size="12" aria-hidden="true" />
                </span>
              </button>
            </div>
            <div class="ob-actions rise" style="--d: 0.05s">
              <button
                v-if="games.length"
                type="button"
                class="ob-btn ob-btn--primary"
                :disabled="chosenGames.size === 0"
                @click="commitGames"
                v-text="ot('gamesAdd', { n: chosenGames.size })"
              ></button>
              <button
                type="button"
                class="ob-btn ob-btn--ghost"
@click="step = 4"
                v-text="ot(games.length ? 'skip' : 'next')"
              ></button>
            </div>
          </template>
        </section>

        <section v-else-if="step === 4" key="exceptions" class="ob-card">
          <div class="ob-head">
            <div class="ob-badge rise" style="--d: 0.05s">
              <AppWindow :size="22" aria-hidden="true" />
            </div>
            <h2 class="ob-title rise" style="--d: 0.06s" v-text="ot('excTitle')"></h2>
            <p class="ob-sub rise" style="--d: 0.11s" v-text="ot('excSub')"></p>
          </div>
          <div class="ob-seg rise" style="--d: 0.05s">
            <button
              type="button"
              class="ob-seg-btn"
              :class="{ 'ob-seg-btn--active': excTab === 'apps' }"
              @click="openProcessTab"
            >
              <LayoutGrid :size="14" aria-hidden="true" />
              <span v-text="ot('excTabApps')"></span>
            </button>
            <button
              type="button"
              class="ob-seg-btn"
              :class="{ 'ob-seg-btn--active': excTab === 'file' }"
              @click="excTab = 'file'"
            >
              <FolderOpen :size="14" aria-hidden="true" />
              <span v-text="ot('excTabFile')"></span>
            </button>
          </div>
          <div v-if="excTab === 'apps'" class="ob-pane rise" style="--d: 0.15s">
            <div class="ob-search">
              <Search :size="14" class="ob-search-icon" aria-hidden="true" />
              <input
                v-model="processQuery"
                class="ob-search-input"
                :placeholder="ot('excSearch')"
              />
            </div>
            <div v-if="!processesLoaded" class="ob-wait">
              <span class="ob-arc" aria-hidden="true"></span>
              <span class="ob-wait-text" v-text="ot('excLoading')"></span>
            </div>
            <div v-else-if="filteredProcesses.length" class="ob-list ob-list--tall">
              <button
                v-for="app in filteredProcesses"
                :key="app.path"
                type="button"
                class="ob-row"
                :class="{ 'ob-row--picked': pickedPaths.has(app.path) }"
                @click="togglePicked(app)"
              >
                <span class="ob-row-icon">
                  <img v-if="icons[app.path]" :src="icons[app.path]" alt="" />
                  <span
                    v-else
                    class="ob-row-letter"
                    v-text="app.name.slice(0, 1).toUpperCase()"
                  ></span>
                </span>
                <span class="ob-row-text">
                  <span class="ob-row-name" v-text="app.name"></span>
                  <span class="ob-row-path mono" v-text="shortPath(app.path)"></span>
                </span>
                <span class="ob-check" :class="{ 'ob-check--on': pickedPaths.has(app.path) }">
                  <Check :size="12" aria-hidden="true" />
                </span>
              </button>
            </div>
            <p v-else class="ob-note" v-text="ot('excEmpty')"></p>
          </div>
          <div v-else class="ob-pane ob-pane--file rise" style="--d: 0.1s">
            <button type="button" class="ob-btn ob-btn--ghost" @click="pickExecutable">
              <FolderOpen :size="15" aria-hidden="true" />
              <span v-text="ot('excBrowse')"></span>
            </button>
          </div>
          <div v-if="picked.length" class="ob-chips">
            <span v-for="app in picked" :key="app.path" class="ob-chip">
              <img v-if="icons[app.path]" :src="icons[app.path]" alt="" />
              <span v-text="app.name"></span>
              <button
                type="button"
                class="ob-chip-x"
                @click="removePicked(app.path)"
                aria-label="remove"
              >
                <X :size="11" aria-hidden="true" />
              </button>
            </span>
          </div>
          <div class="ob-actions rise" style="--d: 0.48s">
            <button
              v-if="picked.length"
              type="button"
              class="ob-btn ob-btn--primary"
              @click="commitPicked"
              v-text="ot('excAdd', { n: picked.length })"
            ></button>
            <button
              type="button"
              class="ob-btn ob-btn--ghost"
              @click="step = 5"
              v-text="ot('skip')"
            ></button>
          </div>
        </section>

        <section v-else-if="step === 5" key="discord" class="ob-card">
          <div class="ob-head">
            <div class="ob-badge rise" style="--d: 0.05s">
              <MessageCircle :size="22" aria-hidden="true" />
            </div>
            <h2 class="ob-title rise" style="--d: 0.06s" v-text="ot('dcTitle')"></h2>
            <p class="ob-sub rise" style="--d: 0.11s" v-text="ot('dcSub')"></p>
          </div>
          <div class="dc-stage rise" style="--d: 0.13s">
            <div class="dc-profile">
              <div class="dc-banner"></div>
              <div class="dc-avatar">
                <img src="/avatar.jpg" alt="" />
                <span class="dc-dot"></span>
              </div>
              <div class="dc-body">
                <p class="dc-name">Wawity User</p>
                <p class="dc-handle">wawityuser</p>
                <p class="dc-about" v-text="ot('dcAbout')"></p>
                <div class="dc-activity">
                  <p class="dc-activity-label" v-text="ot('dcActivityLabel')"></p>
                  <div class="dc-activity-row">
                    <span class="dc-activity-icon">
                      <img src="/rpc.jpg" alt="" />
                    </span>
                    <span class="dc-activity-info">
                      <span class="dc-activity-name">wawity</span>
                      <span class="dc-activity-state" v-text="ot('dcState')"></span>
                      <span class="dc-activity-time">
                        <Gamepad2 :size="12" aria-hidden="true" />
                        <span class="mono">20:52</span>
                      </span>
                    </span>
                  </div>
                </div>
                <div class="dc-message-btn" v-text="ot('dcBtn')"></div>
              </div>
            </div>
            <div class="dc-side">
              <p class="dc-caption" v-text="ot('dcCaptionChat')"></p>
              <div class="dc-member">
                <img class="dc-member-avatar" src="/avatar.jpg" alt="" />
                <span class="dc-member-info">
                  <span class="dc-member-name">Wawity User</span>
                  <span class="dc-member-game">
                    <Gamepad2 :size="11" aria-hidden="true" />
                    <span>wawity</span>
                  </span>
                </span>
              </div>
            </div>
          </div>
          <div class="ob-setting rise" style="--d: 0.15s">
            <div class="ob-setting-left">
              <MessageCircle :size="16" class="ob-setting-icon" aria-hidden="true" />
              <span class="ob-setting-title" v-text="ot('dcToggle')"></span>
            </div>
            <button
              type="button"
              class="ob-toggle"
              :class="{ 'ob-toggle--on': discordOn }"
              @click="discordOn = !discordOn"
              role="switch"
              :aria-checked="discordOn"
            >
              <span class="ob-toggle-thumb" :class="{ 'ob-toggle-thumb--on': discordOn }"></span>
            </button>
          </div>
          <p class="ob-fineprint rise" style="--d: 0.16s" v-text="ot('dcPrivacyNote')"></p>
          <div class="ob-actions rise" style="--d: 0.56s">
            <button
              type="button"
              class="ob-btn ob-btn--primary"
@click="step = 6"
              v-text="ot('next')"
            ></button>
          </div>
        </section>

        <section v-else-if="step === 6" key="killswitch" class="ob-card">
          <div class="ob-head">
            <div class="ob-badge ob-badge--danger rise" style="--d: 0.05s">
              <ShieldAlert :size="22" aria-hidden="true" />
            </div>
            <h2 class="ob-title rise" style="--d: 0.06s" v-text="ot('ksTitle')"></h2>
            <p class="ob-sub rise" style="--d: 0.11s" v-text="ot('ksWhat')"></p>
          </div>
          <div class="ob-cols rise" style="--d: 0.13s">
            <div class="ob-col ob-col--good">
              <p class="ob-col-title">
                <ThumbsUp :size="13" aria-hidden="true" />
                <span v-text="ot('pros')"></span>
              </p>
              <ul>
                <li v-for="(line, i) in ksPros" :key="i" v-text="line"></li>
              </ul>
            </div>
            <div class="ob-col ob-col--bad">
              <p class="ob-col-title">
                <ThumbsDown :size="13" aria-hidden="true" />
                <span v-text="ot('cons')"></span>
              </p>
              <ul>
                <li v-for="(line, i) in ksCons" :key="i" v-text="line"></li>
              </ul>
            </div>
          </div>
          <p class="ob-fit rise" style="--d: 0.42s">
            <strong v-text="ot('fitFor')"></strong>
            <span v-text="ot('ksFor')"></span>
          </p>
          <p class="ob-fit rise" style="--d: 0.16s">
            <strong v-text="ot('notFor')"></strong>
            <span v-text="ot('ksNot')"></span>
          </p>
          <div class="ob-setting rise" style="--d: 0.52s">
            <div class="ob-setting-left">
              <ShieldCheck :size="16" class="ob-setting-icon" aria-hidden="true" />
              <span class="ob-setting-title" v-text="ot('ksToggle')"></span>
            </div>
            <button
              type="button"
              class="ob-toggle"
              :class="{ 'ob-toggle--on': killSwitchOn }"
              @click="killSwitchOn = !killSwitchOn"
              role="switch"
              :aria-checked="killSwitchOn"
            >
              <span class="ob-toggle-thumb" :class="{ 'ob-toggle-thumb--on': killSwitchOn }"></span>
            </button>
          </div>
          <div class="ob-actions rise" style="--d: 0.58s">
            <button
              type="button"
              class="ob-btn ob-btn--primary"
@click="step = 7"
              v-text="ot('next')"
            ></button>
          </div>
        </section>

        <section v-else-if="step === 7" key="alwayson" class="ob-card">
          <div class="ob-head">
            <div class="ob-badge rise" style="--d: 0.05s">
              <Lock :size="22" aria-hidden="true" />
            </div>
            <h2 class="ob-title rise" style="--d: 0.06s" v-text="ot('aoTitle')"></h2>
            <p class="ob-sub rise" style="--d: 0.11s" v-text="ot('aoWhat')"></p>
          </div>
          <div class="ob-cols rise" style="--d: 0.13s">
            <div class="ob-col ob-col--good">
              <p class="ob-col-title">
                <ThumbsUp :size="13" aria-hidden="true" />
                <span v-text="ot('pros')"></span>
              </p>
              <ul>
                <li v-for="(line, i) in aoPros" :key="i" v-text="line"></li>
              </ul>
            </div>
            <div class="ob-col ob-col--bad">
              <p class="ob-col-title">
                <ThumbsDown :size="13" aria-hidden="true" />
                <span v-text="ot('cons')"></span>
              </p>
              <ul>
                <li v-for="(line, i) in aoCons" :key="i" v-text="line"></li>
              </ul>
            </div>
          </div>
          <p class="ob-fit rise" style="--d: 0.42s">
            <strong v-text="ot('fitFor')"></strong>
            <span v-text="ot('aoFor')"></span>
          </p>
          <p class="ob-fit rise" style="--d: 0.16s">
            <strong v-text="ot('notFor')"></strong>
            <span v-text="ot('aoNot')"></span>
          </p>
          <div class="ob-setting rise" style="--d: 0.52s">
            <div class="ob-setting-left">
              <Lock :size="16" class="ob-setting-icon" aria-hidden="true" />
              <span class="ob-setting-title" v-text="ot('aoToggle')"></span>
            </div>
            <button
              type="button"
              class="ob-toggle"
              :class="{ 'ob-toggle--on': alwaysOnOn }"
              @click="alwaysOnOn = !alwaysOnOn"
              role="switch"
              :aria-checked="alwaysOnOn"
            >
              <span class="ob-toggle-thumb" :class="{ 'ob-toggle-thumb--on': alwaysOnOn }"></span>
            </button>
          </div>
          <p v-if="alwaysOnOn" class="ob-fineprint" v-text="ot('aoNote')"></p>
          <div class="ob-actions rise" style="--d: 0.58s">
            <button
              type="button"
              class="ob-btn ob-btn--primary"
@click="step = 8"
              v-text="ot('next')"
            ></button>
          </div>
        </section>

        <section v-else-if="step === 8" key="privacy" class="ob-card">
          <div class="ob-head">
            <div class="ob-badge rise" style="--d: 0.05s">
              <BarChart3 :size="22" aria-hidden="true" />
            </div>
            <h2 class="ob-title rise" style="--d: 0.06s" v-text="ot('tmTitle')"></h2>
            <p class="ob-sub rise" style="--d: 0.11s" v-text="ot('tmSub')"></p>
          </div>
          <div class="ob-cols rise" style="--d: 0.13s">
            <div class="ob-col ob-col--good">
              <p class="ob-col-title">
                <ThumbsUp :size="13" aria-hidden="true" />
                <span v-text="ot('tmYesTitle')"></span>
              </p>
              <ul>
                <li v-for="(line, i) in tmYes" :key="i" v-text="line"></li>
              </ul>
            </div>
            <div class="ob-col ob-col--bad">
              <p class="ob-col-title">
                <ThumbsDown :size="13" aria-hidden="true" />
                <span v-text="ot('tmNoTitle')"></span>
              </p>
              <ul>
                <li v-for="(line, i) in tmNo" :key="i" v-text="line"></li>
              </ul>
            </div>
          </div>
          <div class="ob-setting rise" style="--d: 0.16s">
            <div class="ob-setting-left">
              <BarChart3 :size="16" class="ob-setting-icon" aria-hidden="true" />
              <span class="ob-setting-title" v-text="ot('tmToggle')"></span>
            </div>
            <button
              type="button"
              class="ob-toggle"
              :class="{ 'ob-toggle--on': telemetryOn }"
              @click="telemetryOn = !telemetryOn"
              role="switch"
              :aria-checked="telemetryOn"
            >
              <span class="ob-toggle-thumb" :class="{ 'ob-toggle-thumb--on': telemetryOn }"></span>
            </button>
          </div>
          <p class="ob-fineprint rise" style="--d: 0.56s" v-text="ot('tmNote')"></p>
          <div class="ob-actions rise" style="--d: 0.62s">
            <button
              type="button"
              class="ob-btn ob-btn--primary"
@click="step = 9"
              v-text="ot('next')"
            ></button>
          </div>
        </section>

        <section v-else-if="step === 9" key="subscription" class="ob-card ob-card--hero">
          <div class="ob-badge ob-badge--success rise" style="--d: 0.05s">
            <Rocket :size="22" aria-hidden="true" />
          </div>
          <h2 class="ob-title ob-title--big rise" style="--d: 0.06s" v-text="ot('subTitle')"></h2>
          <p class="ob-sub rise" style="--d: 0.11s" v-text="ot('subSub')"></p>
          <div class="ob-pills rise" style="--d: 0.06s">
            <button type="button" class="ob-pill ob-pill--glow" @click="step = 10">
              <span class="ob-pill-glyph">
                <Link2 :size="15" aria-hidden="true" />
              </span>
              <span class="ob-pill-copy">
                <span class="ob-pill-name" v-text="ot('subYes')"></span>
              </span>
              <ArrowRight :size="15" class="ob-pill-arrow" aria-hidden="true" />
            </button>
            <button type="button" class="ob-pill" :disabled="finishing" @click="finalize(false)">
              <span class="ob-pill-glyph">
                <Sparkles :size="15" aria-hidden="true" />
              </span>
              <span class="ob-pill-copy">
                <span class="ob-pill-name" v-text="ot('subNo')"></span>
                <span class="ob-pill-hint" v-text="ot('subNoHint')"></span>
              </span>
              <ArrowRight :size="15" class="ob-pill-arrow" aria-hidden="true" />
            </button>
          </div>
        </section>

        <section v-else key="coach" class="ob-card">
          <div class="ob-head">
            <div class="ob-badge rise" style="--d: 0.05s">
              <Link2 :size="22" aria-hidden="true" />
            </div>
            <h2 class="ob-title rise" style="--d: 0.06s" v-text="ot('coachTitle')"></h2>
            <p class="ob-sub rise" style="--d: 0.11s" v-text="ot('coachSub')"></p>
          </div>
          <div class="ob-coach rise" style="--d: 0.13s">
            <div class="ob-bubble">
              <span v-text="ot('coachStep1')"></span>
              <ArrowDown :size="14" class="ob-bubble-arrow" aria-hidden="true" />
            </div>
            <div class="ob-mock">
              <span class="ob-mock-input mono">https://sub.example.com/token…</span>
              <span class="ob-mock-btn" v-text="ot('coachFetch')"></span>
            </div>
            <div class="ob-bubble ob-bubble--below">
              <ArrowUp :size="14" class="ob-bubble-arrow" aria-hidden="true" />
              <span v-text="ot('coachStep2')"></span>
            </div>
          </div>
          <div class="ob-actions rise" style="--d: 0.16s">
            <button
              type="button"
              class="ob-btn ob-btn--primary"
              :disabled="finishing"
              @click="finalize(true)"
              v-text="ot('coachGo')"
            ></button>
          </div>
        </section>
      </Transition>
    </main>

    <footer v-if="step > 0" class="ob-footer">
      <button v-if="step < 10" type="button" class="ob-back" @click="goBack">
        <ArrowLeft :size="14" aria-hidden="true" />
        <span v-text="ot('back')"></span>
      </button>
      <div class="ob-dots" aria-hidden="true">
        <span v-for="i in 9" :key="i" class="ob-dot" :class="{ 'ob-dot--on': step >= i }"></span>
      </div>
    </footer>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch, onMounted, onBeforeUnmount } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import { open } from '@tauri-apps/api/dialog';
import {
  Languages,
  BarChart3,
  Gamepad2,
  AppWindow,
  LayoutGrid,
  FolderOpen,
  Search,
  Check,
  X,
  MessageCircle,
  ShieldAlert,
  ShieldCheck,
  Lock,
  ThumbsUp,
  ThumbsDown,
  Rocket,
  Link2,
  Sparkles,
  ArrowDown,
  ArrowUp,
  ArrowLeft,
  ArrowRight,
  HardDriveDownload,
  Loader2,
} from '../lib/appIcons';
import { useVpnStore } from '../stores/vpn';
import { useAppIcon } from '../composables/useAppIcon';
import { useNotifications } from '../composables/useNotifications';
import { t, setLanguage } from '../i18n';
import NeutronStar from './NeutronStar.vue';

interface DetectedGame {
  key: string;
  displayName: string;
  exePaths: string[];
  recommended: boolean;
}

interface RunningApp {
  name: string;
  path: string;
}

const emit = defineEmits<{
  (e: 'done', payload: { goToServers: boolean }): void;
}>();

const store = useVpnStore();
const { pushToast } = useNotifications();
const { iconSrc } = useAppIcon();

const booted = ref(false);
let bootRaf = 0;

onMounted(() => {
  const reveal = () => {
    bootRaf = requestAnimationFrame(() => {
      bootRaf = requestAnimationFrame(() => {
        booted.value = true;
      });
    });
  };
  const fonts = (document as any).fonts;
  if (fonts && typeof fonts.ready?.then === 'function') {
    let settled = false;
    const go = () => {
      if (settled) return;
      settled = true;
      reveal();
    };
    fonts.ready.then(go).catch(go);
    setTimeout(go, 400);
  } else {
    reveal();
  }
});

onBeforeUnmount(() => {
  if (bootRaf) cancelAnimationFrame(bootRaf);
});

const step = ref(0);

const scanStarted = ref(false);
const scanning = ref(false);
const games = ref<DetectedGame[]>([]);
const chosenGames = reactive(new Set<string>());

const excTab = ref<'apps' | 'file'>('apps');
const processes = ref<RunningApp[]>([]);
const processesLoaded = ref(false);
const processesRequested = ref(false);
const processQuery = ref('');
const picked = ref<RunningApp[]>([]);

const discordOn = ref(true);
const killSwitchOn = ref(true);
const alwaysOnOn = ref(false);
const telemetryOn = ref(false);
const finishing = ref(false);

const icons = reactive<Record<string, string>>({});

function ot(key: string, params?: Record<string, string | number>): string {
  return t(`onboarding.${key}`, params);
}

const ksPros = computed(() => [ot('ksPros1'), ot('ksPros2'), ot('ksPros3')]);
const ksCons = computed(() => [ot('ksCons1'), ot('ksCons2')]);
const aoPros = computed(() => [ot('aoPros1'), ot('aoPros2'), ot('aoPros3')]);
const aoCons = computed(() => [ot('aoCons1'), ot('aoCons2')]);
const tmYes = computed(() => [ot('tmYes1'), ot('tmYes2'), ot('tmYes3')]);
const tmNo = computed(() => [ot('tmNo1'), ot('tmNo2'), ot('tmNo3')]);

function chooseLanguage(next: 'ru' | 'en') {
  setLanguage(next);
  store.updateSettings({ language: next });
  step.value = 2;
}

function goBack() {
  if (step.value > 0) step.value -= 1;
}



interface ForeignSub {
  name: string;
  url: string | null;
  inlineLinks: string[];
}

const MIG_CLIENTS = ['v2rayN', 'Clash Verge', 'Clash / mihomo', 'Nekoray', 'Hiddify'];

const migState = ref<'scanning' | 'found' | 'none'>('scanning');
const migLaunched = ref(false);
const migFound = ref<ForeignSub[]>([]);
const migSkipped = reactive(new Set<string>());
const migImporting = ref(false);
const migImportLabel = ref('');
const migClientLabel = ref(MIG_CLIENTS[0]);
let migLabelTimer = 0;

watch(step, (value) => {
  if (value === 2 && !migLaunched.value) {
    migLaunched.value = true;
    void startMigrationScan();
  }
});

async function startMigrationScan() {
  migState.value = 'scanning';
  let idx = 0;
  migClientLabel.value = MIG_CLIENTS[0];
  migLabelTimer = window.setInterval(() => {
    idx += 1;
    migClientLabel.value = MIG_CLIENTS[idx % MIG_CLIENTS.length];
  }, 460);

  let result: { subscriptions: ForeignSub[] } | null = null;
  try {
    result = await invoke<{ clients: unknown[]; subscriptions: ForeignSub[] }>(
      'scan_foreign_clients',
    );
  } catch {}

  
  const elapsed = idx * 460;
  if (elapsed < 2000) {
    await new Promise((resolve) => setTimeout(resolve, 2000 - elapsed));
  }
  window.clearInterval(migLabelTimer);

  const importable = (result?.subscriptions ?? []).filter(
    (sub) => typeof sub.url === 'string' && sub.url.length > 10,
  );
  if (importable.length === 0) {
    migState.value = 'none';
    return;
  }
  migFound.value = importable.slice(0, 12);
  migState.value = 'found';
}

function rowKey(i: number): string {
  return `${migFound.value[i]?.name ?? ''}#${i}`;
}

function toggleMigRow(i: number) {
  const key = rowKey(i);
  if (migSkipped.has(key)) migSkipped.delete(key);
  else migSkipped.add(key);
}

const migPicked = computed(() => migFound.value.length - migSkipped.size);

function shortUrl(url: string): string {
  try {
    const parsed = new URL(url);
    return parsed.hostname.replace(/^www\./, '');
  } catch {
    return url.slice(0, 40);
  }
}

async function commitMigration() {
  if (migImporting.value) return;
  migImporting.value = true;
  const total = migPicked.value;
  migImportLabel.value = ot('migImport', { n: total });
  let imported = 0;
  let failed = 0;
  try {
    for (let i = 0; i < migFound.value.length; i++) {
      if (migSkipped.has(rowKey(i))) continue;
      const sub = migFound.value[i];
      if (!sub.url) continue;
      try {
        const preview = await store.fetchSubscriptionPreview(sub.url);
        if (preview.servers.length > 0) {
          store.addSubscription(sub.url, sub.name, preview.servers);
          imported += 1;
        } else {
          failed += 1;
        }
      } catch {
        failed += 1;
      }
      migImportLabel.value = ot('migProgress', {
        done: imported + failed,
        total,
      });
    }
  } finally {
    migImporting.value = false;
  }
  if (imported > 0) {
    pushToast('success', ot('migDone'), ot('migDoneDesc', { n: imported, total: imported + failed }), 4500);
  } else if (failed > 0) {
    pushToast('error', ot('migFail'), ot('migFailDesc'), 6000);
  }
  step.value = 3;
}

function skipMigration() {
  step.value = 3;
}

async function fetchIcons(paths: string[]) {
  const missing = [...new Set(paths.filter((path) => path && !icons[path]))].slice(0, 400);
  if (missing.length === 0) return;
  for (let at = 0; at < missing.length; at += 64) {
    const slice = missing.slice(at, at + 64);
    try {
      const loaded = await invoke<Array<string | null>>('collect_app_icons', {
        paths: slice,
      });
      loaded.forEach((image, index) => {
        if (image) icons[slice[index]] = image;
      });
    } catch {
      return;
    }
  }
}

async function runGameScan() {
  scanStarted.value = true;
  scanning.value = true;
  try {
    const found = await invoke<DetectedGame[]>('scan_installed_games');
    games.value = found;
    for (const game of found) {
      if (game.recommended) chosenGames.add(game.key);
    }
    fetchIcons(found.map((game) => game.exePaths[0]).filter(Boolean));
  } catch {
    games.value = [];
  } finally {
    scanning.value = false;
  }
}

function toggleGame(key: string) {
  if (chosenGames.has(key)) chosenGames.delete(key);
  else chosenGames.add(key);
}

async function commitGames() {
  const paths = games.value
    .filter((game) => chosenGames.has(game.key))
    .flatMap((game) => game.exePaths);
  if (paths.length) {
    try {
      await store.addBypassApps(paths);
    } catch {}
  }
  step.value = 4;
}

async function openProcessTab() {
  excTab.value = 'apps';
  if (processesRequested.value) return;
  processesRequested.value = true;
  try {
    const raw = await invoke<RunningApp[]>('list_installed_apps');
    processes.value = raw;
    fetchIcons(raw.map((app) => app.path));
  } catch {
    processes.value = [];
  } finally {
    processesLoaded.value = true;
  }
}

watch(step, (now) => {
  if (now === 3 && !processesRequested.value) {
    openProcessTab();
  }
});

const filteredProcesses = computed(() => {
  const query = processQuery.value.trim().toLowerCase();
  if (!query) return processes.value;
  return processes.value.filter(
    (app) => app.name.toLowerCase().includes(query) || app.path.toLowerCase().includes(query),
  );
});

const pickedPaths = computed(() => new Set(picked.value.map((app) => app.path)));

function togglePicked(app: RunningApp) {
  if (pickedPaths.value.has(app.path)) removePicked(app.path);
  else picked.value = [...picked.value, app];
}

function removePicked(path: string) {
  picked.value = picked.value.filter((app) => app.path !== path);
}

async function pickExecutable() {
  try {
    const chosen = await open({
      multiple: true,
      filters: [{ name: ot('excFilter'), extensions: ['exe'] }],
    });
    if (!chosen) return;
    const list = Array.isArray(chosen) ? chosen : [chosen];
    for (const path of list) {
      if (pickedPaths.value.has(path)) continue;
      const name = (path.split(/[\\/]/).pop() || path).replace(/\.exe$/i, '');
      picked.value = [...picked.value, { name, path }];
    }
    fetchIcons(list);
  } catch {}
}

async function commitPicked() {
  const paths = picked.value.map((app) => app.path);
  if (paths.length) {
    try {
      await store.addBypassApps(paths);
    } catch {}
  }
  step.value = 5;
}

function shortPath(path: string): string {
  const parts = path.split(/[\\/]/);
  if (parts.length <= 3) return path;
  return `…\\${parts.slice(-2).join('\\')}`;
}

async function finalize(goToServers: boolean) {
  if (finishing.value) return;
  finishing.value = true;
  try {
    store.updateSettings({
      kill_switch: killSwitchOn.value,
      discord_rpc: discordOn.value,
      telemetry: telemetryOn.value,
    });
    if (alwaysOnOn.value) {
      await store.setAlwaysOn(true);
    }
  } catch {}
  emit('done', { goToServers });
}
</script>

<style scoped>


.ob-mig-scan {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 14px;
  padding: 34px 0;
}

.ob-mig-orb {
  position: relative;
  width: 58px;
  height: 58px;
  border-radius: 50%;
  border: 2px solid rgba(167, 139, 250, 0.16);
  border-top-color: rgba(167, 139, 250, 0.9);
  animation: migSpin 1.05s linear infinite;
}

.ob-mig-orb::after {
  content: '';
  position: absolute;
  inset: -10px;
  border-radius: 50%;
  background: radial-gradient(circle, rgba(124, 92, 255, 0.22), transparent 68%);
  animation: migPulse 1.8s ease-in-out infinite;
}

@keyframes migSpin {
  to {
    transform: rotate(360deg);
  }
}

@keyframes migPulse {
  0%,
  100% {
    opacity: 0.55;
    transform: scale(1);
  }
  50% {
    opacity: 1;
    transform: scale(1.12);
  }
}

.ob-mig-status {
  margin: 6px 0 0;
  font-size: 13px;
  color: var(--foreground);
  letter-spacing: 0.01em;
}

.ob-mig-client {
  margin: 0;
  min-height: 18px;
  font-size: 11px;
  color: rgba(235, 238, 250, 0.42);
  transition:
    opacity 200ms ease,
    filter 220ms ease,
    transform 220ms ease;
}

.ob-mig-list {
  max-height: 240px;
  overflow-y: auto;
}

.ob-row-meta {
  font-size: 10px;
  color: rgba(235, 238, 250, 0.38);
}
</style>

<style scoped>
.ob-shell {
  opacity: 0;
  transition: opacity 0.32s ease;
}

.ob-shell--booted {
  opacity: 1;
}

.ob-stage-inner {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.ob-shell {
  position: fixed;
  inset: 0;
  z-index: 300;
  display: flex;
  flex-direction: column;
  background: #04050a;
  color: var(--foreground);
  font-family: var(--font-sans);
  user-select: none;
  overflow: hidden;
}

.mono {
  font-family: var(--font-mono);
  font-variant-numeric: tabular-nums;
}

.ob-vignette {
  position: absolute;
  inset: 0;
  z-index: 1;
  pointer-events: none;
  background: radial-gradient(ellipse at center, transparent 42%, rgba(0, 0, 0, 0.5) 100%);
}

.rise {
  opacity: 0;
  animation: ob-rise 0.24s ease-out forwards;
  animation-delay: var(--d, 0s);
}

@keyframes ob-rise {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

.ob-top {
  position: relative;
  z-index: 2;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 18px 24px 0;
  flex-shrink: 0;
}

.ob-wordmark {
  display: flex;
  align-items: center;
  gap: 9px;
}
.ob-logo {
  width: 22px;
  height: 22px;
  object-fit: contain;
}
.ob-name {
  font-size: 15px;
  font-weight: 700;
  letter-spacing: 0.02em;
}

.ob-step {
  font-size: 11px;
  color: color-mix(in oklch, var(--muted-foreground) 80%, transparent);
}

.ob-stage {
  position: relative;
  z-index: 2;
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 18px 24px;
  overflow-y: auto;
}

.ob-intro {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  gap: 6px;
  padding: 24px;
}

.ob-intro-halo {
  position: relative;
  width: 116px;
  height: 116px;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 14px;
}

.ob-intro-logo {
  width: 88px;
  height: 88px;
  object-fit: contain;
  position: relative;
  filter: drop-shadow(0 0 26px rgba(167, 139, 250, 0.5));
  animation: ob-float 5.5s ease-in-out infinite;
}

@keyframes ob-float {
  0%,
  100% {
    transform: translateY(0);
  }
  50% {
    transform: translateY(-9px);
  }
}

.ob-hero-title {
  margin: 0;
  font-size: 34px;
  font-weight: 700;
  letter-spacing: -0.025em;
  background: linear-gradient(100deg, #ffffff 25%, #c9b8ff 45%, #ffffff 65%);
  background-size: 220% 100%;
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
  animation:
    ob-rise 0.65s cubic-bezier(0.22, 0.9, 0.3, 1) forwards,
    ob-shimmer 6s ease-in-out 1.4s infinite;
}

.ob-hero-sub {
  margin: 0;
  font-size: 14.5px;
  color: rgba(235, 238, 250, 0.55);
}

.ob-intro-btn {
  margin-top: 26px;
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 13px 30px;
  flex: 0 0 auto;
}

@keyframes ob-shimmer {
  0% {
    background-position: 130% 0;
  }
  100% {
    background-position: -130% 0;
  }
}

.ob-card {
  width: 100%;
  max-width: 580px;
  max-height: calc(100vh - 160px);
  overflow-y: auto;
  overflow-x: hidden;
  padding: 24px 24px 0;
  border-radius: 20px;
  border: 1px solid rgba(255, 255, 255, 0.09);
  background: rgba(10, 11, 20, 0.34);
  backdrop-filter: blur(7px);
  -webkit-backdrop-filter: blur(7px);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.07),
    0 18px 44px rgba(0, 0, 0, 0.4);
  display: flex;
  flex-direction: column;
  gap: 14px;
  scrollbar-width: thin;
  scrollbar-color: rgba(255, 255, 255, 0.08) transparent;
}

.ob-card--hero {
  align-items: center;
  text-align: center;
  padding-top: 36px;
  padding-bottom: 36px;
}

.ob-head {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.ob-badge {
  width: 46px;
  height: 46px;
  border-radius: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.07);
  color: rgba(235, 238, 250, 0.75);
  margin-bottom: 4px;
  flex-shrink: 0;
  border: 1px solid rgba(255, 255, 255, 0.08);
}

.ob-badge--danger {
  color: rgba(235, 238, 250, 0.75);
}
.ob-badge--success {
  color: rgba(235, 238, 250, 0.75);
}

.ob-title {
  margin: 0;
  font-size: 22px;
  font-weight: 650;
  letter-spacing: -0.02em;
  background: linear-gradient(100deg, #ffffff 25%, #cfc2ff 48%, #ffffff 70%);
  background-size: 220% 100%;
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
  animation:
    ob-rise 0.65s cubic-bezier(0.22, 0.9, 0.3, 1) forwards,
    ob-shimmer 7s ease-in-out 1.2s infinite;
}

.ob-title--big {
  font-size: 26px;
}

.ob-sub {
  font-size: 13px;
  line-height: 1.55;
  color: rgba(235, 238, 250, 0.55);
  margin: 0;
}

.ob-note {
  font-size: 12.5px;
  line-height: 1.5;
  color: rgba(235, 238, 250, 0.6);
  margin: 0;
}

.ob-fineprint {
  font-size: 11.5px;
  line-height: 1.5;
  color: rgba(235, 238, 250, 0.4);
  margin: 0;
}

.ob-fit {
  font-size: 12.5px;
  line-height: 1.5;
  color: rgba(235, 238, 250, 0.6);
  margin: 0;
}

.ob-fit strong {
  color: rgba(235, 238, 250, 0.85);
  font-weight: 600;
}

.ob-lang {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  gap: 48px;
  padding: 16px;
}

.ob-lang-title {
  margin: 0;
  font-size: 38px;
  font-weight: 700;
  letter-spacing: -0.03em;
  line-height: 1.15;
  background: linear-gradient(120deg, #ffffff 20%, #d4bfff 50%, #ffffff 80%);
  background-size: 220% 100%;
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
  animation:
    ob-rise 0.65s cubic-bezier(0.22, 0.9, 0.3, 1) forwards,
    ob-shimmer 7s ease-in-out 1.2s infinite;
}

.ob-lang-opts {
  display: flex;
  flex-direction: column;
  gap: 14px;
  width: 100%;
  max-width: 320px;
}

.ob-lang-btn {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 18px 26px;
  border-radius: 999px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  background: rgba(255, 255, 255, 0.055);
  backdrop-filter: blur(11px);
  -webkit-backdrop-filter: blur(11px);
  color: var(--foreground);
  cursor: pointer;
  font-family: var(--font-sans);
  transition:
    transform 0.3s cubic-bezier(0.34, 1.4, 0.64, 1),
    border-color 0.25s ease,
    background 0.25s ease,
    box-shadow 0.25s ease;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.08);
}

.ob-lang-btn:hover {
  transform: translateY(-3px) scale(1.015);
  border-color: rgba(167, 139, 250, 0.55);
  background: rgba(167, 139, 250, 0.1);
  box-shadow:
    0 12px 32px rgba(0, 0, 0, 0.35),
    0 0 28px rgba(167, 139, 250, 0.14),
    inset 0 1px 0 rgba(255, 255, 255, 0.1);
}

.ob-lang-btn:active {
  transform: scale(0.97);
}

.ob-lang-label {
  font-size: 16px;
  font-weight: 600;
  letter-spacing: -0.01em;
}

.ob-lang-hint {
  font-size: 12px;
  color: rgba(235, 238, 250, 0.4);
  font-style: italic;
}

.ob-pills {
  display: flex;
  flex-direction: column;
  gap: 10px;
  width: 100%;
  max-width: 400px;
  margin: 8px auto 0;
}

.ob-pill {
  display: flex;
  align-items: center;
  gap: 12px;
  width: 100%;
  padding: 12px 20px;
  border-radius: 999px;
  border: 1px solid rgba(255, 255, 255, 0.11);
  background: rgba(255, 255, 255, 0.05);
  color: var(--foreground);
  cursor: pointer;
  font-family: var(--font-sans);
  text-align: left;
  transition:
    transform 0.28s cubic-bezier(0.34, 1.4, 0.64, 1),
    border-color 0.25s ease,
    background 0.25s ease,
    box-shadow 0.25s ease;
}

.ob-pill:hover {
  transform: translateX(4px);
  border-color: rgba(167, 139, 250, 0.5);
  background: rgba(167, 139, 250, 0.1);
  box-shadow:
    0 10px 26px rgba(0, 0, 0, 0.32),
    0 0 20px rgba(167, 139, 250, 0.12);
}

.ob-pill:hover .ob-pill-arrow {
  opacity: 1;
  transform: translateX(2px);
}
.ob-pill:active {
  transform: translateX(2px) scale(0.985);
}
.ob-pill:disabled {
  opacity: 0.6;
  cursor: default;
}

.ob-pill--glow {
  border-color: rgba(167, 139, 250, 0.42);
  background: linear-gradient(135deg, rgba(124, 92, 255, 0.22), rgba(124, 92, 255, 0.07));
}

.ob-pill-glyph {
  width: 34px;
  height: 34px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  background: linear-gradient(180deg, rgba(167, 139, 250, 0.28), rgba(124, 92, 255, 0.16));
  color: #cfc2ff;
}

.ob-pill-copy {
  display: flex;
  flex-direction: column;
  gap: 1px;
  min-width: 0;
}

.ob-pill-name {
  font-size: 13.5px;
  font-weight: 600;
}
.ob-pill-hint {
  font-size: 11px;
  color: rgba(235, 238, 250, 0.45);
}

.ob-pill-arrow {
  margin-left: auto;
  flex-shrink: 0;
  opacity: 0.45;
  transition:
    opacity 0.25s ease,
    transform 0.25s ease;
}

.ob-actions {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
  position: sticky;
  bottom: 0;
  margin: 4px -24px 0;
  padding: 12px 24px 20px;
  background: linear-gradient(to bottom, transparent, rgba(8, 7, 16, 0.92) 28%);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  z-index: 2;
}

.ob-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 7px;
  flex: 1;
  min-width: 150px;
  padding: 13px 22px;
  border-radius: 999px;
  border: 1px solid transparent;
  font-size: 13px;
  font-weight: 700;
  font-family: var(--font-sans);
  cursor: pointer;
  transition:
    transform 0.25s cubic-bezier(0.34, 1.4, 0.64, 1),
    opacity 0.2s ease,
    box-shadow 0.25s ease,
    background 0.2s ease;
}

.ob-btn--primary {
  background: var(--primary);
  color: var(--primary-foreground);
}

.ob-btn--primary:hover:not(:disabled) {
  opacity: 0.88;
  transform: translateY(-2px);
  box-shadow: 0 10px 26px rgba(0, 0, 0, 0.35);
}

.ob-btn--ghost {
  background: rgba(255, 255, 255, 0.05);
  border-color: var(--border);
  color: var(--foreground);
}

.ob-btn--ghost:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.09);
  transform: translateY(-2px);
}

.ob-btn:active:not(:disabled) {
  transform: scale(0.96);
}
.ob-btn:disabled {
  opacity: 0.45;
  cursor: default;
}

.ob-wait {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 20px 4px;
}

.ob-arc {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  border: 2px solid rgba(167, 139, 250, 0.2);
  border-top-color: #a78bfa;
  animation: ob-spin 0.8s linear infinite;
  flex-shrink: 0;
}

@keyframes ob-spin {
  to {
    transform: rotate(360deg);
  }
}

.ob-wait-text {
  font-size: 13px;
  color: rgba(235, 238, 250, 0.6);
}

.ob-list {
  display: flex;
  flex-direction: column;
  max-height: 240px;
  overflow-y: auto;
  border-radius: 14px;
  border: 1px solid rgba(255, 255, 255, 0.07);
  background: rgba(0, 0, 0, 0.18);
}

.ob-list--tall {
  max-height: 260px;
}

.ob-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 11px 14px;
  border: none;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  background: transparent;
  color: var(--foreground);
  cursor: pointer;
  text-align: left;
  font-family: var(--font-sans);
  transition: background 0.18s ease;
}

.ob-row:last-child {
  border-bottom: none;
}
.ob-row:hover {
  background: rgba(255, 255, 255, 0.045);
}
.ob-row--picked {
  background: rgba(167, 139, 250, 0.08);
}
.ob-row--picked:hover {
  background: rgba(167, 139, 250, 0.12);
}

.ob-row-icon {
  width: 30px;
  height: 30px;
  border-radius: 8px;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.06);
  flex-shrink: 0;
}

.ob-row-icon img {
  width: 22px;
  height: 22px;
  object-fit: contain;
}

.ob-row-letter {
  font-size: 13px;
  font-weight: 700;
  color: rgba(235, 238, 250, 0.55);
}

.ob-row-text {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
  flex: 1;
}

.ob-row-name {
  font-size: 13px;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.ob-row-path {
  font-size: 10.5px;
  color: rgba(235, 238, 250, 0.35);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.ob-tag {
  font-size: 10.5px;
  font-weight: 600;
  letter-spacing: 0.02em;
  padding: 3px 9px;
  border-radius: 999px;
  background: rgba(167, 139, 250, 0.14);
  color: #cfc2ff;
  flex-shrink: 0;
  white-space: nowrap;
}

.ob-check {
  width: 20px;
  height: 20px;
  border-radius: 7px;
  border: 1px solid rgba(255, 255, 255, 0.16);
  display: flex;
  align-items: center;
  justify-content: center;
  color: transparent;
  flex-shrink: 0;
  transition: all 0.22s cubic-bezier(0.34, 1.4, 0.64, 1);
}

.ob-check--on {
  background: linear-gradient(180deg, rgba(167, 139, 250, 0.9), rgba(124, 92, 255, 0.8));
  border-color: rgba(167, 139, 250, 0.6);
  color: #fff;
}

.ob-seg {
  display: flex;
  gap: 4px;
  padding: 4px;
  border-radius: 999px;
  border: 1px solid rgba(255, 255, 255, 0.07);
  background: rgba(255, 255, 255, 0.045);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  align-self: flex-start;
}

.ob-seg-btn {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  padding: 8px 15px;
  border-radius: 999px;
  border: none;
  background: transparent;
  color: rgba(235, 238, 250, 0.55);
  font-size: 12.5px;
  font-weight: 600;
  font-family: var(--font-sans);
  cursor: pointer;
  transition: all 0.22s ease;
}

.ob-seg-btn--active {
  background: rgba(255, 255, 255, 0.11);
  color: var(--foreground);
}

.ob-pane {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.ob-pane--file {
  align-items: flex-start;
  padding: 8px 0;
}

.ob-search {
  position: relative;
  display: flex;
  align-items: center;
}

.ob-search-icon {
  position: absolute;
  left: 12px;
  color: rgba(235, 238, 250, 0.4);
  pointer-events: none;
}

.ob-search-input {
  width: 100%;
  padding: 10px 12px 10px 34px;
  border-radius: 12px;
  border: 1px solid rgba(255, 255, 255, 0.09);
  background: rgba(0, 0, 0, 0.22);
  color: var(--foreground);
  font-size: 13px;
  font-family: var(--font-sans);
  outline: none;
  transition:
    border-color 0.2s ease,
    box-shadow 0.2s ease;
}

.ob-search-input:focus {
  border-color: rgba(167, 139, 250, 0.45);
  box-shadow: 0 0 0 3px rgba(167, 139, 250, 0.12);
}

.ob-search-input::placeholder {
  color: rgba(235, 238, 250, 0.35);
}

.ob-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 7px;
}

.ob-chip {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  padding: 6px 8px 6px 10px;
  border-radius: 999px;
  border: 1px solid rgba(167, 139, 250, 0.3);
  background: rgba(167, 139, 250, 0.1);
  font-size: 12px;
  font-weight: 500;
  animation: ob-pop 0.32s cubic-bezier(0.34, 1.56, 0.64, 1);
}

@keyframes ob-pop {
  from {
    opacity: 0;
    transform: scale(0.8);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

.ob-chip img {
  width: 15px;
  height: 15px;
  object-fit: contain;
}

.ob-chip-x {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 17px;
  height: 17px;
  border-radius: 50%;
  border: none;
  background: rgba(255, 255, 255, 0.1);
  color: rgba(235, 238, 250, 0.7);
  cursor: pointer;
  transition: background 0.18s ease;
}

.ob-chip-x:hover {
  background: rgba(255, 255, 255, 0.2);
}

.ob-setting {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 14px;
  padding: 14px 16px;
  border-radius: 14px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(255, 255, 255, 0.035);
}

.ob-setting-left {
  display: flex;
  align-items: center;
  gap: 11px;
  min-width: 0;
}
.ob-setting-icon {
  color: rgba(235, 238, 250, 0.55);
  flex-shrink: 0;
}
.ob-setting-title {
  font-size: 13px;
  font-weight: 500;
}

.ob-toggle {
  position: relative;
  width: 44px;
  height: 24px;
  border-radius: 999px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  background: rgba(255, 255, 255, 0.08);
  box-shadow: inset 0 1px 3px rgba(0, 0, 0, 0.3);
  cursor: pointer;
  flex-shrink: 0;
  transition:
    background 0.25s ease,
    border-color 0.25s ease;
}

.ob-toggle--on {
  background: linear-gradient(180deg, rgba(167, 139, 250, 0.85), rgba(124, 92, 255, 0.75));
  border-color: rgba(167, 139, 250, 0.6);
}

.ob-toggle-thumb {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: #fff;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.4);
  transition: transform 0.3s cubic-bezier(0.34, 1.4, 0.64, 1);
}

.ob-toggle-thumb--on {
  transform: translateX(20px);
}

.ob-cols {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
}

.ob-col {
  padding: 13px 14px;
  border-radius: 14px;
  border: 1px solid rgba(255, 255, 255, 0.07);
  background: rgba(255, 255, 255, 0.03);
}

.ob-col--good {
  border-color: color-mix(in oklch, var(--success) 25%, transparent);
}
.ob-col--bad {
  border-color: color-mix(in oklch, var(--destructive) 22%, transparent);
}

.ob-col-title {
  display: flex;
  align-items: center;
  gap: 7px;
  margin: 0 0 8px;
  font-size: 12px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.ob-col--good .ob-col-title {
  color: var(--success);
}
.ob-col--bad .ob-col-title {
  color: var(--destructive);
}

.ob-col ul {
  margin: 0;
  padding-left: 16px;
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.ob-col li {
  font-size: 12px;
  line-height: 1.45;
  color: rgba(235, 238, 250, 0.65);
}

.dc-stage {
  display: grid;
  grid-template-columns: 240px 1fr;
  gap: 14px;
  align-items: start;
}

.dc-profile {
  border-radius: 14px;
  overflow: hidden;
  background: #232428;
  border: 1px solid rgba(255, 255, 255, 0.06);
  font-family: var(--font-sans);
}

.dc-banner {
  height: 56px;
  background: linear-gradient(115deg, #5b4b9e, #7c5cff 55%, #3b2f6e);
}

.dc-avatar {
  position: relative;
  width: 64px;
  height: 64px;
  margin: -32px 0 0 14px;
  border-radius: 50%;
  border: 4px solid #232428;
}

.dc-avatar img {
  width: 100%;
  height: 100%;
  border-radius: 50%;
  object-fit: cover;
}

.dc-dot {
  position: absolute;
  right: -1px;
  bottom: -1px;
  width: 17px;
  height: 17px;
  border-radius: 50%;
  background: #23a55a;
  border: 4px solid #232428;
}

.dc-body {
  padding: 10px 14px 14px;
}
.dc-name {
  margin: 0;
  font-size: 15px;
  font-weight: 700;
  color: #f2f3f5;
}
.dc-handle {
  margin: 1px 0 0;
  font-size: 11.5px;
  color: #b5bac1;
}
.dc-about {
  margin: 8px 0 0;
  font-size: 12px;
  color: #dbdee1;
}

.dc-activity {
  margin-top: 10px;
  padding: 9px 10px;
  border-radius: 9px;
  background: #111214;
}

.dc-activity-label {
  margin: 0 0 7px;
  font-size: 9.5px;
  font-weight: 700;
  letter-spacing: 0.03em;
  color: #b5bac1;
}

.dc-activity-row {
  display: flex;
  gap: 9px;
  align-items: center;
}

.dc-activity-icon {
  width: 58px;
  height: 58px;
  border-radius: 10px;
  background: #232428;
  overflow: hidden;
  flex-shrink: 0;
}

.dc-activity-icon img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.dc-activity-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}
.dc-activity-name {
  font-size: 13px;
  font-weight: 700;
  color: #f2f3f5;
}
.dc-activity-state {
  font-size: 11px;
  color: #b5bac1;
}
.dc-activity-time {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 10.5px;
  color: #23a55a;
}

.dc-message-btn {
  margin-top: 11px;
  padding: 7px 0;
  border-radius: 7px;
  background: #5865f2;
  color: #fff;
  font-size: 12.5px;
  font-weight: 600;
  text-align: center;
}

.dc-side {
  display: flex;
  flex-direction: column;
  gap: 10px;
  min-width: 0;
  justify-content: center;
}

.dc-caption {
  margin: 0;
  font-size: 11.5px;
  font-weight: 600;
  color: rgba(235, 238, 250, 0.5);
}

.dc-member {
  display: flex;
  align-items: center;
  gap: 9px;
  padding: 8px 11px;
  border-radius: 10px;
  background: #2b2d31;
  border: 1px solid rgba(255, 255, 255, 0.05);
}

.dc-member-avatar {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  object-fit: cover;
}

.dc-member-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.dc-member-name {
  font-size: 12.5px;
  font-weight: 600;
  color: #f2f3f5;
}

.dc-member-game {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 10.5px;
  color: #b5bac1;
}

.dc-member-game svg {
  color: #23a55a;
}

.ob-coach {
  display: flex;
  flex-direction: column;
  gap: 10px;
  align-items: stretch;
}

.ob-bubble {
  display: flex;
  align-items: center;
  gap: 9px;
  align-self: flex-start;
  padding: 10px 14px;
  border-radius: 14px;
  border: 1px solid rgba(167, 139, 250, 0.35);
  background: rgba(167, 139, 250, 0.1);
  font-size: 12.5px;
  color: rgba(235, 238, 250, 0.85);
}

.ob-bubble--below {
  align-self: flex-end;
}

.ob-bubble-arrow {
  color: #a78bfa;
  animation: ob-nudge 1.6s ease-in-out infinite;
  flex-shrink: 0;
}

@keyframes ob-nudge {
  0%,
  100% {
    transform: translateY(0);
  }
  50% {
    transform: translateY(4px);
  }
}

.ob-mock {
  display: flex;
  gap: 8px;
  align-items: center;
  padding: 10px 12px;
  border-radius: 14px;
  border: 1px dashed rgba(167, 139, 250, 0.5);
  background: rgba(0, 0, 0, 0.25);
  animation: ob-glowpulse 2.4s ease-in-out infinite;
}

@keyframes ob-glowpulse {
  0%,
  100% {
    box-shadow: 0 0 0 rgba(167, 139, 250, 0);
  }
  50% {
    box-shadow: 0 0 22px rgba(167, 139, 250, 0.22);
  }
}

.ob-mock-input {
  flex: 1;
  padding: 9px 12px;
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.08);
  font-size: 11.5px;
  color: rgba(235, 238, 250, 0.5);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.ob-mock-btn {
  padding: 9px 16px;
  border-radius: 10px;
  background: var(--primary);
  color: var(--primary-foreground);
  font-size: 12px;
  font-weight: 700;
  flex-shrink: 0;
}

.ob-footer {
  position: relative;
  z-index: 2;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 24px 18px;
  flex-shrink: 0;
  min-height: 40px;
}

.ob-back {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  padding: 8px 14px;
  border-radius: 999px;
  border: 1px solid rgba(255, 255, 255, 0.09);
  background: rgba(255, 255, 255, 0.04);
  color: rgba(235, 238, 250, 0.65);
  font-size: 12px;
  font-weight: 600;
  font-family: var(--font-sans);
  cursor: pointer;
  transition: all 0.22s ease;
}

.ob-back:hover {
  background: rgba(255, 255, 255, 0.08);
  color: var(--foreground);
}

.ob-dots {
  display: flex;
  gap: 7px;
  margin-left: auto;
}

.ob-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.14);
  transition: all 0.35s cubic-bezier(0.34, 1.4, 0.64, 1);
}

.ob-dot--on {
  background: #a78bfa;
  box-shadow: 0 0 10px rgba(167, 139, 250, 0.6);
  transform: scale(1.15);
}

.pane-enter-active {
  transition: opacity 0.16s ease;
}

.pane-leave-active {
  transition: opacity 0.12s ease;
}

.pane-enter-from {
  opacity: 0;
}

.pane-leave-to {
  opacity: 0;
}

@media (max-width: 640px) {
  .dc-stage {
    grid-template-columns: 1fr;
  }
}
</style>
