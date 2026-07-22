import { reactive, h, type Component } from 'vue'
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
})

const ask = (title: string, message: string, opts?: Omit<ConfirmOptions, 'title' | 'message'>): Promise<boolean> => {
  state.title = title
  state.message = message
  state.type = opts?.type || 'warning'
  state.confirmText = opts?.confirmText || '确认'
  state.cancelText = opts?.cancelText || '取消'
  state.closeOnClickOverlay = opts?.closeOnClickOverlay ?? false
  state.closeOnPressEscape = opts?.closeOnPressEscape ?? true
  state.visible = true
  return new Promise(resolve => { state._resolve = resolve })
}

const handleConfirm = () => {
  state.visible = false
  state._resolve?.(true)
  state._resolve = null
}

const handleCancel = () => {
  state.visible = false
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
        'onUpdate:visible': (v: boolean) => { if (!v) handleCancel() },
        onConfirm: handleConfirm,
        onCancel: handleCancel,
      })
  },
}

export function useConfirmDialog() {
  return {
    confirm: { ask },
  }
}

export { ask as confirmAsk }
