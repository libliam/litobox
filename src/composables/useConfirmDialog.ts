import { reactive, h, nextTick, type Component } from 'vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import type { ConfirmOptions } from '@/components/ConfirmDialog.vue'

interface ConfirmState {
  visible: boolean
  title: string
  message: string
  type: 'warning' | 'danger'
  confirmText: string
  cancelText: string
  closeOnClickOverlay: boolean
  closeOnPressEscape: boolean
  _resolve: null | ((v: boolean) => void)
  _confirmed: boolean
}

const state = reactive<ConfirmState>({
  visible: false,
  title: '',
  message: '',
  type: 'warning',
  confirmText: '确认',
  cancelText: '取消',
  closeOnClickOverlay: false,
  closeOnPressEscape: true,
  _resolve: null,
  _confirmed: false,
})

const ask = (title: string, message: string, opts?: Omit<ConfirmOptions, 'title' | 'message'>): Promise<boolean> => {
  console.log('[ConfirmDialog] ask 被调用, visible=true')
  state.title = title
  state.message = message
  state.type = opts?.type || 'warning'
  state.confirmText = opts?.confirmText || '确认'
  state.cancelText = opts?.cancelText || '取消'
  state.closeOnClickOverlay = opts?.closeOnClickOverlay ?? false
  state.closeOnPressEscape = opts?.closeOnPressEscape ?? true
  state._confirmed = false
  state.visible = true
  return new Promise(resolve => {
    console.log('[ConfirmDialog] Promise 创建，_resolve 已设置')
    state._resolve = resolve
  })
}

const handleConfirm = () => {
  console.log('[ConfirmDialog] handleConfirm 触发, resolve(true)')
  state._confirmed = true
  state.visible = false
  state._resolve?.(true)
  state._resolve = null
}

const handleCancel = () => {
  console.log('[ConfirmDialog] handleCancel 触发, _confirmed=', state._confirmed)
  if (state._confirmed) {
    console.log('[ConfirmDialog] 已确认，跳过取消')
    return
  }
  state.visible = false
  console.log('[ConfirmDialog] resolve(false)')
  state._resolve?.(false)
  state._resolve = null
}

export const ConfirmDialogWrapper: Component = {
  name: 'ConfirmDialogWrapper',
  setup() {
    return () =>
      h(ConfirmDialog, {
        visible: state.visible,
        title: state.title,
        message: state.message,
        type: state.type,
        confirmText: state.confirmText,
        cancelText: state.cancelText,
        closeOnClickOverlay: state.closeOnClickOverlay,
        closeOnPressEscape: state.closeOnPressEscape,
        'onUpdate:visible': (v: boolean) => {
          console.log('[ConfirmDialogWrapper] onUpdate:visible, v=', v)
          if (!v) {
            // ponytail: 延迟到 nextTick，让 onConfirm/onCancel 先跑完
            // 否则 update:visible 的 handleCancel 会先清空 _resolve，导致 confirm 事件无效
            nextTick(() => handleCancel())
          }
        },
        onConfirm: () => {
          console.log('[ConfirmDialogWrapper] onConfirm 事件')
          handleConfirm()
        },
        onCancel: () => {
          console.log('[ConfirmDialogWrapper] onCancel 事件')
          handleCancel()
        },
      })
  },
}

export function useConfirmDialog() {
  return {
    confirm: { ask },
  }
}

export { ask as confirmAsk }
