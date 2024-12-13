// src/router/index.ts
import { createRouter, createWebHistory, RouteRecordRaw } from 'vue-router';
//首页
import Home from '../views/Home.vue';
//文件列表
import HdfsFolderView from '../views/HdfsFolderView.vue';
//单个文件查看
import HdfsFileView from '../views/HdfsFileView.vue';
const routes: Array<RouteRecordRaw> = [
{ path: '/', name: 'Home', component: Home },
{ path: '/HdfsFolderView/:id', name: 'HdfsFolderView', component: HdfsFolderView },
{ path: '/HdfsFileView/:id', name: 'HdfsFileView', component: HdfsFileView },
];

const router = createRouter({
history: createWebHistory(),
routes,
});

export default router;