<template>
  <div class="tool-container">
    <!-- Tab 栏 -->
    <div class="tool-card sticky-card cert-tabs">
      <div class="card-header">
        <span class="card-title">证书查看器</span>
        <div class="card-actions">
          <el-button
            v-if="activeTab === 'store'"
            size="small"
            :loading="loading"
            @click="fetchStore"
          >
            刷新
          </el-button>
          <el-button
            v-if="activeTab === 'file'"
            size="small"
            @click="selectFile"
          >
            📂 选择文件
          </el-button>
        </div>
      </div>
      <div class="card-body cert-tab-bar">
        <span
          class="cert-tab"
          :class="{ active: activeTab === 'store' }"
          @click="activeTab = 'store'"
        >
          证书存储
        </span>
        <span
          class="cert-tab"
          :class="{ active: activeTab === 'file' }"
          @click="activeTab = 'file'"
        >
          文件解析
        </span>
      </div>
    </div>

    <!-- 错误提示 -->
    <div v-if="error" class="error-message" style="margin-bottom: 12px;">
      {{ error }}
    </div>

    <!-- 证书存储 Tab -->
    <div v-if="activeTab === 'store'" class="cert-layout">
      <!-- 左侧列表 -->
      <div class="cert-left">
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">证书列表</span>
            <span class="card-sub" v-if="storeData">
              {{ currentStoreCerts.length }} 个证书
            </span>
          </div>
          <div class="card-body" style="padding: 8px 12px;">
            <!-- 子 Tab -->
            <div class="sub-tabs">
              <span
                v-for="st in storeTabs"
                :key="st.key"
                class="sub-tab"
                :class="{ active: currentStore === st.key }"
                @click="currentStore = st.key"
              >
                {{ st.label }}
              </span>
            </div>
            <!-- 搜索 -->
            <el-input
              v-model="searchQuery"
              size="small"
              placeholder="搜索主题/颁发者..."
              clearable
              style="margin-bottom: 8px;"
            />
            <!-- 加载中 -->
            <div v-if="loading" class="loading-tip">加载中...</div>
            <!-- 空 -->
            <div v-else-if="filteredCerts.length === 0" class="empty-tip">
              {{ storeData ? '没有匹配的证书' : '点击刷新加载证书' }}
            </div>
            <!-- 证书列表 -->
            <div v-else class="cert-list">
              <div
                v-for="cert in filteredCerts"
                :key="cert.thumbprint"
                class="cert-item"
                :class="{ selected: selectedCert?.thumbprint === cert.thumbprint }"
                @click="selectCert(cert)"
              >
                <div class="cert-item-subject">
                  <span class="expiry-dot" :class="cert.is_expired ? 'expired' : (certDaysLeft(cert) <= 30 ? 'soon' : 'valid')"></span>
                  {{ extractCN(cert.subject) }}
                </div>
                <div class="cert-item-issuer">{{ extractCN(cert.issuer) }}</div>
                <div class="cert-item-date">
                  {{ cert.not_after }}
                  <span v-if="cert.is_expired" class="tag-expired">已过期</span>
                  <span v-else-if="certDaysLeft(cert) <= 30" class="tag-soon">{{ certDaysLeft(cert) }}天后过期</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 右侧详情 -->
      <div class="cert-right">
        <div v-if="detailLoading" class="tool-card">
          <div class="card-body">
            <div class="loading-tip">加载详情中...</div>
          </div>
        </div>
        <div v-else-if="certDetail" class="tool-card">
          <div class="card-header">
            <span class="card-title">证书详情</span>
            <div class="card-actions">
              <el-button size="small" @click="copyPem">复制 PEM</el-button>
              <el-button size="small" @click="exportCert">导出证书</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-descriptions :column="2" border size="small">
              <el-descriptions-item label="主题" :span="2">
                <span class="mono-text">{{ certDetail.subject }}</span>
              </el-descriptions-item>
              <el-descriptions-item label="颁发者" :span="2">
                <span class="mono-text">{{ certDetail.issuer }}</span>
              </el-descriptions-item>
              <el-descriptions-item label="版本">{{ certDetail.version }}</el-descriptions-item>
              <el-descriptions-item label="序列号">
                <span class="mono-text">{{ certDetail.serial_number }}</span>
              </el-descriptions-item>
              <el-descriptions-item label="生效日期">{{ certDetail.not_before }}</el-descriptions-item>
              <el-descriptions-item label="到期日期">
                {{ certDetail.not_after }}
                <span v-if="certDetail.is_expired" class="tag-expired">已过期 {{ Math.abs(certDetail.days_until_expiry) }} 天</span>
                <span v-else-if="certDetail.days_until_expiry <= 30" class="tag-soon">还有 {{ certDetail.days_until_expiry }} 天</span>
                <span v-else class="tag-valid">还有 {{ certDetail.days_until_expiry }} 天</span>
              </el-descriptions-item>
              <el-descriptions-item label="SHA1 指纹" :span="2">
                <span class="mono-text">{{ certDetail.thumbprint }}</span>
              </el-descriptions-item>
              <el-descriptions-item label="SHA256 指纹" :span="2">
                <span class="mono-text">{{ certDetail.thumbprint_sha256 }}</span>
              </el-descriptions-item>
              <el-descriptions-item label="签名算法">{{ certDetail.signature_algorithm }}</el-descriptions-item>
              <el-descriptions-item label="公钥">{{ certDetail.public_key }}</el-descriptions-item>
              <el-descriptions-item v-if="certDetail.san.length > 0" label="主题备用名称" :span="2">
                <div v-for="s in certDetail.san" :key="s" class="tag-item">{{ s }}</div>
              </el-descriptions-item>
              <el-descriptions-item v-if="certDetail.key_usage.length > 0" label="密钥用法" :span="2">
                <div v-for="k in certDetail.key_usage" :key="k" class="tag-item">{{ k }}</div>
              </el-descriptions-item>
              <el-descriptions-item v-if="certDetail.enhanced_key_usage.length > 0" label="增强密钥用法" :span="2">
                <div v-for="k in certDetail.enhanced_key_usage" :key="k" class="tag-item">{{ k }}</div>
              </el-descriptions-item>
              <el-descriptions-item label="基本约束">{{ certDetail.basic_constraints }}</el-descriptions-item>
            </el-descriptions>
          </div>
        </div>
        <div v-else class="tool-card">
          <div class="card-body">
            <div class="empty-tip">点击左侧证书查看详情</div>
          </div>
        </div>
      </div>
    </div>

    <!-- 文件解析 Tab -->
    <div v-if="activeTab === 'file'" class="cert-layout">
      <div class="cert-left">
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">解析证书文件</span>
          </div>
          <div class="card-body">
            <div class="file-upload-area" @click="selectFile">
              <span class="upload-icon">📄</span>
              <span class="upload-text">点击选择 .cer / .crt / .pfx 文件</span>
              <span class="upload-hint">或拖拽文件到此处（暂不支持拖拽）</span>
            </div>
            <div v-if="filePath" class="file-info">
              <span class="file-name">{{ filePath }}</span>
              <el-button size="small" type="danger" text @click="clearFile">清除</el-button>
            </div>
            <div v-if="isPfxFile" style="margin-top: 8px;">
              <el-input
                v-model="pfxPassword"
                type="password"
                size="small"
                placeholder="PFX 密码（无密码则留空）"
                show-password
              />
            </div>
            <el-button
              v-if="filePath"
              type="primary"
              size="small"
              :loading="loading"
              style="margin-top: 8px; width: 100%;"
              @click="parseFile"
            >
              解析证书
            </el-button>
          </div>
        </div>
      </div>

      <div class="cert-right">
        <div v-if="certDetail" class="tool-card">
          <div class="card-header">
            <span class="card-title">证书详情</span>
            <div class="card-actions">
              <el-button size="small" @click="copyPem">复制 PEM</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-descriptions :column="2" border size="small">
              <el-descriptions-item label="主题" :span="2">
                <span class="mono-text">{{ certDetail.subject }}</span>
              </el-descriptions-item>
              <el-descriptions-item label="颁发者" :span="2">
                <span class="mono-text">{{ certDetail.issuer }}</span>
              </el-descriptions-item>
              <el-descriptions-item label="版本">{{ certDetail.version }}</el-descriptions-item>
              <el-descriptions-item label="序列号">
                <span class="mono-text">{{ certDetail.serial_number }}</span>
              </el-descriptions-item>
              <el-descriptions-item label="生效日期">{{ certDetail.not_before }}</el-descriptions-item>
              <el-descriptions-item label="到期日期">
                {{ certDetail.not_after }}
                <span v-if="certDetail.is_expired" class="tag-expired">已过期 {{ Math.abs(certDetail.days_until_expiry) }} 天</span>
                <span v-else-if="certDetail.days_until_expiry <= 30" class="tag-soon">还有 {{ certDetail.days_until_expiry }} 天</span>
                <span v-else class="tag-valid">还有 {{ certDetail.days_until_expiry }} 天</span>
              </el-descriptions-item>
              <el-descriptions-item label="SHA1 指纹" :span="2">
                <span class="mono-text">{{ certDetail.thumbprint }}</span>
              </el-descriptions-item>
              <el-descriptions-item label="SHA256 指纹" :span="2">
                <span class="mono-text">{{ certDetail.thumbprint_sha256 }}</span>
              </el-descriptions-item>
              <el-descriptions-item label="签名算法">{{ certDetail.signature_algorithm }}</el-descriptions-item>
              <el-descriptions-item label="公钥">{{ certDetail.public_key }}</el-descriptions-item>
              <el-descriptions-item v-if="certDetail.san.length > 0" label="主题备用名称" :span="2">
                <div v-for="s in certDetail.san" :key="s" class="tag-item">{{ s }}</div>
              </el-descriptions-item>
              <el-descriptions-item v-if="certDetail.key_usage.length > 0" label="密钥用法" :span="2">
                <div v-for="k in certDetail.key_usage" :key="k" class="tag-item">{{ k }}</div>
              </el-descriptions-item>
              <el-descriptions-item v-if="certDetail.enhanced_key_usage.length > 0" label="增强密钥用法" :span="2">
                <div v-for="k in certDetail.enhanced_key_usage" :key="k" class="tag-item">{{ k }}</div>
              </el-descriptions-item>
              <el-descriptions-item label="基本约束">{{ certDetail.basic_constraints }}</el-descriptions-item>
            </el-descriptions>
          </div>
        </div>
        <div v-else class="tool-card">
          <div class="card-body">
            <div class="empty-tip">选择证书文件并点击解析</div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { readCertStore, getCertDetail, parseCertFile } from '../utils/systemInfoClient'
import type { CertStoreList, CertInfo, CertDetail } from '../utils/systemInfoClient'
import { useToolboxStore } from '../store'
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'

const store = useToolboxStore()

// ============ 状态 ============

const activeTab = ref<'store' | 'file'>('store')
const loading = ref(false)
const error = ref('')

// 证书存储
const storeData = ref<CertStoreList | null>(null)
const currentStore = ref<'personal' | 'root' | 'ca'>('personal')
const searchQuery = ref('')
const selectedCert = ref<CertInfo | null>(null)
const certDetail = ref<CertDetail | null>(null)
const detailLoading = ref(false)

const storeTabs = [
  { key: 'personal' as const, label: '个人' },
  { key: 'root' as const, label: '受信任的根' },
  { key: 'ca' as const, label: '中间 CA' },
]

// 文件解析
const filePath = ref('')
const pfxPassword = ref('')

// ============ 计算属性 ============

const currentStoreCerts = computed(() => {
  if (!storeData.value) return []
  switch (currentStore.value) {
    case 'personal': return storeData.value.personal
    case 'root': return storeData.value.root
    case 'ca': return storeData.value.ca
  }
})

const filteredCerts = computed(() => {
  if (!searchQuery.value.trim()) return currentStoreCerts.value
  const q = searchQuery.value.toLowerCase()
  return currentStoreCerts.value.filter(
    c => c.subject.toLowerCase().includes(q) || c.issuer.toLowerCase().includes(q)
  )
})

const isPfxFile = computed(() => {
  return filePath.value.toLowerCase().endsWith('.pfx')
})

// ============ 方法 ============

function extractCN(dn: string): string {
  const m = dn.match(/CN=([^,]+)/)
  return m ? m[1].trim() : dn
}

function certDaysLeft(cert: CertInfo): number {
  const d = new Date(cert.not_after)
  return Math.floor((d.getTime() - Date.now()) / (1000 * 60 * 60 * 24))
}

async function fetchStore() {
  loading.value = true
  error.value = ''
  try {
    storeData.value = await readCertStore()
    selectedCert.value = null
    certDetail.value = null
  } catch (e) {
    error.value = String(e)
    ElMessage.error('读取证书存储失败: ' + String(e))
  } finally {
    loading.value = false
  }
}

async function selectCert(cert: CertInfo) {
  selectedCert.value = cert
  detailLoading.value = true
  error.value = ''
  try {
    certDetail.value = await getCertDetail(cert.thumbprint, cert.store_name)
  } catch (e) {
    error.value = String(e)
    ElMessage.error('获取证书详情失败: ' + String(e))
    certDetail.value = null
  } finally {
    detailLoading.value = false
  }
}

async function selectFile() {
  try {
    const result = await open({
      filters: [
        {
          name: '证书文件',
          extensions: ['cer', 'crt', 'pfx', 'pem', 'der', 'p7b', 'p12'],
        },
      ],
      multiple: false,
    })
    if (result) {
      filePath.value = result as string
      pfxPassword.value = ''
      certDetail.value = null
    }
  } catch (e) {
    ElMessage.error('选择文件失败: ' + String(e))
  }
}

function clearFile() {
  filePath.value = ''
  pfxPassword.value = ''
  certDetail.value = null
}

async function parseFile() {
  if (!filePath.value) return
  loading.value = true
  error.value = ''
  try {
    const password = pfxPassword.value || undefined
    certDetail.value = await parseCertFile(filePath.value, password)
    store.addHistory({
      tool: 'certViewer',
      action: '文件解析',
      inputPreview: filePath.value.slice(-50),
      outputPreview: certDetail.value.subject.slice(0, 50),
      inputFull: filePath.value,
      outputFull: JSON.stringify(certDetail.value),
    })
  } catch (e) {
    error.value = String(e)
    ElMessage.error('解析证书失败: ' + String(e))
    certDetail.value = null
  } finally {
    loading.value = false
  }
}

async function copyPem() {
  if (!certDetail.value?.raw_pem) return
  try {
    await navigator.clipboard.writeText(certDetail.value.raw_pem)
    ElMessage.success('PEM 已复制到剪贴板')
  } catch (e) {
    ElMessage.error('复制失败: ' + String(e))
  }
}

async function exportCert() {
  if (!certDetail.value?.raw_pem) return
  try {
    const filename = `certificate_${new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19)}.cer`
    const savedPath = await invoke<string>('save_text_with_dialog', { content: certDetail.value.raw_pem, filename })
    if (savedPath) {
      ElMessage.success(`已导出到: ${savedPath}`)
    }
  } catch (e) {
    ElMessage.error('导出失败: ' + String(e))
  }
}

// ============ 生命周期 ============

onMounted(() => {
  fetchStore()
})
</script>

<style scoped>
.cert-layout {
  display: flex;
  gap: 12px;
  align-items: flex-start;
}

.cert-left {
  flex: 0 0 40%;
  min-width: 0;
}

.cert-right {
  flex: 1;
  min-width: 0;
}

/* Tab 栏 */
.cert-tab-bar {
  display: flex;
  gap: 0;
  padding: 0 10px;
}

.cert-tab {
  padding: 6px 16px;
  font-size: 13px;
  color: var(--text-secondary);
  cursor: pointer;
  border-bottom: 2px solid transparent;
  transition: color 0.2s, border-color 0.2s;
}

.cert-tab:hover {
  color: var(--accent-color);
}

.cert-tab.active {
  color: var(--accent-color);
  border-bottom-color: var(--accent-color);
}

/* 子 Tab */
.sub-tabs {
  display: flex;
  gap: 4px;
  margin-bottom: 8px;
}

.sub-tab {
  padding: 4px 10px;
  font-size: 12px;
  color: var(--text-secondary);
  cursor: pointer;
  border-radius: 4px;
  border: 1px solid transparent;
  transition: all 0.2s;
}

.sub-tab:hover {
  border-color: var(--border-color);
  color: var(--text-primary);
}

.sub-tab.active {
  color: var(--accent-color);
  border-color: var(--accent-color);
  background: rgba(0, 212, 255, 0.06);
}

/* 证书列表 */
.cert-list {
  max-height: 500px;
  overflow-y: auto;
}

.cert-item {
  padding: 8px 10px;
  border-radius: 6px;
  cursor: pointer;
  border: 1px solid transparent;
  transition: all 0.15s;
  margin-bottom: 4px;
}

.cert-item:hover {
  background: var(--bg-input);
  border-color: var(--border-color);
}

.cert-item.selected {
  border-color: var(--accent-color);
  background: rgba(0, 212, 255, 0.05);
}

.cert-item-subject {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  display: flex;
  align-items: center;
  gap: 6px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.cert-item-issuer {
  font-size: 11px;
  color: var(--text-secondary);
  margin-top: 2px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.cert-item-date {
  font-size: 11px;
  color: var(--text-secondary);
  margin-top: 2px;
  display: flex;
  align-items: center;
  gap: 6px;
}

/* 过期状态指示 */
.expiry-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.expiry-dot.valid {
  background: var(--accent-green);
}

.expiry-dot.soon {
  background: var(--accent-orange);
}

.expiry-dot.expired {
  background: var(--accent-red);
}

.tag-expired {
  font-size: 11px;
  color: var(--accent-red);
  background: rgba(239, 68, 68, 0.1);
  padding: 1px 6px;
  border-radius: 3px;
}

.tag-soon {
  font-size: 11px;
  color: var(--accent-orange);
  background: rgba(245, 158, 11, 0.1);
  padding: 1px 6px;
  border-radius: 3px;
}

.tag-valid {
  font-size: 11px;
  color: var(--accent-green);
  background: rgba(16, 185, 129, 0.1);
  padding: 1px 6px;
  border-radius: 3px;
}

.tag-item {
  display: inline-block;
  font-size: 12px;
  color: var(--accent-cyan);
  background: rgba(0, 212, 255, 0.06);
  padding: 2px 8px;
  border-radius: 4px;
  margin: 2px 4px 2px 0;
}

/* 详情 */
.mono-text {
  font-family: 'Consolas', 'Courier New', monospace;
  font-size: 12px;
  word-break: break-all;
}

/* 文件上传 */
.file-upload-area {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 32px 16px;
  border: 1px dashed var(--border-color);
  border-radius: 8px;
  cursor: pointer;
  transition: border-color 0.2s;
}

.file-upload-area:hover {
  border-color: var(--accent-color);
}

.upload-icon {
  font-size: 32px;
}

.upload-text {
  font-size: 14px;
  color: var(--text-primary);
}

.upload-hint {
  font-size: 12px;
  color: var(--text-secondary);
}

.file-info {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: 8px;
  padding: 6px 10px;
  background: var(--bg-input);
  border-radius: 4px;
}

.file-name {
  font-size: 12px;
  color: var(--text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* 通用 */
.loading-tip,
.empty-tip {
  text-align: center;
  padding: 24px 0;
  font-size: 13px;
  color: var(--text-secondary);
}
</style>