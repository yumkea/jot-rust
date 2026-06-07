<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useI18n, getLocalizedDateLabel } from '../composables/useI18n'
import * as api from '../api'

const { language, t } = useI18n()

interface NoteRecord {
  id: string
  title: string
  content: string
  updated_at: string
}

const props = defineProps<{
  activeNoteId: string
  isOpen: boolean
  width?: number
}>()

const emit = defineEmits<{
  (e: 'select-note', note: { id: string; title: string; content: string }): void
}>()

const allNotes = ref<NoteRecord[]>([])
const expandedGroups = ref<Record<string, boolean>>({})

const fetchNotes = async (): Promise<void> => {
  const result = await api.listNotes()
  allNotes.value = result as NoteRecord[]
}

watch(
  () => props.isOpen,
  (open) => {
    if (open) fetchNotes()
  },
  { immediate: true }
)

defineExpose({
  refresh: fetchNotes
})

const formatDateLabel = (dateStr: string): string => getLocalizedDateLabel(dateStr, language.value)

const formatTime = (dateStr: string): string => {
  const timePart = dateStr.split(' ')[1]
  return timePart ? timePart.slice(0, 5) : '00:00'
}

const groupedNotes = computed(() => {
  const groups: Record<
    string,
    { label: string; isExpanded: boolean; records: { id: string; time: string; title: string; content: string }[] }
  > = {}

  allNotes.value.forEach((rec) => {
    const label = formatDateLabel(rec.updated_at)
    if (!groups[label]) {
      groups[label] = { label, isExpanded: expandedGroups.value[label] ?? true, records: [] }
    }
    groups[label].records.push({
      id: rec.id,
      time: formatTime(rec.updated_at),
      title: rec.title || t('editor.untitled'),
      content: rec.content
    })
  })

  return Object.values(groups)
})

const toggleGroup = (label: string): void => {
  expandedGroups.value[label] = !(expandedGroups.value[label] ?? true)
}

const selectNote = (note: { id: string; title: string; content: string }): void => emit('select-note', note)
</script>

<template>
  <aside
    class="note-history scrollbar-hide"
    :class="{ 'is-open': isOpen }"
    :style="{ width: isOpen ? (width ? width + 'px' : '160px') : '0' }"
  >
    <div class="history-container" :style="{ width: width ? width + 'px' : '160px' }">
      <div v-for="group in groupedNotes" :key="group.label" class="history-group">
        <div class="group-header" @click="toggleGroup(group.label)">
          <span class="group-label">{{ group.label }}</span>
          <div class="group-icons" :class="{ 'is-expanded': group.isExpanded }">
            <svg viewBox="0 0 24 24" class="chevron-icon">
              <path
                d="M6 9l6 6 6-6"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
            </svg>
          </div>
        </div>

        <transition name="expand">
          <div v-if="group.isExpanded" class="group-items">
            <div
              v-for="item in group.records"
              :key="item.id"
              class="history-item"
              :class="{ 'is-active': item.id === activeNoteId }"
              @click="selectNote(item)"
            >
              <span class="item-time">{{ item.time }}</span>
              <span class="item-title">{{ item.title }}</span>
              <svg viewBox="0 0 24 24" class="item-arrow">
                <path
                  d="M9 18l6-6-6-6"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2.5"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                />
              </svg>
            </div>
          </div>
        </transition>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.note-history {
  width: 0;
  padding: 20px 0;
  overflow-y: auto;
  overflow-x: hidden;
  flex-shrink: 0;
}

.note-history.is-open {
  width: 160px;
}

.history-container {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 0 10px;
  box-sizing: border-box;
}

.history-group {
  display: flex;
  flex-direction: column;
}

.group-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  cursor: pointer;
  padding: 4px 0;
  margin-bottom: 4px;
}

.group-label {
  font-size: var(--panel-font-size-label);
  font-weight: 500;
  color: var(--text-low);
  transition: color 0.2s;
}

.group-header:hover .group-label {
  color: var(--text-main);
}

.group-header:hover .group-icons {
  color: var(--text-main);
}

.group-icons {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  color: var(--text-low);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.chevron-icon {
  width: 14px;
  height: 14px;
  transition: transform 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275);
  transform: rotate(-90deg);
}

.group-icons.is-expanded {
  color: var(--text-main);
}

.group-icons.is-expanded .chevron-icon {
  transform: rotate(0deg);
}

.expand-enter-active,
.expand-leave-active {
  transition: all 0.3s ease;
  max-height: 500px;
  overflow: hidden;
}

.expand-enter-from,
.expand-leave-to {
  max-height: 0;
  opacity: 0;
  transform: translateY(-5px);
}

.group-items {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding-left: 4px;
  border-left: 1px solid var(--border-color);
  margin-left: 4px;
}

.history-item {
  display: flex;
  align-items: center;
  gap: 10px;
  cursor: pointer;
  padding: 4px 6px;
  border-radius: 4px;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
  overflow: hidden;
}

.history-item:hover {
  background: var(--hover-bg);
  padding-left: 8px;
}

.history-item.is-active {
  background: var(--tab-active-bg);
  border-left: 2px solid var(--text-main);
  padding-left: 6px;
}

.history-item.is-active .item-title {
  color: var(--text-main);
}

.item-time {
  font-size: var(--panel-font-size-small);
  color: var(--text-low);
  font-family: monospace;
  width: 35px;
  flex-shrink: 0;
  transition: color 0.2s;
}

.history-item:hover .item-time {
  color: var(--text-secondary);
}

.item-title {
  flex: 1;
  font-size: var(--panel-font-size-item);
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  transition: color 0.2s;
}

.history-item:hover .item-title {
  color: var(--text-main);
}

.item-arrow {
  width: 12px;
  height: 12px;
  color: var(--text-main);
  opacity: 0;
  transform: translateX(-10px);
  transition: all 0.3s cubic-bezier(0.175, 0.885, 0.32, 1.275);
}

.history-item:hover .item-arrow,
.history-item.is-active .item-arrow {
  opacity: 1;
  transform: translateX(0);
}

.scrollbar-hide::-webkit-scrollbar {
  display: none;
}

.scrollbar-hide {
  -ms-overflow-style: none;
  scrollbar-width: none;
}
</style>
