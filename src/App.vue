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
    <QuickLaunchOverlay v-model:visible="store.isQuickLaunchOpen" />
    <ConfirmDialogWrapper />
  </div>
</template>

<script setup lang="ts">
import { watch, onMounted, onUnmounted, computed, defineAsyncComponent } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useToolboxStore } from '@/store'
import { storeToRefs } from 'pinia'
import SidebarNav from '@/components/SidebarNav.vue'
import TabBar from '@/components/TabBar.vue'
import HomeView from '@/views/HomeView.vue'
// 首页同步加载（默认 Tab），其余工具页按需加载：
// defineAsyncComponent + 动态 import 让 vite 为每个页面拆出独立 chunk，
// 首屏只加载首页 + 公共依赖，其他工具页代码在使用时才请求，显著缩短启动白屏。
const JsonTool = defineAsyncComponent(() => import('@/views/JsonTool.vue'))
const StringTool = defineAsyncComponent(() => import('@/views/StringTool.vue'))
const PinyinTool = defineAsyncComponent(() => import('@/views/PinyinTool.vue'))
const EncodeTool = defineAsyncComponent(() => import('@/views/EncodeTool.vue'))
const TimeTool = defineAsyncComponent(() => import('@/views/TimeTool.vue'))
const URLTool = defineAsyncComponent(() => import('@/views/URLTool.vue'))
const RegexTool = defineAsyncComponent(() => import('@/views/RegexTool.vue'))
const BaseConverter = defineAsyncComponent(() => import('@/views/BaseConverter.vue'))
const IpSubnetTool = defineAsyncComponent(() => import('@/views/IpSubnetTool.vue'))
const UUIDTool = defineAsyncComponent(() => import('@/views/UUIDTool.vue'))
const DevTools = defineAsyncComponent(() => import('@/views/DevTools.vue'))
const FileProcessing = defineAsyncComponent(() => import('@/views/FileProcessing.vue'))
const SqlTool = defineAsyncComponent(() => import('@/views/SqlTool.vue'))
const JSTool = defineAsyncComponent(() => import('@/views/JSTool.vue'))
const MockDataTool = defineAsyncComponent(() => import('@/views/MockDataTool.vue'))
const OcrTool = defineAsyncComponent(() => import('@/views/OcrTool.vue'))
const DiffTool = defineAsyncComponent(() => import('@/views/DiffTool.vue'))
const JsonDiffTool = defineAsyncComponent(() => import('@/views/JsonDiffTool.vue'))
const ClipboardTool = defineAsyncComponent(() => import('@/views/ClipboardTool.vue'))
const ClipboardConvertTool = defineAsyncComponent(() => import('@/views/ClipboardConvertTool.vue'))
const CsvTool = defineAsyncComponent(() => import('@/views/CsvTool.vue'))
const ExcelTool = defineAsyncComponent(() => import('@/views/ExcelTool.vue'))
const SchemaTool = defineAsyncComponent(() => import('@/views/SchemaTool.vue'))
const OpenApiTool = defineAsyncComponent(() => import('@/views/OpenApiTool.vue'))
const TemplateTool = defineAsyncComponent(() => import('@/views/TemplateTool.vue'))
const GitStatsTool = defineAsyncComponent(() => import('@/views/GitStatsTool.vue'))
const StaticServerTool = defineAsyncComponent(() => import('@/views/StaticServerTool.vue'))
const ImageCompareTool = defineAsyncComponent(() => import('@/views/ImageCompareTool.vue'))
const ExifTool = defineAsyncComponent(() => import('@/views/ExifTool.vue'))
const CodeFormatterTool = defineAsyncComponent(() => import('@/views/CodeFormatterTool.vue'))
const PdfTool = defineAsyncComponent(() => import('@/views/PdfTool.vue'))
const HashTool = defineAsyncComponent(() => import('@/views/HashTool.vue'))
const XmlYamlTool = defineAsyncComponent(() => import('@/views/XmlYamlTool.vue'))
const DedupTool = defineAsyncComponent(() => import('@/views/DedupTool.vue'))
const NameCaseTool = defineAsyncComponent(() => import('@/views/NameCaseTool.vue'))
const TcConvertTool = defineAsyncComponent(() => import('@/views/TcConvertTool.vue'))
const CssTool = defineAsyncComponent(() => import('@/views/CssTool.vue'))
const JwtTool = defineAsyncComponent(() => import('@/views/JwtTool.vue'))
const WordCountTool = defineAsyncComponent(() => import('@/views/WordCountTool.vue'))
const CronTool = defineAsyncComponent(() => import('@/views/CronTool.vue'))
const MarkdownTool = defineAsyncComponent(() => import('@/views/MarkdownTool.vue'))
const ColorTool = defineAsyncComponent(() => import('@/views/ColorTool.vue'))
const PasswordTool = defineAsyncComponent(() => import('@/views/PasswordTool.vue'))
const QrTool = defineAsyncComponent(() => import('@/views/QrTool.vue'))
const BarcodeTool = defineAsyncComponent(() => import('@/views/BarcodeTool.vue'))
const SnippetTool = defineAsyncComponent(() => import('@/views/SnippetTool.vue'))
const HttpTool = defineAsyncComponent(() => import('@/views/HttpTool.vue'))
const CurlTool = defineAsyncComponent(() => import('@/views/CurlTool.vue'))
const HistoryView = defineAsyncComponent(() => import('@/views/HistoryView.vue'))
const WorkflowView = defineAsyncComponent(() => import('@/views/WorkflowView.vue'))
const NoteEditor = defineAsyncComponent(() => import('@/views/NoteEditor.vue'))
const CalculatorTool = defineAsyncComponent(() => import('@/views/CalculatorTool.vue'))
const SystemInfoView = defineAsyncComponent(() => import('@/views/SystemInfoView.vue'))
const NetworkInfoView = defineAsyncComponent(() => import('@/views/NetworkInfoView.vue'))
const ProcessListView = defineAsyncComponent(() => import('@/views/ProcessListView.vue'))
const HardwareInfoView = defineAsyncComponent(() => import('@/views/HardwareInfoView.vue'))
const SoftwareEnvView = defineAsyncComponent(() => import('@/views/SoftwareEnvView.vue'))
const SqliteViewerView = defineAsyncComponent(() => import('@/views/SqliteViewerView.vue'))
const DiskSpaceAnalyzer = defineAsyncComponent(() => import('@/views/DiskSpaceAnalyzer.vue'))
const FileSearcher = defineAsyncComponent(() => import('@/views/FileSearcher.vue'))
const IconGenerator = defineAsyncComponent(() => import('@/views/IconGenerator.vue'))
const ImageToolEnhanced = defineAsyncComponent(() => import('@/views/ImageToolEnhanced.vue'))
const AudioTool = defineAsyncComponent(() => import('@/views/AudioTool.vue'))
const VideoTool = defineAsyncComponent(() => import('@/views/VideoTool.vue'))
const MediaInfoTool = defineAsyncComponent(() => import('@/views/MediaInfoTool.vue'))
const ZipTool = defineAsyncComponent(() => import('@/views/ZipTool.vue'))
const MermaidTool = defineAsyncComponent(() => import('@/views/MermaidTool.vue'))
const PomodoroTool = defineAsyncComponent(() => import('@/views/PomodoroTool.vue'))
const ServiceListView = defineAsyncComponent(() => import('@/views/ServiceListView.vue'))
const HotkeyView = defineAsyncComponent(() => import('@/views/HotkeyView.vue'))
const HostsView = defineAsyncComponent(() => import('@/views/HostsView.vue'))
const NetworkConnections = defineAsyncComponent(() => import('@/views/NetworkConnections.vue'))
const ScheduledTasksView = defineAsyncComponent(() => import('@/views/ScheduledTasksView.vue'))
const StartupItemsView = defineAsyncComponent(() => import('@/views/StartupItemsView.vue'))
const EnvVarsView = defineAsyncComponent(() => import('@/views/EnvVarsView.vue'))
const CertViewer = defineAsyncComponent(() => import('@/views/CertViewer.vue'))
const BoostView = defineAsyncComponent(() => import('@/views/BoostView.vue'))
const PasswordVault = defineAsyncComponent(() => import('@/views/PasswordVault.vue'))
const FileRenamer = defineAsyncComponent(() => import('@/views/FileRenamer.vue'))
const QuickLaunchTool = defineAsyncComponent(() => import('@/views/QuickLaunchTool.vue'))
const SvgTool = defineAsyncComponent(() => import('@/views/SvgTool.vue'))
const BatchReplaceTool = defineAsyncComponent(() => import('@/views/BatchReplaceTool.vue'))
const ChangelogView = defineAsyncComponent(() => import('@/views/ChangelogView.vue'))
import CommandPalette from '@/components/CommandPalette.vue'
import QuickLaunchOverlay from '@/components/QuickLaunchOverlay.vue'
import { ConfirmDialogWrapper } from '@/composables/useConfirmDialog'

// toolId → 组件 映射表（替代 v-if 链）
const toolComponentMap: Record<string, any> = {
  home: HomeView,
  json: JsonTool,
  string: StringTool,
  pinyin: PinyinTool,
  encode: EncodeTool,
  time: TimeTool,
  url: URLTool,
  regex: RegexTool,
  baseConverter: BaseConverter,
  ipSubnet: IpSubnetTool,
  uuid: UUIDTool,
  devtools: DevTools,
  fileprocessing: FileProcessing,
  zipTool: ZipTool,
  mermaid: MermaidTool,
  pomodoro: PomodoroTool,
  sql: SqlTool,
  batchReplace: BatchReplaceTool,
  js: JSTool,
  mockData: MockDataTool,
  ocr: OcrTool,
  diff: DiffTool,
  jsonDiff: JsonDiffTool,
  clipboard: ClipboardTool,
  clipboardConvert: ClipboardConvertTool,
  image: ImageToolEnhanced,
  csv: CsvTool,
  excelTool: ExcelTool,
  schemaTool: SchemaTool,
  openApi: OpenApiTool,
  templateTool: TemplateTool,
  gitStats: GitStatsTool,
  staticServer: StaticServerTool,
  codeFormatter: CodeFormatterTool,
  imageCompare: ImageCompareTool,
  exif: ExifTool,
  pdf: PdfTool,
  hash: HashTool,
  xmlYaml: XmlYamlTool,
  dedup: DedupTool,
  nameCase: NameCaseTool,
  tcConvert: TcConvertTool,
  css: CssTool,
  jwt: JwtTool,
  wordCount: WordCountTool,
  cron: CronTool,
  markdown: MarkdownTool,
  color: ColorTool,
  password: PasswordTool,
  qr: QrTool,
  barcode: BarcodeTool,
  snippet: SnippetTool,
  http: HttpTool,
  curl: CurlTool,
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
  certViewer: CertViewer,
  boost: BoostView,
  passwordVault: PasswordVault,
  fileRenamer: FileRenamer,
  quickLaunch: QuickLaunchTool,
  svg: SvgTool,
  changelog: ChangelogView,
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
let windowResizeHandler: (() => void) | null = null
let windowMoveHandler: (() => void) | null = null
let windowStateTimer: number | null = null

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

// 保存窗口大小和位置
// 由 Rust 侧（window_state.rs）读取 innerSize/innerPosition 并持久化到 JSON 文件，
// 启动时在 Rust setup 阶段立即恢复（窗口首次显示前），避免前端加载完再 setSize 的尺寸跳变。
const saveWindowState = async () => {
  try {
    await invoke('save_window_state')
  } catch {}
}

onMounted(async () => {
  applyTheme(store.config.theme)

  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
    if (store.config.theme === 'auto') {
      applyTheme('auto')
    }
  })

  // 窗口尺寸/位置已由 Rust setup 阶段恢复，此处只监听变化并保存

  // 监听窗口大小和位置变化（300ms 防抖，避免拖动时高频保存）
  const win = getCurrentWindow()
  const debouncedSave = () => {
    if (windowStateTimer) window.clearTimeout(windowStateTimer)
    windowStateTimer = window.setTimeout(() => saveWindowState(), 300)
  }
  windowResizeHandler = await win.onResized(debouncedSave)
  windowMoveHandler = await win.onMoved(debouncedSave)

  unlistenShortcut = await listen('global-shortcut-triggered', (event) => {
    const toolId = event.payload as string
    if (toolId === '__quick_launch__') {
      if (store.isQuickLaunchOpen) {
        store.closeQuickLaunch()
      } else {
        store.openQuickLaunch()
      }
    } else if (toolId) {
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
  // 窗口状态保存由 onResized/onMoved 防抖触发；退出确认时 Rust 侧兜底保存，无需 beforeunload
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
  if (windowResizeHandler) {
    windowResizeHandler()
  }
  if (windowMoveHandler) {
    windowMoveHandler()
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
  min-width: 0; /* ponytail: flex 子项默认 min-width:auto 会阻止被压缩，导致内部百分比布局失效 */
}

.app-main {
  flex: 1;
  overflow: hidden;
  min-height: 0; /* ponytail: flex 子项默认 min-height:auto，阻止子元素被压缩 */
  min-width: 0;
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
