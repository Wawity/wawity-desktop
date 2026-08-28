import { createRouter, createWebHistory } from 'vue-router';
import Connection from '../views/Connection.vue';
import Settings from '../views/Settings.vue';
import Servers from '../views/Servers.vue';
import Analysis from '../views/Analysis.vue';

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'connection',
      component: Connection,
    },
    {
      path: '/settings',
      name: 'settings',
      component: Settings,
    },
    {
      path: '/servers',
      name: 'servers',
      component: Servers,
    },
    {
      path: '/analysis',
      name: 'analysis',
      component: Analysis,
    },
    {
      path: '/roles',
      redirect: { path: '/settings', query: { tab: 'split' } },
    },
    {
      path: '/extra/reachability',
      name: 'reachability',
      component: () => import('../views/extra/Reachability.vue'),
      meta: { standalone: true },
    },
    {
      path: '/extra/speedtest',
      name: 'speedtest',
      component: () => import('../views/extra/SpeedTest.vue'),
      meta: { standalone: true },
    },
    {
      path: '/extra/leaks',
      name: 'leaks',
      component: () => import('../views/extra/Leaks.vue'),
      meta: { standalone: true },
    },
  ],
});

export default router;
