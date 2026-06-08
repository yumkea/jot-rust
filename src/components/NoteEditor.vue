<script setup lang="ts">
import { ref, nextTick, onMounted, onBeforeUnmount, computed } from 'vue'
import { useEditor, EditorContent } from '@tiptap/vue-3'
import { Editor, Extension } from '@tiptap/core'
import StarterKit from '@tiptap/starter-kit'
import Placeholder from '@tiptap/extension-placeholder'
import TaskList from '@tiptap/extension-task-list'
import TaskItem from '@tiptap/extension-task-item'
import { TableKit } from '@tiptap/extension-table'
import 'katex/dist/katex.min.css'
import { Node as ProsemirrorNode } from 'prosemirror-model'
import { NoLinkInsideCode, StrictCode, TyporaBlockMath, TyporaBold, TyporaInlineMath, TyporaItalic, TyporaTaskList } from '../utils/editorRendering'
import { editorJsonToMarkdown, normalizeContentForEditor } from '../utils/markdown'
import * as api from '../api'

// --- 接口定义 ---
interface HeadingItem {
  level: number
  text: string
  pos: number
}

// --- 右键菜单状态 ---
const showContextMenu = ref(false)
const contextMenuX = ref(0)
const contextMenuY = ref(0)
const isTableContextMenu = ref(false)

// --- Slash Command 状态 ---
interface SlashCommandItem {
  id: string
  label: string
  keywords: string[]
  icon: string
  action: (ed: Editor) => void
}

const showSlashMenu = ref(false)
const slashMenuX = ref(0)
const slashMenuY = ref(0)
const slashQuery = ref('')
const slashSelectedIndex = ref(0)
let slashFrom = 0 // 记录 `/` 字符在文档中的位置

const slashCommands = computed<SlashCommandItem[]>(() => [
  {
    id: 'task',
    label: props.t('slash.taskList'),
    keywords: ['tk', 'task', '任务', 'rw'],
    icon: '☑',
    action: (ed) => ed.chain().focus().toggleTaskList().run()
  },
  {
    id: 'table',
    label: props.t('slash.table'),
    keywords: ['tb', 'table', '表格', 'bg'],
    icon: '▦',
    action: (ed) => ed.chain().focus().insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run()
  }
])

const filteredSlashCommands = ref<SlashCommandItem[]>([])

// --- 属性与事件 ---
const props = defineProps<{
  noteId: string
  title: string
  initialContent: string
  shortcutDeleteLine: string
  shortcutDeleteWord: string
  shortcutTableAddRowBelow: string
  shortcutTableDeleteRow: string
  shortcutToggleBold: string
  t: (key: string) => string
}>()

const emit = defineEmits<{
  (e: 'save-start'): void
  (e: 'save-success', time: string): void
  (e: 'update-content', content: string): void
  (e: 'update-headings', headings: HeadingItem[]): void
}>()

// --- 助手函数 ---
const getHeadings = (editorInstance: Editor): HeadingItem[] => {
  const headings: HeadingItem[] = []
  editorInstance.state.doc.descendants((node: ProsemirrorNode, pos: number) => {
    if (node.type.name === 'heading') {
      headings.push({
        level: node.attrs.level,
        text: node.textContent,
        pos: pos
      })
    }
  })
  return headings
}

/**
 * 将设置格式的快捷键 (如 "Ctrl+Shift+K") 转换为 ProseMirror 格式 (如 "Mod-Shift-k")
 */
const toProseMirrorKey = (shortcut: string): string => {
  return shortcut
    .split('+')
    .map((part) => {
      const lower = part.toLowerCase()
      if (lower === 'ctrl') return 'Mod'
      if (lower === 'alt') return 'Alt'
      if (lower === 'shift') return 'Shift'
      if (lower === 'meta') return 'Mod'
      if (lower === 'esc') return 'Escape'
      if (lower === 'space') return 'Space'
      // 单字母键保持小写
      if (part.length === 1) return part.toLowerCase()
      return part
    })
    .join('-')
}

/**
 * 自定义快捷键扩展：删除行、删除词（根据 props 配置动态绑定）
 */
const deleteLineHandler = ({ editor: ed }: { editor: Editor }): boolean => {
  const { state, dispatch } = ed.view
  const { $from } = state.selection
  const lineStart = $from.start()
  const lineEnd = $from.end()
  const from = Math.max(lineStart - 1, 0)
  const to = Math.min(lineEnd + 1, state.doc.content.size)
  dispatch(state.tr.delete(from, to))
  return true
}

const deleteWordHandler = ({ editor: ed }: { editor: Editor }): boolean => {
  const { state, dispatch } = ed.view
  const { from } = state.selection
  const $pos = state.doc.resolve(from)
  const textBefore = $pos.parent.textBetween(0, $pos.parentOffset)
  if (!textBefore) return false
  // 分隔符包括空白和中文标点
  const sep = /[\s，。、；：！？""''（）【】《》·…—]/
  // 从末尾跳过尾随分隔符，然后删除到前一个分隔符或行首
  let end = textBefore.length
  // 跳过光标前的连续分隔符
  while (end > 0 && sep.test(textBefore[end - 1])) end--
  if (end === 0) {
    // 全是分隔符，删除全部
    dispatch(state.tr.delete(from - textBefore.length, from))
    return true
  }
  // 向前找到词的起点（遇到分隔符停止）
  let start = end
  while (start > 0 && !sep.test(textBefore[start - 1])) start--
  dispatch(state.tr.delete(from - (textBefore.length - start), from))
  return true
}

const tableAddRowBelowHandler = ({ editor: ed }: { editor: Editor }): boolean => {
  if (!ed.isActive('table')) return false
  return ed.chain().focus().addRowAfter().run()
}

const tableDeleteRowHandler = ({ editor: ed }: { editor: Editor }): boolean => {
  if (!ed.isActive('table')) return false
  return ed.chain().focus().deleteRow().run()
}

const toggleBoldHandler = ({ editor: ed }: { editor: Editor }): boolean => {
  return ed.chain().focus().toggleBold().run()
}

const CustomKeymap = Extension.create({
  name: 'customKeymap',
  addKeyboardShortcuts() {
    const shortcuts: Record<string, (args: { editor: Editor }) => boolean> = {}
    shortcuts[toProseMirrorKey(props.shortcutDeleteLine)] = deleteLineHandler
    shortcuts[toProseMirrorKey(props.shortcutDeleteWord)] = deleteWordHandler
    shortcuts[toProseMirrorKey(props.shortcutTableAddRowBelow)] = tableAddRowBelowHandler
    shortcuts[toProseMirrorKey(props.shortcutTableDeleteRow)] = tableDeleteRowHandler
    shortcuts[toProseMirrorKey(props.shortcutToggleBold)] = toggleBoldHandler
    return shortcuts
  }
})

let forceMarkdownCopy = false

const getCurrentMarkdown = (): string => {
  if (!editor.value) return ''
  return editorJsonToMarkdown(editor.value.getJSON())
}

// --- 编辑器初始化 ---
const editor = useEditor({
  content: normalizeContentForEditor(props.initialContent),
  autofocus: true,
  extensions: [
    StarterKit.configure({
      code: false,
      italic: false,
      bold: false
    }),
    TyporaBold,
    TyporaItalic,
    StrictCode,
    NoLinkInsideCode,
    TaskList,
    TaskItem.configure({ nested: true }),
    TableKit,
    Placeholder.configure({
      placeholder: props.t('editor.placeholder')
    }),
    // LaTeX 公式支持：Typora 风格，$...$ 行内公式, $$...$$ 块级公式
    TyporaInlineMath.configure({
      katexOptions: {
        throwOnError: false
      }
    }),
    TyporaBlockMath.configure({
      katexOptions: {
        displayMode: true,
        throwOnError: false
      }
    }),
    TyporaTaskList,
    CustomKeymap
  ],
  editorProps: {
    attributes: {
      class: 'prose-mirror-editor'
    },
    handleKeyDown: (_view, event) => {
      if (event.key === 'Tab') {
        event.preventDefault()
        editor.value?.chain().focus().insertContent('  ').run()
        return true
      }
      if (!showSlashMenu.value) return false
      if (event.key === 'ArrowDown') {
        event.preventDefault()
        slashSelectedIndex.value = (slashSelectedIndex.value + 1) % filteredSlashCommands.value.length
        return true
      }
      if (event.key === 'ArrowUp') {
        event.preventDefault()
        slashSelectedIndex.value = (slashSelectedIndex.value - 1 + filteredSlashCommands.value.length) % filteredSlashCommands.value.length
        return true
      }
      if (event.key === 'Enter' || event.key === ' ') {
        event.preventDefault()
        executeSlashCommand(slashSelectedIndex.value)
        return true
      }
      if (event.key === 'Escape') {
        event.preventDefault()
        showSlashMenu.value = false
        return true
      }
      return false
    },
    handleDOMEvents: {
      copy: (view, event) => {
        const clipboard = event.clipboardData
        if (!clipboard) return false
        const { from, to } = view.state.selection
        const isAllSelected = from <= 1 && to >= view.state.doc.content.size
        if (!forceMarkdownCopy && !isAllSelected) return false
        event.preventDefault()
        clipboard.setData('text/plain', getCurrentMarkdown())
        forceMarkdownCopy = false
        return true
      }
    }
  },
  onUpdate: ({ editor: editorInstance }) => {
    const markdown = editorJsonToMarkdown(editorInstance.getJSON())
    emit('update-content', markdown)
    emit('update-headings', getHeadings(editorInstance))
    debouncedSave(markdown)
  },
  onCreate: ({ editor: editorInstance }) => {
    emit('update-headings', getHeadings(editorInstance))
  },
  onTransaction: ({ editor: editorInstance }) => {
    const { state } = editorInstance
    const { from, empty } = state.selection
    if (!empty) {
      showSlashMenu.value = false
      return
    }
    const $pos = state.doc.resolve(from)
    const textBefore = $pos.parent.textBetween(0, $pos.parentOffset)
    // 只有在行首输入 / 时才触发命令菜单
    const match = textBefore.match(/^\/(\w*)$/)
    if (match) {
      slashFrom = from - match[0].length
      slashQuery.value = match[1]
      const q = match[1].toLowerCase()
      filteredSlashCommands.value = q
        ? slashCommands.value.filter(cmd => cmd.keywords.some(kw => kw.startsWith(q)))
        : [...slashCommands.value]
      if (filteredSlashCommands.value.length > 0) {
        slashSelectedIndex.value = 0
        // 定位菜单到光标位置，带边界检测
        const coords = editorInstance.view.coordsAtPos(from)
        const menuWidth = 180
        const menuHeight = filteredSlashCommands.value.length * 32 + 8
        const maxX = window.innerWidth - menuWidth - 8
        const maxY = window.innerHeight - menuHeight - 8
        slashMenuX.value = Math.min(coords.left, maxX)
        slashMenuY.value = coords.bottom + 4 > maxY
          ? coords.top - menuHeight - 4
          : coords.bottom + 4
        showSlashMenu.value = true
      } else {
        showSlashMenu.value = false
      }
    } else {
      showSlashMenu.value = false
    }
  }
})

// --- Slash Command 执行 ---
const executeSlashCommand = (index: number): void => {
  const cmd = filteredSlashCommands.value[index]
  if (!cmd || !editor.value) return
  showSlashMenu.value = false
  // 删除 /xxx 文本
  const { state } = editor.value.view
  const from = slashFrom
  const to = state.selection.from
  editor.value.view.dispatch(state.tr.delete(from, to))
  // 执行命令
  nextTick(() => {
    if (editor.value) cmd.action(editor.value)
  })
}

// --- 对外暴露的方法 ---
const scrollToHeading = (pos: number): void => {
  if (editor.value) {
    editor.value.commands.focus(pos)
    const dom = editor.value.view.nodeDOM(pos) as HTMLElement
    if (dom) {
      dom.scrollIntoView({ behavior: 'smooth', block: 'center' })
    }
  }
}

const setEditorContent = (content: string): void => {
  if (editor.value) {
    editor.value.commands.setContent(normalizeContentForEditor(content))
  }
}

defineExpose({ scrollToHeading, setEditorContent })

// --- 右键菜单逻辑 ---
const handleContextMenu = (e: MouseEvent): void => {
  e.preventDefault()
  if (editor.value) {
    const pos = editor.value.view.posAtCoords({ left: e.clientX, top: e.clientY })
    if (pos) {
      editor.value.chain().focus().setTextSelection(pos.pos).run()
    }
    isTableContextMenu.value = editor.value.isActive('table')
  } else {
    isTableContextMenu.value = false
  }
  // 边界检测，防止菜单被裁剪
  const menuWidth = 180
  const menuHeight = isTableContextMenu.value ? Math.floor(window.innerHeight * 0.58) : 160
  const maxX = window.innerWidth - menuWidth - 8
  const maxY = window.innerHeight - menuHeight - 8
  contextMenuX.value = Math.min(e.clientX, maxX)
  contextMenuY.value = Math.min(e.clientY, maxY)
  showContextMenu.value = true

  const closeMenu = (): void => {
    showContextMenu.value = false
    isTableContextMenu.value = false
    document.removeEventListener('click', closeMenu)
    document.removeEventListener('contextmenu', closeMenu)
  }
  setTimeout(() => {
    document.addEventListener('click', closeMenu)
    document.addEventListener('contextmenu', closeMenu)
  }, 0)
}

const clearDocument = (): void => {
  if (editor.value) {
    editor.value.commands.clearContent()
  }
  showContextMenu.value = false
  isTableContextMenu.value = false
}

const exportMarkdown = (): void => {
  if (!editor.value) return
  const content = getCurrentMarkdown()
  const blob = new Blob([content], { type: 'text/markdown;charset=utf-8' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `${props.title || 'note'}.md`
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)
  showContextMenu.value = false
  isTableContextMenu.value = false
}

const exportHTML = (): void => {
  if (!editor.value) return
  const html = editor.value.getHTML()
  const safeTitle = props.title.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;')
  const fullHtml = `<!DOCTYPE html><html><head><meta charset="utf-8"><title>${safeTitle}</title><link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.16.37/dist/katex.min.css"></head><body>${html}</body></html>`
  const blob = new Blob([fullHtml], { type: 'text/html;charset=utf-8' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `${props.title || 'note'}.html`
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)
  showContextMenu.value = false
  isTableContextMenu.value = false
}

const selectAll = (): void => {
  if (editor.value) {
    editor.value.commands.selectAll()
    forceMarkdownCopy = true
  }
  showContextMenu.value = false
  isTableContextMenu.value = false
}

const runTableCommand = (action: (ed: Editor) => boolean): void => {
  if (!editor.value) return
  action(editor.value)
  showContextMenu.value = false
  isTableContextMenu.value = false
}

// --- 保存逻辑 (Debounce) ---
let debounceTimeout: ReturnType<typeof setTimeout> | null = null
let lastSaveTime = Date.now()
const FORCED_SAVE_INTERVAL = 5000

/**
 * 防抖保存，确保不会频繁写入磁盘
 */
const debouncedSave = (content: string): void => {
  const now = Date.now()
  if (now - lastSaveTime > FORCED_SAVE_INTERVAL) {
    saveNow(content)
    return
  }

  if (debounceTimeout) clearTimeout(debounceTimeout)
  debounceTimeout = setTimeout(() => {
    saveNow(content)
  }, 500)
}

// --- 渲染辅助 ---
const isContentEmpty = (content: string): boolean => {
  if (!content) return true
  const cleanContent = content
    .replace(/[`*_#>\[\]()!|-]/g, '')
    .replace(/\s+/g, '')
    .trim()
  return cleanContent === ''
}

/**
 * 立即执行保存
 */
const saveNow = async (content: string): Promise<void> => {
  if (debounceTimeout) clearTimeout(debounceTimeout)

  const isEmpty = isContentEmpty(content)

  // 用精准查询替代全量 listNotes，避免性能浪费与并发竞态
  try {
    const existingNote = await api.getNote(props.noteId)
    const isAlreadyInDB = existingNote !== null

    if (isEmpty) {
      if (!isAlreadyInDB) {
        // 数据库没记录且当前为空 -> 保持不保存
        return
      } else {
        // 数据库有记录但当前被清空了 -> 更新数据库
        emit('save-start')
        await api.saveNote(props.noteId, props.title, content)
        lastSaveTime = Date.now()
        const now = new Date()
        emit('save-success', now.toLocaleTimeString('zh-CN', { hour12: false }))
        return
      }
    }
  } catch (e) {
    console.error('Check DB status failed', e)
  }

  emit('save-start')
  try {
    await api.saveNote(props.noteId, props.title, content)
    lastSaveTime = Date.now()
    const now = new Date()
    emit('save-success', now.toLocaleTimeString('zh-CN', { hour12: false }))
  } catch (error) {
    console.error('Failed to save note:', error)
    emit('save-success', 'Save failed')
  }
}

onMounted(() => {
  // 延迟聚焦，确保编辑器完全初始化
  setTimeout(() => {
    if (editor.value) {
      editor.value.commands.focus()
    }
  }, 100)
})

onBeforeUnmount(() => {
  if (debounceTimeout) clearTimeout(debounceTimeout)
  editor.value?.destroy()
})
</script>

<template>
  <section class="editor-area" @click="editor?.chain().focus().run()" @contextmenu="handleContextMenu">
    <editor-content :editor="editor" class="editor-content-wrapper" />

    <!-- 右键菜单 -->
    <div
      v-if="showContextMenu"
      :class="['context-menu', { 'table-context-menu': isTableContextMenu }]"
      :style="{ left: contextMenuX + 'px', top: contextMenuY + 'px' }"
    >
      <div class="context-menu-item" @click="selectAll">
        <span class="ctx-label">{{ t('ctx.selectAll') }}</span>
        <span class="ctx-shortcut">Ctrl+A</span>
      </div>
      <template v-if="isTableContextMenu">
        <div class="context-menu-divider"></div>
        <div class="context-menu-item" @click="runTableCommand((ed) => ed.chain().focus().addRowBefore().run())">
          <span class="ctx-label">{{ t('ctx.addRowBefore') }}</span>
        </div>
        <div class="context-menu-item" @click="runTableCommand((ed) => ed.chain().focus().addRowAfter().run())">
          <span class="ctx-label">{{ t('ctx.addRowAfter') }}</span>
        </div>
        <div class="context-menu-item" @click="runTableCommand((ed) => ed.chain().focus().deleteRow().run())">
          <span class="ctx-label">{{ t('ctx.deleteRow') }}</span>
        </div>
        <div class="context-menu-item" @click="runTableCommand((ed) => ed.chain().focus().addColumnBefore().run())">
          <span class="ctx-label">{{ t('ctx.addColumnBefore') }}</span>
        </div>
        <div class="context-menu-item" @click="runTableCommand((ed) => ed.chain().focus().addColumnAfter().run())">
          <span class="ctx-label">{{ t('ctx.addColumnAfter') }}</span>
        </div>
        <div class="context-menu-item" @click="runTableCommand((ed) => ed.chain().focus().deleteColumn().run())">
          <span class="ctx-label">{{ t('ctx.deleteColumn') }}</span>
        </div>
        <div class="context-menu-item" @click="runTableCommand((ed) => ed.chain().focus().deleteTable().run())">
          <span class="ctx-label">{{ t('ctx.deleteTable') }}</span>
        </div>
      </template>
      <div class="context-menu-divider"></div>
      <div class="context-menu-item" @click="clearDocument">
        <span class="ctx-label">{{ t('ctx.clearDocument') }}</span>
      </div>
      <div class="context-menu-divider"></div>
      <div class="context-menu-item" @click="exportMarkdown">
        <span class="ctx-label">{{ t('ctx.exportMarkdown') }}</span>
        <span class="ctx-shortcut">.md</span>
      </div>
      <div class="context-menu-item" @click="exportHTML">
        <span class="ctx-label">{{ t('ctx.exportHTML') }}</span>
        <span class="ctx-shortcut">.html</span>
      </div>
    </div>

    <!-- Slash Command 菜单 -->
    <Teleport to="body">
      <div
        v-if="showSlashMenu && filteredSlashCommands.length > 0"
        class="slash-menu"
        :style="{ left: slashMenuX + 'px', top: slashMenuY + 'px' }"
      >
        <div
          v-for="(cmd, i) in filteredSlashCommands"
          :key="cmd.id"
          class="slash-menu-item"
          :class="{ selected: i === slashSelectedIndex }"
          @mouseenter="slashSelectedIndex = i"
          @mousedown.prevent="executeSlashCommand(i)"
        >
          <span class="slash-icon">{{ cmd.icon }}</span>
          <span class="slash-label">{{ cmd.label }}</span>
          <span class="slash-hint">{{ cmd.keywords[0] }}</span>
        </div>
      </div>
    </Teleport>
  </section>
</template>

<style>
/* 这里不使用 scoped，因为 Tiptap 生成的 HTML 在组件作用域外 */
.editor-area {
  flex: 1;
  padding: 1px 1px 10px 10px;
  overflow-y: auto;
  scrollbar-gutter: stable;
  display: flex; /* 开启 flex 布局 */
  flex-direction: column;
  cursor: text; /* 鼠标移动到此处显示文本输入指针 */
}

.editor-content-wrapper {
  flex: 1; /* 让编辑器包装层填满剩余空间 */
  display: flex;
  flex-direction: column;
}

/* 自定义滚动条样式 */
.editor-area::-webkit-scrollbar {
  width: 6px;
}

.editor-area::-webkit-scrollbar-track {
  background: var(--border-color);
  border-radius: 10px;
}

.editor-area::-webkit-scrollbar-thumb {
  background: var(--text-low);
  border-radius: 10px;
  border: 1px solid rgba(0, 0, 0, 0.2);
  transition: background 0.2s;
}

.editor-area::-webkit-scrollbar-thumb:hover {
  background: var(--text-secondary);
}

.editor-area::-webkit-scrollbar-thumb:active {
  background: var(--text-main);
}

.prose-mirror-editor {
  flex: 1; /* 关键：让编辑区 div 填满高度 */
  width: 100%;
  color: var(--text-main);
  font-size: 18px;
  outline: none;
  font-family: inherit;
  line-height: 1.6;
  caret-color: var(--accent-color);
}

.prose-mirror-editor p.is-editor-empty:first-child::before {
  content: attr(data-placeholder);
  float: left;
  color: var(--text-low);
  pointer-events: none;
  height: 0;
}

.prose-mirror-editor h1 {
  font-size: 2em;
  margin: 0.5em 0;
}
.prose-mirror-editor h2 {
  font-size: 1.5em;
  margin: 0.4em 0;
}
.prose-mirror-editor p {
  margin: 0.2em 0;
}
.prose-mirror-editor ul,
.prose-mirror-editor ol {
  padding-left: 1.2em;
}
.prose-mirror-editor ul {
  list-style-type: disc;
}
.prose-mirror-editor ol {
  list-style-type: decimal;
}
.prose-mirror-editor code {
  background: var(--hover-bg);
  padding: 2px 4px;
  border-radius: 4px;
}

/* 编辑区内任何残留链接都按普通文本显示，避免浏览器默认蓝色样式 */
.prose-mirror-editor a,
.prose-mirror-editor a:visited,
.prose-mirror-editor a:hover,
.prose-mirror-editor a:active {
  color: inherit !important;
  text-decoration: none !important;
  pointer-events: none;
  cursor: text;
}

/* LaTeX 公式样式 */
.prose-mirror-editor .tiptap-mathematics-render {
  padding: 2px 4px;
  cursor: pointer;
}

.prose-mirror-editor .tiptap-mathematics-render--editable:hover {
  background: var(--hover-bg);
  border-radius: 4px;
}

/* 块级公式 */
.prose-mirror-editor div.tiptap-mathematics-render {
  display: block;
  text-align: center;
  margin: 1em 0;
  padding: 1em;
  background: var(--hover-bg);
  border-radius: 8px;
}

/* 公式选中状态 */
.prose-mirror-editor .tiptap-mathematics-render.ProseMirror-selectednode {
  outline: 2px solid var(--accent-color);
  border-radius: 4px;
}

/* 右键菜单样式 */
.context-menu {
  position: fixed;
  z-index: 9999;
  min-width: 180px;
  background: var(--bg-main);
  border: 1px solid var(--border-active);
  border-radius: 8px;
  padding: 4px 0;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
}

.context-menu.table-context-menu {
  max-height: 58vh;
  overflow-y: auto;
  overscroll-behavior: contain;
}

.context-menu.table-context-menu::-webkit-scrollbar {
  width: 6px;
}

.context-menu.table-context-menu::-webkit-scrollbar-track {
  background: transparent;
}

.context-menu.table-context-menu::-webkit-scrollbar-thumb {
  background: var(--text-low);
  border-radius: 8px;
}

.context-menu.table-context-menu::-webkit-scrollbar-thumb:hover {
  background: var(--text-secondary);
}

.context-menu-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 12px;
  cursor: pointer;
  font-size: 13px;
  color: var(--text-secondary);
  transition: all 0.15s ease;
  gap: 16px;
}

.context-menu-item:hover {
  background: var(--hover-bg);
  color: var(--text-main);
}

.ctx-label {
  white-space: nowrap;
}

.ctx-shortcut {
  font-size: 11px;
  color: var(--text-low);
  font-family: ui-monospace, SFMono-Regular, monospace;
}

.context-menu-divider {
  height: 1px;
  background: var(--border-color);
  margin: 2px 8px;
}

/* Slash Command 菜单样式 */
.slash-menu {
  position: fixed;
  z-index: 9999;
  min-width: 180px;
  background: var(--bg-main);
  border: 1px solid var(--border-active);
  border-radius: 8px;
  padding: 4px 0;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
}

.slash-menu-item {
  display: flex;
  align-items: center;
  padding: 6px 12px;
  cursor: pointer;
  font-size: 13px;
  color: var(--text-secondary);
  transition: all 0.15s ease;
  gap: 10px;
}

.slash-menu-item.selected,
.slash-menu-item:hover {
  background: var(--hover-bg);
  color: var(--text-main);
}

.slash-icon {
  font-size: 16px;
  width: 24px;
  text-align: center;
  flex-shrink: 0;
}

.slash-label {
  flex: 1;
  white-space: nowrap;
}

.slash-hint {
  font-size: 11px;
  color: var(--text-low);
  font-family: ui-monospace, SFMono-Regular, monospace;
}

/* 任务列表样式 */
.prose-mirror-editor ul[data-type="taskList"] {
  list-style: none;
  padding-left: 0;
}

.prose-mirror-editor ul[data-type="taskList"] li {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  margin: 4px 0;
}

.prose-mirror-editor ul[data-type="taskList"] li > label {
  flex-shrink: 0;
  margin-top: 4px;
  user-select: none;
  position: relative;
  width: 18px;
  height: 18px;
}

.prose-mirror-editor ul[data-type="taskList"] li > label input[type="checkbox"] {
  appearance: none;
  -webkit-appearance: none;
  width: 18px;
  height: 18px;
  border: 2px solid var(--text-low);
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.2s ease;
  position: relative;
  background: transparent;
  margin: 0;
}

.prose-mirror-editor ul[data-type="taskList"] li > label input[type="checkbox"]:hover {
  border-color: var(--accent-color);
  box-shadow: 0 0 0 3px rgba(var(--accent-rgb), 0.15);
}

.prose-mirror-editor ul[data-type="taskList"] li > label input[type="checkbox"]:checked {
  background: var(--accent-color);
  border-color: var(--accent-color);
}

.prose-mirror-editor ul[data-type="taskList"] li > label input[type="checkbox"]:checked::after {
  content: '';
  position: absolute;
  left: 4px;
  top: 1px;
  width: 6px;
  height: 10px;
  border: solid #fff;
  border-width: 0 2px 2px 0;
  transform: rotate(45deg);
}

.prose-mirror-editor ul[data-type="taskList"] li > div {
  flex: 1;
  padding-top: 0;
}

.prose-mirror-editor ul[data-type="taskList"] li[data-checked="true"] > div {
  text-decoration: line-through;
  color: var(--text-low);
}

/* 表格样式 */
.prose-mirror-editor table {
  border-collapse: collapse;
  width: 100%;
  margin: 0.8em 0;
  overflow: hidden;
}

.prose-mirror-editor th,
.prose-mirror-editor td {
  border: 1px solid var(--border-active);
  padding: 8px 12px;
  vertical-align: top;
  min-width: 80px;
  position: relative;
}

.prose-mirror-editor th {
  background: var(--hover-bg);
  font-weight: 600;
  text-align: left;
}

.prose-mirror-editor td {
  background: transparent;
}

.prose-mirror-editor td p,
.prose-mirror-editor th p {
  margin: 0;
}

/* 表格选中状态 */
.prose-mirror-editor .selectedCell {
  background: rgba(var(--accent-rgb), 0.1);
}

.prose-mirror-editor .selectedCell::after {
  content: '';
  position: absolute;
  inset: 0;
  border: 2px solid rgba(var(--accent-rgb), 0.4);
  pointer-events: none;
}
</style>
