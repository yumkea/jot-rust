<script setup lang="ts">
import iconPin from '../assets/icons/pin.svg'
import iconMinimize from '../assets/icons/minimize.svg'
import iconMaximize from '../assets/icons/maximize.svg'
import iconClose from '../assets/icons/close.svg'
import { ref, onMounted, onBeforeUnmount } from 'vue'

const props = withDefaults(defineProps<{
  isPinned: boolean
  t?: (key: string) => string
}>(), {
  t: (key: string) => key
})

defineEmits<{
  (e: 'toggle-pin'): void
}>()

const isMaximized = ref(false)
let unsubscribeMaximized: (() => void) | null = null

const minimize = (): void => window.electronAPI.minimize()

const maximize = (): void => {
  window.electronAPI.maximize()
}
const close = (): void => window.electronAPI.close()

onMounted(async () => {
  isMaximized.value = await window.electronAPI.isMaximized()
  unsubscribeMaximized = window.electronAPI.onMaximizedStatus((status) => {
    isMaximized.value = status
  })
})

onBeforeUnmount(() => {
  unsubscribeMaximized?.()
  unsubscribeMaximized = null
})
</script>

<template>
  <div class="window-controls no-drag">
    <button class="control-icon pin" :class="{ pinned: isPinned }" @click="$emit('toggle-pin')">
      <img :src="iconPin" class="icon-small" draggable="false" />
      <span class="pin-tooltip">{{ props.t('titlebar.pin') }}</span>
    </button>
    <button class="control-icon" @click="minimize">
      <img :src="iconMinimize" class="icon-small" draggable="false" />
      <span class="ctrl-tooltip">{{ props.t('titlebar.minimize') }}</span>
    </button>
    <button class="control-icon maximize" :class="{ maximized: isMaximized }" @click="maximize">
      <img :src="iconMaximize" class="icon-small" draggable="false" />
      <span class="ctrl-tooltip">{{ props.t('titlebar.maximize') }}</span>
    </button>
    <button class="control-icon close" @click="close">
      <img :src="iconClose" class="icon-small" draggable="false" />
      <span class="ctrl-tooltip">{{ props.t('titlebar.close') }}</span>
    </button>
  </div>
</template>

<style scoped>
.window-controls {
  display: flex;
  gap: 0px;
  align-items: center;
  flex-shrink: 0;
  min-width: 128px; /* 显式保证四个按钮（32*4）的最小宽度空间 */
}

.control-icon {
  width: 32px;
  height: 30px;
  background: transparent;
  border: none;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.2s;
  position: relative;
}

.icon-small {
  width: 20px;
  height: 20px;
  filter: var(--icon-filter);
  transition: filter 0.2s;
}

.control-icon.pin.pinned .icon-small {
  filter: var(--icon-active-filter);
}

.control-icon.maximize.maximized .icon-small {
  filter: var(--icon-active-filter);
}

.control-icon:hover {
  background: var(--hover-bg);
}

.pin-tooltip {
  position: absolute;
  top: calc(100% + 6px);
  left: 50%;
  transform: translateX(-50%) translateY(-2px);
  white-space: nowrap;
  font-size: 12px;
  font-weight: 500;
  color: var(--text-main);
  background: var(--overlay-bg);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  padding: 4px 10px;
  pointer-events: none;
  opacity: 0;
  transition: opacity 0.2s ease, transform 0.2s ease;
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
  z-index: 1000;
  letter-spacing: 0.3px;
}

.control-icon.pin:hover .pin-tooltip {
  opacity: 1;
  transform: translateX(-50%) translateY(0);
  transition-delay: 0.4s;
}

.ctrl-tooltip {
  position: absolute;
  top: calc(100% + 6px);
  left: 50%;
  transform: translateX(-50%) translateY(-2px);
  white-space: nowrap;
  font-size: 12px;
  font-weight: 500;
  color: var(--text-main);
  background: var(--overlay-bg);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  padding: 4px 10px;
  pointer-events: none;
  opacity: 0;
  transition: opacity 0.2s ease, transform 0.2s ease;
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
  z-index: 1000;
  letter-spacing: 0.3px;
}

.control-icon:hover .ctrl-tooltip {
  opacity: 1;
  transform: translateX(-50%) translateY(0);
  transition-delay: 0.4s;
}

.control-icon.close:hover .icon-small {
  filter: invert(41%) sepia(85%) saturate(3015%) hue-rotate(338deg) brightness(101%) contrast(100%);
}

.no-drag {
  -webkit-app-region: no-drag;
}
</style>
