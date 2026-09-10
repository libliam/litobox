<template>
  <div class="tool-container cheatsheet-container">
    <!-- Tab 栏 -->
    <div class="tool-card tab-card">
      <el-tabs v-model="activeTab" class="cheatsheet-tabs">
        <el-tab-pane v-for="sheet in cheatSheets" :key="sheet.id" :label="sheet.name" :name="sheet.id" />
      </el-tabs>
    </div>

    <!-- 速查表内容 -->
    <div v-for="sheet in cheatSheets" :key="sheet.id" v-show="activeTab === sheet.id" class="tool-card sheet-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">{{ sheet.name }}</span>
          <span v-if="sheet.description" class="sheet-desc">{{ sheet.description }}</span>
        </div>
        <div class="card-actions">
          <el-input
            v-model="searchMap[sheet.id]"
            placeholder="搜索..."
            size="small"
            clearable
            style="width: 220px"
          >
            <template #prefix>
              <el-icon><Search /></el-icon>
            </template>
          </el-input>
        </div>
      </div>
      <div class="card-body sheet-body">
        <div class="table-wrapper">
          <el-table
            :data="filteredRows(sheet)"
            border
            stripe
            size="small"
            height="100%"
            style="width: 100%"
          >
            <el-table-column
              v-for="col in sheet.columns"
              :key="col.key"
              :prop="col.key"
              :label="col.label"
              :width="col.width"
            >
              <template #default="{ row }">
                <span
                  v-if="col.copyable"
                  class="copyable-cell"
                  @click="copyCell(row[col.key])"
                >{{ row[col.key] }}</span>
                <span v-else>{{ row[col.key] }}</span>
              </template>
            </el-table-column>
          </el-table>
        </div>
        <div class="table-footer">
          共 {{ filteredRows(sheet).length }} 条 / {{ sheet.rows.length }} 条
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue'
import { ElMessage } from 'element-plus'
import { Search } from '@element-plus/icons-vue'
import { cheatSheets, type CheatSheetData } from '@/utils/cheatSheets'

const activeTab = ref(cheatSheets[0]?.id || '')

const searchMap = reactive<Record<string, string>>({})
cheatSheets.forEach(s => { searchMap[s.id] = '' })

const filteredRows = (sheet: CheatSheetData) => {
  const kw = (searchMap[sheet.id] || '').trim().toLowerCase()
  if (!kw) return sheet.rows
  return sheet.rows.filter(row =>
    sheet.columns.some(col => String(row[col.key] || '').toLowerCase().includes(kw))
  )
}

const copyCell = async (text: string) => {
  if (!text) return
  try {
    await navigator.clipboard.writeText(text)
    ElMessage.success(`已复制: ${text}`)
  } catch {
    ElMessage.error('复制失败')
  }
}
</script>

<style scoped>
.cheatsheet-container {
  display: flex;
  flex-direction: column;
  overflow: hidden;
  gap: 16px;
}

.tab-card {
  flex-shrink: 0;
}

.sheet-card {
  flex: 1;
  min-height: 300px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.sheet-body {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.table-wrapper {
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.cheatsheet-tabs :deep(.el-tabs__header) {
  margin-bottom: 0;
  padding-left: 8px;
}

.cheatsheet-tabs :deep(.el-tabs__nav-wrap) { padding-left: 4px; }
.cheatsheet-tabs :deep(.el-tabs__item) { color: var(--text-secondary); font-size: 14px; font-weight: 500; }
.cheatsheet-tabs :deep(.el-tabs__item.is-active) { color: var(--accent-cyan); }
.cheatsheet-tabs :deep(.el-tabs__active-bar) { background-color: var(--accent-cyan); }
.cheatsheet-tabs :deep(.el-tabs__nav-wrap::after) { background-color: var(--border-color); }

.sheet-desc {
  font-size: 12px;
  color: var(--text-muted);
  font-weight: normal;
  text-transform: none;
  letter-spacing: normal;
}

.copyable-cell {
  cursor: pointer;
  color: var(--accent-cyan);
  font-family: 'JetBrains Mono', monospace;
  transition: opacity 0.2s;
}

.copyable-cell:hover {
  opacity: 0.7;
  text-decoration: underline;
}

.table-footer {
  flex-shrink: 0;
  margin-top: 8px;
  font-size: 12px;
  color: var(--text-muted);
  text-align: right;
}
</style>
