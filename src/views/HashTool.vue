<template>
  <div class="tool-container">
    <div class="tool-card sticky-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">操作</span>
          <el-tooltip placement="bottom" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>• 支持 MD5、SHA-1、SHA-256、SHA-512</p>
                <p>• 支持文本和文件哈希计算</p>
                <p>• 可选 HMAC 密钥进行密钥哈希</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-label">算法</div>
            <el-checkbox-group v-model="selectedAlgorithms" size="small">
              <el-checkbox-button label="md5">MD5</el-checkbox-button>
              <el-checkbox-button label="sha1">SHA-1</el-checkbox-button>
              <el-checkbox-button label="sha256">SHA-256</el-checkbox-button>
              <el-checkbox-button label="sha512">SHA-512</el-checkbox-button>
            </el-checkbox-group>
          </div>
          <div class="action-group">
            <div class="group-label">HMAC密钥</div>
            <el-input v-model="hmacKey" placeholder="可选" size="small" style="width: 140px" clearable />
          </div>
          <div class="action-group">
            <div class="group-label">执行</div>
            <div class="group-buttons">
              <el-button type="primary" size="small" @click="handleHash">计算</el-button>
              <el-button size="small" @click="handleFileHash">文件哈希</el-button>
            </div>
            <input ref="fileInput" type="file" style="display: none" @change="handleFileChange" />
          </div>
        </div>
      </div>
    </div>

    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">输入 (文本或文件)</span>
        <div class="card-actions">
          <VariablePicker @select="handleInsertVariable" />
          <el-button size="small" @click="handleClear">清空</el-button>
          <el-button size="small" @click="handlePaste">粘贴</el-button>
        </div>
      </div>
      <div class="card-body">
        <el-input v-model="input" type="textarea" :rows="6" placeholder="请输入文本，或点击「文件哈希」上传文件..." resize="vertical" />
        <div v-if="fileName" class="file-info">
          <el-tag size="small" type="success">{{ fileName }} ({{ formatFileSize(fileSize) }})</el-tag>
        </div>
      </div>
    </div>

    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">哈希结果</span>
        <el-button v-if="results.length > 0" size="small" @click="handleCopyAll">复制全部</el-button>
      </div>
      <div class="card-body">
        <div v-if="results.length > 0" class="result-list">
          <div v-for="result in results" :key="result.algorithm" class="result-item">
            <span class="result-algo">{{ result.algorithm }}</span>
            <span class="result-hash" @click="handleCopyOne(result.hash)">{{ result.hash }}</span>
            <el-tooltip content="点击复制" placement="top">
              <el-button size="small" text @click="handleCopyOne(result.hash)">复制</el-button>
            </el-tooltip>
          </div>
        </div>
        <div v-else class="empty-tip">计算后将在此显示哈希值</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import { hashText, hmacText, hashFile, type HashAlgorithm, type HashResult } from '@/utils/hashUtils'
import { useToolboxStore } from '@/store'
import VariablePicker from '@/components/VariablePicker.vue'

const store = useToolboxStore()

const input = ref('')
const selectedAlgorithms = ref<HashAlgorithm[]>(['md5', 'sha256'])
const hmacKey = ref('')
const results = ref<HashResult[]>([])
const fileName = ref('')
const fileSize = ref(0)
const fileInput = ref<HTMLInputElement>()

const handleHash = () => {
  if (!input.value.trim() && !fileName.value) {
    ElMessage.warning('请输入文本或选择文件')
    return
  }
  if (selectedAlgorithms.value.length === 0) {
    ElMessage.warning('请至少选择一种算法')
    return
  }

  results.value = []
  const text = input.value

  for (const algo of selectedAlgorithms.value) {
    try {
      const hash = hmacKey.value
        ? hmacText(text, hmacKey.value, algo)
        : hashText(text, algo)
      results.value.push({ algorithm: algo.toUpperCase(), hash })
    } catch (e: any) {
      results.value.push({ algorithm: algo.toUpperCase(), hash: `错误: ${e.message}` })
    }
  }

  store.addHistory({
    tool: 'hash',
    action: hmacKey.value ? 'HMAC计算' : '哈希计算',
    inputPreview: text.slice(0, 50),
    outputPreview: results.value.map(r => r.hash.slice(0, 16)).join(', '),
  })

  ElMessage.success('计算完成')
}

const handleFileHash = () => {
  fileInput.value?.click()
}

const handleFileChange = async (e: Event) => {
  const target = e.target as HTMLInputElement
  const file = target.files?.[0]
  if (!file) return

  fileName.value = file.name
  fileSize.value = file.size
  input.value = `[文件: ${file.name}]`

  // 文件哈希只支持 SHA 系列
  const shaAlgos = selectedAlgorithms.value.filter(a => a !== 'md5')
  if (shaAlgos.length === 0) {
    ElMessage.warning('文件哈希不支持 MD5，已自动选择 SHA-256')
    selectedAlgorithms.value = ['sha256']
  }

  results.value = []
  for (const algo of shaAlgos) {
    try {
      const hash = await hashFile(file, algo as 'sha1' | 'sha256' | 'sha512')
      results.value.push({ algorithm: algo.toUpperCase(), hash })
    } catch (e: any) {
      results.value.push({ algorithm: algo.toUpperCase(), hash: `错误: ${e.message}` })
    }
  }

  ElMessage.success('文件哈希计算完成')
  target.value = ''
}

const handleClear = () => {
  input.value = ''
  results.value = []
  fileName.value = ''
  fileSize.value = 0
}

const handlePaste = async () => {
  try {
    input.value = await navigator.clipboard.readText()
  } catch {
    ElMessage.warning('无法读取剪贴板')
  }
}

const handleInsertVariable = (value: string) => {
  input.value = value
}

const handleCopyOne = (hash: string) => {
  navigator.clipboard.writeText(hash)
  ElMessage.success('已复制')
}

const handleCopyAll = () => {
  const text = results.value.map(r => `${r.algorithm}: ${r.hash}`).join('\n')
  navigator.clipboard.writeText(text)
  ElMessage.success('已复制全部结果')
}

const formatFileSize = (bytes: number): string => {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
}
</script>

<style scoped>
.tool-container {
  height: 100vh;
  overflow-y: auto;
  padding: 20px;
  background: var(--bg-primary);
}

.tool-card {
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  margin-bottom: 16px;
  overflow: hidden;
  transition: border-color 0.3s;
}
.tool-card:last-child { margin-bottom: 0; }
.tool-card:hover { border-color: rgba(0, 212, 255, 0.3); }

.sticky-card {
  position: sticky;
  top: 0;
  z-index: 10;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
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
.card-actions { display: flex; align-items: center; gap: 6px; }
.header-left { display: flex; align-items: center; gap: 8px; }
.card-body { padding: 16px 20px; }

.hint-icon {
  font-size: 15px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: color 0.2s;
  flex-shrink: 0;
}
.hint-icon:hover { color: var(--accent-cyan); }
.tooltip-content { max-width: 320px; line-height: 1.6; }
.tooltip-content p { margin: 2px 0; }

.action-grid { display: flex; flex-wrap: wrap; gap: 16px; align-items: center; }
.action-group { display: flex; align-items: center; gap: 8px; }
.group-label { color: var(--text-secondary); font-size: 13px; white-space: nowrap; }
.group-buttons { display: flex; gap: 6px; }

.empty-tip {
  text-align: center;
  color: var(--text-muted);
  padding: 40px 0;
  font-size: 14px;
}

.file-info { margin-top: 8px; }

.result-list { display: flex; flex-direction: column; gap: 8px; }
.result-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 12px;
  background: var(--bg-input);
  border-radius: 6px;
  border: 1px solid var(--border-color);
}
.result-algo {
  font-weight: 600;
  font-size: 12px;
  color: var(--accent-cyan);
  min-width: 70px;
  flex-shrink: 0;
}
.result-hash {
  flex: 1;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  color: var(--text-primary);
  word-break: break-all;
  cursor: pointer;
}
.result-hash:hover { color: var(--accent-cyan); }
</style>
