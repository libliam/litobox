<script setup lang="ts">
import { ref, watch, onBeforeUnmount, type PropType } from 'vue'

export interface ConfirmOptions {
  title: string
  message: string
  type?: 'warning' | 'danger'
  confirmText?: string
  cancelText?: string
  closeOnClickOverlay?: boolean
  closeOnPressEscape?: boolean
}

const props = defineProps({
  visible: Boolean,
  title: String,
  message: String,
  type: {
    type: String as PropType<'warning' | 'danger'>,
    default: 'warning',
  },
  confirmText: {
    type: String,
    default: '确认',
  },
  cancelText: {
    type: String,
    default: '取消',
  },
  closeOnClickOverlay: {
    type: Boolean,
    default: false,
  },
  closeOnPressEscape: {
    type: Boolean,
    default: true,
  },
})

const emit = defineEmits<{
  (e: 'update:visible', v: boolean): void
  (e: 'confirm'): void
  (e: 'cancel'): void
}>()

const handleCancel = () => {
  emit('update:visible', false)
  emit('cancel')
}

const handleConfirm = () => {
  emit('update:visible', false)
  emit('confirm')
}

const handleOverlayClick = () => {
  if (props.closeOnClickOverlay) {
    handleCancel()
  }
}

// ESC 键关闭
const handleKeydown = (e: KeyboardEvent) => {
  if (e.key === 'Escape' && props.closeOnPressEscape && props.visible) {
    handleCancel()
  }
}

// 在组件内监听键盘事件
const isListening = ref(false)
const ensureKeyListener = () => {
  if (props.visible && !isListening.value) {
    document.addEventListener('keydown', handleKeydown)
    isListening.value = true
  } else if (!props.visible && isListening.value) {
    document.removeEventListener('keydown', handleKeydown)
    isListening.value = false
  }
}

watch(() => props.visible, ensureKeyListener)
onBeforeUnmount(() => {
  if (isListening.value) {
    document.removeEventListener('keydown', handleKeydown)
  }
})
</script>

<template>
  <Teleport to="body">
    <Transition name="confirm-fade">
      <div v-if="visible" class="confirm-overlay" @click.self="handleOverlayClick">
        <Transition name="confirm-zoom" appear>
          <div v-if="visible" class="confirm-dialog" role="dialog" aria-modal="true">
            <div class="confirm-header">
              <span class="confirm-title">{{ title }}</span>
              <button class="confirm-close" @click="handleCancel" aria-label="关闭">×</button>
            </div>
            <div class="confirm-body">
              <div class="confirm-icon" :class="type">
                <svg v-if="type === 'warning'" viewBox="0 0 24 24" width="32" height="32" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/>
                  <line x1="12" y1="9" x2="12" y2="13"/>
                  <line x1="12" y1="17" x2="12.01" y2="17"/>
                </svg>
                <svg v-else-if="type === 'danger'" viewBox="0 0 24 24" width="32" height="32" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <circle cx="12" cy="12" r="10"/>
                  <line x1="15" y1="9" x2="9" y2="15"/>
                  <line x1="9" y1="9" x2="15" y2="15"/>
                </svg>
              </div>
              <div class="confirm-message">{{ message }}</div>
            </div>
            <div class="confirm-footer">
              <button class="confirm-btn cancel" @click="handleCancel">{{ cancelText }}</button>
              <button class="confirm-btn ok" :class="type" @click="handleConfirm">{{ confirmText }}</button>
            </div>
          </div>
        </Transition>
      </div>
    </Transition>
  </Teleport>
</template>

<style>
.confirm-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 2000;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.5);
  backdrop-filter: blur(2px);
}

.confirm-dialog {
  width: 420px;
  max-width: 90%;
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.5), 0 0 0 1px rgba(0, 212, 255, 0.1);
  overflow: hidden;
}

.confirm-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 20px;
  background: rgba(0, 0, 0, 0.2);
  border-bottom: 1px solid var(--border-color);
}

.confirm-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--accent-cyan);
  text-transform: uppercase;
  letter-spacing: 1px;
}

.confirm-close {
  width: 24px;
  height: 24px;
  border: none;
  background: transparent;
  color: var(--text-muted);
  font-size: 18px;
  cursor: pointer;
  border-radius: 4px;
  transition: all 0.2s;
}

.confirm-close:hover {
  background: rgba(255, 255, 255, 0.1);
  color: var(--text-primary);
}

.confirm-body {
  display: flex;
  align-items: flex-start;
  gap: 16px;
  padding: 24px 20px;
}

.confirm-icon {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}

.confirm-icon.warning { color: var(--accent-orange, #f59e0b); }
.confirm-icon.danger { color: var(--accent-red, #ef4444); }

.confirm-message {
  flex: 1;
  font-size: 14px;
  line-height: 1.7;
  color: var(--text-primary);
  white-space: pre-line;
  padding-top: 4px;
}

.confirm-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  padding: 12px 20px 16px;
  border-top: 1px solid var(--border-color);
}

.confirm-btn {
  padding: 7px 20px;
  font-size: 13px;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s;
  border: 1px solid var(--border-color);
}

.confirm-btn.cancel {
  background: var(--bg-input);
  color: var(--text-secondary);
}

.confirm-btn.cancel:hover {
  border-color: var(--accent-cyan);
  color: var(--accent-cyan);
}

.confirm-btn.ok {
  border: none;
  color: #fff;
}

.confirm-btn.ok.warning {
  background: linear-gradient(135deg, #f59e0b, #d97706);
}

.confirm-btn.ok.danger {
  background: linear-gradient(135deg, #ef4444, #dc2626);
}

.confirm-btn.ok:hover {
  opacity: 0.9;
  transform: translateY(-1px);
}

.confirm-fade-enter-active,
.confirm-fade-leave-active {
  transition: opacity 0.2s ease;
}

.confirm-fade-enter-from,
.confirm-fade-leave-to {
  opacity: 0;
}

.confirm-zoom-enter-active {
  transition: all 0.2s ease;
}

.confirm-zoom-enter-from {
  transform: scale(0.92) translateY(-10px);
  opacity: 0;
}
</style>
