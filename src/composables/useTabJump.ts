// 通用多 tab 跳转消费：配合 store.openToolTab(toolId, tabId) 使用。
// 全局搜索命中某工具的子 tab 后，openToolTab 设置 pending 并打开工具，
// 目标组件通过本 composable 在激活时切到对应 tab。
//
// 关键点（见 AGENTS 踩坑 12 + el-tabs KeepAlive 问题）：
// - 组件被 KeepAlive 缓存时，watch 在 deactivated 状态仍会触发，但 el-tabs 未激活，
//   v-model 改动可能没同步到内部状态；所以 watch 只设值不清除。
// - 真正的清除交给 onMounted/onActivated，确保组件激活时再设一次，el-tabs 能切到对应 tab。
// - 若组件当前已激活，watch 设值后可立即清除（isActive 标记）。
import { ref, watch, onMounted, onActivated, onDeactivated, type Ref } from 'vue'
import { useToolboxStore } from '@/store'

export function useTabJump(activeTab: Ref<string>, toolId: string) {
  const store = useToolboxStore()
  const isActive = ref(false)

  const apply = () => {
    const p = store.tabJumpPending
    if (!p || p.toolId !== toolId) return
    activeTab.value = p.tabId
    // 已激活则立即清除；未激活留到 onActivated 再设再清，确保 el-tabs 接收到
    if (isActive.value) store.tabJumpPending = null
  }

  const consume = () => {
    const p = store.tabJumpPending
    if (p && p.toolId === toolId) {
      activeTab.value = p.tabId
      store.tabJumpPending = null
    }
  }

  onMounted(() => { isActive.value = true; consume() })
  onActivated(() => { isActive.value = true; consume() })
  onDeactivated(() => { isActive.value = false })
  watch(() => store.tabJumpPending, apply)
}
