/**
 * 国际化 (i18n) 组合式函数
 * 支持中英文切换
 */
import { ref, type Ref } from 'vue'

// 支持的语言类型
export type Language = 'en' | 'zh'

// 翻译文本定义
const translations: Record<Language, Record<string, string>> = {
  en: {
    // General Settings
    'settings.general': 'General',
    'settings.shortcuts': 'Shortcuts',
    'settings.about': 'About',
    'settings.theme': 'Theme',
    'settings.theme.dark': 'Dark',
    'settings.theme.light': 'Light',
    'settings.accentColor': 'Accent Color',
    'settings.autoLaunch': 'Auto Launch',
    'settings.autoLaunch.on': 'ON',
    'settings.autoLaunch.off': 'OFF',
    'settings.closeAction': 'Close Button Behavior',
    'settings.closeAction.hide': 'Hide to Tray',
    'settings.closeAction.quit': 'Quit App',
    'settings.language': 'Language',
    'settings.language.en': 'English',
    'settings.language.zh': '中文',

    // Sidebar
    'sidebar.outline': 'Outline',
    'sidebar.search': 'Search',
    'sidebar.history': 'History',
    'sidebar.settings': 'Settings',

    // Shortcuts
    'shortcut.hideWindow': 'Hide the window',
    'shortcut.showWindow': 'Show window at cursor',
    'shortcut.newNote': 'Create new note',
    'shortcut.toggleBold': 'Toggle bold',
    'shortcut.deleteLine': 'Delete line',
    'shortcut.deleteWord': 'Delete word before cursor',
    'shortcut.tableAddRowBelow': 'Table: add row below',
    'shortcut.tableDeleteRow': 'Table: delete current row',
    'shortcut.global': 'Global',
    'shortcut.recording': 'Press keys...',

    // About
    'about.quote': 'Graceful stories originate in an unplanned jot.',

    // History Panel
    'history.today': 'Today',
    'history.yesterday': 'Yesterday',
    'history.searchPlaceholder': 'Search notes...',
    'history.searchTitle': 'Title',
    'history.searchContent': 'Content',
    'history.searchEmpty': 'No results',

    // Editor
    'editor.placeholder': 'Fleeting ideas, captured...',
    'editor.untitled': 'Untitled',

    // Slash Commands
    'slash.taskList': 'Task List',
    'slash.table': 'Table',

    // Context Menu
    'ctx.selectAll': 'Select All',
    'ctx.addRowBefore': 'Insert Row Above',
    'ctx.addRowAfter': 'Insert Row Below',
    'ctx.deleteRow': 'Delete Row',
    'ctx.addColumnBefore': 'Insert Column Left',
    'ctx.addColumnAfter': 'Insert Column Right',
    'ctx.deleteColumn': 'Delete Column',
    'ctx.deleteTable': 'Delete Table',
    'ctx.clearDocument': 'Clear Document',
    'ctx.exportMarkdown': 'Export Markdown',
    'ctx.exportHTML': 'Export HTML',

    // Tray
    'tray.show': 'Show Jot',
    'tray.quit': 'Quit',

    // Footer
    'footer.saving': 'Saving...',
    'footer.saved': 'Saved at',
    'footer.updated': 'updated at',
    'footer.chars': 'chars',

    // Tooltips
    'titlebar.newNote': 'New Note',
    'titlebar.pin': 'Pin Window',
    'titlebar.minimize': 'Minimize',
    'titlebar.maximize': 'Maximize',
    'titlebar.close': 'Close',
    'sidebar.resetSize': 'Double-click to reset size',

    // Accent Color
    'settings.accentColor.reset': 'Reset'
  },
  zh: {
    // 通用设置
    'settings.general': '通用',
    'settings.shortcuts': '快捷键',
    'settings.about': '关于',
    'settings.theme': '主题',
    'settings.theme.dark': '深色',
    'settings.theme.light': '浅色',
    'settings.accentColor': '主题色',
    'settings.autoLaunch': '开机启动',
    'settings.autoLaunch.on': '开启',
    'settings.autoLaunch.off': '关闭',
    'settings.closeAction': '关闭按钮行为',
    'settings.closeAction.hide': '最小化到托盘',
    'settings.closeAction.quit': '退出应用',
    'settings.language': '语言',
    'settings.language.en': 'English',
    'settings.language.zh': '中文',

    // 侧边栏
    'sidebar.outline': '大纲',
    'sidebar.search': '搜索',
    'sidebar.history': '历史',
    'sidebar.settings': '设置',

    // 快捷键
    'shortcut.hideWindow': '隐藏窗口',
    'shortcut.showWindow': '在光标处显示窗口',
    'shortcut.newNote': '创建新笔记',
    'shortcut.toggleBold': '加粗/取消加粗',
    'shortcut.deleteLine': '删除整行',
    'shortcut.deleteWord': '删除光标前一个词',
    'shortcut.tableAddRowBelow': '表格：在下方插入行',
    'shortcut.tableDeleteRow': '表格：删除当前行',
    'shortcut.global': '全局',
    'shortcut.recording': '请按下按键...',

    // 关于
    'about.quote': '优雅的故事源自不经意间的速记。',

    // 历史面板
    'history.today': '今天',
    'history.yesterday': '昨天',
    'history.searchPlaceholder': '搜索笔记...',
    'history.searchTitle': '标题',
    'history.searchContent': '正文',
    'history.searchEmpty': '无搜索结果',

    // 编辑器
    'editor.placeholder': '灵感闪现，即刻记录...',
    'editor.untitled': '无标题',

    // 斜杠命令
    'slash.taskList': '任务列表',
    'slash.table': '表格',

    // 右键菜单
    'ctx.selectAll': '全选',
    'ctx.addRowBefore': '在上方插入行',
    'ctx.addRowAfter': '在下方插入行',
    'ctx.deleteRow': '删除当前行',
    'ctx.addColumnBefore': '在左侧插入列',
    'ctx.addColumnAfter': '在右侧插入列',
    'ctx.deleteColumn': '删除当前列',
    'ctx.deleteTable': '删除表格',
    'ctx.clearDocument': '清空文档',
    'ctx.exportMarkdown': '导出 Markdown',
    'ctx.exportHTML': '导出 HTML',

    // 托盘
    'tray.show': '显示 Jot',
    'tray.quit': '退出',

    // 底栏
    'footer.saving': '保存中...',
    'footer.saved': '已保存于',
    'footer.updated': '更新于',
    'footer.chars': '字符',

    // 工具提示
    'titlebar.newNote': '新建笔记',
    'titlebar.pin': '置顶窗口',
    'titlebar.minimize': '最小化',
    'titlebar.maximize': '最大化',
    'titlebar.close': '关闭',
    'sidebar.resetSize': '双击重置窗口大小',

    // 主题色
    'settings.accentColor.reset': '重置'
  }
}

// 全局语言状态
const currentLanguage: Ref<Language> = ref('en')

/**
 * 国际化组合式函数
 */
export function useI18n(): {
  language: Ref<Language>
  t: (key: string) => string
  setLanguage: (lang: Language) => void
} {
  const t = (key: string): string => {
    return translations[currentLanguage.value][key] || key
  }

  const setLanguage = (lang: Language): void => {
    currentLanguage.value = lang
  }

  return {
    language: currentLanguage,
    t,
    setLanguage
  }
}

/**
 * 获取日期标签的本地化格式
 */
export function getLocalizedDateLabel(dateStr: string, language: Language): string {
  const date = new Date(dateStr)
  const now = new Date()
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate())
  const yesterday = new Date(today)
  yesterday.setDate(yesterday.getDate() - 1)

  const checkDate = new Date(date.getFullYear(), date.getMonth(), date.getDate())

  if (checkDate.getTime() === today.getTime()) {
    return translations[language]['history.today']
  }
  if (checkDate.getTime() === yesterday.getTime()) {
    return translations[language]['history.yesterday']
  }

  return dateStr.split(' ')[0]
}
