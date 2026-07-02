<template>
  <div class="app-layout">
    <SidebarNav v-model="activeTool" />
    
    <div class="app-content">
      <main class="app-main">
        <KeepAlive :max="4">
          <HomeView v-if="activeTool === 'home'" :key="'home'" :on-select-tool="handleSelectTool" />
          <JsonTool v-else-if="activeTool === 'json'" :key="'json'" />
          <StringTool v-else-if="activeTool === 'string'" :key="'string'" />
          <EncodeTool v-else-if="activeTool === 'encode'" :key="'encode'" />
          <TimeTool v-else-if="activeTool === 'time'" :key="'time'" />
          <URLTool v-else-if="activeTool === 'url'" :key="'url'" />
          <RegexTool v-else-if="activeTool === 'regex'" :key="'regex'" />
          <BaseConverter v-else-if="activeTool === 'baseConverter'" :key="'baseConverter'" />
          <UUIDTool v-else-if="activeTool === 'uuid'" :key="'uuid'" />
          <DevTools v-else-if="activeTool === 'devtools'" :key="'devtools'" />
          <FileProcessing v-else-if="activeTool === 'fileprocessing'" :key="'fileprocessing'" />
          <SqlTool v-else-if="activeTool === 'sql'" :key="'sql'" />
          <JSTool v-else-if="activeTool === 'js'" :key="'js'" />
          <MockDataTool v-else-if="activeTool === 'mockData'" :key="'mockData'" />
          <OcrTool v-else-if="activeTool === 'ocr'" :key="'ocr'" />
          <DiffTool v-else-if="activeTool === 'diff'" :key="'diff'" />
          <ClipboardTool v-else-if="activeTool === 'clipboard'" :key="'clipboard'" />
          <ImageTool v-else-if="activeTool === 'image'" :key="'image'" />
          <CsvTool v-else-if="activeTool === 'csv'" :key="'csv'" />
          <PdfTool v-else-if="activeTool === 'pdf'" :key="'pdf'" />
          <HashTool v-else-if="activeTool === 'hash'" :key="'hash'" />
          <XmlYamlTool v-else-if="activeTool === 'xmlYaml'" :key="'xmlYaml'" />
          <DedupTool v-else-if="activeTool === 'dedup'" :key="'dedup'" />
          <CssTool v-else-if="activeTool === 'css'" :key="'css'" />
          <JwtTool v-else-if="activeTool === 'jwt'" :key="'jwt'" />
          <WordCountTool v-else-if="activeTool === 'wordCount'" :key="'wordCount'" />
          <CronTool v-else-if="activeTool === 'cron'" :key="'cron'" />
          <MarkdownTool v-else-if="activeTool === 'markdown'" :key="'markdown'" />
          <ColorTool v-else-if="activeTool === 'color'" :key="'color'" />
          <PasswordTool v-else-if="activeTool === 'password'" :key="'password'" />
          <QrTool v-else-if="activeTool === 'qr'" :key="'qr'" />
          <SnippetTool v-else-if="activeTool === 'snippet'" :key="'snippet'" />
          <HttpTool v-else-if="activeTool === 'http'" :key="'http'" />
          <HistoryView v-else-if="activeTool === 'history'" :key="'history'" />
          <WorkflowView v-else-if="activeTool === 'workflow'" :key="'workflow'" />
          <NoteEditor v-else-if="activeTool === 'note'" :key="'note'" />
          <CalculatorTool v-else-if="activeTool === 'calculator'" :key="'calculator'" />
        </KeepAlive>
      </main>
      
      <div class="app-footer">
        <span>© 2026 栗的百宝箱 · Made by liam</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { watch, onMounted, onUnmounted } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { useToolboxStore } from '@/store'
import { storeToRefs } from 'pinia'
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
import PdfTool from '@/views/PdfTool.vue'
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
import HistoryView from '@/views/HistoryView.vue'
import WorkflowView from '@/views/WorkflowView.vue'
import NoteEditor from '@/views/NoteEditor.vue'
import CalculatorTool from '@/views/CalculatorTool.vue'

const store = useToolboxStore()
const { activeTool } = storeToRefs(store)

// 初始化 activeTool
activeTool.value = store.config.lastTool

let unlistenShortcut: (() => void) | null = null

const handleSelectTool = (toolId: string) => {
  activeTool.value = toolId
}

watch(activeTool, (newTool: string) => {
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
