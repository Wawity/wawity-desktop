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
    {
      path: '/extra/dns-bench',
      name: 'dns-bench',
      component: () => import('../views/extra/DnsBench.vue'),
      meta: { standalone: true },
    },
    {
      path: '/extra/node-pulse',
      name: 'node-pulse',
      component: () => import('../views/extra/NodePulse.vue'),
      meta: { standalone: true },
    },
    {
      path: '/extra/port-audit',
      name: 'port-audit',
      component: () => import('../views/extra/PortAudit.vue'),
      meta: { standalone: true },
    },
    {
      path: '/extra/firewall',
      name: 'firewall',
      component: () => import('../views/extra/Firewall.vue'),
      meta: { standalone: true },
    },
    {
      path: '/extra/snippets',
      name: 'snippets',
      component: () => import('../views/extra/Snippets.vue'),
      meta: { standalone: true },
    },
  ],
});

export default router;
