<template>
  <div class="app-layout">
    <SidebarNav v-model="activeTool" />

    <div class="app-content">
      <TabBar />
      <main class="app-main">
        <KeepAlive :max="8">
          <component
            :is="toolComponentMap[activeTabId]"
            :key="store.getTabKey(activeTabId)"
            v-bind="activeTabId === 'home' ? { onSelectTool: handleSelectTool } : {}"
          />
        </KeepAlive>
      </main>

      <div class="app-footer">
        <span>© 2026 栗的百宝箱 · Made by liam</span>
      </div>
    </div>
    <CommandPalette />
    <ConfirmDialogWrapper />
  </div>
</template>

<script setup lang="ts">
import { watch, onMounted, onUnmounted, computed } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { useToolboxStore } from '@/store'
import { storeToRefs } from 'pinia'
import SidebarNav from '@/components/SidebarNav.vue'
import TabBar from '@/components/TabBar.vue'
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
import SystemInfoView from '@/views/SystemInfoView.vue'
import NetworkInfoView from '@/views/NetworkInfoView.vue'
import ProcessListView from '@/views/ProcessListView.vue'
import HardwareInfoView from '@/views/HardwareInfoView.vue'
import SoftwareEnvView from '@/views/SoftwareEnvView.vue'
import SqliteViewerView from '@/views/SqliteViewerView.vue'
import DiskSpaceAnalyzer from '@/views/DiskSpaceAnalyzer.vue'
import FileSearcher from '@/views/FileSearcher.vue'
import IconGenerator from '@/views/IconGenerator.vue'
import ImageToolEnhanced from '@/views/ImageToolEnhanced.vue'
import AudioTool from '@/views/AudioTool.vue'
import VideoTool from '@/views/VideoTool.vue'
import MediaInfoTool from '@/views/MediaInfoTool.vue'
import ServiceListView from '@/views/ServiceListView.vue'
import HotkeyView from '@/views/HotkeyView.vue'
import HostsView from '@/views/HostsView.vue'
import NetworkConnections from '@/views/NetworkConnections.vue'
import ScheduledTasksView from '@/views/ScheduledTasksView.vue'
import StartupItemsView from '@/views/StartupItemsView.vue'
import EnvVarsView from '@/views/EnvVarsView.vue'
import BoostView from '@/views/BoostView.vue'
import CommandPalette from '@/components/CommandPalette.vue'
import { ConfirmDialogWrapper } from '@/composables/useConfirmDialog'

// toolId → 组件 映射表（替代 v-if 链）
const toolComponentMap: Record<string, any> = {
  home: HomeView,
  json: JsonTool,
  string: StringTool,
  encode: EncodeTool,
  time: TimeTool,
  url: URLTool,
  regex: RegexTool,
  baseConverter: BaseConverter,
  uuid: UUIDTool,
  devtools: DevTools,
  fileprocessing: FileProcessing,
  sql: SqlTool,
  js: JSTool,
  mockData: MockDataTool,
  ocr: OcrTool,
  diff: DiffTool,
  clipboard: ClipboardTool,
  image: ImageTool,
  csv: CsvTool,
  pdf: PdfTool,
  hash: HashTool,
  xmlYaml: XmlYamlTool,
  dedup: DedupTool,
  css: CssTool,
  jwt: JwtTool,
  wordCount: WordCountTool,
  cron: CronTool,
  markdown: MarkdownTool,
  color: ColorTool,
  password: PasswordTool,
  qr: QrTool,
  snippet: SnippetTool,
  http: HttpTool,
  history: HistoryView,
  workflow: WorkflowView,
  note: NoteEditor,
  calculator: CalculatorTool,
  systemInfo: SystemInfoView,
  networkInfo: NetworkInfoView,
  processList: ProcessListView,
  hardwareInfo: HardwareInfoView,
  softwareEnv: SoftwareEnvView,
  sqliteViewer: SqliteViewerView,
  diskAnalyzer: DiskSpaceAnalyzer,
  fileSearcher: FileSearcher,
  iconGenerator: IconGenerator,
  imageToolEnhanced: ImageToolEnhanced,
  audioTool: AudioTool,
  videoTool: VideoTool,
  mediaInfo: MediaInfoTool,
  serviceList: ServiceListView,
  hotkeyViewer: HotkeyView,
  hostsManager: HostsView,
  networkConnections: NetworkConnections,
  scheduledTasks: ScheduledTasksView,
  startupItems: StartupItemsView,
  envVars: EnvVarsView,
  boost: BoostView,
}

const store = useToolboxStore()
const { activeTabId } = storeToRefs(store)

// 兼容 SidebarNav 的 v-model="activeTool"：activeTool 仍是 computed 指向 activeTabId
const activeTool = computed({
  get: () => store.activeTabId,
  set: (val: string) => store.openTab(val),  // SidebarNav 设置时走 openTab
})

// 初始化：恢复上次使用的工具（单 tab，不持久化 tab 列表）
store.openTab(store.config.lastTool || 'home')

let unlistenShortcut: (() => void) | null = null
let unlistenPalette: (() => void) | null = null
let globalKeydownHandler: ((e: KeyboardEvent) => void) | null = null

const handleSelectTool = (toolId: string) => {
  store.openTab(toolId)
  store.addRecentTool(toolId)
}

// lastTool 跟随 activeTabId 变化
watch(activeTabId, (newTool: string) => {
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
    if (toolId) {
      store.openTab(toolId)
      store.addRecentTool(toolId)
    }
  })

  unlistenPalette = await listen('command-palette-triggered', () => {
    // 全局热键也是 toggle 行为，与应用内 Ctrl+P 保持一致
    if (store.isCommandPaletteOpen) {
      store.closeCommandPalette()
    } else {
      store.openCommandPalette()
    }
  })

  // 应用内 Ctrl+P toggle 命令面板（仅应用激活时生效）
  globalKeydownHandler = (e: KeyboardEvent) => {
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'p') {
      e.preventDefault()
      if (store.isCommandPaletteOpen) {
        store.closeCommandPalette()
      } else {
        store.openCommandPalette()
      }
    }
  }
  window.addEventListener('keydown', globalKeydownHandler)
})

onUnmounted(() => {
  if (unlistenShortcut) {
    unlistenShortcut()
  }
  if (unlistenPalette) {
    unlistenPalette()
  }
  if (globalKeydownHandler) {
    window.removeEventListener('keydown', globalKeydownHandler)
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
