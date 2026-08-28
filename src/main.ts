import { createApp } from 'vue';
import { createPinia } from 'pinia';
import './i18n';
import { initTelemetry, watchVueErrors } from './lib/telemetry';
import { preloadUiFlags } from './lib/uiFlags';
import './assets/global.css';

preloadUiFlags();

const params = new URLSearchParams(window.location.search);

if (params.has('notify')) {
  import('./NotifyPopup.vue').then(({ default: NotifyPopup }) => {
    createApp(NotifyPopup).mount('#app');
  });
} else if (params.has('tray')) {
  import('./TrayPopup.vue').then(({ default: TrayPopup }) => {
    const app = createApp(TrayPopup);
    app.use(createPinia());
    app.mount('#app');
  });
} else {
  Promise.all([import('./App.vue'), import('./router')]).then(([appModule, routerModule]) => {
    const app = createApp(appModule.default);
    app.use(createPinia());
    app.use(routerModule.default);
    watchVueErrors(app);
    initTelemetry();
    app.mount('#app');
  });
}
