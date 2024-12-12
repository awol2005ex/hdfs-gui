// src/router/index.ts
import { createRouter, createWebHistory, RouteRecordRaw } from 'vue-router';
import Home from '../views/Home.vue';
import HdfsFolderView from '../views/HdfsFolderView.vue';
const routes: Array<RouteRecordRaw> = [
{ path: '/', name: 'Home', component: Home },
{ path: '/HdfsFolderView/:id', name: 'HdfsFolderView', component: HdfsFolderView },
];

const router = createRouter({
history: createWebHistory(),
routes,
});

export default router;