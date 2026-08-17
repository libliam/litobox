import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import ElementPlus from 'element-plus'
import 'element-plus/dist/index.css'
import App from './App.vue'
import './style/main.css'
import './style/theme.css'

const app = createApp(App)
const pinia = createPinia()

app.use(pinia)
app.use(ElementPlus)
app.mount('#app')

// 启动期间主窗口保持隐藏（tauri.conf visible:false），界面就绪后再显示，消除启动白屏。
// 非 Tauri 环境（纯浏览器访问 dev url）invoke 失败，忽略即可。
invoke('app_ready').catch(() => {})