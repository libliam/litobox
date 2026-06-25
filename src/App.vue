<template>
  <div class="app-layout">
    <SidebarNav v-model="activeTool" />
    
    <div class="app-content">
      <main class="app-main">
        <HomeView v-if="activeTool === 'home'" :on-select-tool="handleSelectTool" />
        <JsonTool v-else-if="activeTool === 'json'" />
        <StringTool v-else-if="activeTool === 'string'" />
        <EncodeTool v-else-if="activeTool === 'encode'" />
        <TimeTool v-else-if="activeTool === 'time'" />
        <URLTool v-else-if="activeTool === 'url'" />
        <RegexTool v-else-if="activeTool === 'regex'" />
        <BaseConverter v-else-if="activeTool === 'baseConverter'" />
        <UUIDTool v-else-if="activeTool === 'uuid'" />
        <DevTools v-else-if="activeTool === 'devtools'" />
        <FileProcessing v-else-if="activeTool === 'fileprocessing'" />
        <SqlTool v-else-if="activeTool === 'sql'" />
        <JSTool v-else-if="activeTool === 'js'" />
        <MockDataTool v-else-if="activeTool === 'mockData'" />
        <OcrTool v-else-if="activeTool === 'ocr'" />
        <DiffTool v-else-if="activeTool === 'diff'" />
        <ClipboardTool v-else-if="activeTool === 'clipboard'" />
        <ImageTool v-else-if="activeTool === 'image'" />
        <CsvTool v-else-if="activeTool === 'csv'" />
        <PdfTool v-else-if="activeTool === 'pdf'" />
        <HashTool v-else-if="activeTool === 'hash'" />
        <XmlYamlTool v-else-if="activeTool === 'xmlYaml'" />
        <DedupTool v-else-if="activeTool === 'dedup'" />
        <CssTool v-else-if="activeTool === 'css'" />
        <JwtTool v-else-if="activeTool === 'jwt'" />
        <WordCountTool v-else-if="activeTool === 'wordCount'" />
        <CronTool v-else-if="activeTool === 'cron'" />
        <MarkdownTool v-else-if="activeTool === 'markdown'" />
        <ColorTool v-else-if="activeTool === 'color'" />
        <PasswordTool v-else-if="activeTool === 'password'" />
        <QrTool v-else-if="activeTool === 'qr'" />
        <SnippetTool v-else-if="activeTool === 'snippet'" />
        <HttpTool v-else-if="activeTool === 'http'" />
        <HistoryView v-else-if="activeTool === 'history'" />
      </main>
      
      <div class="app-footer">
        <span>© 2026 栗的百宝箱 · Made by liam</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { useToolboxStore } from '@/store'
import SidebarNav from '@/components/SidebarNav.vue'
import HomeView from '@/views/HomeView.vue'
import JsonTool from '@/views/JsonTool.vue'
import StringTool from '@/views/StringTool.vue'
import EncodeTool from '@/views/EncodeTool.vue'
import TimeTool from '@/views/TimeTool.vue'
import URLTool from '@/views/URLTool.vue'
import RegexTool from '@/views/RegexTool.vue'
import BaseConverter from '@/views/BaseConverter.vue'
import UUIDTool from '@/views/UUIDTool.vue'
import DevTools from '@/views/DevTools.vue'
import FileProcessing from '@/views/FileProcessing.vue'
import SqlTool from '@/views/SqlTool.vue'
import JSTool from '@/views/JSTool.vue'
import MockDataTool from '@/views/MockDataTool.vue'
import OcrTool from '@/views/OcrTool.vue'
import DiffTool from '@/views/DiffTool.vue'
import ClipboardTool from '@/views/ClipboardTool.vue'
import ImageTool from '@/views/ImageTool.vue'
import CsvTool from '@/views/CsvTool.vue'
import HashTool from '@/views/HashTool.vue'
import XmlYamlTool from '@/views/XmlYamlTool.vue'
import DedupTool from '@/views/DedupTool.vue'
import CssTool from '@/views/CssTool.vue'
import JwtTool from '@/views/JwtTool.vue'
import WordCountTool from '@/views/WordCountTool.vue'
import CronTool from '@/views/CronTool.vue'
import MarkdownTool from '@/views/MarkdownTool.vue'
import ColorTool from '@/views/ColorTool.vue'
import PasswordTool from '@/views/PasswordTool.vue'
import QrTool from '@/views/QrTool.vue'
import SnippetTool from '@/views/SnippetTool.vue'
import HttpTool from '@/views/HttpTool.vue'
import PdfTool from '@/views/PdfTool.vue'
import HistoryView from '@/views/HistoryView.vue'

const store = useToolboxStore()
const activeTool = ref(store.config.lastTool)

let unlistenShortcut: (() => void) | null = null

const handleSelectTool = (toolId: string) => {
  activeTool.value = toolId
}

watch(activeTool, (newTool) => {
  store.saveConfig({ lastTool: newTool })
})

const applyTheme = (theme: string) => {
  const html = document.documentElement
  html.classList.remove('dark', 'light')
  
  if (theme === 'dark') {
    html.classList.add('dark')
  } else if (theme === 'light') {
    html.classList.add('light')
  } else {
    const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches
    if (prefersDark) {
      html.classList.add('dark')
    } else {
      html.classList.add('light')
    }
  }
}

onMounted(async () => {
  applyTheme(store.config.theme)
  
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
    if (store.config.theme === 'auto') {
      applyTheme('auto')
    }
  })
  
  unlistenShortcut = await listen('global-shortcut-triggered', (event) => {
    const toolId = event.payload as string
    if (toolId && toolId !== activeTool.value) {
      activeTool.value = toolId
      store.addRecentTool(toolId)
    }
  })
})

onUnmounted(() => {
  if (unlistenShortcut) {
    unlistenShortcut()
  }
})
</script>

<style scoped>
.app-layout {
  display: flex;
  height: 100vh;
  background: var(--bg-primary);
}

.app-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  height: 100vh;
}

.app-main {
  flex: 1;
  overflow: hidden;
}

.app-footer {
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-secondary);
  border-top: 1px solid var(--border-color);
  font-size: 11px;
  color: var(--text-muted);
  letter-spacing: 0.5px;
  flex-shrink: 0;
}
</style>
