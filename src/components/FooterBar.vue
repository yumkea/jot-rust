<script setup lang="ts">
import { computed } from 'vue'
import iconResize from '../assets/icons/resize.svg'

const props = withDefaults(defineProps<{
  onResize: (direction: string, e: MouseEvent) => void
  onReset: () => void
  lastSavedTime?: string
  selectedWordCount?: number
  wordCount?: number
  t?: (key: string) => string
}>(), {
  selectedWordCount: 0,
  wordCount: 0,
  t: (key: string) => key
})

const saveStatusText = computed(() => {
  if (!props.lastSavedTime) return ''
  if (props.lastSavedTime === '__SAVE_FAILED__') return props.t('footer.saveFailed')
  return `${props.t('footer.updated')} ${props.lastSavedTime}`
})

const wordStatusText = computed(() => {
  if (props.selectedWordCount > 0) {
    return `${props.selectedWordCount} / ${props.wordCount} ${props.t('footer.words')}`
  }
  return `${props.wordCount} ${props.t('footer.words')}`
})
</script>

<template>
  <footer class="footer-bar no-drag">
    <div class="status-container">
      <span v-if="saveStatusText" class="save-time">
        {{ saveStatusText }}
      </span>
      <span v-if="saveStatusText" class="status-divider"></span>
      <span class="word-count">{{ wordStatusText }}</span>
    </div>
    <div class="resize-handle" @mousedown="onResize('se', $event)" @dblclick="onReset">
      <img :src="iconResize" class="icon-resize" draggable="false" />
    </div>
  </footer>
</template>

<style scoped>
.footer-bar {
  height: 24px;
  flex-shrink: 0;
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 4px;
  margin-top: auto;
}

.status-container {
  flex: 1; /* 撑满剩余空间 */
  height: 100%;
  display: flex;
  align-items: center;
  gap: 8px;
  padding-left: 10px;
  user-select: none;
}

.word-count,
.save-time {
  font-size: 11px; /* 稍微调大一点 */
  color: var(--text-low);
  font-family: ui-monospace, SFMono-Regular, monospace;
}

.status-divider {
  width: 1px;
  height: 10px;
  background: var(--text-low);
  opacity: 0.45;
}

.resize-handle {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: grab;
}

.resize-handle:active {
  cursor: grabbing;
}

.icon-resize {
  width: 18px;
  height: 18px;
  filter: var(--icon-filter);
  transition: opacity 0.2s;
}

.resize-handle:hover .icon-resize {
  opacity: 0.7;
}

.no-drag {
  -webkit-app-region: no-drag;
}
</style>
