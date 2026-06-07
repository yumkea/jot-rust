<script setup lang="ts">
import iconResize from '../assets/icons/resize.svg'

const props = withDefaults(defineProps<{
  onResize: (direction: string, e: MouseEvent) => void
  onReset: () => void
  lastSavedTime?: string
  t?: (key: string) => string
}>(), {
  t: (key: string) => key
})
</script>

<template>
  <footer class="footer-bar no-drag">
    <div class="status-container">
      <span v-if="lastSavedTime" class="save-time">
        {{ props.t('footer.updated') }} {{ lastSavedTime }}
      </span>
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

.save-time {
  font-size: 11px; /* 稍微调大一点 */
  color: rgba(255, 255, 255, 0.4);
  font-family: ui-monospace, SFMono-Regular, monospace;
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
  filter: brightness(0) invert(1);
  transition: opacity 0.2s;
}

.resize-handle:hover .icon-resize {
  opacity: 0.7;
}

.no-drag {
  -webkit-app-region: no-drag;
}
</style>
