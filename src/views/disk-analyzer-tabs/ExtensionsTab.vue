<template>
  <div class="extensions-tab">
    <div class="tab-toolbar">
      <span class="muted">共 {{ total }} 种扩展名</span>
    </div>

    <el-table :data="exts" v-loading="loading" border size="small" style="width: 100%">
      <el-table-column label="扩展名" width="140">
        <template #default="{ row }">
          <span class="ext-tag">{{ row.extension || '(无扩展名)' }}</span>
        </template>
      </el-table-column>
      <el-table-column prop="fileCount" label="文件数" width="120" />
      <el-table-column label="总大小" width="240">
        <template #default="{ row }">
          <div class="size-cell">
            <span>{{ formatBytes(row.totalSize) }}</span>
            <el-progress
              :percentage="Math.min(100, row.percent)"
              :stroke-width="6"
              :show-text="false"
              style="flex: 1; margin-left: 8px"
            />
          </div>
        </template>
      </el-table-column>
      <el-table-column label="占比" width="80">
        <template #default="{ row }">{{ row.percent.toFixed(1) }}%</template>
      </el-table-column>
    </el-table>

    <div v-if="hasMore" class="load-more">
      <el-button size="small" @click="loadMore" :loading="loading">加载更多</el-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { formatBytes } from '@/utils/systemInfoClient'
import { diskGetExtensionStats } from '@/utils/diskAnalyzerClient'
import type { ExtensionStat } from '@/utils/diskAnalyzerTypes'

const props = defineProps<{ scanId: string }>()

const exts = ref<ExtensionStat[]>([])
const loading = ref(false)
const total = ref(0)
const offset = ref(0)
const pageSize = 100
const hasMore = ref(false)

const fetchExts = async (reset = false) => {
  if (reset) {
    offset.value = 0
    exts.value = []
  }
  loading.value = true
  try {
    const page = await diskGetExtensionStats(props.scanId, pageSize, offset.value)
    if (reset) {
      exts.value = page.items
    } else {
      exts.value.push(...page.items)
    }
    total.value = page.total
    hasMore.value = exts.value.length < total.value
    offset.value += page.items.length
  } catch (e) {
    ElMessage.error(`加载类型统计失败: ${e}`)
  } finally {
    loading.value = false
  }
}

const loadMore = () => fetchExts(false)

onMounted(() => fetchExts(true))
</script>

<style scoped>
.extensions-tab { padding: 8px 0; }
.tab-toolbar { margin-bottom: 10px; }
.muted { font-size: 13px; color: var(--text-secondary, #888); }
.ext-tag {
  font-family: monospace;
  background: var(--bg-alt, #1e2a3a);
  padding: 2px 6px;
  border-radius: 3px;
}
.size-cell { display: flex; align-items: center; }
.load-more { text-align: center; margin-top: 12px; }
</style>
