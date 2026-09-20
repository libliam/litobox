<template>
  <div class="tool-container mindmap-editor">
    <!-- 顶部工具栏 -->
    <div class="tool-card sticky-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">脑图画布</span>
          <el-tooltip placement="top" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>自由创建/编辑思维导图</p>
                <p>· 双击节点编辑文字</p>
                <p>· Tab 新增子节点，Enter 新增兄弟节点</p>
                <p>· <b>拖拽节点到另一个节点上</b>可改变父子关系</p>
                <p>· <b>左键拖空白处</b>框选多个节点</p>
                <p>· <b>右键拖空白处</b>平移画布，滚轮缩放</p>
                <p>· 方向键平移，按 0 适配画布</p>
                <p>· 右键菜单操作选中节点</p>
                <p>· 支持 4 种布局、8 种主题</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
        <div class="action-bar">
          <el-button size="small" @click="createNew">
            <el-icon><Plus /></el-icon> 新建
          </el-button>
          <el-button size="small" @click="saveCurrent" :loading="saving">
            <el-icon><Check /></el-icon> 保存
          </el-button>
          <el-button size="small" @click="renameCurrent">
            <el-icon><Edit /></el-icon> 重命名
          </el-button>
          <el-button size="small" @click="deleteCurrent" type="danger">
            <el-icon><Delete /></el-icon> 删除
          </el-button>
          <el-divider direction="vertical" />
          <el-dropdown @command="handleImport">
            <el-button size="small">
              <el-icon><Upload /></el-icon> 导入
              <el-icon class="el-icon--right"><ArrowDown /></el-icon>
            </el-button>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item command="blank">空白脑图</el-dropdown-item>
                <el-dropdown-item command="json">JSON 文件</el-dropdown-item>
                <el-dropdown-item command="markdown">Markdown 文本</el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
          <el-dropdown @command="handleExport">
            <el-button size="small">
              <el-icon><Download /></el-icon> 导出
              <el-icon class="el-icon--right"><ArrowDown /></el-icon>
            </el-button>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item command="png">PNG 图片</el-dropdown-item>
                <el-dropdown-item command="svg">SVG 矢量</el-dropdown-item>
                <el-dropdown-item command="json">JSON 数据</el-dropdown-item>
                <el-dropdown-item command="markdown">Markdown</el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
          <el-divider direction="vertical" />
          <el-button size="small" @click="centerView" title="居中显示">
            <el-icon><Aim /></el-icon> 居中
          </el-button>
          <el-button size="small" @click="fitView" title="适配画布">
            <el-icon><FullScreen /></el-icon> 适配
          </el-button>
          <el-divider direction="vertical" />
          <el-select v-model="direction" size="small" style="width: 110px" @change="changeDirection">
            <el-option label="两侧展开" :value="2" />
            <el-option label="向右展开" :value="1" />
            <el-option label="向左展开" :value="0" />
            <el-option label="向下展开" :value="3" />
          </el-select>
          <el-select v-model="theme" size="small" style="width: 130px" @change="changeTheme">
            <el-option v-for="t in themeOptions" :key="t.value" :label="t.label" :value="t.value" />
          </el-select>
        </div>
      </div>
    </div>

    <!-- 主体：左侧列表 + 右侧画布 -->
    <div class="tool-card editor-main">
      <div class="card-body editor-body">
        <!-- 左侧列表 -->
        <div class="list-pane">
          <div class="pane-title">脑图列表 ({{ mindmapList.length }})</div>
          <div class="mindmap-list">
            <div
              v-for="item in mindmapList"
              :key="item.id"
              class="mindmap-item"
              :class="{ active: currentId === item.id }"
              @click="loadMindmap(item.id)"
            >
              <div class="item-title">{{ item.title }}</div>
              <div class="item-meta">{{ item.updated_at.slice(0, 16).replace('T', ' ') }}</div>
            </div>
            <div v-if="mindmapList.length === 0" class="empty-list">
              暂无脑图，点「新建」开始
            </div>
          </div>
        </div>

        <!-- 右侧画布 -->
        <div class="canvas-pane">
          <div class="canvas-header">
            <span class="pane-title">{{ currentTitle || '未命名脑图' }}</span>
            <span class="save-status" :class="{ saved: !dirty }">
              {{ dirty ? '未保存' : '已保存' }}
            </span>
          </div>
          <div ref="mapContainer" class="map-container"></div>
        </div>
      </div>
    </div>

    <!-- 空白处右键菜单 -->
    <div
      v-show="blankMenuVisible"
      class="blank-context-menu"
      :style="{ left: blankMenuX + 'px', top: blankMenuY + 'px' }"
      @click.stop
    >
      <div class="menu-item" @click="centerView">
        <el-icon><Aim /></el-icon> 居中显示
      </div>
      <div class="menu-item" @click="fitView">
        <el-icon><FullScreen /></el-icon> 适配画布
      </div>
      <div class="menu-item" @click="selectAllNodes">
        <el-icon><Select /></el-icon> 全选节点
      </div>
    </div>

    <!-- 导入 Markdown 弹窗 -->
    <el-dialog v-model="importMdVisible" title="导入 Markdown" width="600px">
      <el-input
        v-model="importMdText"
        type="textarea"
        :rows="12"
        placeholder="# 根节点&#10;## 子节点 A&#10;- 内容 1&#10;## 子节点 B&#10;### 更深层级"
      />
      <template #footer>
        <el-button @click="importMdVisible = false">取消</el-button>
        <el-button type="primary" @click="doImportMarkdown">导入</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage, ElMessageBox } from 'element-plus'
import { QuestionFilled, Plus, Check, Edit, Delete, Upload, Download, ArrowDown, Aim, FullScreen, Select } from '@element-plus/icons-vue'
import MindElixir from 'mind-elixir'
import 'mind-elixir/style.css'
import { zh_CN } from 'mind-elixir/i18n'
import domtoimage from 'dom-to-image-more'
import { saveFileWithDialog } from '@/utils/fileSaver'

// ============ 状态 ============
const mindmapList = ref<any[]>([])
const currentId = ref<string>('')
const currentTitle = ref('')
const direction = ref<number>(2) // SIDE
const theme = ref('light')
const dirty = ref(false)
const saving = ref(false)

const mapContainer = ref<HTMLElement | null>(null)
let mei: MindElixir | null = null

const importMdVisible = ref(false)
const importMdText = ref('')

// 空白处右键菜单
const blankMenuVisible = ref(false)
const blankMenuX = ref(0)
const blankMenuY = ref(0)
let blankCtxHandler: ((e: MouseEvent) => void) | null = null

let saveTimer: ReturnType<typeof setTimeout> | null = null

// ============ 主题预设 ============
// ponytail: 基于 MindElixir 默认主题结构，只改 palette + 关键色，复用默认 cssVar 避免样式错乱
const buildTheme = (name: string, type: 'light' | 'dark', palette: string[], accent: string, rootBg: string, mainBg: string, bg: string) => {
  const base = type === 'dark' ? MindElixir.DARK_THEME : MindElixir.THEME
  return {
    name,
    type,
    palette,
    cssVar: {
      ...base.cssVar,
      '--bgcolor': bg,
      '--root-bgcolor': rootBg,
      '--main-bgcolor': mainBg,
      '--accent-color': accent,
      '--selected': accent
    }
  }
}

const themes: Record<string, any> = {
  latte: buildTheme('Latte', 'light',
    ['#dd7878', '#ea76cb', '#8839ef', '#e64553', '#fe640b', '#df8e1d', '#40a02b', '#209fb5', '#1e66f5', '#7287fd'],
    '#e64553', '#4c4f69', '#ffffff', '#f6f6f6'),
  dark: buildTheme('Dark', 'dark',
    ['#848FA0', '#748BE9', '#D2F9FE', '#4145A5', '#789AFA', '#706CF4', '#EF987F', '#775DD5', '#FCEECF', '#DA7FBC'],
    '#789AFA', '#2d3748', '#4c4f69', '#252526'),
  cyan: buildTheme('青色科技', 'light',
    ['#00d4ff', '#06b6d4', '#0ea5e9', '#3b82f6', '#6366f1', '#8b5cf6', '#0891b2', '#0284c7', '#0369a1', '#075985'],
    '#00d4ff', '#0e7490', '#ffffff', '#f0fdfa'),
  purple: buildTheme('紫色梦幻', 'light',
    ['#a78bfa', '#8b5cf6', '#c084fc', '#d946ef', '#ec4899', '#f472b6', '#7c3aed', '#6d28d9', '#5b21b6', '#4c1d95'],
    '#8b5cf6', '#6d28d9', '#ffffff', '#faf5ff'),
  green: buildTheme('绿色自然', 'light',
    ['#34d399', '#10b981', '#22c55e', '#84cc16', '#14b8a6', '#06b6d4', '#059669', '#047857', '#065f46', '#064e3b'],
    '#10b981', '#047857', '#ffffff', '#f0fdf4'),
  orange: buildTheme('橙色暖阳', 'light',
    ['#fbbf24', '#f97316', '#f59e0b', '#eab308', '#f43f5e', '#ef4444', '#ea580c', '#c2410c', '#9a3412', '#7c2d12'],
    '#f97316', '#c2410c', '#ffffff', '#fffbeb'),
  rose: buildTheme('红粉浪漫', 'light',
    ['#fb7185', '#f472b6', '#e879f9', '#a78bfa', '#c084fc', '#d946ef', '#e11d48', '#be123c', '#9f1239', '#881337'],
    '#fb7185', '#be123c', '#ffffff', '#fff1f2'),
  ocean: buildTheme('深海蓝', 'dark',
    ['#38bdf8', '#0ea5e9', '#0284c7', '#0369a1', '#075985', '#0c4a6e', '#22d3ee', '#06b6d4', '#0891b2', '#0e7490'],
    '#38bdf8', '#0c4a6e', '#1e3a5f', '#0f172a'),
}

const themeOptions = [
  { value: 'latte', label: '浅色·Latte' },
  { value: 'dark', label: '深色·Dark' },
  { value: 'cyan', label: '青色科技' },
  { value: 'purple', label: '紫色梦幻' },
  { value: 'green', label: '绿色自然' },
  { value: 'orange', label: '橙色暖阳' },
  { value: 'rose', label: '红粉浪漫' },
  { value: 'ocean', label: '深海蓝' },
]

// ============ 列表加载 ============
const loadList = async () => {
  try {
    mindmapList.value = await invoke('db_mindmap_list')
  } catch (e: any) {
    console.error('load list error:', e)
  }
}

// ============ 新建 ============
const createNew = async () => {
  const id = crypto.randomUUID()
  const data = MindElixir.new('新脑图')
  const dataJson = JSON.stringify(data)
  await invoke('db_mindmap_save', {
    id,
    title: '新脑图',
    data_json: dataJson,
    direction: 'side',
    theme: 'light'
  })
  await loadList()
  loadMindmap(id)
}

// ============ 加载 ============
const loadMindmap = async (id: string) => {
  if (dirty.value && currentId.value) {
    await saveCurrent()
  }
  try {
    const record: any = await invoke('db_mindmap_get', { id })
    if (!record) return
    currentId.value = id
    currentTitle.value = record.title
    direction.value = dirToNum(record.direction)
    theme.value = record.theme

    if (!mei) {
      await initMindmap()
    }
    const data = JSON.parse(record.data_json)
    mei!.refresh(data)
    // refresh 会重置视图，优先恢复保存的位置/缩放，没有则居中
    await nextTick()
    if (!restoreViewState(data.viewState)) {
      mei!.toCenter()
    }
    dirty.value = false
  } catch (e: any) {
    console.error('load error:', e)
  }
}

// ============ 初始化 MindElixir ============
const initMindmap = async () => {
  if (!mapContainer.value) return
  mei = new MindElixir({
    el: mapContainer.value,
    direction: direction.value as any,
    editable: true,
    contextMenu: {
      focus: true,
      link: true,
      locale: zh_CN
    },
    toolBar: false,
    keypress: true,
    allowUndo: true,
    // ponytail: mouseSelectionButton=0 → 左键框选节点，右键平移画布
    mouseSelectionButton: 0,
    theme: themes[theme.value] || MindElixir.THEME
  })
  await mei.init(MindElixir.new('新脑图'))

  // 监听操作，标记 dirty
  mei.bus.addListener('operation', () => {
    dirty.value = true
    scheduleAutoSave()
  })

  // 监听视图变化（平移/缩放），保存视图状态
  mei.bus.addListener('scale', () => {
    dirty.value = true
    scheduleAutoSave()
  })
  mei.bus.addListener('move', () => {
    dirty.value = true
    scheduleAutoSave()
  })

  // 空白处右键菜单：监听 showContextMenu，非节点时显示自定义菜单
  blankCtxHandler = (e: MouseEvent) => {
    const target = e.target as HTMLElement
    const isNode = !!target.closest('.me-tpc, .me-parent, .me-root')
    if (!isNode) {
      blankMenuX.value = e.clientX
      blankMenuY.value = e.clientY
      blankMenuVisible.value = true
    }
  }
  mei.bus.addListener('showContextMenu', blankCtxHandler)
}

// ============ 自动保存（防抖 1.5s） ============
const scheduleAutoSave = () => {
  if (saveTimer) clearTimeout(saveTimer)
  saveTimer = setTimeout(() => {
    saveCurrent()
  }, 1500)
}

// ============ 保存 ============
const saveCurrent = async () => {
  if (!mei || !currentId.value) return
  saving.value = true
  try {
    const data = mei.getData() as any
    // ponytail: mind-elixir 不存视图状态，手动把平移+缩放塞进 data_json
    data.viewState = getViewState()
    const dataJson = JSON.stringify(data)
    await invoke('db_mindmap_save', {
      id: currentId.value,
      title: currentTitle.value,
      data_json: dataJson,
      direction: numToDir(direction.value),
      theme: theme.value
    })
    dirty.value = false
    await loadList()
  } catch (e: any) {
    ElMessage.error('保存失败: ' + e.message)
  } finally {
    saving.value = false
  }
}

// ============ 重命名 ============
const renameCurrent = async () => {
  try {
    const { value } = await ElMessageBox.prompt('输入新名称', '重命名', {
      inputValue: currentTitle.value,
      confirmButtonText: '确定',
      cancelButtonText: '取消'
    })
    if (value) {
      currentTitle.value = value
      await saveCurrent()
    }
  } catch { /* cancel */ }
}

// ============ 删除 ============
const deleteCurrent = async () => {
  if (!currentId.value) return
  try {
    await ElMessageBox.confirm(`确定删除「${currentTitle.value}」？`, '删除确认', {
      type: 'warning',
      confirmButtonText: '删除',
      cancelButtonText: '取消'
    })
    await invoke('db_mindmap_delete', { id: currentId.value })
    currentId.value = ''
    currentTitle.value = ''
    if (mei) mei.refresh(MindElixir.new('新脑图'))
    await loadList()
    ElMessage.success('已删除')
  } catch { /* cancel */ }
}

// ============ 布局切换 ============
const changeDirection = (_val: number) => {
  if (!mei) return
  // 先保存当前数据，destroy 后重建再加载
  const data = mei.getData()
  if (blankCtxHandler) mei.bus?.removeListener('showContextMenu', blankCtxHandler)
  mei.destroy()
  mei = null
  initMindmap().then(() => {
    if (mei) mei.refresh(data)
  })
  dirty.value = true
  scheduleAutoSave()
}

// ============ 主题切换 ============
const changeTheme = (val: string) => {
  if (!mei) return
  mei.changeTheme(themes[val] || MindElixir.THEME)
  dirty.value = true
  scheduleAutoSave()
}

// ============ 视图控制 ============
const centerView = () => {
  if (mei) mei.toCenter()
  blankMenuVisible.value = false
}

const fitView = () => {
  if (mei) mei.scaleFit()
  blankMenuVisible.value = false
}

// 读取当前视图状态（平移 + 缩放）
const getViewState = () => {
  if (!mei || !mei.map) return null
  const transform = mei.map.style.transform || ''
  const m = transform.match(/translate3d\(([^,]+),\s*([^,]+),\s*[^)]+\)\s*scale\(([^)]+)\)/)
  if (!m) return null
  return { x: parseFloat(m[1]), y: parseFloat(m[2]), scale: parseFloat(m[3]) }
}

// 恢复视图状态
const restoreViewState = (state: any) => {
  if (!mei || !mei.map || !state) return false
  const { x, y, scale } = state
  if (typeof x !== 'number' || typeof y !== 'number' || typeof scale !== 'number') return false
  mei.map.style.transform = `translate3d(${x}px, ${y}px, 0) scale(${scale})`
  ;(mei as any).scaleVal = scale
  return true
}

// 全选所有节点
const selectAllNodes = () => {
  if (!mei || !mapContainer.value) return
  const nodes = Array.from(mapContainer.value.querySelectorAll('.me-tpc'))
  if (nodes.length > 0) {
    mei.selectNodes(nodes as any)
  }
  blankMenuVisible.value = false
}

// 键盘方向键平移画布
const onKeydown = (e: KeyboardEvent) => {
  if (!mei) return
  // 输入框中不处理
  const tag = (e.target as HTMLElement)?.tagName
  if (tag === 'INPUT' || tag === 'TEXTAREA' || (e.target as HTMLElement)?.isContentEditable) return

  const step = 80
  switch (e.key) {
    case 'ArrowUp':    mei.move(0, step, true); break
    case 'ArrowDown':  mei.move(0, -step, true); break
    case 'ArrowLeft':  mei.move(step, 0, true); break
    case 'ArrowRight': mei.move(-step, 0, true); break
    case '0':          mei.scaleFit(); break
  }
}

// ============ 导入 ============
const handleImport = (cmd: string) => {
  if (cmd === 'blank') {
    if (mei) {
      mei.refresh(MindElixir.new('新脑图'))
      dirty.value = true
      scheduleAutoSave()
    }
  } else if (cmd === 'json') {
    importJsonFile()
  } else if (cmd === 'markdown') {
    importMdText.value = ''
    importMdVisible.value = true
  }
}

const importJsonFile = async () => {
  const input = document.createElement('input')
  input.type = 'file'
  input.accept = '.json'
  input.onchange = async () => {
    const file = input.files?.[0]
    if (!file) return
    const text = await file.text()
    try {
      const data = JSON.parse(text)
      if (mei) {
        mei.refresh(data)
        dirty.value = true
        scheduleAutoSave()
        ElMessage.success('导入成功')
      }
    } catch {
      ElMessage.error('JSON 格式错误')
    }
  }
  input.click()
}

const doImportMarkdown = () => {
  const data = markdownToMindElixir(importMdText.value)
  if (mei) {
    mei.refresh(data)
    dirty.value = true
    scheduleAutoSave()
    importMdVisible.value = false
    ElMessage.success('导入成功')
  }
}

// ============ 导出 ============
const handleExport = async (cmd: string) => {
  if (!mei) return
  if (cmd === 'png') {
    await exportPng()
  } else if (cmd === 'svg') {
    await exportSvg()
  } else if (cmd === 'json') {
    exportJson()
  } else if (cmd === 'markdown') {
    exportMarkdown()
  }
}

const exportPng = async () => {
  if (!mapContainer.value || !mei) return
  try {
    mei.toCenter()
    await new Promise(r => setTimeout(r, 200))
    const isDark = themes[theme.value]?.type === 'dark'
    const bgColor = isDark ? themes[theme.value].cssVar['--bgcolor'] || '#1a1a2e' : '#ffffff'
    const dataUrl = await domtoimage.toPng(mapContainer.value, {
      bgcolor: bgColor,
      scale: 2,
      quality: 1
    })
    const base64 = dataUrl.split(',')[1]
    const binary = atob(base64)
    const bytes = new Uint8Array(binary.length)
    for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i)
    const blob = new Blob([bytes], { type: 'image/png' })
    await saveFileWithDialog(blob, `${currentTitle.value || 'mindmap'}.png`, 'png')
  } catch (e: any) {
    ElMessage.error('导出失败: ' + e.message)
  }
}

const exportSvg = async () => {
  if (!mapContainer.value || !mei) return
  try {
    mei.toCenter()
    await new Promise(r => setTimeout(r, 200))
    const isDark = themes[theme.value]?.type === 'dark'
    const bgColor = isDark ? themes[theme.value].cssVar['--bgcolor'] || '#1a1a2e' : '#ffffff'
    // ponytail: 节点是 HTML div，连线才是 svg，必须用 toSvg 把整个画布 DOM 序列化进 foreignObject
    const dataUrl = await domtoimage.toSvg(mapContainer.value, {
      bgcolor: bgColor,
      scale: 2,
      quality: 1
    })
    // ponytail: 只反转 escapeXhtml 的三种编码（%25/%23/%0A），不能用 decodeURIComponent
    // 否则 CSS url() 里的 %3C 等会被解成 < 破坏 XML
    let svgContent = dataUrl.slice(dataUrl.indexOf(',') + 1)
    svgContent = svgContent.replace(/%25/g, '%').replace(/%23/g, '#').replace(/%0A/g, '\n')
    const blob = new Blob([svgContent], { type: 'image/svg+xml;charset=utf-8' })
    await saveFileWithDialog(blob, `${currentTitle.value || 'mindmap'}.svg`, 'svg')
  } catch (e: any) {
    ElMessage.error('导出失败: ' + e.message)
  }
}

const exportJson = () => {
  if (!mei) return
  const data = JSON.stringify(mei.getData(), null, 2)
  const blob = new Blob([data], { type: 'application/json' })
  saveFileWithDialog(blob, `${currentTitle.value || 'mindmap'}.json`, 'json')
}

const exportMarkdown = () => {
  if (!mei) return
  const data = mei.getData()
  const md = nodeToMarkdown(data.nodeData, 1)
  const blob = new Blob([md], { type: 'text/markdown' })
  saveFileWithDialog(blob, `${currentTitle.value || 'mindmap'}.md`, 'md')
}

// ============ Markdown ↔ MindElixir 转换 ============
// ponytail: 简单转换器，# 标题 → 层级，- 列表 → 子节点
const markdownToMindElixir = (md: string): any => {
  const lines = md.split('\n').map(l => l.trimEnd()).filter(l => l.trim())
  const root: any = { topic: '根节点', id: genId(), children: [] }
  // 栈：每一层级的当前节点
  const stack: any[] = [root]

  for (const line of lines) {
    const headerMatch = line.match(/^(#{1,6})\s+(.+)/)
    const listMatch = line.match(/^[-*+]\s+(.+)/)

    if (headerMatch) {
      const level = headerMatch[1].length
      const topic = headerMatch[2]
      // 调整栈到对应层级
      while (stack.length > level) stack.pop()
      const parent = stack[stack.length - 1] || root
      const node = { topic, id: genId(), children: [] }
      if (!parent.children) parent.children = []
      parent.children.push(node)
      stack[level] = node
      stack.length = level + 1
    } else if (listMatch) {
      const topic = listMatch[1]
      const parent = stack[stack.length - 1] || root
      const node = { topic, id: genId(), children: [] }
      if (!parent.children) parent.children = []
      parent.children.push(node)
    }
  }

  // 如果根节点没有子节点，把第一个标题当根
  if (root.children.length === 1 && lines[0]?.startsWith('#')) {
    return { nodeData: root.children[0] }
  }
  return { nodeData: root }
}

const nodeToMarkdown = (node: any, level: number): string => {
  if (level <= 6) {
    let md = '#'.repeat(level) + ' ' + node.topic + '\n'
    if (node.children) {
      for (const child of node.children) {
        md += nodeToMarkdown(child, level + 1)
      }
    }
    return md
  } else {
    // 超过 6 级用列表
    let md = '- ' + node.topic + '\n'
    if (node.children) {
      for (const child of node.children) {
        md += '  ' + nodeToMarkdown(child, level + 1)
      }
    }
    return md
  }
}

const genId = () => Math.random().toString(36).slice(2, 10)

// ============ 方向转换 ============
const dirToNum = (d: string): number => {
  const map: Record<string, number> = { left: 0, right: 1, side: 2, down: 3 }
  return map[d] ?? 2
}
const numToDir = (n: number): string => {
  const map: Record<number, string> = { 0: 'left', 1: 'right', 2: 'side', 3: 'down' }
  return map[n] ?? 'side'
}

// ============ 生命周期 ============
const onDocClick = () => {
  blankMenuVisible.value = false
}

onMounted(async () => {
  await loadList()
  await nextTick()
  await initMindmap()
  // 默认加载第一个
  if (mindmapList.value.length > 0) {
    loadMindmap(mindmapList.value[0].id)
  } else {
    createNew()
  }
  window.addEventListener('keydown', onKeydown)
  document.addEventListener('click', onDocClick)
})

onUnmounted(() => {
  if (saveTimer) clearTimeout(saveTimer)
  window.removeEventListener('keydown', onKeydown)
  document.removeEventListener('click', onDocClick)
  if (mei) {
    if (blankCtxHandler) mei.bus?.removeListener('showContextMenu', blankCtxHandler)
    mei.destroy()
    mei = null
  }
})
</script>

<style scoped>
.mindmap-editor {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.mindmap-editor .editor-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.mindmap-editor .editor-body {
  flex: 1;
  display: grid;
  grid-template-columns: 240px 1fr;
  min-height: 0;
  padding: 0;
  gap: 0;
}

.mindmap-editor .list-pane {
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--border-color);
  min-height: 0;
}

.mindmap-editor .pane-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--accent-cyan);
  text-transform: uppercase;
  letter-spacing: 1px;
  padding: 12px 16px 6px;
}

.mindmap-editor .mindmap-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.mindmap-editor .mindmap-item {
  padding: 10px 12px;
  border-radius: 6px;
  cursor: pointer;
  margin-bottom: 4px;
  transition: background 0.2s;
  border: 1px solid transparent;
}

.mindmap-editor .mindmap-item:hover {
  background: var(--bg-hover, rgba(0, 212, 255, 0.08));
}

.mindmap-editor .mindmap-item.active {
  background: var(--bg-hover, rgba(0, 212, 255, 0.15));
  border-color: var(--accent-cyan);
}

.mindmap-editor .item-title {
  font-size: 13px;
  color: var(--text-primary);
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.mindmap-editor .item-meta {
  font-size: 11px;
  color: var(--text-secondary);
  margin-top: 2px;
}

.mindmap-editor .empty-list {
  text-align: center;
  color: var(--text-secondary);
  font-size: 13px;
  padding: 40px 16px;
}

.mindmap-editor .canvas-pane {
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-height: 0;
}

.mindmap-editor .canvas-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 1px solid var(--border-color);
}

.mindmap-editor .save-status {
  font-size: 12px;
  color: var(--warning, #f59e0b);
  padding-right: 16px;
}

.mindmap-editor .save-status.saved {
  color: var(--success, #34d399);
}

.mindmap-editor .map-container {
  flex: 1;
  overflow: hidden;
  background: var(--mm-bg, #f5f5f5);
  min-height: 0;
}

/* 空白处右键菜单 */
.mindmap-editor .blank-context-menu {
  position: fixed;
  z-index: 9999;
  min-width: 140px;
  background: var(--bg-card, #1e1e2e);
  border: 1px solid var(--border-color, #333);
  border-radius: 8px;
  padding: 6px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
  user-select: none;
}

.mindmap-editor .blank-context-menu .menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  font-size: 13px;
  color: var(--text-primary, #e0e0e0);
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.15s;
}

.mindmap-editor .blank-context-menu .menu-item:hover {
  background: var(--bg-hover, rgba(0, 212, 255, 0.15));
  color: var(--accent-cyan, #00d4ff);
}

.mindmap-editor .blank-context-menu .menu-item .el-icon {
  font-size: 16px;
}
</style>
