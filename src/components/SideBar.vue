<script setup lang="ts">
import { computed } from 'vue'
import iconOutline from '../assets/icons/list.svg'
import iconSearch from '../assets/icons/search.svg'
import iconHistory from '../assets/icons/history.svg'
import iconSettings from '../assets/icons/settings.svg'

const props = defineProps<{
  activeTab: string
  t: (key: string) => string
}>()

const emit = defineEmits<{
  (e: 'update:active-tab', tab: string): void
  (e: 'reset-size'): void
}>()

const toggleTab = (tab: string): void => {
  if (props.activeTab === tab) {
    emit('update:active-tab', 'none')
  } else {
    emit('update:active-tab', tab)
  }
}

const indicatorY = computed(() => {
  switch (props.activeTab) {
    case 'outline': return 0
    case 'history': return 44
    case 'search': return 88
    case 'settings': return 132
    default: return 0
  }
})
</script>

<template>
  <aside class="sidebar no-drag">
    <div class="top-icons">
      <div 
        class="active-indicator" 
        :class="{ hidden: activeTab === 'none' }"
        :style="{ transform: `translateY(${indicatorY}px)` }"
      ></div>
      <button 
        :class="{ active: activeTab === 'outline' }" 
        @click="toggleTab('outline')"
      >
        <img :src="iconOutline" class="icon" draggable="false" />
        <span class="tooltip">{{ t('sidebar.outline') }}</span>
      </button>

      <button 
        :class="{ active: activeTab === 'history' }" 
        @click="toggleTab('history')"
      >
        <img :src="iconHistory" class="icon" draggable="false" />
        <span class="tooltip">{{ t('sidebar.history') }}</span>
      </button>

      <button 
        :class="{ active: activeTab === 'search' }" 
        @click="toggleTab('search')"
      >
        <img :src="iconSearch" class="icon" draggable="false" />
        <span class="tooltip">{{ t('sidebar.search') }}</span>
      </button>

      <button 
        :class="{ active: activeTab === 'settings' }" 
        @click="toggleTab('settings')"
      >
        <img :src="iconSettings" class="icon" draggable="false" />
        <span class="tooltip">{{ t('sidebar.settings') }}</span>
      </button>
    </div>

    <div class="sidebar-spacer" @dblclick="emit('reset-size')">
      <span class="tooltip spacer-tooltip">{{ t('sidebar.resetSize') }}</span>
    </div>

    <div class="bottom-logo">
      <span class="logo-text">Jot</span>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  width: 48px;
  flex-shrink: 0;
  position: relative;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  align-items: center;
  padding: 24px 0;
  border-right: 1px solid var(--border-color);
  background: var(--bg-main);
  border-radius: 8px 0 0 8px;
  overflow: visible;
  z-index: 100;
}

.top-icons {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}

.active-indicator {
  position: absolute;
  left: -12px;
  width: 4px;
  height: 24px;
  background: var(--text-main);
  border-radius: 0 4px 4px 0;
  transition: all 0.4s cubic-bezier(0.18, 0.89, 0.32, 1.28);
  margin-top: 4px; /* 对齐图标中心 */
}

.active-indicator.hidden {
  opacity: 0;
  transform: scaleY(0);
}

.top-icons button {
  background: transparent;
  border: none;
  width: 32px;
  height: 32px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 10px;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
}

.top-icons button:hover {
  background: var(--hover-bg);
}

/* Custom tooltip */
.tooltip {
  position: absolute;
  left: calc(100% + 10px);
  top: 50%;
  transform: translateY(-50%);
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
  transform: translateY(-50%) translateX(-4px);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
  z-index: 1000;
  letter-spacing: 0.3px;
}

.top-icons button:hover .tooltip {
  opacity: 1;
  transform: translateY(-50%) translateX(0);
  transition-delay: 0.4s;
}

.top-icons button.active {
  background: var(--tab-active-bg);
}

.icon {
  width: 24px;
  height: 24px;
  filter: var(--icon-filter);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  opacity: 0.8;
}

.top-icons button:hover .icon,
.top-icons button.active .icon {
  filter: var(--icon-active-filter);
  opacity: 1;
  transform: scale(1.05);
}

.sidebar-spacer {
  flex: 1;
  width: 100%;
  position: relative;
}

.sidebar-spacer:hover .spacer-tooltip {
  opacity: 1;
  transform: translateY(-50%) translateX(0);
  transition-delay: 0.4s;
}

.logo-text {
  font-family: 'Comic Sans MS', cursive, sans-serif;
  font-size: 14px;
  color: var(--text-main);
  font-weight: 800;
  letter-spacing: 1px;
  opacity: 0.9;
  transition: all 0.3s ease;
  user-select: none;
}

.sidebar:hover .logo-text {
  opacity: 1;
  text-shadow: 0 0 8px rgba(255, 255, 255, 0.2);
}

.no-drag {
  -webkit-app-region: no-drag;
}
</style>
