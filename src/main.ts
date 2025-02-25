import { createApp } from "vue";
import App from "./App.vue";
import router from './router';
// 引入 element-plus
import ElementPlus from 'element-plus'
import 'element-plus/dist/index.css'


import JsonViewer from "vue3-json-viewer";
// if you used v1.0.5 or latster ,you should add import "vue3-json-viewer/dist/index.css"
import "vue3-json-viewer/dist/index.css";


createApp(App).use(router).use(JsonViewer).use(ElementPlus).mount("#app");
