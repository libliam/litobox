<template>
  <div class="tool-container">
    <el-tabs v-model="activeTab" class="crypto-tabs" @tab-click="handleTabClick">
      <el-tab-pane label="哈希" name="hash" />
      <el-tab-pane label="HMAC" name="hmac" />
      <el-tab-pane label="AES" name="aes" />
      <el-tab-pane label="RSA" name="rsa" />
    </el-tabs>

    <!-- 哈希 Tab -->
    <div v-if="activeTab === 'hash'" class="tool-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">哈希计算</span>
          <el-tooltip placement="bottom" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>• 支持 MD5、SHA1、SHA256、SHA512</p>
                <p>• 输入明文后一键出密文</p>
                <p>• 可勾选多种算法同时计算</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
        <div class="card-actions">
          <VariablePicker @select="handleInsertVariable" />
          <el-button size="small" @click="handleClear">清空</el-button>
          <el-button size="small" @click="handlePaste">粘贴</el-button>
        </div>
      </div>
      <div class="card-body">
        <el-input v-model="hashInput" type="textarea" :rows="4" placeholder="请输入明文..." resize="vertical" />
      </div>
    </div>

    <div v-if="activeTab === 'hash'" class="tool-card">
      <div class="card-header">
        <span class="card-title">算法选择</span>
      </div>
      <div class="card-body">
        <el-checkbox-group v-model="hashAlgorithms">
          <el-checkbox label="MD5" />
          <el-checkbox label="SHA1" />
          <el-checkbox label="SHA256" />
          <el-checkbox label="SHA512" />
        </el-checkbox-group>
      </div>
    </div>

    <div v-if="activeTab === 'hash'" class="tool-card">
      <div class="card-header">
        <span class="card-title">操作</span>
        <el-button size="small" type="primary" @click="handleHash">计算哈希</el-button>
      </div>
      <div class="card-body">
        <div v-for="algo in hashAlgorithms" :key="algo" class="hash-result-item">
          <span class="hash-label">{{ algo }}</span>
          <el-input :model-value="hashResults[algo] || ''" readonly size="small" class="hash-value">
            <template #append>
              <el-button size="small" @click="handleCopy(hashResults[algo] || '')">复制</el-button>
            </template>
          </el-input>
        </div>
        <div v-if="hashAlgorithms.length === 0" class="empty-hint">请选择至少一种算法</div>
      </div>
    </div>

    <!-- HMAC Tab -->
    <div v-if="activeTab === 'hmac'" class="tool-card">
      <div class="card-header">
        <span class="card-title">输入</span>
        <div class="card-actions">
          <el-button size="small" @click="handleClear">清空</el-button>
          <el-button size="small" @click="handlePaste">粘贴</el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="input-row">
          <label>密钥：</label>
          <el-input v-model="hmacKey" placeholder="请输入 HMAC 密钥" />
        </div>
        <el-input v-model="hmacInput" type="textarea" :rows="4" placeholder="请输入明文..." resize="vertical" style="margin-top: 12px" />
      </div>
    </div>

    <div v-if="activeTab === 'hmac'" class="tool-card">
      <div class="card-header">
        <span class="card-title">算法</span>
      </div>
      <div class="card-body">
        <el-radio-group v-model="hmacAlgorithm" size="small">
          <el-radio-button label="MD5" />
          <el-radio-button label="SHA1" />
          <el-radio-button label="SHA256" />
          <el-radio-button label="SHA512" />
        </el-radio-group>
      </div>
    </div>

    <div v-if="activeTab === 'hmac'" class="tool-card">
      <div class="card-header">
        <span class="card-title">操作</span>
        <el-button size="small" type="primary" @click="handleHmac">计算 HMAC</el-button>
      </div>
      <div class="card-body">
        <el-input :model-value="hmacResult" readonly type="textarea" :rows="3" resize="vertical" />
        <div class="copy-row">
          <el-button size="small" @click="handleCopy(hmacResult)">复制</el-button>
        </div>
      </div>
    </div>

    <!-- AES Tab -->
    <div v-if="activeTab === 'aes'" class="tool-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">AES 加解密</span>
          <el-tooltip placement="bottom" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>• 支持 ECB / CBC 模式</p>
                <p>• 密钥自动补齐/截断至 32 字节</p>
                <p>• CBC 模式需要 16 字节 IV</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
        <div class="card-actions">
          <el-button size="small" @click="handleClear">清空</el-button>
          <el-button size="small" @click="handlePaste">粘贴</el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="input-row">
          <label>密钥：</label>
          <el-input v-model="aesKey" placeholder="请输入密钥（自动补齐至32字节）" />
        </div>
        <div class="input-row" style="margin-top: 12px">
          <label>模式：</label>
          <el-radio-group v-model="aesMode" size="small">
            <el-radio-button label="ECB" />
            <el-radio-button label="CBC" />
          </el-radio-group>
        </div>
        <div v-if="aesMode === 'CBC'" class="input-row" style="margin-top: 12px">
          <label>IV：</label>
          <el-input v-model="aesIv" placeholder="请输入 IV（16字节）" />
        </div>
        <el-input v-model="aesInput" type="textarea" :rows="4" placeholder="请输入明文或密文..." resize="vertical" style="margin-top: 12px" />
      </div>
    </div>

    <div v-if="activeTab === 'aes'" class="tool-card">
      <div class="card-header">
        <span class="card-title">操作</span>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-buttons">
              <el-button size="small" type="primary" @click="handleAesEncrypt">加密</el-button>
              <el-button size="small" @click="handleAesDecrypt">解密</el-button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div v-if="activeTab === 'aes'" class="tool-card">
      <div class="card-header">
        <span class="card-title">输出</span>
        <el-button size="small" @click="handleCopy(aesResult)">复制</el-button>
      </div>
      <div class="card-body">
        <el-input :model-value="aesResult" readonly type="textarea" :rows="4" resize="vertical" :class="{ 'error': aesError }" />
        <div v-if="aesErrorMessage" class="error-message">{{ aesErrorMessage }}</div>
      </div>
    </div>

    <!-- RSA Tab -->
    <div v-if="activeTab === 'rsa'" class="tool-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">RSA 简易加解密</span>
          <el-tooltip placement="bottom" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>• 简易模拟实现，非真实 RSA 运算</p>
                <p>• 仅用于演示加解密流程</p>
                <p>• 生产环境请使用 node-rsa 等库</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
        <div class="card-actions">
          <el-button size="small" @click="handleClear">清空</el-button>
          <el-button size="small" @click="handlePaste">粘贴</el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="input-row">
          <label>公钥：</label>
          <el-input v-model="rsaPublicKey" placeholder="请输入公钥（用于模拟）" />
        </div>
        <div class="input-row" style="margin-top: 12px">
          <label>私钥：</label>
          <el-input v-model="rsaPrivateKey" placeholder="请输入私钥（用于模拟）" />
        </div>
        <el-input v-model="rsaInput" type="textarea" :rows="4" placeholder="请输入明文或密文..." resize="vertical" style="margin-top: 12px" />
      </div>
    </div>

    <div v-if="activeTab === 'rsa'" class="tool-card">
      <div class="card-header">
        <span class="card-title">操作</span>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-buttons">
              <el-button size="small" type="primary" @click="handleRsaEncrypt">加密</el-button>
              <el-button size="small" @click="handleRsaDecrypt">解密</el-button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div v-if="activeTab === 'rsa'" class="tool-card">
      <div class="card-header">
        <span class="card-title">输出</span>
        <el-button size="small" @click="handleCopy(rsaResult)">复制</el-button>
      </div>
      <div class="card-body">
        <el-input :model-value="rsaResult" readonly type="textarea" :rows="4" resize="vertical" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import * as crypto from '@/utils/cryptoUtils'
import { useToolboxStore } from '@/store'
import VariablePicker from '@/components/VariablePicker.vue'

const store = useToolboxStore()
const activeTab = ref('hash')

// Hash
const hashInput = ref('')
const hashAlgorithms = ref<string[]>(['MD5', 'SHA256'])
const hashResults = ref<Record<string, string>>({})

// HMAC
const hmacInput = ref('')
const hmacKey = ref('')
const hmacAlgorithm = ref<'MD5' | 'SHA1' | 'SHA256' | 'SHA512'>('SHA256')
const hmacResult = ref('')

// AES
const aesInput = ref('')
const aesKey = ref('')
const aesMode = ref<'ECB' | 'CBC'>('ECB')
const aesIv = ref('')
const aesResult = ref('')
const aesError = ref(false)
const aesErrorMessage = ref('')

// RSA
const rsaInput = ref('')
const rsaPublicKey = ref('')
const rsaPrivateKey = ref('')
const rsaResult = ref('')

const handleTabClick = () => {}

const handleClear = () => {
  hashInput.value = ''
  hmacInput.value = ''
  hmacKey.value = ''
  aesInput.value = ''
  aesKey.value = ''
  aesIv.value = ''
  rsaInput.value = ''
  rsaPublicKey.value = ''
  rsaPrivateKey.value = ''
  hashResults.value = {}
  hmacResult.value = ''
  aesResult.value = ''
  aesError.value = false
  aesErrorMessage.value = ''
  rsaResult.value = ''
}

const handlePaste = async () => {
  try {
    const text = await navigator.clipboard.readText()
    if (activeTab.value === 'hash') hashInput.value = text
    else if (activeTab.value === 'hmac') hmacInput.value = text
    else if (activeTab.value === 'aes') aesInput.value = text
    else if (activeTab.value === 'rsa') rsaInput.value = text
  } catch {
    ElMessage.warning('无法读取剪贴板')
  }
}

const handleInsertVariable = (value: string) => {
  if (activeTab.value === 'hash') hashInput.value = value
  else if (activeTab.value === 'hmac') hmacInput.value = value
  else if (activeTab.value === 'aes') aesInput.value = value
  else if (activeTab.value === 'rsa') rsaInput.value = value
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

const handleHash = () => {
  if (!hashInput.value.trim()) {
    ElMessage.warning('请输入明文')
    return
  }
  const results: Record<string, string> = {}
  if (hashAlgorithms.value.includes('MD5')) results.MD5 = crypto.md5(hashInput.value)
  if (hashAlgorithms.value.includes('SHA1')) results.SHA1 = crypto.sha1(hashInput.value)
  if (hashAlgorithms.value.includes('SHA256')) results.SHA256 = crypto.sha256(hashInput.value)
  if (hashAlgorithms.value.includes('SHA512')) results.SHA512 = crypto.sha512(hashInput.value)
  hashResults.value = results
  store.addHistory({
    tool: 'crypto',
    action: 'hash',
    inputPreview: hashInput.value.slice(0, 50),
    outputPreview: results.SHA256?.slice(0, 50) || '',
    inputFull: hashInput.value,
    outputFull: Object.entries(results).map(([k, v]) => `${k}: ${v}`).join('\n'),
  })
  ElMessage.success('计算完成')
}

const handleHmac = () => {
  if (!hmacInput.value.trim() || !hmacKey.value.trim()) {
    ElMessage.warning('请输入明文和密钥')
    return
  }
  hmacResult.value = crypto.hmac(hmacInput.value, hmacKey.value, hmacAlgorithm.value)
  store.addHistory({
    tool: 'crypto',
    action: 'hmac',
    inputPreview: hmacInput.value.slice(0, 50),
    outputPreview: hmacResult.value.slice(0, 50),
    inputFull: hmacInput.value,
    outputFull: hmacResult.value,
  })
  ElMessage.success('计算完成')
}

const handleAesEncrypt = () => {
  if (!aesInput.value.trim() || !aesKey.value.trim()) {
    ElMessage.warning('请输入明文和密钥')
    return
  }
  if (aesMode.value === 'CBC' && !aesIv.value.trim()) {
    ElMessage.warning('CBC 模式需要 IV')
    return
  }
  try {
    aesResult.value = crypto.aesEncrypt(aesInput.value, aesKey.value, aesMode.value, aesIv.value)
    aesError.value = false
    aesErrorMessage.value = ''
    store.addHistory({
      tool: 'crypto',
      action: 'aes-encrypt',
      inputPreview: aesInput.value.slice(0, 50),
      outputPreview: aesResult.value.slice(0, 50),
      inputFull: aesInput.value,
      outputFull: aesResult.value,
    })
    ElMessage.success('加密完成')
  } catch (e: any) {
    aesError.value = true
    aesErrorMessage.value = e.message || '加密失败'
  }
}

const handleAesDecrypt = () => {
  if (!aesInput.value.trim() || !aesKey.value.trim()) {
    ElMessage.warning('请输入密文和密钥')
    return
  }
  if (aesMode.value === 'CBC' && !aesIv.value.trim()) {
    ElMessage.warning('CBC 模式需要 IV')
    return
  }
  aesResult.value = crypto.aesDecrypt(aesInput.value, aesKey.value, aesMode.value, aesIv.value)
  aesError.value = aesResult.value.includes('解密失败')
  aesErrorMessage.value = aesError.value ? aesResult.value : ''
  if (!aesError.value) {
    store.addHistory({
      tool: 'crypto',
      action: 'aes-decrypt',
      inputPreview: aesInput.value.slice(0, 50),
      outputPreview: aesResult.value.slice(0, 50),
      inputFull: aesInput.value,
      outputFull: aesResult.value,
    })
    ElMessage.success('解密完成')
  }
}

const handleRsaEncrypt = () => {
  if (!rsaInput.value.trim() || !rsaPublicKey.value.trim()) {
    ElMessage.warning('请输入明文和公钥')
    return
  }
  rsaResult.value = crypto.rsaEncrypt(rsaInput.value, rsaPublicKey.value)
  store.addHistory({
    tool: 'crypto',
    action: 'rsa-encrypt',
    inputPreview: rsaInput.value.slice(0, 50),
    outputPreview: rsaResult.value.slice(0, 50),
    inputFull: rsaInput.value,
    outputFull: rsaResult.value,
  })
  ElMessage.success('加密完成')
}

const handleRsaDecrypt = () => {
  if (!rsaInput.value.trim() || !rsaPrivateKey.value.trim()) {
    ElMessage.warning('请输入密文和私钥')
    return
  }
  rsaResult.value = crypto.rsaDecrypt(rsaInput.value, rsaPrivateKey.value)
  ElMessage.success('解密完成')
}

// 自动执行：输入变化时自动计算哈希
watch([hashInput, hashAlgorithms], () => {
  if (hashInput.value.trim() && hashAlgorithms.value.length > 0) {
    handleHash()
  }
}, { deep: true })
</script>

<style scoped>
.tool-container {
  height: 100%;
  overflow-y: auto;
  padding: 0;
}

/* 二级 Tab（子功能切换） */
.crypto-tabs {
  margin-bottom: 8px;
  margin-top: -4px;
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  padding: 4px 8px 4px 12px;
}

.crypto-tabs :deep(.el-tabs__header) {
  margin-bottom: 0;
  padding-left: 0;
  border-bottom: none;
}

.crypto-tabs :deep(.el-tabs__nav-wrap) {
  padding-left: 0;
}

.crypto-tabs :deep(.el-tabs__nav-wrap::after) {
  display: none;
}

.crypto-tabs :deep(.el-tabs__item) {
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

.crypto-tabs :deep(.el-tabs__item:hover) {
  color: var(--text-primary);
  background: rgba(0, 212, 255, 0.05);
}

.crypto-tabs :deep(.el-tabs__item.is-active) {
  color: var(--accent-cyan);
  font-weight: 500;
  background: rgba(0, 212, 255, 0.1);
}

.crypto-tabs :deep(.el-tabs__active-bar) {
  display: none;
}

html.light .crypto-tabs {
  background: var(--bg-card);
  border-color: var(--border-color);
}

html.light .crypto-tabs :deep(.el-tabs__item:hover) {
  background: rgba(8, 145, 178, 0.05);
}

html.light .crypto-tabs :deep(.el-tabs__item.is-active) {
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

.action-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
  align-items: center;
}

.action-group {
  display: flex;
  align-items: center;
  gap: 8px;
}

.group-label {
  color: var(--text-secondary);
  font-size: 13px;
  white-space: nowrap;
}

.group-buttons {
  display: flex;
  gap: 6px;
}

.input-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.input-row label {
  font-size: 13px;
  color: var(--text-secondary);
  white-space: nowrap;
  min-width: 60px;
}

.hash-result-item {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 10px;
}

.hash-result-item:last-child {
  margin-bottom: 0;
}

.hash-label {
  font-size: 13px;
  color: var(--text-secondary);
  min-width: 60px;
  font-weight: 500;
}

.hash-value {
  flex: 1;
}

.copy-row {
  margin-top: 8px;
  display: flex;
  justify-content: flex-end;
}

.empty-hint {
  color: var(--text-secondary);
  font-size: 13px;
  text-align: center;
  padding: 16px;
}

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
