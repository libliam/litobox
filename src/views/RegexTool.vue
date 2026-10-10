<template>
  <div class="tool-container">
    <el-tabs v-model="activeTab" class="regex-tool-tabs" @tab-click="handleTabClick">

      <!-- Tab 1: 正则测试 -->
      <el-tab-pane label="测试" name="test">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <div class="header-left">
              <span class="card-title">正则表达式</span>
              <el-tooltip placement="top" effect="dark">
                <template #content>
                  <div class="tooltip-content">
                    <p>输入正则表达式，支持常用标志</p>
                    <p><code>g</code> 全局匹配 <code>i</code> 忽略大小写 <code>m</code> 多行</p>
                  </div>
                </template>
                <el-icon class="hint-icon"><QuestionFilled /></el-icon>
              </el-tooltip>
            </div>
            <div class="card-actions">
              <el-button size="small" type="primary" @click="handleTest">测试</el-button>
              <el-button size="small" @click="handleReplace">替换</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input
              v-model="pattern"
              placeholder="输入正则表达式，如: (\d{4})-(\d{2})-(\d{2})"
              size="default"
              style="margin-bottom: 12px"
            >
              <template #prepend>/</template>
              <template #append>/</template>
            </el-input>
            <div class="flags-row">
              <el-checkbox v-model="flagG" label="g" size="small">全局</el-checkbox>
              <el-checkbox v-model="flagI" label="i" size="small">忽略大小写</el-checkbox>
              <el-checkbox v-model="flagM" label="m" size="small">多行</el-checkbox>
            </div>
          </div>
        </div>

        <!-- 输入卡片 -->
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">测试文本</span>
            <div class="card-actions">
              <VariablePicker @select="handleInsertVariable" />
              <el-button size="small" @click="handleClear">清空</el-button>
              <el-button size="small" @click="handlePaste">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input
              v-model="testText"
              type="textarea"
              :rows="5"
              placeholder="输入测试文本..."
              resize="vertical"
            />
          </div>
        </div>

        <!-- 替换输入 -->
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">替换字符串</span>
          </div>
          <div class="card-body">
            <el-input
              v-model="replacePattern"
              placeholder="替换字符串，如: $1/$2/$3 或 $&"
              size="small"
            />
          </div>
        </div>

        <!-- 匹配结果 -->
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">匹配结果</span>
            <span v-if="matches.length > 0" class="match-count">{{ matches.length }} 个匹配</span>
          </div>
          <div class="card-body">
            <div v-if="errorMessage" class="error-message">{{ errorMessage }}</div>
            <div v-else-if="matches.length > 0">
              <!-- 分组捕获详情 -->
              <div v-if="hasGroups" class="groups-section">
                <div class="section-label">分组捕获</div>
                <div class="group-list">
                  <div v-for="(match, idx) in matches" :key="idx" class="group-item">
                    <span class="group-index">#{{ idx + 1 }}</span>
                    <div class="group-content">
                      <div class="group-full">{{ match.text }}</div>
                      <div v-if="match.captures && match.captures.length > 0" class="group-captures">
                        <span v-for="(c, ci) in match.captures" :key="ci" class="capture-tag">
                          <span class="capture-num">{{ ci + 1 }}</span>{{ c }}
                        </span>
                      </div>
                      <div v-else-if="match.groups && Object.keys(match.groups).length > 0" class="group-captures">
                        <span v-for="(g, key) in match.groups" :key="key" class="capture-tag">
                          <span class="capture-num">{{ key }}</span>{{ g }}
                        </span>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
              <!-- 简单匹配列表 -->
              <div v-else class="match-list">
                <div v-for="(match, index) in matches" :key="index" class="match-item">
                  <span class="match-index">#{{ index + 1 }}</span>
                  <code class="match-text">{{ match.text }}</code>
                  <span class="match-pos">位置: {{ match.index }}</span>
                </div>
              </div>
            </div>
            <div v-else-if="testResult !== null" class="no-match">未找到匹配</div>
            <div v-else class="stats-empty">点击"测试"按钮查看匹配结果</div>
          </div>
        </div>

        <!-- 替换结果 -->
        <div v-if="replacedText" class="tool-card">
          <div class="card-header">
            <span class="card-title">替换结果</span>
            <el-button size="small" @click="handleCopyReplaced">复制</el-button>
          </div>
          <div class="card-body">
            <el-input
              :model-value="replacedText"
              type="textarea"
              :rows="4"
              readonly
              resize="vertical"
            />
          </div>
        </div>
      </el-tab-pane>

      <!-- Tab 2: 常用模板 -->
      <el-tab-pane label="模板" name="templates">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <div class="header-left">
              <span class="card-title">常用正则模板</span>
              <el-tooltip placement="top" effect="dark">
                <template #content>
                  <div class="tooltip-content">
                    <p>点击模板自动填入正则表达式</p>
                    <p>可直接在测试Tab中使用</p>
                  </div>
                </template>
                <el-icon class="hint-icon"><QuestionFilled /></el-icon>
              </el-tooltip>
            </div>
          </div>
        </div>

        <div v-for="(group, gi) in templateGroups" :key="gi" class="tool-card template-group-card">
          <div class="card-header">
            <span class="card-title">{{ group.name }}</span>
          </div>
          <div class="card-body">
            <div class="template-grid">
              <div
                v-for="(tpl, ti) in group.items"
                :key="ti"
                class="template-item"
                @click="useTemplate(tpl)"
              >
                <div class="template-name">{{ tpl.name }}</div>
                <code class="template-pattern">{{ tpl.pattern }}</code>
                <div class="template-desc">{{ tpl.desc }}</div>
              </div>
            </div>
          </div>
        </div>
      </el-tab-pane>

      <!-- Tab 3: 正则可视化 -->
      <el-tab-pane label="可视化" name="visualize">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <div class="header-left">
              <span class="card-title">正则表达式可视化</span>
              <el-tooltip placement="top" effect="dark">
                <template #content>
                  <div class="tooltip-content">
                    <p>输入正则表达式，生成结构流程图</p>
                    <p>支持字符类、量词、分组、断言等</p>
                  </div>
                </template>
                <el-icon class="hint-icon"><QuestionFilled /></el-icon>
              </el-tooltip>
            </div>
            <div class="card-actions">
              <el-button size="small" type="primary" @click="handleVisualize">生成可视化</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input
              v-model="visPattern"
              placeholder="输入正则表达式，如: (\d{4})-(\d{2})-(\d{2})"
              size="default"
            >
              <template #prepend>/</template>
              <template #append>/</template>
            </el-input>
          </div>
        </div>

        <div v-if="visError" class="error-message" style="margin: 12px 0">{{ visError }}</div>

        <div v-if="visNodes.length" class="tool-card">
          <div class="card-header">
            <span class="card-title">结构图</span>
          </div>
          <div class="card-body">
            <div class="regex-flow">
              <template v-for="(node, idx) in visNodes" :key="idx">
                <div v-if="idx > 0 && node.type !== 'alt'" class="flow-arrow">→</div>
                <div v-if="node.type === 'alt'" class="flow-alt-sep">|</div>
                <div class="flow-node" :class="'node-' + node.type">
                  <div class="node-label">{{ node.label }}</div>
                  <div v-if="node.detail" class="node-detail">{{ node.detail }}</div>
                  <div v-if="node.quantifier" class="node-quantifier">{{ node.quantifier }}</div>
                </div>
              </template>
            </div>
          </div>
        </div>

        <div v-if="visTree" class="tool-card">
          <div class="card-header">
            <span class="card-title">语法树</span>
          </div>
          <div class="card-body">
            <div class="regex-tree">
              <div class="tree-node tree-root">
                <span class="tree-label">Expression</span>
                <div class="tree-children">
                  <template v-for="child in visTree" :key="child.label + child.detail">
                    <div class="tree-branch">
                      <span class="tree-label" :class="'tree-' + child.type">{{ child.label }}</span>
                      <span v-if="child.detail" class="tree-detail">{{ child.detail }}</span>
                      <span v-if="child.quantifier" class="tree-quantifier">{{ child.quantifier }}</span>
                      <div v-if="child.children && child.children.length" class="tree-children">
                        <div v-for="(sub, si) in child.children" :key="si" class="tree-leaf">
                          <span class="tree-label" :class="'tree-' + sub.type">{{ sub.label }}</span>
                          <span v-if="sub.detail" class="tree-detail">{{ sub.detail }}</span>
                          <span v-if="sub.quantifier" class="tree-quantifier">{{ sub.quantifier }}</span>
                        </div>
                      </div>
                    </div>
                  </template>
                </div>
              </div>
            </div>
          </div>
        </div>
      </el-tab-pane>

    </el-tabs>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import { testRegex, type RegexMatch } from '@/utils/regexUtils'
import { useToolboxStore } from '@/store'
import VariablePicker from '@/components/VariablePicker.vue'

const store = useToolboxStore()

// ============ Tab 状态 ============
const activeTab = ref('test')

// ============ 正则测试 ============
const pattern = ref('')
const testText = ref('')
const flagG = ref(true)
const flagI = ref(false)
const flagM = ref(false)
const replacePattern = ref('')
const matches = ref<RegexMatch[]>([])
const replacedText = ref('')
const errorMessage = ref('')
const testResult = ref<boolean | null>(null)

const hasGroups = computed(() => matches.value.some(m =>
  (m.groups && Object.keys(m.groups).length > 0) || (m.captures && m.captures.length > 0)
))

const getFlags = () => {
  let flags = ''
  if (flagG.value) flags += 'g'
  if (flagI.value) flags += 'i'
  if (flagM.value) flags += 'm'
  return flags || 'g'
}

const handleTest = () => {
  if (!pattern.value) {
    ElMessage.warning('请输入正则表达式')
    return
  }
  if (!testText.value) {
    ElMessage.warning('请输入测试文本')
    return
  }

  const result = testRegex(pattern.value, testText.value, getFlags())
  if (!result.success) {
    errorMessage.value = result.error || '测试失败'
    matches.value = []
    testResult.value = null
    return
  }

  errorMessage.value = ''
  matches.value = result.matches
  testResult.value = result.matches.length > 0
  replacedText.value = ''

  if (result.matches.length > 0) {
    ElMessage.success(`找到 ${result.matches.length} 个匹配`)
  } else {
    ElMessage.info('未找到匹配')
  }

  store.addHistory({
    tool: 'regex',
    action: 'test',
    inputPreview: pattern.value.slice(0, 50),
    outputPreview: `${result.matches.length} matches`,
    inputFull: `Pattern: ${pattern.value}\nFlags: ${getFlags()}\nText: ${testText.value}`,
    outputFull: result.matches.map(m => m.text).join('\n'),
  })
}

const handleReplace = () => {
  if (!pattern.value || !testText.value) {
    ElMessage.warning('请先输入正则表达式和测试文本')
    return
  }

  const result = testRegex(pattern.value, testText.value, getFlags(), replacePattern.value)
  if (!result.success) {
    errorMessage.value = result.error || '替换失败'
    return
  }

  replacedText.value = result.replacedText || ''
  ElMessage.success('替换完成')

  store.addHistory({
    tool: 'regex',
    action: 'replace',
    inputPreview: pattern.value.slice(0, 50),
    outputPreview: replacedText.value.slice(0, 50),
    inputFull: `Pattern: ${pattern.value}\nText: ${testText.value}\nReplace: ${replacePattern.value}`,
    outputFull: replacedText.value,
  })
}

const handleClear = () => {
  testText.value = ''
}

const handlePaste = async () => {
  try {
    const text = await navigator.clipboard.readText()
    testText.value = text
  } catch {
    ElMessage.warning('无法读取剪贴板')
  }
}

const handleInsertVariable = (value: string) => {
  testText.value = value
}

const handleCopyReplaced = async () => {
  if (!replacedText.value) {
    ElMessage.warning('没有可复制的内容')
    return
  }
  try {
    await navigator.clipboard.writeText(replacedText.value)
    ElMessage.success('已复制到剪贴板')
  } catch {
    ElMessage.error('复制失败')
  }
}

const handleTabClick = () => {}

// ============ 模板库 ============
const templateGroups = [
  {
    name: '验证类',
    items: [
      { name: '手机号', pattern: '^1[3-9]\\d{9}$', desc: '中国大陆手机号' },
      { name: '身份证', pattern: '^\\d{15}|\\d{17}[\\dXx]$', desc: '15位或18位身份证号' },
      { name: '邮箱', pattern: '^[\\w.-]+@[\\w.-]+\\.\\w{2,}$', desc: '电子邮箱地址' },
      { name: 'URL', pattern: '^https?://[\\w.-]+(:\\d+)?(/[\\w./?-]*)*$', desc: 'HTTP/HTTPS 链接' },
      { name: 'IP地址', pattern: '^(\\d{1,3}\\.){3}\\d{1,3}$', desc: 'IPv4 地址格式' },
      { name: '端口号', pattern: '^([1-9]\\d{0,3}|[1-5]\\d{4}|6[0-4]\\d{3}|65[0-4]\\d{2}|655[0-2]\\d|6553[0-5])$', desc: '1-65535 端口号' },
    ]
  },
  {
    name: '格式类',
    items: [
      { name: '日期(YYYY-MM-DD)', pattern: '^(\\d{4})-(\\d{2})-(\\d{2})$', desc: '年-月-日格式' },
      { name: '时间(HH:mm:ss)', pattern: '^([01]\\d|2[0-3]):([0-5]\\d):([0-5]\\d)$', desc: '24小时制时间' },
      { name: '日期时间', pattern: '^(\\d{4})-(\\d{2})-(\\d{2}) ([01]\\d|2[0-3]):([0-5]\\d):([0-5]\\d)$', desc: '完整日期时间' },
      { name: 'UUID', pattern: '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$', desc: '标准 UUID 格式' },
      { name: 'MD5', pattern: '^[0-9a-fA-F]{32}$', desc: '32位 MD5 哈希' },
      { name: 'SHA256', pattern: '^[0-9a-fA-F]{64}$', desc: '64位 SHA256 哈希' },
    ]
  },
  {
    name: '提取类',
    items: [
      { name: '提取数字', pattern: '\\d+', desc: '匹配所有数字' },
      { name: '提取中文', pattern: '[\\u4e00-\\u9fa5]+', desc: '匹配中文字符' },
      { name: '提取英文', pattern: '[a-zA-Z]+', desc: '匹配英文单词' },
      { name: '提取URL', pattern: 'https?://[\\w.-]+(:\\d+)?(/[\\w./?-]*)*', desc: '匹配文本中的链接' },
      { name: '提取邮箱', pattern: '[\\w.-]+@[\\w.-]+\\.\\w{2,}', desc: '匹配文本中的邮箱' },
      { name: '提取HTML标签', pattern: '<([a-z][a-z0-9]*)\\b[^>]*>(.*?)</\\1>', desc: '匹配成对HTML标签' },
    ]
  },
  {
    name: '密码/安全',
    items: [
      { name: '6-16位数字字母', pattern: '^[a-zA-Z0-9]{6,16}$', desc: '数字+字母组合密码' },
      { name: '含特殊字符', pattern: '^(?=.*[a-zA-Z])(?=.*\\d)(?=.*[^a-zA-Z0-9]).{8,}$', desc: '至少8位，含字母+数字+特殊字符' },
      { name: '不含连续字符', pattern: '^(?!.*(.)\\1{2})[a-zA-Z0-9]{6,}$', desc: '无连续3个相同字符' },
    ]
  }
]

const useTemplate = (tpl: { name: string; pattern: string; desc: string }) => {
  pattern.value = tpl.pattern
  activeTab.value = 'test'
  ElMessage.success(`已填入「${tpl.name}」正则表达式`)
}

// ============ 正则可视化 ============
interface VisNode {
  type: string
  label: string
  detail?: string
  quantifier?: string
  children?: VisNode[]
}

const visPattern = ref('')
const visError = ref('')
const visNodes = ref<VisNode[]>([])
const visTree = ref<VisNode[]>([])

const ESCAPE_MAP: Record<string, string> = {
  'd': '数字 [0-9]',
  'D': '非数字',
  'w': '单词字符 [a-zA-Z0-9_]',
  'W': '非单词字符',
  's': '空白字符',
  'S': '非空白字符',
  'b': '单词边界',
  'B': '非单词边界',
  'n': '换行符',
  't': '制表符',
  'r': '回车符',
  '.': '点号',
}

function parseQuantifier(str: string, pos: number): { q: string; next: number } {
  if (pos >= str.length) return { q: '', next: pos }
  const ch = str[pos]
  if (ch === '*') return { q: '0次或多次 (*)', next: pos + 1 }
  if (ch === '+') return { q: '1次或多次 (+)', next: pos + 1 }
  if (ch === '?') return { q: '0或1次 (?)', next: pos + 1 }
  if (ch === '{') {
    const end = str.indexOf('}', pos)
    if (end !== -1) {
      const inner = str.substring(pos + 1, end)
      return { q: `重复 ${inner} 次`, next: end + 1 }
    }
  }
  return { q: '', next: pos }
}

function parseCharClass(str: string, pos: number): { node: VisNode; next: number } {
  let i = pos + 1 // skip '['
  let negated = false
  if (str[i] === '^') { negated = true; i++ }
  let content = ''
  while (i < str.length && str[i] !== ']') {
    content += str[i]
    i++
  }
  const label = negated ? '排除字符类' : '字符类'
  const detail = `[${negated ? '^' : ''}${content}]`
  return { node: { type: 'charclass', label, detail }, next: i + 1 }
}

function parseGroup(str: string, pos: number): { node: VisNode; next: number } {
  let i = pos + 1 // skip '('
  let type = 'group'
  let label = '捕获分组'
  let detail = '(...)'

  if (str[i] === '?') {
    i++
    if (str[i] === ':') { type = 'noncap'; label = '非捕获分组'; detail = '(?:...)'; i++ }
    else if (str[i] === '=') { type = 'lookahead'; label = '正向先行断言'; detail = '(?=...)'; i++ }
    else if (str[i] === '!') { type = 'lookahead'; label = '负向先行断言'; detail = '(?!...)'; i++ }
    else if (str[i] === '<' && str[i + 1] === '=') { type = 'lookbehind'; label = '正向后行断言'; detail = '(?<=...)'; i += 2 }
    else if (str[i] === '<' && str[i + 1] === '!') { type = 'lookbehind'; label = '负向后行断言'; detail = '(?<!...)'; i += 2 }
    else if (str[i] === '<') {
      // named group (?<name>...)
      const end = str.indexOf('>', i)
      const name = str.substring(i + 1, end)
      type = 'namedgroup'; label = `命名分组 "${name}"`; detail = `(?<${name}>...)`
      i = end + 1
    }
  }

  // parse children until matching ')'
  const children: VisNode[] = []
  let depth = 1
  while (i < str.length && depth > 0) {
    if (str[i] === '(') depth++
    if (str[i] === ')') { depth--; if (depth === 0) break }
    if (str[i] === '|') {
      children.push({ type: 'alt', label: '或', detail: '|' })
      i++
      continue
    }
    const { node, next } = parseToken(str, i)
    children.push(node)
    i = next
  }

  return { node: { type, label, detail, children }, next: i + 1 }
}

function parseToken(str: string, pos: number): { node: VisNode; next: number } {
  if (pos >= str.length) return { node: { type: 'end', label: '' }, next: pos }
  const ch = str[pos]

  // character class
  if (ch === '[') {
    const { node, next } = parseCharClass(str, pos)
    const { q, next: n2 } = parseQuantifier(str, next)
    if (q) node.quantifier = q
    return { node, next: n2 }
  }

  // group
  if (ch === '(') {
    const { node, next } = parseGroup(str, pos)
    const { q, next: n2 } = parseQuantifier(str, next)
    if (q) node.quantifier = q
    return { node, next: n2 }
  }

  // escape
  if (ch === '\\' && pos + 1 < str.length) {
    const nextCh = str[pos + 1]
    const desc = ESCAPE_MAP[nextCh]
    const node: VisNode = {
      type: 'escape',
      label: desc ? desc : `转义字符`,
      detail: `\\${nextCh}`,
    }
    const { q, next: n2 } = parseQuantifier(str, pos + 2)
    if (q) node.quantifier = q
    return { node, next: n2 }
  }

  // dot
  if (ch === '.') {
    const node: VisNode = { type: 'dot', label: '任意字符', detail: '.' }
    const { q, next: n2 } = parseQuantifier(str, pos + 1)
    if (q) node.quantifier = q
    return { node, next: n2 }
  }

  // anchors
  if (ch === '^') {
    return { node: { type: 'anchor', label: '行首', detail: '^' }, next: pos + 1 }
  }
  if (ch === '$') {
    return { node: { type: 'anchor', label: '行尾', detail: '$' }, next: pos + 1 }
  }

  // literal
  const node: VisNode = { type: 'literal', label: '字面量', detail: ch }
  const { q, next: n2 } = parseQuantifier(str, pos + 1)
  if (q) node.quantifier = q
  return { node, next: n2 }
}

function parseRegex(pattern: string): VisNode[] {
  const nodes: VisNode[] = []
  let pos = 0
  while (pos < pattern.length) {
    if (pattern[pos] === '|') {
      nodes.push({ type: 'alt', label: '或', detail: '|' })
      pos++
      continue
    }
    const { node, next } = parseToken(pattern, pos)
    nodes.push(node)
    pos = next
  }
  return nodes
}

const handleVisualize = () => {
  visError.value = ''
  visNodes.value = []
  visTree.value = []
  if (!visPattern.value.trim()) {
    visError.value = '请输入正则表达式'
    return
  }
  try {
    // 验证正则是否合法
    new RegExp(visPattern.value)
    const tree = parseRegex(visPattern.value)
    visTree.value = tree
    // 扁平化用于流程图（展开分组的子节点）
    const flat: VisNode[] = []
    for (const node of tree) {
      flat.push(node)
      if (node.children && node.children.length) {
        for (const child of node.children) {
          flat.push(child)
        }
      }
    }
    visNodes.value = flat
  } catch (e: any) {
    visError.value = `正则表达式语法错误: ${e.message}`
  }
}
</script>

<style scoped>
/* ===== 一级 Tab 样式 ===== */
.regex-tool-tabs :deep(.el-tabs__header) {
  margin-bottom: 16px;
  padding-left: 8px;
  position: sticky;
  top: 0;
  z-index: 20;
  background: var(--bg-primary);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}

html.light .regex-tool-tabs :deep(.el-tabs__header) {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.regex-tool-tabs :deep(.el-tabs__nav-wrap) {
  padding-left: 4px;
}

.regex-tool-tabs :deep(.el-tabs__item) {
  color: var(--text-secondary);
  font-size: 14px;
  font-weight: 500;
}

.regex-tool-tabs :deep(.el-tabs__item.is-active) {
  color: var(--accent-cyan);
}

.regex-tool-tabs :deep(.el-tabs__active-bar) {
  background-color: var(--accent-cyan);
}

.regex-tool-tabs :deep(.el-tabs__nav-wrap::after) {
  background-color: var(--border-color);
}

/* ===== 工具卡片 ===== */
.tool-card {
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  margin-bottom: 16px;
  overflow: hidden;
  transition: border-color 0.3s;
}

.tool-card:last-child {
  margin-bottom: 0;
}

.tool-card:hover {
  border-color: rgba(0, 212, 255, 0.3);
}

.sticky-card {
  position: sticky;
  top: 0;
  z-index: 10;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

/* 标题栏 */
.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  background: rgba(0, 0, 0, 0.2);
  border-bottom: 1px solid var(--border-color);
}

.card-title {
  font-weight: 600;
  font-size: 14px;
  color: var(--accent-cyan);
  text-transform: uppercase;
  letter-spacing: 1px;
}

.card-body {
  padding: 16px 20px;
}

.card-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

/* 提示图标 */
.hint-icon {
  font-size: 15px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: color 0.2s;
  flex-shrink: 0;
}

.hint-icon:hover {
  color: var(--accent-cyan);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.tooltip-content {
  max-width: 320px;
  line-height: 1.6;
}

.tooltip-content p {
  margin: 2px 0;
}

.tooltip-content code {
  background: rgba(0, 212, 255, 0.1);
  padding: 1px 4px;
  border-radius: 3px;
  font-size: 12px;
}

/* 标志行 */
.flags-row {
  display: flex;
  gap: 16px;
  margin-top: 8px;
}

/* 匹配计数 */
.match-count {
  font-size: 13px;
  color: var(--accent-cyan);
}

/* 分组捕获 */
.groups-section {
  margin-bottom: 8px;
}

.section-label {
  font-size: 13px;
  color: var(--text-secondary);
  margin-bottom: 8px;
  font-weight: 500;
}

.group-list {
  max-height: 300px;
  overflow-y: auto;
}

.group-item {
  display: flex;
  gap: 8px;
  padding: 8px 12px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  margin-bottom: 8px;
}

.group-index {
  font-size: 11px;
  color: var(--accent-cyan);
  background: rgba(0, 212, 255, 0.1);
  padding: 2px 6px;
  border-radius: 3px;
  min-width: 30px;
  text-align: center;
  flex-shrink: 0;
}

.group-content {
  flex: 1;
  min-width: 0;
}

.group-full {
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  color: var(--text-primary);
  background: rgba(0, 212, 255, 0.05);
  padding: 4px 8px;
  border-radius: 3px;
  word-break: break-all;
  margin-bottom: 6px;
}

.group-captures {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.capture-tag {
  font-size: 12px;
  color: var(--text-secondary);
  background: rgba(0, 212, 255, 0.08);
  padding: 2px 8px;
  border-radius: 3px;
  border: 1px solid var(--border-color);
}

.capture-num {
  color: var(--accent-cyan);
  font-weight: 600;
  margin-right: 4px;
}

/* 简单匹配列表 */
.match-list {
  max-height: 300px;
  overflow-y: auto;
}

.match-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 12px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  margin-bottom: 8px;
}

.match-index {
  font-size: 11px;
  color: var(--accent-cyan);
  background: rgba(0, 212, 255, 0.1);
  padding: 2px 6px;
  border-radius: 3px;
  min-width: 30px;
  text-align: center;
}

.match-text {
  flex: 1;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  color: var(--text-primary);
  background: rgba(0, 212, 255, 0.05);
  padding: 4px 8px;
  border-radius: 3px;
  word-break: break-all;
}

.match-pos {
  font-size: 11px;
  color: var(--text-muted);
  white-space: nowrap;
}

.no-match {
  color: var(--text-muted);
  text-align: center;
  padding: 20px;
}

.error-message {
  padding: 8px 12px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid var(--accent-red);
  border-radius: 4px;
  color: var(--accent-red);
  font-size: 13px;
  line-height: 1.5;
}

.stats-empty {
  text-align: center;
  padding: 40px 0;
  color: var(--text-muted);
  font-style: italic;
}

/* ===== 模板库 ===== */
.template-group-card:hover {
  border-color: rgba(0, 212, 255, 0.3);
}

.template-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 12px;
}

.template-item {
  padding: 12px 16px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  cursor: pointer;
  transition: border-color 0.2s, background 0.2s;
}

.template-item:hover {
  border-color: var(--accent-cyan);
  background: rgba(0, 212, 255, 0.05);
}

.template-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
  margin-bottom: 6px;
}

.template-pattern {
  display: block;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 12px;
  color: var(--accent-cyan);
  background: rgba(0, 212, 255, 0.08);
  padding: 4px 8px;
  border-radius: 3px;
  margin-bottom: 6px;
  word-break: break-all;
}

.template-desc {
  font-size: 12px;
  color: var(--text-secondary);
}

/* ===== 正则可视化 ===== */
.regex-flow {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 6px;
  padding: 16px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  min-height: 80px;
}
.flow-node {
  padding: 8px 14px;
  border-radius: 6px;
  font-size: 13px;
  border: 1px solid var(--border-color);
  background: var(--bg-card);
  text-align: center;
  min-width: 60px;
}
.node-label {
  font-weight: 600;
  color: var(--text-primary);
}
.node-detail {
  font-size: 11px;
  color: var(--accent-cyan);
  font-family: 'Consolas', monospace;
  margin-top: 2px;
}
.node-quantifier {
  font-size: 10px;
  color: var(--accent-orange);
  margin-top: 2px;
}
.node-literal { border-color: var(--accent-green); }
.node-charclass { border-color: var(--accent-blue); }
.node-dot { border-color: var(--accent-cyan); }
.node-escape { border-color: var(--accent-cyan); }
.node-anchor { border-color: var(--accent-orange); background: rgba(245, 158, 11, 0.1); }
.node-group { border-color: var(--accent-cyan); background: rgba(0, 212, 255, 0.08); }
.node-noncap { border-color: var(--text-secondary); }
.node-lookahead, .node-lookbehind { border-color: var(--accent-red); background: rgba(239, 68, 68, 0.08); }
.node-namedgroup { border-color: var(--accent-cyan); }
.node-alt { border: none; background: transparent; color: var(--accent-orange); font-weight: 700; font-size: 18px; }
.flow-arrow { color: var(--text-secondary); font-size: 16px; }
.flow-alt-sep { color: var(--accent-orange); font-weight: 700; font-size: 18px; }

.regex-tree {
  padding: 16px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 6px;
}
.tree-node { margin-bottom: 8px; }
.tree-root > .tree-label {
  font-weight: 700;
  color: var(--accent-cyan);
  font-size: 15px;
}
.tree-children {
  margin-left: 24px;
  border-left: 2px dashed var(--border-color);
  padding-left: 16px;
  margin-top: 6px;
}
.tree-branch { margin: 6px 0; }
.tree-leaf { margin: 4px 0; }
.tree-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
}
.tree-literal { color: var(--accent-green); }
.tree-charclass { color: var(--accent-blue); }
.tree-dot, .tree-escape { color: var(--accent-cyan); }
.tree-anchor { color: var(--accent-orange); }
.tree-group, .tree-namedgroup { color: var(--accent-cyan); }
.tree-noncap { color: var(--text-secondary); }
.tree-lookahead, .tree-lookbehind { color: var(--accent-red); }
.tree-alt { color: var(--accent-orange); font-weight: 700; }
.tree-detail {
  font-size: 11px;
  color: var(--accent-cyan);
  font-family: 'Consolas', monospace;
  margin-left: 8px;
}
.tree-quantifier {
  font-size: 10px;
  color: var(--accent-orange);
  margin-left: 8px;
}
</style>
