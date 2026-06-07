<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import { useI18n } from '../composables/useI18n'

const { t } = useI18n()

interface NoteRecord {
  id: string
  title: string
  content: string
  updated_at: string
}

interface SearchNoteRecord extends NoteRecord {
  plainContent: string
}

interface SearchResultRecord extends SearchNoteRecord {
  matchedField: 'title' | 'content'
  titleHtml: string
  previewHtml: string
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
const searchQuery = ref('')
const searchMode = ref<'title' | 'content'>('title')
const searchInputRef = ref<HTMLInputElement | null>(null)

const fetchNotes = async (): Promise<void> => {
  const result = await window.api.listNotes()
  allNotes.value = result as NoteRecord[]
}

const escapeHtml = (value: string): string =>
  value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')

const escapeRegExp = (value: string): string => value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')

const highlightKeyword = (value: string, keyword: string): string => {
  const escapedValue = escapeHtml(value)
  if (!keyword) return escapedValue

  const pattern = new RegExp(`(${escapeRegExp(keyword)})`, 'ig')
  return escapedValue.replace(pattern, '<mark>$1</mark>')
}

const createSnippet = (content: string, keyword: string): string => {
  const normalized = content.replace(/\s+/g, ' ').trim()
  if (!normalized) return ''
  if (!keyword) return normalized.slice(0, 110)

  const lowerContent = normalized.toLowerCase()
  const lowerKeyword = keyword.toLowerCase()
  const matchIndex = lowerContent.indexOf(lowerKeyword)

  if (matchIndex < 0) return normalized.slice(0, 110)

  const padding = 26
  const start = Math.max(0, matchIndex - padding)
  const end = Math.min(normalized.length, matchIndex + keyword.length + 54)
  const prefix = start > 0 ? '...' : ''
  const suffix = end < normalized.length ? '...' : ''
  return `${prefix}${normalized.slice(start, end)}${suffix}`
}

const formatUpdatedAt = (value: string): string => {
  if (!value) return ''
  const [date = '', time = ''] = value.split(' ')
  return time ? `${date} ${time.slice(0, 5)}` : date
}

const extractPlainText = (content: string): string => {
  if (!content) return ''
  if (!/<\/?[a-z][\s\S]*>/i.test(content)) return content.replace(/\s+/g, ' ').trim()

  const doc = new DOMParser().parseFromString(content, 'text/html')
  return (doc.body.textContent ?? '').replace(/\s+/g, ' ').trim()
}

watch(
  () => props.isOpen,
  (open) => {
    if (!open) return

    fetchNotes()
    nextTick(() => searchInputRef.value?.focus())
  },
  { immediate: true }
)

defineExpose({
  refresh: fetchNotes
})

const searchableNotes = computed<SearchNoteRecord[]>(() => {
  return allNotes.value.map((rec) => ({
    ...rec,
    plainContent: extractPlainText(rec.content)
  }))
})

const filteredNotes = computed<SearchResultRecord[]>(() => {
  const keyword = searchQuery.value.trim().toLowerCase()
  const mode = searchMode.value

  return searchableNotes.value
    .filter((rec) => {
      if (!keyword) return true
      const source = mode === 'title' ? rec.title : rec.plainContent
      return (source || '').toLowerCase().includes(keyword)
    })
    .map((rec) => {
      const previewSource = mode === 'content' ? rec.plainContent : rec.title || rec.plainContent
      const previewText = createSnippet(previewSource, keyword)

      return {
        ...rec,
        matchedField: mode,
        titleHtml: highlightKeyword(rec.title || t('editor.untitled'), keyword),
        previewHtml: highlightKeyword(previewText || rec.plainContent || t('history.searchEmpty'), keyword)
      }
    })
})

const resultSummary = computed(() => `${filteredNotes.value.length}/${searchableNotes.value.length}`)

const clearSearch = (): void => {
  searchQuery.value = ''
  nextTick(() => searchInputRef.value?.focus())
}
</script>

<template>
  <aside
    class="note-search scrollbar-hide"
    :class="{ 'is-open': isOpen }"
    :style="{ width: isOpen ? (width ? width + 'px' : '160px') : '0' }"
  >
    <div class="search-container" :style="{ width: width ? width + 'px' : '160px' }">
      <div class="search-bar">
        <div class="search-panel-head">
          <div class="search-title-block">
            <div class="search-title">{{ t('sidebar.search') }}</div>
            <div class="search-meta">{{ resultSummary }}</div>
          </div>
          <div class="mode-switch">
            <button
              type="button"
              class="mode-btn"
              :class="{ active: searchMode === 'title' }"
              @click="searchMode = 'title'"
            >
              {{ t('history.searchTitle') }}
            </button>
            <button
              type="button"
              class="mode-btn"
              :class="{ active: searchMode === 'content' }"
              @click="searchMode = 'content'"
            >
              {{ t('history.searchContent') }}
            </button>
          </div>
        </div>

        <div class="search-input-wrap">
          <input
            ref="searchInputRef"
            v-model="searchQuery"
            type="text"
            class="search-input"
            :placeholder="t('history.searchPlaceholder')"
          />
          <button v-if="searchQuery" type="button" class="clear-btn" @click="clearSearch">×</button>
        </div>
      </div>

      <div v-if="filteredNotes.length === 0" class="empty-state">
        <div class="empty-title">{{ t('history.searchEmpty') }}</div>
        <div class="empty-hint">{{ searchQuery ? searchQuery : t('history.searchPlaceholder') }}</div>
      </div>

      <div v-else class="results-list">
        <div
          v-for="item in filteredNotes"
          :key="item.id"
          class="result-item"
          :class="{ 'is-active': item.id === activeNoteId }"
          @click="emit('select-note', { id: item.id, title: item.title, content: item.content })"
        >
          <div class="result-copy">
            <div class="result-meta">
              <span class="result-tag">{{ item.matchedField === 'title' ? t('history.searchTitle') : t('history.searchContent') }}</span>
              <span class="result-time">{{ formatUpdatedAt(item.updated_at) }}</span>
            </div>
            <div class="title-line" v-html="item.titleHtml"></div>
            <div class="content-line" v-html="item.previewHtml"></div>
          </div>
          <svg viewBox="0 0 24 24" class="result-arrow">
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
    </div>
  </aside>
</template>

<style scoped>
.note-search {
  width: 0;
  padding: 20px 0;
  overflow-y: auto;
  overflow-x: hidden;
  flex-shrink: 0;
}

.note-search.is-open {
  width: 160px;
}

.search-container {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 0 10px;
  box-sizing: border-box;
}

.search-bar {
  display: flex;
  flex-direction: column;
  gap: 10px;
  position: sticky;
  top: 0;
  z-index: 2;
  background: var(--bg-main);
  padding-bottom: 10px;
  border-bottom: 1px solid var(--border-color);
}

.search-panel-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.search-title-block {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.search-title {
  font-size: var(--panel-font-size-label);
  font-weight: 500;
  color: var(--text-main);
  letter-spacing: 0.02em;
}

.search-meta {
  font-size: var(--panel-font-size-small);
  color: var(--text-low);
}

.search-input-wrap {
  position: relative;
}

.search-input {
  width: 100%;
  height: 30px;
  border: 1px solid var(--border-color);
  border-radius: 9px;
  background: var(--hover-bg);
  color: var(--text-main);
  font-size: 12px;
  padding: 0 30px 0 10px;
  box-sizing: border-box;
  outline: none;
  transition: border-color 0.15s ease, box-shadow 0.15s ease, background 0.15s ease;
}

.search-input:focus {
  border-color: var(--border-active);
  box-shadow: 0 0 0 1px rgba(var(--accent-rgb), 0.12);
  background: var(--bg-main);
}

.search-input::placeholder {
  color: var(--text-low);
}

.clear-btn {
  position: absolute;
  top: 50%;
  right: 7px;
  transform: translateY(-50%);
  width: 18px;
  height: 18px;
  border: none;
  border-radius: 999px;
  background: rgba(var(--accent-rgb), 0.12);
  color: var(--text-secondary);
  cursor: pointer;
  font-size: 13px;
  line-height: 1;
}

.clear-btn:hover {
  background: rgba(var(--accent-rgb), 0.18);
  color: var(--text-main);
}

.mode-switch {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  border: 1px solid var(--border-color);
  border-radius: 999px;
  padding: 2px;
  width: fit-content;
  background: var(--hover-bg);
}

.mode-btn {
  height: 22px;
  min-width: 38px;
  border: none;
  border-radius: 999px;
  background: transparent;
  color: var(--text-low);
  font-size: 10px;
  cursor: pointer;
  padding: 0 8px;
  transition: background 0.15s ease, color 0.15s ease;
}

.mode-btn:hover {
  color: var(--text-secondary);
}

.mode-btn.active {
  background: rgba(var(--accent-rgb), 0.14);
  color: var(--text-main);
}

.results-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding-left: 4px;
  margin-left: 4px;
  border-left: 1px solid var(--border-color);
}

.result-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 6px 6px 6px 8px;
  border-radius: 6px;
  cursor: pointer;
  border-left: 2px solid transparent;
  background: transparent;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
  overflow: hidden;
}

.result-item:hover {
  background: var(--hover-bg);
  padding-left: 10px;
}

.result-item.is-active {
  border-left-color: var(--text-main);
  background: var(--tab-active-bg);
  padding-left: 8px;
}

.result-copy {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.result-meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.result-tag {
  display: inline-flex;
  align-items: center;
  height: 18px;
  padding: 0 7px;
  border-radius: 999px;
  background: rgba(var(--accent-rgb), 0.12);
  color: var(--text-secondary);
  font-size: 10px;
}

.result-time {
  font-size: var(--panel-font-size-small);
  color: var(--text-low);
  white-space: nowrap;
}

.title-line {
  font-size: var(--panel-font-size-item);
  color: var(--text-secondary);
  line-height: 1.4;
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 1;
  -webkit-box-orient: vertical;
}

.content-line {
  font-size: var(--panel-font-size-small);
  color: var(--text-low);
  line-height: 1.5;
  overflow: hidden;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}

.result-item:hover .title-line,
.result-item.is-active .title-line {
  color: var(--text-main);
}

.title-line :deep(mark),
.content-line :deep(mark) {
  background: rgba(var(--accent-rgb), 0.18);
  color: var(--text-main);
  padding: 0 2px;
  border-radius: 4px;
}

.result-arrow {
  width: 12px;
  height: 12px;
  color: var(--text-main);
  opacity: 0;
  transform: translateX(-10px);
  transition: all 0.3s cubic-bezier(0.175, 0.885, 0.32, 1.275);
  flex-shrink: 0;
}

.result-item:hover .result-arrow,
.result-item.is-active .result-arrow {
  opacity: 1;
  transform: translateX(0);
}

.empty-state {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 18px 10px;
  text-align: center;
  border: 1px dashed var(--border-color);
  border-radius: 10px;
  color: var(--text-low);
  background: var(--hover-bg);
}

.empty-title {
  font-size: 12px;
  color: var(--text-secondary);
}

.empty-hint {
  font-size: 11px;
  color: var(--text-low);
  word-break: break-word;
}

.scrollbar-hide::-webkit-scrollbar {
  display: none;
}

.scrollbar-hide {
  -ms-overflow-style: none;
  scrollbar-width: none;
}
</style>
