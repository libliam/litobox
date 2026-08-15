<template>
  <div class="tool-container">
    <div class="tool-card sticky-card">
      <div class="card-header">
        <span class="card-title">操作</span>
        <div class="tab-switch">
          <el-radio-group v-model="mode" size="small">
            <el-radio-button label="hash">哈希摘要</el-radio-button>
            <el-radio-button label="crypto">加密解密</el-radio-button>
          </el-radio-group>
        </div>
      </div>
      <div class="card-body">
        <!-- 哈希模式 -->
        <div v-if="mode === 'hash'" class="action-grid">
          <div class="action-group" style="--group-color: #00d4ff">
            <div class="group-label">
              算法
              <el-tooltip placement="bottom" effect="dark">
                <template #content>
                  <div class="tooltip-content">
                    <p><b>MD5</b>：128位，速度快，已不安全，仅用于文件校验</p>
                    <p><b>SHA-1</b>：160位，已被攻破，不推荐安全场景</p>
                    <p><b>SHA-224</b>：224位，SHA-2家族，适合空间受限场景</p>
                    <p><b>SHA-256</b>：256位，安全可靠，目前最广泛使用</p>
                    <p><b>SHA-384</b>：384位，SHA-2家族，安全性高于SHA-256</p>
                    <p><b>SHA-512</b>：512位，安全性最高，64位系统性能更优</p>
                    <p><b>SHA-3</b>：256位，第三代哈希标准，抗碰撞能力更强</p>
                    <p><b>RIPEMD-160</b>：160位，比特币地址使用，欧洲标准</p>
                  </div>
                </template>
                <el-icon class="hint-icon"><QuestionFilled /></el-icon>
              </el-tooltip>
            </div>
            <el-checkbox-group v-model="selectedAlgorithms" size="small">
              <el-checkbox-button label="md5">MD5</el-checkbox-button>
              <el-checkbox-button label="sha1">SHA-1</el-checkbox-button>
              <el-checkbox-button label="sha224">SHA-224</el-checkbox-button>
              <el-checkbox-button label="sha256">SHA-256</el-checkbox-button>
              <el-checkbox-button label="sha384">SHA-384</el-checkbox-button>
              <el-checkbox-button label="sha512">SHA-512</el-checkbox-button>
              <el-checkbox-button label="sha3">SHA-3</el-checkbox-button>
              <el-checkbox-button label="ripemd160">RIPEMD-160</el-checkbox-button>
            </el-checkbox-group>
          </div>
          <div class="action-group" style="--group-color: #f59e0b">
            <div class="group-label">HMAC密钥</div>
            <el-input v-model="hmacKey" placeholder="可选" size="small" style="width: 140px" clearable />
          </div>
          <div class="action-group" style="--group-color: #10b981">
            <div class="group-label">执行</div>
            <div class="group-buttons">
              <el-button type="primary" size="small" @click="handleHash">计算</el-button>
              <el-button size="small" @click="handleFileHash">文件哈希</el-button>
            </div>
            <input ref="fileInput" type="file" style="display: none" @change="handleFileChange" />
          </div>
        </div>

        <!-- 加密解密模式 -->
        <div v-else class="action-grid">
          <div class="action-group" style="--group-color: #8b5cf6">
            <div class="group-label">
              算法
              <el-tooltip placement="bottom" effect="dark">
                <template #content>
                  <div class="tooltip-content">
                    <p><b>AES</b>：最常用对称加密，安全性高，支持128/192/256位密钥</p>
                    <p><b>DES</b>：传统加密，56位密钥已不安全，仅兼容旧系统</p>
                    <p><b>3DES</b>：DES三次增强版，168位密钥，更安全但较慢</p>
                    <p><b>RC4</b>：流加密，速度快但存在安全漏洞，不推荐新系统</p>
                    <p><b>Rabbit</b>：流加密，高性能，安全性优于RC4</p>
                  </div>
                </template>
                <el-icon class="hint-icon"><QuestionFilled /></el-icon>
              </el-tooltip>
            </div>
            <el-radio-group v-model="cryptoAlgo" size="small">
              <el-radio-button label="aes">AES</el-radio-button>
              <el-radio-button label="des">DES</el-radio-button>
              <el-radio-button label="3des">3DES</el-radio-button>
              <el-radio-button label="rc4">RC4</el-radio-button>
              <el-radio-button label="rabbit">Rabbit</el-radio-button>
            </el-radio-group>
          </div>
          <div class="action-group" style="--group-color: #f59e0b">
            <div class="group-label">密钥</div>
            <el-input v-model="cryptoKey" placeholder="请输入密钥" size="small" style="width: 160px" show-password />
          </div>
          <div class="action-group" style="--group-color: #10b981">
            <div class="group-label">执行</div>
            <div class="group-buttons">
              <el-button type="primary" size="small" @click="handleCryptoEncode">加密</el-button>
              <el-button type="primary" size="small" @click="handleCryptoDecode">解密</el-button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">输入</span>
        <div class="card-actions">
          <VariablePicker @select="handleInsertVariable" />
          <el-button size="small" @click="handleClear">清空</el-button>
          <el-button size="small" @click="handlePaste">粘贴</el-button>
        </div>
      </div>
      <div class="card-body">
        <el-input v-model="input" type="textarea" :rows="6" placeholder="请输入内容..." resize="vertical" />
        <div v-if="fileName" class="file-info">
          <el-tag size="small" type="success">{{ fileName }} ({{ formatFileSize(fileSize) }})</el-tag>
        </div>
      </div>
    </div>

    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">{{ mode === 'hash' ? '哈希结果' : '结果' }}</span>
        <div class="card-actions">
          <el-button v-if="mode === 'crypto' && outputValue" size="small" @click="handleOutputToInput">转到输入</el-button>
          <el-button v-if="results.length > 0" size="small" @click="handleCopyAll">复制全部</el-button>
          <el-button v-if="mode === 'crypto' && outputValue" size="small" @click="handleCopyText">复制</el-button>
        </div>
      </div>
      <div class="card-body">
        <!-- 哈希结果列表 -->
        <div v-if="mode === 'hash'">
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
        <!-- 加解密结果 -->
        <div v-else>
          <el-input
            v-model="outputValue"
            type="textarea"
            :rows="8"
            readonly
            resize="vertical"
            :class="{ 'error': isError }"
            placeholder="加密/解密结果将在此显示..."
          />
          <div v-if="errorMessage" class="error-message">{{ errorMessage }}</div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import { hashText, hmacText, hashFile, type HashAlgorithm, type HashResult } from '@/utils/hashUtils'
import { aesEncode, aesDecode, desEncode, desDecode, tripleDesEncode, tripleDesDecode, rc4Encode, rc4Decode, rabbitEncode, rabbitDecode } from '@/utils/encodeUtils'
import { useToolboxStore } from '@/store'
import VariablePicker from '@/components/VariablePicker.vue'

const store = useToolboxStore()

const mode = ref<'hash' | 'crypto'>('hash')
const input = ref('')

// 哈希相关
const selectedAlgorithms = ref<HashAlgorithm[]>(['md5', 'sha256'])
const hmacKey = ref('')
const results = ref<HashResult[]>([])
const fileName = ref('')
const fileSize = ref(0)
const fileInput = ref<HTMLInputElement>()

// 加解密相关
const cryptoAlgo = ref<'aes' | 'des' | '3des' | 'rc4' | 'rabbit'>('aes')
const cryptoKey = ref('')
const outputValue = ref('')
const errorMessage = ref('')
const isError = ref(false)

// ========== 哈希 ==========
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
    inputFull: text,
    outputFull: results.value.map(r => `${r.algorithm}: ${r.hash}`).join('\n'),
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

  // 文件哈希仅支持 SHA-1/256/384/512（Web Crypto API 限制）
  const fileSupportedAlgos = selectedAlgorithms.value.filter(a => ['sha1', 'sha256', 'sha384', 'sha512'].includes(a))
  const unsupported = selectedAlgorithms.value.filter(a => !['sha1', 'sha256', 'sha384', 'sha512'].includes(a))
  if (unsupported.length > 0) {
    ElMessage.warning(`文件哈希不支持 ${unsupported.map(a => a.toUpperCase()).join('、')}，仅支持 SHA-1/256/384/512`)
  }
  if (fileSupportedAlgos.length === 0) {
    ElMessage.warning('已自动选择 SHA-256')
    selectedAlgorithms.value = ['sha256']
  }

  results.value = []
  for (const algo of (fileSupportedAlgos.length > 0 ? fileSupportedAlgos : ['sha256' as const])) {
    try {
      const hash = await hashFile(file, algo as 'sha1' | 'sha256' | 'sha384' | 'sha512')
      results.value.push({ algorithm: algo.toUpperCase(), hash })
    } catch (e: any) {
      results.value.push({ algorithm: algo.toUpperCase(), hash: `错误: ${e.message}` })
    }
  }

  ElMessage.success('文件哈希计算完成')
  target.value = ''
}

// ========== 加解密 ==========
const cryptoEncodeMap = { aes: aesEncode, des: desEncode, '3des': tripleDesEncode, rc4: rc4Encode, rabbit: rabbitEncode }
const cryptoDecodeMap = { aes: aesDecode, des: desDecode, '3des': tripleDesDecode, rc4: rc4Decode, rabbit: rabbitDecode }
const cryptoLabelMap: Record<string, string> = { aes: 'AES', des: 'DES', '3des': '3DES', rc4: 'RC4', rabbit: 'Rabbit' }

const handleCryptoEncode = () => {
  if (!input.value.trim()) {
    ElMessage.warning('请输入内容')
    return
  }
  if (!cryptoKey.value) {
    ElMessage.warning('请输入密钥')
    return
  }
  const fn = cryptoEncodeMap[cryptoAlgo.value]
  const result = fn(input.value, cryptoKey.value)
  outputValue.value = result
  errorMessage.value = ''
  isError.value = result.includes('失败')
  store.addHistory({
    tool: 'hash',
    action: `${cryptoLabelMap[cryptoAlgo.value]}加密`,
    inputPreview: input.value.slice(0, 50),
    outputPreview: outputValue.value.slice(0, 50),
    inputFull: input.value,
    outputFull: outputValue.value,
  })
  ElMessage.success('加密完成')
}

const handleCryptoDecode = () => {
  if (!input.value.trim()) {
    ElMessage.warning('请输入内容')
    return
  }
  if (!cryptoKey.value) {
    ElMessage.warning('请输入密钥')
    return
  }
  const fn = cryptoDecodeMap[cryptoAlgo.value]
  const result = fn(input.value, cryptoKey.value)
  outputValue.value = result
  errorMessage.value = ''
  isError.value = result.includes('失败')
  store.addHistory({
    tool: 'hash',
    action: `${cryptoLabelMap[cryptoAlgo.value]}解密`,
    inputPreview: input.value.slice(0, 50),
    outputPreview: outputValue.value.slice(0, 50),
    inputFull: input.value,
    outputFull: outputValue.value,
  })
  ElMessage.success('解密完成')
}

// ========== 通用 ==========
const handleClear = () => {
  input.value = ''
  outputValue.value = ''
  errorMessage.value = ''
  isError.value = false
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

const handleCopyText = async () => {
  try {
    await navigator.clipboard.writeText(outputValue.value)
    ElMessage.success('已复制')
  } catch {
    ElMessage.error('复制失败')
  }
}

const handleOutputToInput = () => {
  if (!outputValue.value) {
    ElMessage.warning('输出为空')
    return
  }
  input.value = outputValue.value
  outputValue.value = ''
  errorMessage.value = ''
  isError.value = false
  ElMessage.success('已转到输入')
}

const formatFileSize = (bytes: number): string => {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
}
</script>

<style scoped>
.tool-container {
  height: 100%;
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
  font-size: 13px;
  color: var(--accent-cyan);
  text-transform: uppercase;
  letter-spacing: 1px;
}
.card-actions { display: flex; align-items: center; gap: 6px; }
.card-body { padding: 16px 20px; }

.tab-switch { display: flex; align-items: center; }

.action-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  align-items: stretch;
}
.action-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 10px 12px;
  background: rgba(255, 255, 255, 0.02);
  border: 1px solid var(--border-color);
  border-left: 3px solid var(--group-color, var(--accent-cyan));
  border-radius: 6px;
  min-width: 120px;
}
.group-label {
  font-size: 12px;
  color: var(--group-color, var(--text-secondary));
  font-weight: 600;
  letter-spacing: 0.5px;
  display: flex;
  align-items: center;
  gap: 4px;
}
.group-buttons { display: flex; gap: 6px; flex-wrap: wrap; }
.hint-icon {
  font-size: 14px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: color 0.2s;
  flex-shrink: 0;
}
.hint-icon:hover { color: var(--accent-cyan); }
.tooltip-content { max-width: 340px; line-height: 1.8; }
.tooltip-content p { margin: 2px 0; }

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

.error :deep(.el-textarea__inner) {
  border-color: var(--accent-red) !important;
  box-shadow: 0 0 10px rgba(239, 68, 68, 0.2) !important;
}
.error-message {
  color: var(--accent-red);
  font-size: 12px;
  margin-top: 10px;
  padding: 8px 12px;
  background: rgba(239, 68, 68, 0.08);
  border: 1px solid rgba(239, 68, 68, 0.2);
  border-radius: 4px;
}
</style>
