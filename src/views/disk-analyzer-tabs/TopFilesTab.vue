<template>
  <div class="top-files-tab">
    <div class="tab-toolbar">
      <span class="muted">共 {{ total }} 个文件（仅展示 Top {{ pageSize }}）</span>
      <el-button
        size="small"
        type="danger"
        :disabled="selected.length === 0"
        @click="confirmDelete"
      >
        删除勾选 ({{ selected.length }}, {{ formatBytes(selectedSize) }})
      </el-button>
    </div>

    <el-table
      :data="files"
      v-loading="loading"
      border
      size="small"
      @selection-change="onSelectionChange"
      style="width: 100%"
    >
      <el-table-column type="selection" width="40" />
      <el-table-column prop="path" label="路径" min-width="300" show-overflow-tooltip />
      <el-table-column label="大小" width="120">
        <template #default="{ row }">{{ formatBytes(row.sizeBytes) }}</template>
      </el-table-column>
      <el-table-column label="修改时间" width="170">
        <template #default="{ row }">{{ formatTime(row.modifiedMs) }}</template>
      </el-table-column>
      <el-table-column label="操作" width="120">
        <template #default="{ row }">
          <el-button size="small" link @click="locate(row.path)">定位</el-button>
        </template>
      </el-table-column>
    </el-table>

    <div v-if="hasMore" class="load-more">
      <el-button size="small" @click="loadMore" :loading="loading">加载更多</el-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { formatBytes } from '@/utils/systemInfoClient'
import { diskGetTopFiles, diskDeleteFiles, diskLocateInExplorer } from '@/utils/diskAnalyzerClient'
import type { FileInfo } from '@/utils/diskAnalyzerTypes'

const props = defineProps<{ scanId: string }>()

const files = ref<FileInfo[]>([])
const loading = ref(false)
const total = ref(0)
const offset = ref(0)
const pageSize = 100
const hasMore = ref(false)
const selected = ref<FileInfo[]>([])

const selectedSize = computed(() =>
  selected.value.reduce((sum, f) => sum + f.sizeBytes, 0)
)

const formatTime = (ms: number) => {
  if (!ms) return '-'
  return new Date(ms).toLocaleString('zh-CN')
}

const fetchFiles = async (reset = false) => {
  if (reset) {
    offset.value = 0
    files.value = []
  }
  loading.value = true
  try {
    const page = await diskGetTopFiles(props.scanId, pageSize, offset.value)
    if (reset) {
      files.value = page.items
    } else {
      files.value.push(...page.items)
    }
    total.value = page.total
    hasMore.value = files.value.length < total.value
    offset.value += page.items.length
  } catch (e) {
    ElMessage.error(`加载文件列表失败: ${e}`)
  } finally {
    loading.value = false
  }
}

const onSelectionChange = (rows: FileInfo[]) => {
  selected.value = rows
}

const confirmDelete = async () => {
  if (selected.value.length === 0) return
  const paths = selected.value.map((f) => f.path)
  const totalSize = formatBytes(selectedSize.value)
  try {
    await ElMessageBox.confirm(
      `确认将 ${paths.length} 个文件（共 ${totalSize}）送入回收站？`,
      '删除确认',
      { type: 'warning', confirmButtonText: '删除', cancelButtonText: '取消' }
    )
  } catch {
    return // 用户取消
  }
  try {
    const result = await diskDeleteFiles(paths)
    if (result.succeeded.length > 0) {
      ElMessage.success(`已删除 ${result.succeeded.length} 个文件`)
      // 从列表中移除已删除的
      const succSet = new Set(result.succeeded)
      files.value = files.value.filter((f) => !succSet.has(f.path))
    }
    if (result.failed.length > 0) {
      ElMessage.warning(`${result.failed.length} 个文件删除失败`)
      console.error('删除失败详情:', result.failed)
    }
  } catch (e) {
    ElMessage.error(`删除失败: ${e}`)
  }
}

const locate = async (path: string) => {
  try {
    await diskLocateInExplorer(path)
  } catch (e) {
    ElMessage.error(`定位失败: ${e}`)
  }
}

const loadMore = () => fetchFiles(false)

onMounted(() => fetchFiles(true))
</script>

<style scoped>
.top-files-tab { padding: 8px 0; }
.tab-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}
.muted { font-size: 13px; color: var(--text-secondary, #888); }
.load-more { text-align: center; margin-top: 12px; }
</style>
