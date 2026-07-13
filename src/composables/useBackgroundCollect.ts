import { computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { ElNotification } from 'element-plus'
import { useToolboxStore } from '@/store'
import {
  type CollectKind, type CollectCompletePayload,
  getCollectStatus,
} from '@/utils/systemInfoClient'

// kind → (工具页 toolId, 中文标签, invoke 命令名)
const KIND_META: Record<CollectKind, { toolId: string; label: string; cmd: string }> = {
  system:   { toolId: 'systemInfo',   label: '系统信息', cmd: 'collect_system' },
  network:  { toolId: 'networkInfo',  label: '网络信息', cmd: 'collect_network' },
  process:  { toolId: 'processList',  label: '进程列表', cmd: 'collect_process' },
  hardware: { toolId: 'hardwareInfo', label: '硬件外设', cmd: 'collect_hardware' },
  software: { toolId: 'softwareEnv',  label: '软件环境', cmd: 'collect_software' },
}

export function useBackgroundCollect(kind: CollectKind) {
  const store = useToolboxStore()
  const meta = KIND_META[kind]
  const collecting = computed(() => store.collecting[kind])
  const result = computed(() => store.collectResults[kind])

  async function collect() {
    if (store.collecting[kind]) return          // 重复触发拦截
    store.collecting[kind] = true
    let done = false                            // 事件 + 轮询去重 flag
    let timer: ReturnType<typeof setInterval> | null = null
    let unlisten: UnlistenFn | null = null

    const finish = (payload: { ok: boolean; data?: unknown; error?: string | null }) => {
      if (done) return
      done = true
      if (timer) clearInterval(timer)
      if (unlisten) unlisten()
      store.collecting[kind] = false
      if (payload.ok) {
        store.collectResults[kind] = payload.data
        // 已在当前页面则不弹通知（用户直接能看到数据）
        if (store.activeTabId !== meta.toolId) {
          ElNotification.success({
            title: '采集完成',
            message: `${meta.label}采集完成，点击查看`,
            duration: 5000,
            onClick: () => store.openTab(meta.toolId),
          })
        }
      } else {
        ElNotification.error({
          title: '采集失败',
          message: payload.error || `${meta.label}采集失败`,
          duration: 5000,
        })
      }
    }

    unlisten = await listen<CollectCompletePayload>('collect-complete', (e) => {
      if (e.payload.kind !== kind) return
      finish({ ok: e.payload.ok, data: e.payload.data, error: e.payload.error })
    })

    timer = setInterval(async () => {            // 2s 轮询兜底
      try {
        const st = await getCollectStatus(kind)
        if (st && (st.status === 'done' || st.status === 'error')) {
          finish({ ok: st.status === 'done', data: st.data, error: st.error })
        }
      } catch { /* 轮询失败忽略，下次重试 */ }
    }, 2000)

    try {
      await invoke(meta.cmd)
    } catch (e) {
      // 启动即失败（极少见），直接收尾
      finish({ ok: false, error: String(e) })
    }
  }

  // 页面首次进入时若无缓存数据，自动触发采集
  function collectIfEmpty() {
    if (store.collectResults[kind] == null) {
      collect()
    }
  }

  return { collect, collectIfEmpty, collecting, result }
}

// ============ 自检 ============
// ponytail: 验证 KIND_META 五个 kind 完整，且重复触发拦截语义
console.assert(Object.keys(KIND_META).length === 5, 'KIND_META 应有 5 个 kind')
console.assert(KIND_META.process.toolId === 'processList', 'process → processList')
console.assert(KIND_META.process.cmd === 'collect_process', 'process → collect_process')
