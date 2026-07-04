<template>
  <div class="duplicates-tab">
    <div class="tab-toolbar">
      <span class="muted">
        共 {{ total }} 组重复文件 |
        可回收 {{ formatBytes(totalWasted) }}
      </span>
      <el-button
        size="small"
        type="danger"
        :disabled="allSelected.length === 0"
        @click="confirmDeleteAll"
      >
        删除全部勾选 ({{ allSelected.length }}, {{ formatBytes(allSelectedSize) }})
      </el-button>
    </div>

    <el-table
      :data="groups"
      v-loading="loading"
      border
      size="small"
      row-key="groupId"
      style="width: 100%"
    >
      <el-table-column type="expand">
        <template #default="{ row }">
          <div class="group-files">
            <div v-for="file in row.files" :key="file.path" class="group-file-row">
              <el-checkbox
                :model-value="isChecked(row.groupId, file.path)"
                @change="(val: boolean | string | number) => toggleCheck(row.groupId, file.path, !!val)"
              />
              <span class="file-path" :title="file.path">{{ file.path }}</span>
              <el-button size="small" link @click="locate(file.path)">定位</el-button>
            </div>
          </div>
        </template>
      </el-table-column>
      <el-table-column label="组号" width="80">
        <template #default="{ row }">#{{ row.groupId }}</template>
      </el-table-column>
      <el-table-column prop="fileCount" label="文件数" width="100" />
      <el-table-column label="单个大小" width="120">
        <template #default="{ row }">{{ formatBytes(row.fileSize) }}</template>
      </el-table-column>
      <el-table-column label="浪费空间" width="140">
        <template #default="{ row }">
          <span class="wasted">{{ formatBytes(row.wastedBytes) }}</span>
        </template>
      </el-table-column>
      <el-table-column label="本组勾选" width="120">
        <template #default="{ row }">{{ countCheckedInGroup(row.groupId) }} / {{ row.fileCount }}</template>
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
import { diskGetDuplicates, diskDeleteFiles, diskLocateInExplorer } from '@/utils/diskAnalyzerClient'
import type { DuplicateGroup } from '@/utils/diskAnalyzerTypes'

const props = defineProps<{ scanId: string }>()

const groups = ref<DuplicateGroup[]>([])
const loading = ref(false)
const total = ref(0)
const offset = ref(0)
const pageSize = 50
const hasMore = ref(false)

// 选中状态：Map<groupId, Set<filePath>>
const selectedMap = ref<Map<number, Set<string>>>(new Map())

const allSelected = computed(() => {
  const arr: { groupId: number; path: string; size: number }[] = []
  for (const [gid, paths] of selectedMap.value) {
    const g = groups.value.find((x) => x.groupId === gid)
    if (!g) continue
    for (const p of paths) {
      arr.push({ groupId: gid, path: p, size: g.fileSize })
    }
  }
  return arr
})

const allSelectedSize = computed(() =>
  allSelected.value.reduce((s, x) => s + x.size, 0)
)

const totalWasted = computed(() =>
  groups.value.reduce((s, g) => s + g.wastedBytes, 0)
)

const isChecked = (gid: number, path: string) => {
  return selectedMap.value.get(gid)?.has(path) ?? false
}

const toggleCheck = (gid: number, path: string, val: boolean) => {
  if (!selectedMap.value.has(gid)) {
    selectedMap.value.set(gid, new Set())
  }
  const set = selectedMap.value.get(gid)!
  if (val) set.add(path)
  else set.delete(path)
  if (set.size === 0) selectedMap.value.delete(gid)
  // 触发响应式
  selectedMap.value = new Map(selectedMap.value)
}

const countCheckedInGroup = (gid: number) => {
  return selectedMap.value.get(gid)?.size ?? 0
}

const fetchGroups = async (reset = false) => {
  if (reset) {
    offset.value = 0
    groups.value = []
    selectedMap.value.clear()
  }
  loading.value = true
  try {
    const page = await diskGetDuplicates(props.scanId, pageSize, offset.value)
    if (reset) {
      groups.value = page.items
    } else {
      groups.value.push(...page.items)
    }
    total.value = page.total
    hasMore.value = groups.value.length < total.value
    offset.value += page.items.length
  } catch (e) {
    ElMessage.error(`加载重复文件失败: ${e}`)
  } finally {
    loading.value = false
  }
}

const confirmDeleteAll = async () => {
  const paths = allSelected.value.map((x) => x.path)
  if (paths.length === 0) return
  const totalSize = formatBytes(allSelectedSize.value)
  try {
    await ElMessageBox.confirm(
      `确认将 ${paths.length} 个文件（共 ${totalSize}）送入回收站？\n\n注意：每组至少保留 1 个文件，否则数据丢失不可恢复。`,
      '删除确认',
      { type: 'warning', confirmButtonText: '删除', cancelButtonText: '取消' }
    )
  } catch {
    return
  }
  try {
    const result = await diskDeleteFiles(paths)
    if (result.succeeded.length > 0) {
      ElMessage.success(`已删除 ${result.succeeded.length} 个文件`)
      // 从展开组中移除已删除的
      const succSet = new Set(result.succeeded)
      for (const g of groups.value) {
        g.files = g.files.filter((f) => !succSet.has(f.path))
        g.fileCount = g.files.length as number
        g.wastedBytes = g.fileSize * Math.max(0, g.fileCount - 1)
      }
      // 移除空组
      groups.value = groups.value.filter((g) => g.fileCount >= 1)
      selectedMap.value.clear()
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

const loadMore = () => fetchGroups(false)

onMounted(() => fetchGroups(true))
</script>

<style scoped>
.duplicates-tab { padding: 8px 0; }
.tab-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}
.muted { font-size: 13px; color: var(--text-secondary, #888); }
.wasted { color: var(--warning-color, #e6a23c); font-weight: 600; }
.group-files { padding: 8px 16px; background: var(--bg-alt, #1a2332); }
.group-file-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 0;
}
.file-path {
  flex: 1;
  font-family: monospace;
  font-size: 12px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.load-more { text-align: center; margin-top: 12px; }
</style>
