<template>
  <div class="tool-container">
    <!-- 操作卡 -->
    <div class="tool-card sticky-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">TOTP 验证器</span>
          <el-tooltip placement="bottom" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>• 扫码导入：上传 / 拖拽 / Ctrl+V 粘贴 otpauth 二维码图片</p>
                <p>• 支持从剪贴板读取图片或 otpauth:// 链接</p>
                <p>• 也可手动添加密钥（Base32）</p>
                <p>• 验证码实时刷新，点击验证码即可复制</p>
                <p>• 密钥明文存储于本地 SQLite，完全离线</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
        <div class="card-actions">
          <el-button size="small" type="primary" :loading="decoding" @click="triggerFileInput">扫码导入</el-button>
          <el-button size="small" @click="importFromClipboard">从剪贴板导入</el-button>
          <el-button size="small" @click="openAddDialog">手动添加</el-button>
        </div>
      </div>
    </div>

    <!-- 密钥列表卡 -->
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">密钥列表</span>
        <div class="card-actions">
          <el-tag v-if="secrets.length" size="small" type="success">{{ secrets.length }} 个密钥</el-tag>
        </div>
      </div>
      <div class="card-body">
        <div v-if="loading" class="empty-tip">加载中...</div>
        <div v-else-if="!secrets.length" class="empty-tip">
          暂无密钥，点击「扫码导入」或「手动添加」开始
        </div>
        <div
          v-else
          class="secret-list"
          @dragover.prevent
          @drop.prevent="handleDrop"
        >
          <div v-for="s in secrets" :key="s.id" class="secret-item">
            <div class="secret-info">
              <div class="secret-name">{{ s.issuer || '未命名' }}</div>
              <div class="secret-account">{{ s.account || '-' }}</div>
            </div>
            <div class="secret-code">
              <span
                class="code-text"
                :class="{ 'code-expiring': (display[s.id]?.remaining ?? 99) <= 5 }"
                @click="copyCode(s)"
              >{{ display[s.id] ? formatCode(display[s.id]!.code) : '------' }}</span>
              <div class="code-meta">
                <span class="remain">{{ display[s.id]?.remaining ?? '-' }}s</span>
                <el-progress
                  class="code-progress"
                  :percentage="progressPercent(s)"
                  :show-text="false"
                  :stroke-width="4"
                  :color="progressColor(s)"
                />
              </div>
            </div>
            <div class="secret-actions">
              <el-button size="small" text @click="copyCode(s)">复制</el-button>
              <el-button size="small" text @click="openEditDialog(s)">编辑</el-button>
              <el-button size="small" text type="danger" @click="handleDelete(s)">删除</el-button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <input
      ref="fileInput"
      type="file"
      accept="image/*"
      style="display: none"
      @change="handleFileChange"
    />

    <!-- 新增/编辑弹窗 -->
    <el-dialog
      v-model="dialogVisible"
      :title="editingId ? '编辑密钥' : '添加密钥'"
      width="440px"
      class="totp-dialog"
    >
      <el-form label-width="72px" label-position="left">
        <el-form-item label="发行方">
          <el-input v-model="form.issuer" placeholder="如 GitHub" />
        </el-form-item>
        <el-form-item label="账号">
          <el-input v-model="form.account" placeholder="如 user@example.com" />
        </el-form-item>
        <el-form-item label="密钥" required>
          <el-input
            v-model="form.secret"
            type="textarea"
            :rows="2"
            placeholder="Base32 密钥，如 JBSWY3DPEHPK3PXP"
          />
        </el-form-item>
        <el-form-item label="算法">
          <el-select v-model="form.algorithm" style="width: 100%">
            <el-option label="SHA1" value="SHA1" />
            <el-option label="SHA256" value="SHA256" />
            <el-option label="SHA512" value="SHA512" />
          </el-select>
        </el-form-item>
        <el-form-item label="位数">
          <el-radio-group v-model="form.digits">
            <el-radio-button :label="6">6 位</el-radio-button>
            <el-radio-button :label="8">8 位</el-radio-button>
          </el-radio-group>
        </el-form-item>
        <el-form-item label="周期">
          <el-input-number v-model="form.period" :min="1" :max="300" :step="1" controls-position="right" />
          <span class="period-unit">秒</span>
        </el-form-item>
        <div v-if="formError" class="form-error">{{ formError }}</div>
        <div v-if="formPreview" class="form-preview">
          预览：<span class="preview-code">{{ formPreview }}</span>
        </div>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="saving" @click="handleSave">保存</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch, onMounted, onBeforeUnmount } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import jsQR from 'jsqr'
import { useConfirmDialog } from '@/composables/useConfirmDialog'
import {
  totp, formatCode, parseOtpauthUri, validateSecret,
  type TotpAlgorithm, type TotpResult,
} from '@/utils/totp'
import {
  listTotpSecrets, saveTotpSecret, deleteTotpSecret,
  type TotpSecret,
} from '@/utils/dbClient'

// ponytail: TOTP 是密钥管理型工具，验证码是每秒派生的临时值，无文本输入区，
// 因此不接入工作流 / 变量池 / 操作历史（会产生每秒一条的噪音）

const { confirm } = useConfirmDialog()

const loading = ref(false)
const saving = ref(false)
const decoding = ref(false)
const secrets = ref<TotpSecret[]>([])

const fileInput = ref<HTMLInputElement | null>(null)
const now = ref(Date.now())
let tickTimer: number | undefined

// ============ 实时刷新 ============
const tick = () => {
  now.value = Date.now()
}

onMounted(() => {
  tick()
  tickTimer = window.setInterval(tick, 1000)
  window.addEventListener('paste', handlePasteEvent)
  loadSecrets()
})

onBeforeUnmount(() => {
  if (tickTimer) window.clearInterval(tickTimer)
  window.removeEventListener('paste', handlePasteEvent)
})

const display = computed(() => {
  const map: Record<string, TotpResult | null> = {}
  for (const s of secrets.value) {
    try {
      map[s.id] = totp(s.secret, {
        algorithm: s.algorithm as TotpAlgorithm,
        digits: s.digits,
        period: s.period,
        timestamp: now.value,
      })
    } catch {
      map[s.id] = null
    }
  }
  return map
})

const progressPercent = (s: TotpSecret) => {
  const d = display.value[s.id]
  if (!d) return 0
  return Math.round((1 - d.progress) * 100)
}

const progressColor = (s: TotpSecret) => {
  const d = display.value[s.id]
  if (d && d.remaining <= 5) return 'var(--color-danger, #f56c6c)'
  return 'var(--accent-cyan)'
}

// ============ 数据加载 ============
const loadSecrets = async () => {
  loading.value = true
  try {
    secrets.value = await listTotpSecrets()
  } catch (e: any) {
    ElMessage.error('加载密钥失败：' + (e?.message || e))
  } finally {
    loading.value = false
  }
}

const copyCode = async (s: TotpSecret) => {
  const d = display.value[s.id]
  if (!d) {
    ElMessage.warning('无法生成验证码，请检查密钥')
    return
  }
  try {
    await navigator.clipboard.writeText(d.code)
    ElMessage.success('验证码已复制')
  } catch {
    ElMessage.error('复制失败')
  }
}

// ============ 新增 / 编辑弹窗 ============
const dialogVisible = ref(false)
const editingId = ref('')
const formError = ref('')
const form = reactive({
  issuer: '',
  account: '',
  secret: '',
  algorithm: 'SHA1' as TotpAlgorithm,
  digits: 6,
  period: 30,
})

const formPreview = computed(() => {
  if (validateSecret(form.secret)) return ''
  try {
    const r = totp(form.secret, { algorithm: form.algorithm, digits: form.digits, period: form.period })
    return formatCode(r.code)
  } catch {
    return ''
  }
})

const resetForm = () => {
  form.issuer = ''
  form.account = ''
  form.secret = ''
  form.algorithm = 'SHA1'
  form.digits = 6
  form.period = 30
  formError.value = ''
}

const openAddDialog = () => {
  editingId.value = ''
  resetForm()
  dialogVisible.value = true
}

const openEditDialog = (s: TotpSecret) => {
  editingId.value = s.id
  form.issuer = s.issuer
  form.account = s.account
  form.secret = s.secret
  form.algorithm = s.algorithm as TotpAlgorithm
  form.digits = s.digits
  form.period = s.period
  formError.value = ''
  dialogVisible.value = true
}

const handleSave = async () => {
  const err = validateSecret(form.secret)
  if (err) {
    formError.value = err
    return
  }
  formError.value = ''
  saving.value = true
  try {
    const iso = new Date().toISOString()
    const existing = secrets.value.find(s => s.id === editingId.value)
    const item: TotpSecret = {
      id: editingId.value || crypto.randomUUID(),
      issuer: form.issuer.trim(),
      account: form.account.trim(),
      secret: form.secret.replace(/[\s-]/g, '').toUpperCase(),
      algorithm: form.algorithm,
      digits: form.digits,
      period: form.period,
      created_at: existing?.created_at || iso,
      updated_at: iso,
    }
    await saveTotpSecret(item)
    ElMessage.success(editingId.value ? '已更新' : '已添加')
    dialogVisible.value = false
    await loadSecrets()
  } catch (e: any) {
    ElMessage.error('保存失败：' + (e?.message || e))
  } finally {
    saving.value = false
  }
}

const handleDelete = async (s: TotpSecret) => {
  const ok = await confirm.ask('删除确认', `确定删除「${s.issuer || s.account || '该密钥'}」吗？`, {
    type: 'danger',
    confirmText: '删除',
  })
  if (!ok) return
  try {
    await deleteTotpSecret(s.id)
    secrets.value = secrets.value.filter(x => x.id !== s.id)
    ElMessage.success('已删除')
  } catch (e: any) {
    ElMessage.error('删除失败：' + (e?.message || e))
  }
}

// ============ 扫码导入 ============
const triggerFileInput = () => {
  fileInput.value?.click()
}

const handleFileChange = (e: Event) => {
  const file = (e.target as HTMLInputElement).files?.[0]
  if (file) processImageFile(file)
  ;(e.target as HTMLInputElement).value = ''
}

const handleDrop = (e: DragEvent) => {
  const file = e.dataTransfer?.files?.[0]
  if (file && file.type.startsWith('image/')) processImageFile(file)
}

const processImageFile = (file: File) => {
  const reader = new FileReader()
  reader.onload = (e) => decodeFromImage(e.target?.result as string)
  reader.readAsDataURL(file)
}

const decodeFromImage = (dataUrl: string) => {
  decoding.value = true
  const img = new Image()
  img.onload = () => {
    decoding.value = false
    const canvas = document.createElement('canvas')
    canvas.width = img.width
    canvas.height = img.height
    const ctx = canvas.getContext('2d')
    if (!ctx) {
      ElMessage.error('无法创建画布上下文')
      return
    }
    ctx.drawImage(img, 0, 0)
    const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height)
    const code = jsQR(imageData.data, imageData.width, imageData.height)
    if (code && code.data) {
      applyScannedText(code.data)
    } else {
      ElMessage.warning('未检测到二维码，请确保图片清晰')
    }
  }
  img.onerror = () => {
    decoding.value = false
    ElMessage.error('图片加载失败')
  }
  img.src = dataUrl
}

const applyScannedText = (text: string) => {
  const trimmed = text.trim()
  if (!/^otpauth:\/\//i.test(trimmed)) {
    ElMessage.warning('二维码内容不是 otpauth 链接')
    return
  }
  try {
    const p = parseOtpauthUri(trimmed)
    if (p.type === 'hotp') {
      ElMessage.warning('暂仅支持 TOTP，该二维码为 HOTP')
      return
    }
    editingId.value = ''
    form.issuer = p.issuer
    form.account = p.account
    form.secret = p.secret
    form.algorithm = p.algorithm
    form.digits = p.digits
    form.period = p.period
    formError.value = ''
    dialogVisible.value = true
    ElMessage.success('识别成功，请确认后保存')
  } catch (e: any) {
    ElMessage.error('解析失败：' + (e?.message || e))
  }
}

const importFromClipboard = async () => {
  try {
    const items = await navigator.clipboard.read()
    for (const item of items) {
      if (item.types.some(t => t.startsWith('image/'))) {
        const imgType = item.types.find(t => t.startsWith('image/'))!
        const blob = await item.getType(imgType)
        const dataUrl = await blobToDataUrl(blob)
        decodeFromImage(dataUrl)
        return
      }
    }
    const text = await navigator.clipboard.readText()
    if (text) {
      applyScannedText(text)
      return
    }
    ElMessage.warning('剪贴板为空或不支持读取')
  } catch (e: any) {
    ElMessage.error('读取剪贴板失败：' + (e?.message || '可能需要授权'))
  }
}

const blobToDataUrl = (blob: Blob): Promise<string> =>
  new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => resolve(reader.result as string)
    reader.onerror = reject
    reader.readAsDataURL(blob)
  })

const handlePasteEvent = (e: ClipboardEvent) => {
  const items = e.clipboardData?.items
  if (!items) return
  for (const item of items) {
    if (item.type.startsWith('image/')) {
      const file = item.getAsFile()
      if (file) {
        e.preventDefault()
        processImageFile(file)
      }
      break
    }
  }
}

watch(dialogVisible, (v) => {
  if (!v) formError.value = ''
})
</script>

<style scoped>
/* 页面特有样式；布局类（.tool-card/.card-header 等）使用 theme.css 全局定义 */
.empty-tip {
  padding: 24px 0;
  text-align: center;
  color: var(--text-secondary);
  font-size: 13px;
}

.secret-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.secret-item {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 12px 16px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 6px;
}

.secret-info {
  flex: 1;
  min-width: 0;
}

.secret-name {
  color: var(--text-primary);
  font-size: 14px;
  font-weight: 600;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.secret-account {
  margin-top: 2px;
  color: var(--text-secondary);
  font-size: 12px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.secret-code {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 4px;
  min-width: 140px;
}

.code-text {
  color: var(--accent-cyan);
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 24px;
  font-weight: 700;
  letter-spacing: 1px;
  cursor: pointer;
  user-select: none;
  transition: color 0.2s;
}

.code-text:hover {
  opacity: 0.8;
}

.code-expiring {
  color: var(--color-danger, #f56c6c);
}

.code-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
}

.remain {
  color: var(--text-secondary);
  font-size: 12px;
  min-width: 28px;
  text-align: right;
}

.code-progress {
  flex: 1;
}

.secret-actions {
  display: flex;
  gap: 2px;
}

.period-unit {
  margin-left: 8px;
  color: var(--text-secondary);
  font-size: 13px;
}

.form-error {
  margin: 0 0 8px 72px;
  color: var(--color-danger, #f56c6c);
  font-size: 13px;
}

.form-preview {
  margin-left: 72px;
  color: var(--text-secondary);
  font-size: 13px;
}

.preview-code {
  color: var(--accent-cyan);
  font-family: 'JetBrains Mono', 'Consolas', monospace;
  font-size: 15px;
  font-weight: 600;
  letter-spacing: 1px;
}
</style>
