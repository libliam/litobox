<template>
  <div class="tool-container">
    <el-tabs v-model="activeTab" class="url-tabs" @tab-click="handleTabClick">
      <el-tab-pane label="参数解析" name="parse" />
      <el-tab-pane label="参数组装" name="build" />
    </el-tabs>

    <!-- 参数解析 Tab -->
    <div v-if="activeTab === 'parse'" class="tool-card">
      <div class="card-header">
        <span class="card-title">输入 URL</span>
        <div class="card-actions">
          <el-button size="small" @click="handleClear">清空</el-button>
          <el-button size="small" @click="handlePaste">粘贴</el-button>
        </div>
      </div>
      <div class="card-body">
        <el-input v-model="urlInput" type="textarea" :rows="3" placeholder="粘贴完整 URL，自动拆分参数..." resize="vertical" @input="handleUrlParse" />
      </div>
    </div>

    <div v-if="activeTab === 'parse'" class="tool-card">
      <div class="card-header">
        <span class="card-title">URL 结构</span>
      </div>
      <div class="card-body">
        <div class="url-structure">
          <div class="struct-item"><span class="struct-label">协议：</span><span class="struct-value">{{ urlParts.protocol }}</span></div>
          <div class="struct-item"><span class="struct-label">主机：</span><span class="struct-value">{{ urlParts.host }}</span></div>
          <div class="struct-item"><span class="struct-label">路径：</span><span class="struct-value">{{ urlParts.pathname }}</span></div>
          <div class="struct-item"><span class="struct-label">哈希：</span><span class="struct-value">{{ urlParts.hash || '-' }}</span></div>
        </div>
      </div>
    </div>

    <div v-if="activeTab === 'parse'" class="tool-card">
      <div class="card-header">
        <span class="card-title">参数列表</span>
        <el-button size="small" @click="handleCopyParams">复制参数</el-button>
      </div>
      <div class="card-body">
        <div v-if="urlParams.length === 0" class="empty-hint">输入 URL 后自动解析参数</div>
        <el-table v-else :data="urlParams" border size="small" class="param-table">
          <el-table-column prop="key" label="参数名" width="200" />
          <el-table-column prop="value" label="参数值" />
          <el-table-column label="操作" width="80">
            <template #default="{ row }">
              <el-button size="small" @click="handleCopy(row.value)">复制</el-button>
            </template>
          </el-table-column>
        </el-table>
      </div>
    </div>

    <!-- 参数组装 Tab -->
    <div v-if="activeTab === 'build'" class="tool-card">
      <div class="card-header">
        <span class="card-title">基础 URL</span>
      </div>
      <div class="card-body">
        <el-input v-model="baseUrl" placeholder="如 https://api.example.com/v1/users" />
      </div>
    </div>

    <div v-if="activeTab === 'build'" class="tool-card">
      <div class="card-header">
        <span class="card-title">参数列表</span>
        <div class="card-actions">
          <el-button size="small" type="primary" @click="addParam">添加参数</el-button>
          <el-button size="small" @click="clearParams">清空</el-button>
        </div>
      </div>
      <div class="card-body">
        <div v-for="(param, index) in buildParams" :key="index" class="param-row">
          <el-input v-model="param.key" placeholder="参数名" style="width: 180px" />
          <span class="param-eq">=</span>
          <el-input v-model="param.value" placeholder="参数值" />
          <el-button size="small" type="danger" @click="removeParam(index)" :icon="'Delete'">×</el-button>
        </div>
        <div v-if="buildParams.length === 0" class="empty-hint">点击"添加参数"开始组装</div>
      </div>
    </div>

    <div v-if="activeTab === 'build'" class="tool-card">
      <div class="card-header">
        <span class="card-title">生成 URL</span>
        <el-button size="small" @click="handleCopy(builtUrl)">复制</el-button>
      </div>
      <div class="card-body">
        <el-input :model-value="builtUrl" readonly type="textarea" :rows="3" resize="vertical" class="code-input" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { ElMessage } from 'element-plus'

const activeTab = ref('parse')

// 解析
const urlInput = ref('')
const urlParts = ref({ protocol: '', host: '', pathname: '', hash: '' })
const urlParams = ref<{ key: string; value: string }[]>([])

// 组装
const baseUrl = ref('')
const buildParams = ref<{ key: string; value: string }[]>([])

const builtUrl = computed(() => {
  if (!baseUrl.value) return ''
  const params = buildParams.value.filter(p => p.key)
  if (params.length === 0) return baseUrl.value
  const query = params.map(p => `${encodeURIComponent(p.key)}=${encodeURIComponent(p.value)}`).join('&')
  return `${baseUrl.value}?${query}`
})

const handleTabClick = () => {}

const handleUrlParse = () => {
  const url = urlInput.value.trim()
  if (!url) {
    urlParts.value = { protocol: '', host: '', pathname: '', hash: '' }
    urlParams.value = []
    return
  }
  try {
    // 如果用户没有输入协议，自动补全
    let fullUrl = url
    if (!url.startsWith('http://') && !url.startsWith('https://')) {
      fullUrl = 'https://' + url
    }
    const parsed = new URL(fullUrl)
    urlParts.value = {
      protocol: parsed.protocol,
      host: parsed.host,
      pathname: parsed.pathname,
      hash: parsed.hash
    }
    const params: { key: string; value: string }[] = []
    parsed.searchParams.forEach((value, key) => {
      params.push({ key, value })
    })
    urlParams.value = params
  } catch {
    urlParts.value = { protocol: '', host: '', pathname: '', hash: '' }
    urlParams.value = []
  }
}

const addParam = () => {
  buildParams.value.push({ key: '', value: '' })
}

const removeParam = (index: number) => {
  buildParams.value.splice(index, 1)
}

const clearParams = () => {
  buildParams.value = []
}

const handleClear = () => {
  urlInput.value = ''
  urlParts.value = { protocol: '', host: '', pathname: '', hash: '' }
  urlParams.value = []
}

const handlePaste = async () => {
  try {
    const text = await navigator.clipboard.readText()
    urlInput.value = text
    handleUrlParse()
  } catch {
    ElMessage.warning('无法读取剪贴板')
  }
}

const handleCopy = async (text: string) => {
  if (!text) {
    ElMessage.warning('没有可复制的内容')
    return
  }
  try {
    await navigator.clipboard.writeText(text)
    ElMessage.success('已复制到剪贴板')
  } catch {
    ElMessage.error('复制失败')
  }
}

const handleCopyParams = async () => {
  if (urlParams.value.length === 0) {
    ElMessage.warning('没有参数可复制')
    return
  }
  const text = urlParams.value.map(p => `${p.key}=${p.value}`).join('\n')
  await handleCopy(text)
}
</script>

<style scoped>
/* 二级 Tab（子功能切换） */
.url-tabs {
  margin-bottom: 8px;
  margin-top: -4px;
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  padding: 4px 8px 4px 12px;
}

.url-tabs :deep(.el-tabs__header) {
  margin-bottom: 0;
  padding-left: 0;
  border-bottom: none;
}

.url-tabs :deep(.el-tabs__nav-wrap) {
  padding-left: 0;
}

.url-tabs :deep(.el-tabs__nav-wrap::after) {
  display: none;
}

.url-tabs :deep(.el-tabs__item) {
  color: var(--text-secondary);
  font-size: 12px;
  font-weight: 400;
  padding: 0 12px;
  height: 28px;
  line-height: 28px;
  border-radius: 4px;
  margin-right: 4px;
  transition: all 0.2s;
}

.url-tabs :deep(.el-tabs__item:hover) {
  color: var(--text-primary);
  background: rgba(0, 212, 255, 0.05);
}

.url-tabs :deep(.el-tabs__item.is-active) {
  color: var(--accent-cyan);
  font-weight: 500;
  background: rgba(0, 212, 255, 0.1);
}

.url-tabs :deep(.el-tabs__active-bar) {
  display: none;
}

html.light .url-tabs {
  background: var(--bg-card);
  border-color: var(--border-color);
}

html.light .url-tabs :deep(.el-tabs__item:hover) {
  background: rgba(8, 145, 178, 0.05);
}

html.light .url-tabs :deep(.el-tabs__item.is-active) {
  background: rgba(8, 145, 178, 0.1);
}

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

.url-structure {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.struct-item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.struct-label {
  font-size: 13px;
  color: var(--text-secondary);
  min-width: 60px;
}

.struct-value {
  font-size: 14px;
  color: var(--text-primary);
  font-family: 'Courier New', monospace;
}

.param-table {
  width: 100%;
}

.param-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.param-row:last-child {
  margin-bottom: 0;
}

.param-eq {
  font-size: 16px;
  color: var(--text-secondary);
  font-weight: 600;
}

.empty-hint {
  color: var(--text-secondary);
  font-size: 13px;
  text-align: center;
  padding: 16px;
}

.code-input :deep(.el-textarea__inner) {
  font-family: 'Courier New', monospace;
  font-size: 13px;
}
</style>
