<template>
  <div class="folder-tab">
    <!-- 面包屑 -->
    <div class="breadcrumb">
      <el-breadcrumb separator="/">
        <el-breadcrumb-item
          v-for="(crumb, idx) in breadcrumbs"
          :key="idx"
          @click="navigateTo(crumb.path)"
        >
          <span class="crumb-link">{{ crumb.name }}</span>
        </el-breadcrumb-item>
      </el-breadcrumb>
      <el-button size="small" :disabled="!currentParent" @click="goUp">返回上级</el-button>
    </div>

    <el-table :data="folders" v-loading="loading" border size="small" style="width: 100%">
      <el-table-column prop="name" label="名称" min-width="200" />
      <el-table-column label="大小" width="220">
        <template #default="{ row }">
          <div class="size-cell">
            <span>{{ formatBytes(row.sizeBytes) }}</span>
            <el-progress
              :percentage="Math.min(100, row.percentOfRoot)"
              :stroke-width="6"
              :show-text="false"
              style="flex: 1; margin-left: 8px"
            />
          </div>
        </template>
      </el-table-column>
      <el-table-column prop="fileCount" label="文件数" width="100" />
      <el-table-column label="占比" width="80">
        <template #default="{ row }">{{ row.percentOfRoot.toFixed(1) }}%</template>
      </el-table-column>
      <el-table-column label="操作" width="160">
        <template #default="{ row }">
          <el-button size="small" link @click="drillDown(row)">下钻</el-button>
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
import { ref, watch, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { formatBytes } from '@/utils/systemInfoClient'
import { diskGetFolders, diskLocateInExplorer } from '@/utils/diskAnalyzerClient'
import type { FolderInfo } from '@/utils/diskAnalyzerTypes'

const props = defineProps<{ scanId: string; rootPath: string }>()

const folders = ref<FolderInfo[]>([])
const loading = ref(false)
const currentParent = ref<string | null>(null) // null = root
const total = ref(0)
const offset = ref(0)
const pageSize = 100
const hasMore = ref(false)

const breadcrumbs = ref<{ name: string; path: string }[]>([])

const fetchFolders = async (reset = false) => {
  if (reset) {
    offset.value = 0
    folders.value = []
  }
  loading.value = true
  try {
    const page = await diskGetFolders(props.scanId, currentParent.value, pageSize, offset.value)
    if (reset) {
      folders.value = page.items
    } else {
      folders.value.push(...page.items)
    }
    total.value = page.total
    hasMore.value = folders.value.length < total.value
    offset.value += page.items.length
  } catch (e) {
    ElMessage.error(`加载文件夹失败: ${e}`)
  } finally {
    loading.value = false
  }
}

const drillDown = (folder: FolderInfo) => {
  currentParent.value = folder.path
  updateBreadcrumbs(folder.path)
  fetchFolders(true)
}

const goUp = () => {
  if (!currentParent.value) return
  // 找当前父的父
  const pb = currentParent.value
  // 简化：用 props.rootPath 作上限
  if (pb === props.rootPath) {
    currentParent.value = null
  } else {
    // 取最后一段的上级
    const parts = pb.split(/[/\\]/).filter(Boolean)
    if (parts.length <= 1) {
      currentParent.value = null
    } else {
      parts.pop()
      // 重构路径（保留盘符冒号）
      let parent = parts.join('\\')
      if (pb.startsWith('\\\\')) parent = '\\\\' + parent // UNC 路径
      else if (/^[A-Za-z]:/.test(pb)) parent = parts[0] + '\\' + parts.slice(1).join('\\')
      currentParent.value = parent === props.rootPath ? null : parent
    }
  }
  updateBreadcrumbs(currentParent.value || props.rootPath)
  fetchFolders(true)
}

const navigateTo = (path: string) => {
  currentParent.value = path === props.rootPath ? null : path
  updateBreadcrumbs(path)
  fetchFolders(true)
}

const updateBreadcrumbs = (path: string) => {
  const parts = path.split(/[/\\]/).filter(Boolean)
  const crumbs: { name: string; path: string }[] = []
  let acc = ''
  for (const p of parts) {
    acc = acc ? acc + '\\' + p : (path.startsWith('\\\\') ? '\\\\' + p : /^[A-Za-z]:/.test(path) ? p : p)
    crumbs.push({ name: p, path: acc })
  }
  breadcrumbs.value = crumbs
}

const loadMore = () => fetchFolders(false)

const locate = async (path: string) => {
  try {
    await diskLocateInExplorer(path)
  } catch (e) {
    ElMessage.error(`定位失败: ${e}`)
  }
}

onMounted(() => {
  updateBreadcrumbs(props.rootPath)
  fetchFolders(true)
})

// 当 scanId 变化时重新加载
watch(() => props.scanId, () => {
  currentParent.value = null
  updateBreadcrumbs(props.rootPath)
  fetchFolders(true)
})
</script>

<style scoped>
.folder-tab { padding: 8px 0; }
.breadcrumb {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}
.crumb-link { cursor: pointer; color: var(--primary-color, #00d4ff); }
.size-cell { display: flex; align-items: center; }
.load-more { text-align: center; margin-top: 12px; }
</style>
