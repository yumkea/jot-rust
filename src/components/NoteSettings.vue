<script setup lang="ts">
import { useI18n } from '../composables/useI18n'

const { t } = useI18n()

defineProps<{
  isOpen: boolean
  width?: number
  activeSettingsTab: string
}>()

const emit = defineEmits<{
  (e: 'update:activeSettingsTab', tab: string): void
}>()

const selectTab = (tab: string): void => {
  emit('update:activeSettingsTab', tab)
}
</script>

<template>
  <aside
    class="note-settings scrollbar-hide"
    :class="{ 'is-open': isOpen }"
    :style="{
      width: isOpen ? (width ? width + 'px' : '160px') : '0'
    }"
  >
    <div class="settings-container" :style="{ width: width ? width + 'px' : '160px' }">
      <div class="settings-tabs">
        <div class="tab-item" :class="{ active: activeSettingsTab === 'general' }" @click="selectTab('general')">
          {{ t('settings.general') }}
        </div>
        <div class="tab-item" :class="{ active: activeSettingsTab === 'shortcuts' }" @click="selectTab('shortcuts')">
          {{ t('settings.shortcuts') }}
        </div>
        <div class="tab-item" :class="{ active: activeSettingsTab === 'about' }" @click="selectTab('about')">
          {{ t('settings.about') }}
        </div>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.note-settings {
  width: 0;
  padding: 20px 0;
  overflow-y: auto;
  overflow-x: hidden;
  flex-shrink: 0;
}

.note-settings.is-open {
  width: 160px;
}

.settings-container {
  display: flex;
  flex-direction: column;
  gap: 20px;
  padding: 0 10px;
  /* 宽度由内联样式动态设置，不再使用固定值 */
  box-sizing: border-box;
}

.settings-tabs {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.tab-item {
  padding: 4px 6px;
  border-radius: 4px;
  cursor: pointer;
  font-size: var(--panel-font-size-item);
  color: var(--text-secondary);
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
  overflow: hidden;
}

.tab-item:hover {
  background: var(--hover-bg);
  padding-left: 8px;
  color: var(--text-main);
}

.tab-item.active {
  background: var(--tab-active-bg);
  border-left: 2px solid var(--text-main);
  padding-left: 6px;
  color: var(--text-main);
  font-weight: 500;
}

/* 隐藏滚动条 */
.scrollbar-hide::-webkit-scrollbar {
  display: none;
}

.scrollbar-hide {
  -ms-overflow-style: none;
  scrollbar-width: none;
}
</style>
