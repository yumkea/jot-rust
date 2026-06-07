<script setup lang="ts">
import iconAdd from '../assets/icons/addsheet.svg'
import WindowControls from './WindowControls.vue'
import TabItem from './TabItem.vue'

interface Note {
  id: string
  title: string
}

const props = withDefaults(defineProps<{
  isPinned: boolean
  notes: Note[]
  activeNoteId: string
  t?: (key: string) => string
}>(), {
  t: (key: string) => key
})

const emit = defineEmits<{
  (e: 'toggle-pin'): void
  (e: 'add-note'): void
  (e: 'close-all'): void
  (e: 'close-note', id: string): void
  (e: 'switch-note', id: string): void
  (e: 'rename-note', id: string, newTitle: string): void
}>()

const handleWheel = (e: WheelEvent): void => {
  const container = e.currentTarget as HTMLElement
  if (container) {
    container.scrollLeft += e.deltaY
  }
}

// 手动计时检测右键双击（e.detail 对右键不可靠）
let lastRightClickTime = 0
const DOUBLE_CLICK_THRESHOLD = 400 // ms

const handleMouseDown = (e: MouseEvent): void => {
  if (e.button === 2) {
    const now = Date.now()
    if (now - lastRightClickTime < DOUBLE_CLICK_THRESHOLD) {
      emit('close-all')
      lastRightClickTime = 0
    } else {
      lastRightClickTime = now
    }
  }
}
</script>

<template>
  <header class="title-bar">
    <div class="tabs-container">
      <div class="tab-add no-drag" @click.left="emit('add-note')" @mousedown="handleMouseDown" @contextmenu.prevent>
        <img :src="iconAdd" class="icon-tab-add" draggable="false" />
        <span class="add-tooltip">{{ props.t('titlebar.newNote') }}</span>
      </div>
      <div class="tabs-bar no-drag" @wheel.prevent="handleWheel">
        <TabItem
          v-for="note in notes"
          :key="note.id"
          :note="note"
          :is-active="note.id === activeNoteId"
          @switch="emit('switch-note', note.id)"
          @close="emit('close-note', note.id)"
          @rename="(newTitle) => emit('rename-note', note.id, newTitle)"
        />
      </div>
      <div class="tabs-spacer"></div>
    </div>
    <WindowControls :is-pinned="isPinned" :t="props.t" @toggle-pin="emit('toggle-pin')" />
  </header>
</template>

<style scoped>
.title-bar {
  height: 30px;
  flex-shrink: 0;
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 4px; /* 增加整体内边距，避开容器圆角 */
  -webkit-app-region: drag;
  overflow: hidden;
  width: 100%; /* 确保占据全部可用宽度 */
}

.tabs-container {
  flex: 1;
  height: 100%;
  display: flex;
  align-items: flex-end;
  padding-left: 0;
  min-width: 0;
  overflow: hidden;
  justify-content: flex-start;
  gap: 4px; /* 添加加号与标签之间的间距 */
  position: relative;
}

.tabs-spacer {
  flex: 1 0 60px; /* 自动增长占据剩余空间，但最小保证 60px */
  height: 100%;
  position: relative;
}

.tabs-spacer::before {
  content: '';
  position: absolute;
  right: 0; /* 紧贴右侧控制按钮 */
  top: 50%;
  transform: translateY(-50%);
  width: 1px;
  height: 16px;
  background: var(--divider-color);
}

.tabs-spacer::after {
  content: '';
  position: absolute;
  left: 0; /* 紧贴左侧标签页区域 */
  top: 50%;
  transform: translateY(-50%);
  width: 1px;
  height: 16px;
  background: var(--divider-color);
}

.tabs-bar {
  flex: 0 1 auto; /* 根据标签数量自动缩放，不再强制平铺 */
  display: flex;
  gap: 4px;
  height: 26px;
  align-items: flex-end;
  overflow-x: auto;
  overflow-y: hidden;
  scrollbar-width: none;
  min-width: 0;
  padding-left: 2px; /* 增加微小间距，防止第一个标签被边缘遮挡 */
}

.tabs-bar::-webkit-scrollbar {
  display: none;
}

.tab-add {
  flex-shrink: 0;
  width: 28px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  margin-bottom: 2px;
  transition: all 0.2s;
  background: transparent;
  border-radius: 4px;
  position: relative;
}

.icon-tab-add {
  width: 19px;
  height: 19px;
  opacity: 0.85; /* 再次提高基础亮度 */
  filter: var(--icon-filter); /* 移除硬编码 contrast */
  transition: opacity 0.2s;
}

.tab-add:hover {
  background: var(--hover-bg);
}

.tab-add:hover .icon-tab-add {
  opacity: 1;
  filter: var(--icon-active-filter);
}

.add-tooltip {
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

.tab-add:hover .add-tooltip {
  opacity: 1;
  transform: translateX(-50%) translateY(0);
  transition-delay: 0.4s;
}

.no-drag {
  -webkit-app-region: no-drag;
}
</style>
