<script setup lang="ts">
import { ref } from 'vue'

interface Note {
  id: string
  title: string
}

const props = defineProps<{
  note: Note
  isActive: boolean
}>()

const emit = defineEmits<{
  (e: 'switch'): void
  (e: 'close'): void
  (e: 'rename', newTitle: string): void
}>()

const isEditing = ref(false)
const editingTitle = ref('')

const startEditing = (): void => {
  isEditing.value = true
  editingTitle.value = props.note.title
}

const finishEditing = (): void => {
  if (isEditing.value && editingTitle.value.trim()) {
    emit('rename', editingTitle.value.trim())
  }
  isEditing.value = false
}

const handleMouseDown = (e: MouseEvent): void => {
  // 右键双击检测 (button 2 = right, detail 2 = double click)
  if (e.button === 2 && e.detail === 2) {
    emit('close')
  }
}

const vFocus = {
  mounted: (el: HTMLInputElement): void => {
    el.focus()
    el.select()
  }
}
</script>

<template>
  <div
    class="tab"
    :class="{ active: isActive, editing: isEditing }"
    @mousedown="handleMouseDown"
    @click="emit('switch')"
    @dblclick="startEditing"
    @contextmenu.prevent
  >
    <div v-if="isEditing" class="tab-input-wrapper">
      <input
        v-model="editingTitle"
        v-focus
        class="tab-input"
        @blur="finishEditing"
        @keyup.enter="finishEditing"
        @click.stop
      />
      <span class="tab-input-measure">{{ editingTitle || ' ' }}</span>
    </div>
    <span v-else class="tab-text">{{ note.title }}</span>
    <div class="tab-divider"></div>
  </div>
</template>

<style scoped>
.tab {
  flex-shrink: 0;
  height: 24px;
  padding: 0 10px;
  background: transparent;
  border-radius: 4px;
  display: flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  border: none;
  transition: all 0.2s;
  max-width: 120px; /* 稍微放宽一点，或者保持 90px */
  margin-bottom: 2px;
  position: relative;
}

.tab-divider {
  position: absolute;
  right: -2px; /* 放在 gap 的中间 */
  top: 50%;
  transform: translateY(-50%);
  width: 1px;
  height: 12px;
  background: var(--divider-color);
  pointer-events: none;
}

/* 最后一个标签或者激活的标签后的分割线隐藏（可选，根据视觉需求） */
.tab.active .tab-divider {
  opacity: 0;
}

.tab.active {
  background: var(--tab-active-bg);
}

.tab.editing {
  max-width: none; /* 编辑时取消最大宽度限制，让其根据内容伸缩 */
}

.tab:hover:not(.active) {
  background: var(--hover-bg);
}

.tab.active .tab-text {
  color: var(--text-main);
}

.tab-text {
  font-size: 11px;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  user-select: none;
}

.tab-input-wrapper {
  position: relative;
  display: flex;
  align-items: center;
  min-width: 20px;
  max-width: 150px; /* 设置一个最大编辑宽度 */
}

.tab-input {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: transparent;
  border: none;
  color: var(--text-main);
  font-size: 11px;
  padding: 0;
  outline: none;
  box-sizing: border-box;
  z-index: 1;
  display: flex;
  align-items: center;
}

/* 用来撑开父级宽度的隐藏层 */
.tab-input-measure {
  font-size: 11px;
  padding: 0;
  visibility: hidden;
  white-space: pre;
  height: 18px;
  display: inline-block;
}
</style>
