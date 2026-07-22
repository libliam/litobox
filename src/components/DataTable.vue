<script setup lang="ts">
/**
 * DataTable — 公共表格组件
 * 基于 el-table 薄封装：
 * - 默认开启边框 + 斑马纹 + 紧凑尺寸，统一项目视觉
 * - 深色/浅色主题由 theme.css 全局接管，本组件不写主题样式
 * - 透传所有 el-table props/事件/插槽，零学习成本
 *
 * 用法与 el-table 完全一致，只是把 <el-table> 换成 <DataTable>：
 * <DataTable :data="list" @row-click="fn">
 *   <el-table-column prop="name" label="名称" />
 *   <el-table-column label="操作">
 *     <template #default="{ row }">...</template>
 *   </el-table-column>
 * </DataTable>
 */
defineOptions({ name: 'DataTable', inheritAttrs: false })

withDefaults(
  defineProps<{
    /** 边框，默认开 */
    border?: boolean
    /** 斑马纹，默认开 */
    stripe?: boolean
    /** 尺寸，默认 small */
    size?: 'large' | 'default' | 'small'
    /** 空数据文案 */
    emptyText?: string
  }>(),
  {
    border: true,
    stripe: true,
    size: 'small',
    emptyText: '暂无数据',
  }
)
</script>

<template>
  <el-table
    v-bind="$attrs"
    :border="border"
    :stripe="stripe"
    :size="size"
    :empty-text="emptyText"
  >
    <slot />
    <template v-if="$slots.empty" #empty><slot name="empty" /></template>
    <template v-if="$slots.append" #append><slot name="append" /></template>
    <template v-if="$slots.footer" #footer><slot name="footer" /></template>
    <template v-if="$slots.header" #header><slot name="header" /></template>
    <template v-if="$slots.summary" #summary><slot name="summary" /></template>
  </el-table>
</template>
