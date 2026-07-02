<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted, computed, nextTick } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import SideBar from './components/SideBar.vue'
import NoteOutline from './components/NoteOutline.vue'
import NoteSearch from './components/NoteSearch.vue'
import NoteHistory from './components/NoteHistory.vue'
import NoteSettings from './components/NoteSettings.vue'
import TitleBar from './components/TitleBar.vue'
import NoteEditor from './components/NoteEditor.vue'
import FooterBar from './components/FooterBar.vue'
import WindowControls from './components/WindowControls.vue'
import ResizeSensors from './components/ResizeSensors.vue'
import { useWindowResize } from './composables/useWindowResize'
import { useWindowAnimate } from './composables/useWindowAnimate'
import { useI18n, type Language } from './composables/useI18n'
import * as api from './api'

// --- 国际化 ---
const { language, t, setLanguage } = useI18n()

// --- 接口定义 ---
interface Heading {
  level: number
  text: string
  pos: number
}

interface Note {
  id: string
  title: string
  content: string
}

interface EditorInstance {
  scrollToHeading: (pos: number) => void
  setEditorContent: (content: string) => void
}

interface HistoryInstance {
  refresh: () => void
}

// --- 状态管理 ---
const isPinned = ref(false)
const activeTab = ref('none')
const headings = ref<Heading[]>([])
const editorRef = ref<EditorInstance | null>(null)
const historyRef = ref<HistoryInstance | null>(null)
const searchRef = ref<HistoryInstance | null>(null)
const notes = ref<Note[]>([])
const activeNoteId = ref('')
const selectedText = ref('')
const activeSettingsTab = ref('general')
const theme = ref('dark')
const DEFAULT_ACCENT_COLOR = '#13b2ed'
const accentColor = ref(DEFAULT_ACCENT_COLOR)
const isAutoLaunch = ref(false)
const closeAction = ref<'quit' | 'hide'>('hide')
const shortcutInput = ref<HTMLInputElement | null>(null)
const storagePath = ref('')
const isApplyingStoragePath = ref(false)
const isMigratingLegacyDb = ref(false)
const migrationMessage = ref('')

// --- Shortcuts State ---
const defaultShortcuts = {
  toggle_outline: 'Ctrl+L',
  toggle_history: 'Ctrl+H',
  toggle_pin: 'Ctrl+P',
  hide_window: 'Esc',
  show_window: 'Ctrl+J',
  new_note: 'Ctrl+N',
  toggle_bold: 'Ctrl+B',
  delete_line: 'Ctrl+Shift+K',
  delete_word: 'Ctrl+W',
  table_add_row_below: 'Ctrl+Enter',
  table_delete_row: 'Ctrl+Backspace'
}
const customShortcuts = ref({ ...defaultShortcuts })
const globalFlags = ref<Record<string, boolean>>({
  toggle_outline: false,
  toggle_history: false,
  toggle_pin: false,
  hide_window: false,
  show_window: true,
  new_note: false,
  toggle_bold: false,
  delete_line: false,
  delete_word: false,
  table_add_row_below: false,
  table_delete_row: false
})
const recordingShortcut = ref<string | null>(null)

// --- 窗口管理状态 ---
const { startResize } = useWindowResize()
const { animateResize } = useWindowAnimate()
const outlineWidth = ref(160)
const searchWidth = ref(160)
const historyWidth = ref(160)
const settingsWidth = ref(160)
const panelTab = ref('none')
const isPanelOpen = ref(false)
const isResizingPanel = ref(false)
const isResetting = ref(false)
let panelTransitionTimer: number | null = null
let panelTransitionId = 0

const PANEL_OPEN_ANIMATION_MS = 200
const PANEL_CLOSE_ANIMATION_MS = 130
const PANEL_TABS = ['outline', 'search', 'history', 'settings']

const isPanelTab = (tab: string): boolean => PANEL_TABS.includes(tab)

// 获取当前侧边面板宽度
const getPanelWidth = (tab: string): number => {
  if (tab === 'outline') return outlineWidth.value
  if (tab === 'search') return searchWidth.value
  if (tab === 'history') return historyWidth.value
  if (tab === 'settings') return settingsWidth.value
  return 0
}

const clearPanelTransitionTimer = (): void => {
  if (panelTransitionTimer !== null) {
    window.clearTimeout(panelTransitionTimer)
    panelTransitionTimer = null
  }
}

const preventNativeContextMenu = (e: MouseEvent): void => {
  e.preventDefault()
}

// --- 逻辑处理 ---

/**
 * 初始化：从数据库加载最新一条笔记
 */
onMounted(async () => {
  try {
    const savedNotes = await api.listNotes()
    if (savedNotes.length > 0) {
      notes.value = [savedNotes[0]]
      activeNoteId.value = savedNotes[0].id
    } else {
      addNote()
    }
  } catch (e) {
    console.error('Failed to load notes:', e)
    addNote()
  }

  // 加载快捷键设置
  let settings: Record<string, string> = {}
  try {
    settings = await api.getSettings()
    if (settings.theme) {
      theme.value = settings.theme
    }

    // 加载主题色设置
    if (settings.accentColor) {
      accentColor.value = settings.accentColor
    }
    applyAccentColor(accentColor.value)

    // 加载语言设置
    if (settings.language) {
      setLanguage(settings.language as Language)
    }
  } catch (e) {
    console.error('Failed to load settings:', e)
  }

  try {
    storagePath.value = await api.getStoragePath()
  } catch (e) {
    console.error('Failed to load storage path:', e)
  }

  // 加载开机自启设置
  try {
    isAutoLaunch.value = await api.getAutoLaunch()
  } catch (e) {
    console.error('Failed to load auto launch setting:', e)
  }

  // 加载关闭动作设置
  if (settings.closeAction) {
    closeAction.value = settings.closeAction as 'quit' | 'hide'
  }

  Object.keys(defaultShortcuts).forEach((key) => {
    if (settings[key]) {
      (customShortcuts.value as Record<string, string>)[key] = settings[key]
    }
    if (settings[`${key}_is_global`] !== undefined) {
      (globalFlags.value as Record<string, boolean>)[key] = settings[`${key}_is_global`] === 'true'
    }
  })

  // 全局快捷键监听 (仅针对非全局配置的本窗口监听)
  window.addEventListener('keydown', handleGlobalKeyDown, true)
  window.addEventListener('contextmenu', preventNativeContextMenu, true)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleGlobalKeyDown, true)
  window.removeEventListener('contextmenu', preventNativeContextMenu, true)
  clearPanelTransitionTimer()
})

/**
 * 执行快捷键指令 (复用逻辑)
 */
const executeShortcutCommand = async (command: string): Promise<void> => {
  if (command === 'toggle_pin') {
    await togglePin()
  } else if (command === 'toggle_outline') {
    handleActiveTabChange(activeTab.value === 'outline' ? 'none' : 'outline')
  } else if (command === 'toggle_history') {
    handleActiveTabChange(activeTab.value === 'history' ? 'none' : 'history')
  } else if (command === 'hide_window') {
    await api.hideWindow()
  } else if (command === 'show_window') {
    await api.showWindow()
  } else if (command === 'new_note') {
    addNote()
  }
}

const APP_LEVEL_SHORTCUTS = new Set(['toggle_pin', 'toggle_outline', 'toggle_history', 'hide_window', 'new_note'])

/**
 * 将 KeyboardEvent 转换为自定义格式字符串
 */
const getShortcutString = (e: KeyboardEvent): string => {
  const keys: string[] = []
  if (e.ctrlKey) keys.push('Ctrl')
  if (e.altKey) keys.push('Alt')
  if (e.shiftKey) keys.push('Shift')
  if (e.metaKey) keys.push('Meta')
  
  // 转换按键名称
  let keyName = e.key
  if (keyName === ' ') keyName = 'Space'
  if (keyName === 'Escape') keyName = 'Esc'
  if (keyName === 'Control') return keys.join('+') // 忽略单纯的修饰键
  if (keyName === 'Alt') return keys.join('+')
  if (keyName === 'Shift') return keys.join('+')
  if (keyName === 'Meta') return keys.join('+')
  
  // 对于字母键统一转大写，对于 F1-F12 等保持原样或格式化
  if (keyName.length === 1) {
    keys.push(keyName.toUpperCase())
  } else {
    keys.push(keyName)
  }
  
  return keys.join('+')
}

/**
 * 停止记录并保存
 */
const handleShortcutKeyDown = async (e: KeyboardEvent): Promise<void> => {
  if (!recordingShortcut.value) return

  e.preventDefault()
  e.stopPropagation()

  // 忽略单纯的修饰键
  if (['Control', 'Shift', 'Alt', 'Meta'].includes(e.key)) return

  const newShortcut = getShortcutString(e)
  const keyToUpdate = recordingShortcut.value
  if (keyToUpdate) {
    (customShortcuts.value as Record<string, string>)[keyToUpdate] = newShortcut
    recordingShortcut.value = null
    await api.saveSetting(keyToUpdate, newShortcut)

    // 如果修改的是全局快捷键 (show_window)，需要更新后端注册
    if (keyToUpdate === 'show_window') {
      try {
        await api.updateGlobalShortcut(newShortcut)
      } catch (err) {
        console.error('Failed to update global shortcut:', err)
      }
    }
  } else {
    recordingShortcut.value = null
  }
}

/**
 * 新建笔记：仅内存级操作，直到输入内容才持久化
 */
const addNote = (): void => {
  const newNote: Note = {
    id: crypto.randomUUID(),
    title: t('editor.untitled'),
    content: ''
  }
  notes.value.push(newNote)
  activeNoteId.value = newNote.id
}

/**
 * 助手函数：判断内容是否为空（忽略空 HTML 标签）
 */
const isNoteEmpty = (content: string): boolean => {
  if (!content) return true
  const cleanContent = content
    .replace(/[`*_#>\[\]()!|-]/g, '')
    .replace(/\s+/g, '')
    .trim()
  return cleanContent === ''
}

/**
 * 核心快捷键处理逻辑：本地拦截与执行
 */
const handleGlobalKeyDown = (e: KeyboardEvent): void => {
  if (recordingShortcut.value) return

  const shortcutStr = getShortcutString(e)

  // 查找匹配的快捷键，阻止所有配置快捷键的默认行为（防止浏览器默认动作如下载面板）
  for (const [command, mapping] of Object.entries(customShortcuts.value)) {
    if (mapping === shortcutStr) {
      e.preventDefault()
      e.stopPropagation()
      e.returnValue = false
      // 仅对本地处理的快捷键执行命令，全局快捷键由后端处理
      if (APP_LEVEL_SHORTCUTS.has(command)) {
        void executeShortcutCommand(command)
      }
      break
    }
  }
}

/**
 * 开始录制快捷键
 */
const startRecording = (key: string): void => {
  recordingShortcut.value = key
  // 等待 DOM 更新后聚焦隐藏输入框
  setTimeout(() => {
    shortcutInput.value?.focus()
  }, 0)
}

/**
 * 关闭笔记并清理数据库中存在的空笔记
 */
const closeNote = async (id: string): Promise<void> => {
  const index = notes.value.findIndex((n) => n.id === id)
  if (index === -1) return
  
  const noteToDelete = notes.value[index]

  // 如果内容为空，关闭时从数据库中彻底移除
  if (isNoteEmpty(noteToDelete.content)) {
    await api.deleteNote(id)
    historyRef.value?.refresh?.()
  }

  notes.value.splice(index, 1)
  
  // 如果所有标签都关闭了，自动创建一个新标签并聚焦
  if (notes.value.length === 0) {
    addNote()
    return
  }

  // 处理 activeNoteId
  if (activeNoteId.value === id) {
    activeNoteId.value = notes.value[Math.max(0, index - 1)].id
  }
}

/**
 * 更新内存中的笔记内容
 */
const updateNoteContent = (content: string): void => {
  const note = notes.value.find((n) => n.id === activeNoteId.value)
  if (note) note.content = content
}

const updateSelectedText = (content: string): void => {
  selectedText.value = content
}

/**
 * 重置窗口到默认尺寸 (502x350)
 */
const resetSize = (): void => {
  clearPanelTransitionTimer()
  panelTransitionId += 1
  isResetting.value = true
  outlineWidth.value = 160
  searchWidth.value = 160
  historyWidth.value = 160
  settingsWidth.value = 160

  if (panelTab.value !== 'none') {
    const currentX = window.screenX
    const currentY = window.screenY
    const panelWidth = getPanelWidth(panelTab.value)
    
    activeTab.value = 'none'
    panelTab.value = 'none'
    isPanelOpen.value = false
    animateResize(502, 350, currentX + panelWidth, currentY)
  } else {
    activeTab.value = 'none'
    panelTab.value = 'none'
    isPanelOpen.value = false
    animateResize(502, 350, window.screenX, window.screenY)
  }

  setTimeout(() => {
    isResetting.value = false
  }, 300)
}

const handleActiveTabChange = async (nextTab: string): Promise<void> => {
  if (isResetting.value) return

  clearPanelTransitionTimer()

  if (!isPanelTab(nextTab)) {
    activeTab.value = 'none'

    if (!isPanelTab(panelTab.value)) {
      isPanelOpen.value = false
      return
    }

    const closingPanelWidth = getPanelWidth(panelTab.value)
    const transitionId = ++panelTransitionId
    const targetWidth = Math.max(320, window.outerWidth - closingPanelWidth)

    isPanelOpen.value = false
    void animateResize(
      targetWidth,
      window.outerHeight,
      window.screenX + closingPanelWidth,
      window.screenY
    )

    if (panelTransitionId !== transitionId) return
    panelTransitionTimer = window.setTimeout(() => {
      if (panelTransitionId !== transitionId) return
      panelTab.value = 'none'
      panelTransitionTimer = null
    }, PANEL_CLOSE_ANIMATION_MS)
    return
  }

  const currentPanelWidth = isPanelTab(panelTab.value) ? getPanelWidth(panelTab.value) : 0
  const nextPanelWidth = getPanelWidth(nextTab)
  const widthDelta = nextPanelWidth - currentPanelWidth
  const targetWindowWidth = window.outerWidth + widthDelta

  activeTab.value = nextTab
  panelTab.value = nextTab

  if (widthDelta !== 0) {
    await animateResize(
      targetWindowWidth,
      window.outerHeight,
      window.screenX - widthDelta,
      window.screenY
    )
  }

  const transitionId = ++panelTransitionId
  await nextTick()
  requestAnimationFrame(() => {
    if (panelTransitionId === transitionId) {
      isPanelOpen.value = true
    }
  })
}

/**
 * 拖动调整侧边面板宽度逻辑
 */
const startPanelResize = (e: MouseEvent): void => {
  isResizingPanel.value = true
  const startX = e.clientX
  const currentTab = panelTab.value
  const startPanelWidth = getPanelWidth(currentTab)
  let rafId: number | null = null

  const onMouseMove = (moveEvent: MouseEvent): void => {
    if (!isResizingPanel.value) return

    // 使用 requestAnimationFrame 节流更新，避免频繁重绘
    if (rafId !== null) return

    rafId = requestAnimationFrame(() => {
      rafId = null
      const deltaX = moveEvent.clientX - startX
      const newWidth = Math.max(120, Math.min(350, startPanelWidth + deltaX))

      if (currentTab === 'outline') outlineWidth.value = newWidth
      else if (currentTab === 'search') searchWidth.value = newWidth
      else if (currentTab === 'history') historyWidth.value = newWidth
      else if (currentTab === 'settings') settingsWidth.value = newWidth
    })
  }

  const onMouseUp = (): void => {
    isResizingPanel.value = false
    if (rafId !== null) {
      cancelAnimationFrame(rafId)
      rafId = null
    }
    document.removeEventListener('mousemove', onMouseMove)
    document.removeEventListener('mouseup', onMouseUp)
  }

  document.addEventListener('mousemove', onMouseMove)
  document.addEventListener('mouseup', onMouseUp)
}

/**
 * 置顶切换
 */
const togglePin = async (): Promise<void> => {
  const nextPinned = !isPinned.value
  try {
    await api.setAlwaysOnTop(nextPinned)
    isPinned.value = nextPinned
  } catch (e) {
    console.error('Failed to toggle always on top:', e)
  }
}

// --- 保存与状态同步 ---
const lastSavedTime = ref('')

const handleSaveStart = (): void => {}
const handleSaveSuccess = (time: string): void => {
  lastSavedTime.value = time
  historyRef.value?.refresh?.()
  searchRef.value?.refresh?.()
}
const handleUpdateHeadings = (newHeadings: Heading[]): void => {
  headings.value = newHeadings
}
const handleSelectHeading = (pos: number): void => {
  editorRef.value?.scrollToHeading(pos)
}

/**
 * 从列表中选择笔记
 */
const handleSelectNote = (note: { id: string; title: string; content: string }): void => {
  selectedText.value = ''
  const existingNote = notes.value.find((n) => n.id === note.id)
  if (existingNote) {
    activeNoteId.value = note.id
  } else {
    // 如果该笔记未在当前标签页中，则添加它
    notes.value.push({
      id: note.id,
      title: note.title,
      content: note.content
    })
    activeNoteId.value = note.id
  }
}

const switchActiveNote = (id: string): void => {
  selectedText.value = ''
  activeNoteId.value = id
}

const renameNote = (id: string, newTitle: string): void => {
  const note = notes.value.find((n) => n.id === id)
  if (note) {
    note.title = newTitle

    // 只有在内容不为空时才执行数据库保存
    // 如果是空白笔记重命名，仅更新内存状态，直到输入内容才持久化
    if (!isNoteEmpty(note.content)) {
      api.saveNote(note.id, newTitle, note.content).then(() => {
        handleSaveSuccess(new Date().toLocaleTimeString('zh-CN', { hour12: false }))
      })
    }
  }
}

/**
 * 关闭全部标签页
 */
const closeAllNotes = async (): Promise<void> => {
  // 必须使用复制的数组进行循环，因为 closeNote 会修改原有数组
  const ids = notes.value.map((n) => n.id)
  for (const id of ids) {
    await closeNote(id)
  }
}

/**
 * 切换开机自启
 */
const toggleAutoLaunch = async (): Promise<void> => {
  isAutoLaunch.value = !isAutoLaunch.value
  await api.setAutoLaunch(isAutoLaunch.value)
}

/**
 * 切换关闭动作
 */
const toggleCloseAction = async (): Promise<void> => {
  closeAction.value = closeAction.value === 'hide' ? 'quit' : 'hide'
  await api.saveSetting('closeAction', closeAction.value)
}

const migrateToStoragePath = async (nextPath: string): Promise<void> => {
  if (!nextPath || nextPath === storagePath.value || isApplyingStoragePath.value) return

  isApplyingStoragePath.value = true
  try {
    const result = await api.setNotesDirectory(nextPath)
    storagePath.value = result.path

    const savedNotes = await api.listNotes()
    notes.value = savedNotes.length > 0 ? [savedNotes[0]] : []
    activeNoteId.value = savedNotes[0]?.id || ''
    if (notes.value.length === 0) addNote()
    historyRef.value?.refresh?.()
    searchRef.value?.refresh?.()
  } catch (e) {
    console.error('Failed to update storage path:', e)
  } finally {
    isApplyingStoragePath.value = false
  }
}

const chooseStoragePath = async (): Promise<void> => {
  if (isApplyingStoragePath.value) return

  try {
    const selected = await open({
      directory: true,
      multiple: false,
      defaultPath: storagePath.value || undefined,
      title: t('settings.storagePath.dialogTitle')
    })

    if (typeof selected === 'string') {
      await migrateToStoragePath(selected)
    }
  } catch (e) {
    console.error('Failed to choose storage path:', e)
  }
}

const refreshNotesAfterMigration = async (): Promise<void> => {
  const savedNotes = await api.listNotes()
  notes.value = savedNotes.length > 0 ? [savedNotes[0]] : []
  activeNoteId.value = savedNotes[0]?.id || ''
  if (notes.value.length === 0) addNote()
  historyRef.value?.refresh?.()
  searchRef.value?.refresh?.()
}

const chooseLegacyDatabase = async (): Promise<void> => {
  if (isMigratingLegacyDb.value) return

  try {
    const selected = await open({
      directory: false,
      multiple: false,
      filters: [
        { name: 'SQLite Database', extensions: ['db', 'sqlite', 'sqlite3'] }
      ],
      title: t('settings.legacyMigration.dialogTitle')
    })

    if (typeof selected !== 'string') return

    isMigratingLegacyDb.value = true
    migrationMessage.value = ''
    const result = await api.migrateLegacyDatabase(selected)
    await refreshNotesAfterMigration()
    migrationMessage.value = t('settings.legacyMigration.done').replace('{count}', String(result.migrated))
  } catch (e) {
    console.error('Failed to migrate legacy database:', e)
    migrationMessage.value = t('settings.legacyMigration.failed')
  } finally {
    isMigratingLegacyDb.value = false
  }
}

/**
 * 切换主题
 */
const toggleTheme = async (): Promise<void> => {
  theme.value = theme.value === 'dark' ? 'light' : 'dark'
  await api.saveSetting('theme', theme.value)
}

/**
 * 将 hex 颜色转为 r,g,b 字符串
 */
const hexToRgb = (hex: string): string => {
  const r = parseInt(hex.slice(1, 3), 16)
  const g = parseInt(hex.slice(3, 5), 16)
  const b = parseInt(hex.slice(5, 7), 16)
  return `${r}, ${g}, ${b}`
}

/**
 * 将 hex 颜色转为 CSS filter 字符串（用于 SVG 图标着色）
 */
const hexToFilter = (hex: string): string => {
  const r = parseInt(hex.slice(1, 3), 16) / 255
  const g = parseInt(hex.slice(3, 5), 16) / 255
  const b = parseInt(hex.slice(5, 7), 16) / 255

  const max = Math.max(r, g, b)
  const min = Math.min(r, g, b)
  const l = (max + min) / 2

  let h = 0
  let s = 0
  if (max !== min) {
    const d = max - min
    s = l > 0.5 ? d / (2 - max - min) : d / (max + min)
    switch (max) {
      case r: h = ((g - b) / d + (g < b ? 6 : 0)) / 6; break
      case g: h = ((b - r) / d + 2) / 6; break
      case b: h = ((r - g) / d + 4) / 6; break
    }
  }

  const hueRotate = Math.round(h * 360 - 35)
  const saturate = Math.round(s * 100 * 2.5)
  const brightness = Math.round(l * 200 + 20)

  return `invert(48%) sepia(90%) saturate(${saturate}%) hue-rotate(${hueRotate}deg) brightness(${brightness}%) contrast(90%)`
}

/**
 * 应用主题色到 CSS 变量
 */
const applyAccentColor = (color: string): void => {
  const rgb = hexToRgb(color)
  const filter = hexToFilter(color)
  document.documentElement.style.setProperty('--accent-color', color)
  document.documentElement.style.setProperty('--accent-rgb', rgb)
  document.documentElement.style.setProperty('--icon-active-filter', filter)
}

/**
 * 更改主题色
 */
const changeAccentColor = async (e: Event): Promise<void> => {
  const target = e.target as HTMLInputElement
  accentColor.value = target.value
  applyAccentColor(target.value)
  await api.saveSetting('accentColor', target.value)
}

/**
 * 重置主题色为默认值
 */
const resetAccentColor = async (): Promise<void> => {
  accentColor.value = DEFAULT_ACCENT_COLOR
  applyAccentColor(DEFAULT_ACCENT_COLOR)
  await api.saveSetting('accentColor', DEFAULT_ACCENT_COLOR)
}

/**
 * 切换语言
 */
const toggleLanguage = async (): Promise<void> => {
  const newLang: Language = language.value === 'en' ? 'zh' : 'en'
  setLanguage(newLang)
  await api.saveSetting('language', newLang)
}

const activeNote = computed(() => notes.value.find((n) => n.id === activeNoteId.value))

const countWords = (content: string): number => {
  const plainText = content
    .replace(/<[^>]*>/g, ' ')
    .replace(/&nbsp;/gi, ' ')
    .replace(/&[a-z\d#]+;/gi, ' ')
    .replace(/[`*_~>#\[\]()!|-]/g, ' ')
    .replace(/\s+/g, ' ')
    .trim()

  if (!plainText) return 0

  const cjkMatches = plainText.match(/[\p{Script=Han}\p{Script=Hiragana}\p{Script=Katakana}\p{Script=Hangul}]/gu) || []
  const latinMatches = plainText.match(/[A-Za-z0-9]+(?:[.'_-][A-Za-z0-9]+)*/g) || []

  return cjkMatches.length + latinMatches.length
}

const activeWordCount = computed(() => countWords(activeNote.value?.content || ''))
const selectedWordCount = computed(() => countWords(selectedText.value))

// 同步主题类到 html 元素，使 Teleport 到 body 的元素也能继承主题变量
watch(theme, (newTheme) => {
  document.documentElement.classList.toggle('theme-light', newTheme === 'light')
}, { immediate: true })
</script>

<template>
    <div class="wrapper glass border-glow" :class="`theme-${theme}`">
    <!-- 背景/边框感应区 -->
    <ResizeSensors :on-resize="startResize" :on-reset="resetSize" />

    <!-- 左侧侧边栏及其内部面板 -->
    <div class="sidebar-wrapper" :class="{ collapsed: panelTab === 'none' }">
      <SideBar :active-tab="activeTab" :t="t" @update:active-tab="handleActiveTabChange" @reset-size="resetSize" />

      <!-- 分隔线 -->
      <div class="tab-separator" :class="{ visible: panelTab !== 'none' && isPanelOpen }"></div>

      <!-- 面板容器：显示大纲或历史 -->
      <div
        class="sidebar-panels"
        :class="{ open: isPanelOpen, resizing: isResizingPanel }"
        :style="{
          width: panelTab !== 'none' && isPanelOpen ? getPanelWidth(panelTab) + 'px' : '0px',
          '--panel-duration': isPanelOpen ? PANEL_OPEN_ANIMATION_MS + 'ms' : PANEL_CLOSE_ANIMATION_MS + 'ms',
          '--panel-fade-duration': isPanelOpen ? '140ms' : '90ms'
        }"
      >
        <Transition name="panel-fade">
          <NoteOutline
            v-if="panelTab === 'outline'"
            :key="'outline'"
            :headings="headings"
            :is-open="panelTab === 'outline'"
            :width="outlineWidth"
            @select="handleSelectHeading"
          />
          
          <NoteSearch
            v-else-if="panelTab === 'search'"
            :key="'search'"
            ref="searchRef"
            :active-note-id="activeNoteId"
            :is-open="panelTab === 'search'"
            :width="searchWidth"
            @select-note="handleSelectNote"
          />

          <NoteHistory
            v-else-if="panelTab === 'history'"
            :key="'history'"
            ref="historyRef"
            :active-note-id="activeNoteId"
            :is-open="panelTab === 'history'"
            :width="historyWidth"
            @select-note="handleSelectNote"
          />

          <NoteSettings
            v-else-if="panelTab === 'settings'"
            :key="'settings'"
            v-model:active-settings-tab="activeSettingsTab"
            :is-open="panelTab === 'settings'"
            :width="settingsWidth"
          />
        </Transition>
      </div>

      <!-- 拖拽调节区 -->
      <div
        v-if="isPanelOpen && activeTab !== 'none'"
        class="outline-resizer"
        :style="{ left: panelTab !== 'none' ? 52 + getPanelWidth(panelTab) + 'px' : '52px' }"
        @mousedown="startPanelResize"
      ></div>
    </div>

    <!-- 右侧容器 -->
    <main class="main-container">
      <template v-if="activeTab !== 'settings'">
        <TitleBar
          :is-pinned="isPinned"
          :notes="notes"
          :active-note-id="activeNoteId"
          :t="t"
          @toggle-pin="togglePin"
          @add-note="addNote"
          @close-all="closeAllNotes"
          @close-note="closeNote"
          @switch-note="switchActiveNote"
          @rename-note="renameNote"
        />
        <NoteEditor
          v-if="activeNote"
          :key="activeNoteId"
          ref="editorRef"
          :note-id="activeNoteId"
          :title="activeNote?.title || t('editor.untitled')"
          :initial-content="activeNote?.content || ''"
          :shortcut-delete-line="customShortcuts.delete_line"
          :shortcut-delete-word="customShortcuts.delete_word"
          :shortcut-table-add-row-below="customShortcuts.table_add_row_below"
          :shortcut-table-delete-row="customShortcuts.table_delete_row"
          :shortcut-toggle-bold="customShortcuts.toggle_bold"
          :t="t"
          @save-start="handleSaveStart"
          @save-success="handleSaveSuccess"
          @update-content="updateNoteContent"
          @update-selection="updateSelectedText"
          @update-headings="handleUpdateHeadings"
        />
        <FooterBar
          :on-resize="startResize"
          :on-reset="resetSize"
          :last-saved-time="lastSavedTime"
          :selected-word-count="selectedWordCount"
          :word-count="activeWordCount"
          :t="t"
        />
      </template>

      <div v-else class="settings-view">
        <header class="settings-header">
          <div class="settings-title">
            {{
              activeSettingsTab === 'general' ? t('settings.general') : activeSettingsTab === 'shortcuts' ? t('settings.shortcuts') : t('settings.about')
            }}
          </div>
          <WindowControls :is-pinned="isPinned" :t="t" @toggle-pin="togglePin" />
        </header>

        <div v-if="activeSettingsTab === 'general'" class="settings-content general-content">
          <div class="shortcut-list">
            <div class="shortcut-item" @click="toggleTheme">
              <span class="shortcut-label">{{ t('settings.theme') }}</span>
              <div class="shortcut-group">
                <span class="global-badge">{{ theme === 'dark' ? t('settings.theme.dark') : t('settings.theme.light') }}</span>
              </div>
            </div>
            <div class="shortcut-item accent-color-item">
              <span class="shortcut-label">{{ t('settings.accentColor') }}</span>
              <div class="shortcut-group">
                <label class="color-swatch" :style="{ background: accentColor, borderColor: accentColor }">
                  <input type="color" :value="accentColor" @input="changeAccentColor" />
                </label>
                <span 
                  v-if="accentColor !== DEFAULT_ACCENT_COLOR" 
                  class="global-badge reset-color-btn" 
                  @click="resetAccentColor"
                >
                  {{ t('settings.accentColor.reset') }}
                </span>
              </div>
            </div>
            <div class="shortcut-item" @click="toggleLanguage">
              <span class="shortcut-label">{{ t('settings.language') }}</span>
              <div class="shortcut-group">
                <span class="global-badge">{{ language === 'en' ? t('settings.language.en') : t('settings.language.zh') }}</span>
              </div>
            </div>
            <div class="shortcut-item" @click="toggleAutoLaunch">
              <span class="shortcut-label">{{ t('settings.autoLaunch') }}</span>
              <div class="shortcut-group">
                <span class="global-badge">{{ isAutoLaunch ? t('settings.autoLaunch.on') : t('settings.autoLaunch.off') }}</span>
              </div>
            </div>
            <div class="shortcut-item" @click="toggleCloseAction">
              <span class="shortcut-label">{{ t('settings.closeAction') }}</span>
              <div class="shortcut-group">
                <span class="global-badge">{{ closeAction === 'hide' ? t('settings.closeAction.hide') : t('settings.closeAction.quit') }}</span>
              </div>
            </div>
            <div class="storage-card">
              <div class="storage-card-head">
                <div class="storage-title-block">
                  <span class="shortcut-label">{{ t('settings.storagePath') }}</span>
                  <span class="storage-subtitle">{{ t('settings.storagePath.hint') }}</span>
                </div>
                <button class="storage-path-button" :disabled="isApplyingStoragePath" @click="chooseStoragePath">
                  {{ isApplyingStoragePath ? t('settings.storagePath.applying') : t('settings.storagePath.choose') }}
                </button>
              </div>
              <button class="storage-path-display" :disabled="isApplyingStoragePath" @click="chooseStoragePath">
                <span class="storage-path-text">{{ storagePath || t('settings.storagePath.empty') }}</span>
              </button>
              <div class="storage-migration-note">
                {{ t('settings.storagePath.migration') }}
              </div>
            </div>
            <div class="storage-card">
              <div class="storage-card-head">
                <div class="storage-title-block">
                  <span class="shortcut-label">{{ t('settings.legacyMigration') }}</span>
                  <span class="storage-subtitle">{{ t('settings.legacyMigration.hint') }}</span>
                </div>
                <button class="storage-path-button" :disabled="isMigratingLegacyDb" @click="chooseLegacyDatabase">
                  {{ isMigratingLegacyDb ? t('settings.legacyMigration.running') : t('settings.legacyMigration.choose') }}
                </button>
              </div>
              <div class="storage-migration-note">
                {{ t('settings.legacyMigration.keepDb') }}
              </div>
              <div v-if="migrationMessage" class="storage-result">
                {{ migrationMessage }}
              </div>
            </div>
          </div>
        </div>

        <div v-else-if="activeSettingsTab === 'shortcuts'" class="settings-content shortcuts-content">
          <div class="shortcut-list">
            <div
              class="shortcut-item"
              :class="{ recording: recordingShortcut === 'hide_window' }"
              @click="startRecording('hide_window')"
            >
              <span class="shortcut-label">{{ t('shortcut.hideWindow') }}</span>
              <div class="shortcut-keys">
                <kbd v-for="key in customShortcuts.hide_window.split('+')" :key="key">{{ key }}</kbd>
              </div>
              <div v-if="recordingShortcut === 'hide_window'" class="recording-overlay">{{ t('shortcut.recording') }}</div>
            </div>

            <div
              class="shortcut-item"
              :class="{ recording: recordingShortcut === 'show_window' }"
              @click="startRecording('show_window')"
            >
              <span class="shortcut-label">{{ t('shortcut.showWindow') }}</span>
              <div class="shortcut-group">
                <span class="global-badge">{{ t('shortcut.global') }}</span>
                <div class="shortcut-keys">
                  <kbd v-for="key in customShortcuts.show_window.split('+')" :key="key">{{ key }}</kbd>
                </div>
              </div>
              <div v-if="recordingShortcut === 'show_window'" class="recording-overlay">{{ t('shortcut.recording') }}</div>
            </div>

            <div
              class="shortcut-item"
              :class="{ recording: recordingShortcut === 'new_note' }"
              @click="startRecording('new_note')"
            >
              <span class="shortcut-label">{{ t('shortcut.newNote') }}</span>
              <div class="shortcut-keys">
                <kbd v-for="key in customShortcuts.new_note.split('+')" :key="key">{{ key }}</kbd>
              </div>
              <div v-if="recordingShortcut === 'new_note'" class="recording-overlay">{{ t('shortcut.recording') }}</div>
            </div>

            <div
              class="shortcut-item"
              :class="{ recording: recordingShortcut === 'toggle_bold' }"
              @click="startRecording('toggle_bold')"
            >
              <span class="shortcut-label">{{ t('shortcut.toggleBold') }}</span>
              <div class="shortcut-keys">
                <kbd v-for="key in customShortcuts.toggle_bold.split('+')" :key="key">{{ key }}</kbd>
              </div>
              <div v-if="recordingShortcut === 'toggle_bold'" class="recording-overlay">{{ t('shortcut.recording') }}</div>
            </div>

            <div
              class="shortcut-item"
              :class="{ recording: recordingShortcut === 'delete_line' }"
              @click="startRecording('delete_line')"
            >
              <span class="shortcut-label">{{ t('shortcut.deleteLine') }}</span>
              <div class="shortcut-keys">
                <kbd v-for="key in customShortcuts.delete_line.split('+')" :key="key">{{ key }}</kbd>
              </div>
              <div v-if="recordingShortcut === 'delete_line'" class="recording-overlay">{{ t('shortcut.recording') }}</div>
            </div>

            <div
              class="shortcut-item"
              :class="{ recording: recordingShortcut === 'table_add_row_below' }"
              @click="startRecording('table_add_row_below')"
            >
              <span class="shortcut-label">{{ t('shortcut.tableAddRowBelow') }}</span>
              <div class="shortcut-keys">
                <kbd v-for="key in customShortcuts.table_add_row_below.split('+')" :key="key">{{ key }}</kbd>
              </div>
              <div v-if="recordingShortcut === 'table_add_row_below'" class="recording-overlay">{{ t('shortcut.recording') }}</div>
            </div>

            <div
              class="shortcut-item"
              :class="{ recording: recordingShortcut === 'table_delete_row' }"
              @click="startRecording('table_delete_row')"
            >
              <span class="shortcut-label">{{ t('shortcut.tableDeleteRow') }}</span>
              <div class="shortcut-keys">
                <kbd v-for="key in customShortcuts.table_delete_row.split('+')" :key="key">{{ key }}</kbd>
              </div>
              <div v-if="recordingShortcut === 'table_delete_row'" class="recording-overlay">{{ t('shortcut.recording') }}</div>
            </div>

            <div
              class="shortcut-item"
              :class="{ recording: recordingShortcut === 'delete_word' }"
              @click="startRecording('delete_word')"
            >
              <span class="shortcut-label">{{ t('shortcut.deleteWord') }}</span>
              <div class="shortcut-keys">
                <kbd v-for="key in customShortcuts.delete_word.split('+')" :key="key">{{ key }}</kbd>
              </div>
              <div v-if="recordingShortcut === 'delete_word'" class="recording-overlay">{{ t('shortcut.recording') }}</div>
            </div>
          </div>
          <!-- Invisible input to capture keys when recording -->
          <input
            v-if="recordingShortcut"
            ref="shortcutInput"
            class="hidden-input"
            @keydown="handleShortcutKeyDown"
            @blur="recordingShortcut = null"
          />
        </div>

        <div v-else-if="activeSettingsTab === 'about'" class="settings-content about-content">
          <div class="shortcut-list">
            <div class="shortcut-item">
              <span class="shortcut-label">GitHub</span>
              <a class="shortcut-label about-link" href="https://github.com/yumkea/jot" target="_blank">github.com/yumkea/jot</a>
            </div>
          </div>
          <div class="about-quote">{{ t('about.quote') }}</div>
        </div>
      </div>
    </main>
  </div>
</template>

<style>
/* 
  全局基础样式 
  保留在 App.vue 中以维持整体布局骨架
*/
.wrapper {
  position: relative;
  height: calc(100vh - 2px);
  width: calc(100vw - 2px);
  margin: 1px;
  display: flex;
  flex-direction: row;
  border-radius: 8px;
  overflow: hidden;
  box-sizing: border-box;
  color: #fff;
  padding: 4px;
  background: rgba(255, 255, 255, 0.1) !important;
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
  border: 1px solid rgba(255, 255, 255, 0.5) !important;
  box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.2);
  gap: 4px;
}

.main-container {
  flex: 1; /* 右侧区域自动填充剩余空间 */
  min-width: 0; /* 允许收缩，但靠内部元素撑开 */
  display: flex;
  flex-direction: column;
  background: var(--bg-main);
  border-radius: 8px;
  overflow: hidden;
  position: relative;
}

.sidebar-wrapper {
  display: flex;
  flex-direction: row;
  height: 100%;
  flex-shrink: 0;
  background: var(--bg-main);
  border-radius: 8px;
  overflow: visible;
  box-shadow: 4px 0 15px rgba(0, 0, 0, 0.05);
  z-index: 10;
  position: relative;
}

.sidebar-wrapper.collapsed {
  overflow: visible;
}

.sidebar-wrapper.collapsed .sidebar {
  border-radius: 8px;
}

.sidebar-panels {
  display: flex;
  height: 100%;
  flex-shrink: 0;
  overflow: hidden;
  position: relative;
  opacity: 0;
  pointer-events: none;
  z-index: 30;
  box-shadow: none;
  transition:
    width var(--panel-duration, 160ms) cubic-bezier(0.22, 1, 0.36, 1),
    opacity var(--panel-fade-duration, 140ms) ease,
    margin var(--panel-duration, 160ms) cubic-bezier(0.22, 1, 0.36, 1);
  will-change: width, opacity;
}

.sidebar-panels.open {
  opacity: 1;
  pointer-events: auto;
}

.sidebar-panels.resizing {
  transition: none;
}

/* 面板切换动画 */
.panel-fade-enter-active,
.panel-fade-leave-active {
  transition:
    opacity 140ms ease,
    transform 140ms cubic-bezier(0.16, 1, 0.3, 1);
  position: absolute;
  top: 0;
  left: 0;
  height: 100%;
}

.panel-fade-enter-active {
  position: relative; /* 进入的组件设为 relative 以撑开容器 */
}

.panel-fade-enter-from {
  opacity: 0;
  transform: translateX(-6px);
}

.panel-fade-leave-to {
  opacity: 0;
  transform: translateX(6px);
}

.tab-separator {
  width: 0;
  height: 40%;
  background: var(--divider-color);
  border-radius: 1px;
  flex-shrink: 0;
  align-self: center;
  position: relative;
  opacity: 0;
  transform: scaleY(0.5);
  z-index: 35;
  transition:
    width 140ms cubic-bezier(0.16, 1, 0.3, 1),
    opacity 100ms ease,
    transform 140ms cubic-bezier(0.16, 1, 0.3, 1);
}

.tab-separator.visible {
  width: 1px;
  opacity: 1;
  margin: 0 4px;
  transform: scaleY(1);
}

.outline-resizer {
  width: 6px;
  cursor: col-resize;
  position: absolute;
  top: 0;
  bottom: 0;
  transition: background 0.2s;
  z-index: 40;
  margin-left: -3px; /* 使拖拽区中心对准边框 */
  background: transparent;
}

.outline-resizer:hover {
  background: rgba(var(--accent-rgb), 0.3);
}

/* Settings View Styles */
.settings-view {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  padding: 0 4px;
  background: var(--bg-main);
  color: var(--text-main);
  position: relative;
}

.settings-content {
  padding: 0 12px;
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow-y: auto;
  overflow-x: hidden;
  scrollbar-gutter: stable;
}

.settings-content::-webkit-scrollbar {
  width: 6px;
}

.settings-content::-webkit-scrollbar-track {
  background: transparent;
}

.settings-content::-webkit-scrollbar-thumb {
  background: var(--text-low);
  border-radius: 10px;
}

.settings-content::-webkit-scrollbar-thumb:hover {
  background: var(--text-secondary);
}

.settings-content::-webkit-scrollbar-thumb:active {
  background: var(--text-main);
}

.settings-header {
  height: 30px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  -webkit-app-region: drag;
  padding: 0;
}

.settings-title {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-secondary);
  padding-left: 12px;
}

.shortcut-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.shortcut-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 12px;
  border-bottom: 1px solid var(--border-color);
  border-radius: 6px;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
  gap: 16px;
  min-height: 40px;
  cursor: pointer;
}

.shortcut-item:hover {
  background: var(--hover-bg);
  padding-left: 16px;
}

.shortcut-item.recording {
  background: var(--recording-bg);
  border-color: var(--border-active);
}

.recording-overlay {
  position: absolute;
  inset: 0;
  background: var(--overlay-bg);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 11px;
  color: var(--text-secondary);
  font-weight: bold;
  border-radius: 6px;
  z-index: 10;
  pointer-events: none;
}

.hidden-input {
  position: absolute;
  top: -100px;
  opacity: 0;
}

.shortcut-label {
  font-size: 14px;
  color: var(--text-secondary);
  font-weight: 400;
}

.shortcut-keys {
  display: flex;
  gap: 4px;
}

.shortcut-group {
  display: flex;
  align-items: center;
  gap: 12px;
}

.storage-card {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 12px;
  border: 1px solid var(--border-active);
  border-radius: 8px;
  background: linear-gradient(180deg, var(--recording-bg), transparent);
}

.storage-card-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.storage-title-block {
  display: flex;
  flex-direction: column;
  gap: 3px;
  min-width: 0;
}

.storage-subtitle,
.storage-migration-note {
  font-size: 11px;
  line-height: 1.35;
  color: var(--text-low);
}

.storage-result {
  border: 1px solid rgba(var(--accent-rgb), 0.35);
  border-radius: 6px;
  background: rgba(var(--accent-rgb), 0.1);
  color: var(--text-secondary);
  padding: 7px 9px;
  font-size: 11px;
  line-height: 1.35;
}

.storage-path-display {
  width: 100%;
  min-height: 34px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background: var(--kbd-bg);
  color: var(--text-main);
  padding: 7px 9px;
  text-align: left;
  cursor: pointer;
}

.storage-path-display:hover {
  border-color: var(--accent-color);
}

.storage-path-text {
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 11px;
}

.storage-path-button {
  height: 30px;
  border: 1px solid rgba(var(--accent-rgb), 0.45);
  border-radius: 6px;
  background: rgba(var(--accent-rgb), 0.12);
  color: var(--text-main);
  padding: 0 12px;
  font-size: 11px;
  font-weight: 700;
  cursor: pointer;
  white-space: nowrap;
}

.storage-path-button:hover {
  background: rgba(var(--accent-rgb), 0.18);
}

.storage-path-button:disabled {
  opacity: 0.45;
  cursor: default;
}

.global-badge {
  font-size: 10px;
  background: var(--kbd-bg);
  color: var(--text-secondary);
  padding: 2px 6px;
  border-radius: 4px;
  font-weight: 600;
  text-transform: uppercase;
  border: 1px solid var(--border-active);
}

.shortcut-keys kbd {
  background: var(--kbd-bg);
  border: 1px solid var(--border-active);
  border-radius: 4px;
  padding: 2px 6px;
  font-size: 11px;
  color: var(--text-secondary);
  min-width: 20px;
  text-align: center;
}



.about-quote {
  position: absolute;
  bottom: 20px;
  right: 20px;
  font-size: 11px;
  color: var(--text-quote);
  font-style: italic;
  white-space: nowrap;
  text-align: right;
}

.about-link {
  color: var(--accent-color);
  text-decoration: none;
  cursor: pointer;
}
.about-link:hover {
  text-decoration: underline;
}

/* 主题变量 */
:root {
  --accent-color: #0993a9;
  --accent-rgb: 9, 147, 169;
  --bg-main: #000;
  --text-main: #fff;
  --text-secondary: rgba(255, 255, 255, 0.9);
  --text-low: rgba(255, 255, 255, 0.4);
  --text-quote: rgba(255, 255, 255, 0.2);
  --border-color: rgba(255, 255, 255, 0.05);
  --border-active: rgba(255, 255, 255, 0.2);
  --hover-bg: rgba(255, 255, 255, 0.05);
  --recording-bg: rgba(255, 255, 255, 0.08);
  --overlay-bg: rgba(0, 0, 0, 0.85);
  --kbd-bg: rgba(255, 255, 255, 0.1);
  --glass-bg: rgba(255, 255, 255, 0.1);
  --glass-border: rgba(255, 255, 255, 0.5);
  --icon-filter: brightness(0) invert(1);
  --icon-active-filter: invert(41%) sepia(91%) saturate(1352%) hue-rotate(160deg) brightness(95%) contrast(95%);
  --tab-active-bg: rgba(255, 255, 255, 0.1);
  --divider-color: rgba(255, 255, 255, 0.15);
}

.theme-light,
:root.theme-light {
  --bg-main: #ffffff;
  --text-main: #1a1a1a;
  --text-secondary: rgba(0, 0, 0, 0.85);
  --text-low: rgba(0, 0, 0, 0.45);
  --text-quote: rgba(0, 0, 0, 0.3);
  --border-color: rgba(0, 0, 0, 0.08);
  --border-active: rgba(0, 0, 0, 0.15);
  --hover-bg: rgba(0, 0, 0, 0.04);
  --recording-bg: rgba(0, 0, 0, 0.06);
  --overlay-bg: rgba(255, 255, 255, 0.9);
  --kbd-bg: rgba(0, 0, 0, 0.06);
  --glass-bg: rgba(255, 255, 255, 0.3);
  --glass-border: rgba(0, 0, 0, 0.2);
  --icon-filter: brightness(0); /* Black icons */
  --icon-active-filter: invert(41%) sepia(91%) saturate(1352%) hue-rotate(160deg) brightness(95%) contrast(95%); /* Darker blue */
  --tab-active-bg: rgba(0, 0, 0, 0.05);
  --divider-color: rgba(0, 0, 0, 0.1);
}

.wrapper.glass {
  background: var(--glass-bg) !important;
  border-color: var(--glass-border) !important;
}

/* 主题色选择器 */
.accent-color-item {
  cursor: default;
}

.color-swatch {
  display: inline-block;
  width: 22px;
  height: 22px;
  border-radius: 6px;
  border: 2px solid transparent;
  cursor: pointer;
  position: relative;
  transition: transform 0.15s ease, box-shadow 0.15s ease;
  box-shadow: 0 0 0 1px rgba(255, 255, 255, 0.1);
}

.color-swatch:hover {
  transform: scale(1.12);
  box-shadow: 0 0 0 1px rgba(255, 255, 255, 0.1), 0 0 10px rgba(var(--accent-rgb), 0.5);
}

.color-swatch input[type="color"] {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  opacity: 0;
  cursor: pointer;
  border: none;
  padding: 0;
}

.reset-color-btn {
  cursor: pointer;
  transition: all 0.2s ease;
  margin-left: 4px;
}

.reset-color-btn:hover {
  background: var(--hover-bg);
  color: var(--text-main);
  border-color: var(--accent-color);
}
</style>
