import { Extension, InputRule, markInputRule, markPasteRule } from '@tiptap/core'
import Bold, {
  starInputRegex as boldStarInputRegex,
  starPasteRegex as boldStarPasteRegex
} from '@tiptap/extension-bold'
import Code from '@tiptap/extension-code'
import Italic, {
  starInputRegex as italicStarInputRegex,
  starPasteRegex as italicStarPasteRegex
} from '@tiptap/extension-italic'
import { BlockMath, InlineMath } from '@tiptap/extension-mathematics'
import { Fragment } from 'prosemirror-model'
import { Plugin, TextSelection, type EditorState } from 'prosemirror-state'

type WrappedTextRuleConfig = {
  find: RegExp
  markName: string
  prefixGroup: number
  innerGroup: number
  suffixGroup?: number
  blockedMarks?: string[]
  removeMarks?: string[]
}

type WrappedNodeRuleConfig = {
  find: RegExp
  nodeName: string
  prefixGroup: number
  innerGroup: number
  suffixGroup?: number
  blockedMarks?: string[]
  getAttrs: (inner: string) => Record<string, unknown>
  cursorOffset?: number
}

type WrappedMatchParts = {
  prefix: string
  inner: string
  suffix: string
}

const getMatchPart = (match: RegExpMatchArray, index: number | undefined): string => {
  if (index === undefined) return ''
  const value = match[index]
  return typeof value === 'string' ? value : ''
}

const getWrappedMatchParts = (
  match: RegExpMatchArray,
  prefixGroup: number,
  innerGroup: number,
  suffixGroup?: number
): WrappedMatchParts => ({
  prefix: getMatchPart(match, prefixGroup),
  inner: getMatchPart(match, innerGroup),
  suffix: getMatchPart(match, suffixGroup)
})

const hasBlockedMark = (state: EditorState, from: number, blockedMarks: string[]): boolean => {
  const activeMarks = state.doc.resolve(from).marks()
  return activeMarks.some((mark) => blockedMarks.includes(mark.type.name))
}

const createWrappedMarkRule = (config: WrappedTextRuleConfig): InputRule => {
  return new InputRule({
    find: config.find,
    handler: ({ state, range, match }) => {
      const blockedMarks = config.blockedMarks ?? []
      if (blockedMarks.length > 0 && hasBlockedMark(state, range.from, blockedMarks)) return

      const { prefix, inner, suffix } = getWrappedMatchParts(
        match,
        config.prefixGroup,
        config.innerGroup,
        config.suffixGroup
      )
      const markType = state.schema.marks[config.markName]
      if (!markType) return

      const start = range.from + prefix.length
      const { tr } = state
      tr.insertText(inner + suffix, start, range.to)
      tr.addMark(start, start + inner.length, markType.create())

      for (const markName of config.removeMarks ?? []) {
        const removableMark = state.schema.marks[markName]
        if (!removableMark) continue
        tr.removeMark(start, start + inner.length, removableMark)
      }

      tr.setSelection(TextSelection.create(tr.doc, start + inner.length + suffix.length))
      tr.setStoredMarks([])
    }
  })
}

const createWrappedNodeRule = (config: WrappedNodeRuleConfig): InputRule => {
  return new InputRule({
    find: config.find,
    handler: ({ state, range, match }) => {
      const blockedMarks = config.blockedMarks ?? []
      if (blockedMarks.length > 0 && hasBlockedMark(state, range.from, blockedMarks)) return

      const { prefix, inner, suffix } = getWrappedMatchParts(
        match,
        config.prefixGroup,
        config.innerGroup,
        config.suffixGroup
      )
      const nodeType = state.schema.nodes[config.nodeName]
      if (!nodeType) return

      const start = range.from + prefix.length
      const end = range.to
      const { tr } = state
      const replacement = nodeType.create(config.getAttrs(inner))
      const content = suffix ? Fragment.fromArray([replacement, state.schema.text(suffix)]) : replacement
      tr.replaceWith(start, end, content)
      tr.setSelection(TextSelection.create(tr.doc, start + (config.cursorOffset ?? 1) + suffix.length))
    }
  })
}

const TyporaInlineMathRule = createWrappedNodeRule({
  find: /(^|[\s(>])(\$((?:\\\$|[^$\n])+?)\$)([\s),.!?;:])?$/,
  nodeName: 'inlineMath',
  prefixGroup: 1,
  innerGroup: 3,
  suffixGroup: 4,
  blockedMarks: ['code'],
  getAttrs: (inner) => ({ latex: inner.replace(/\\\$/g, '$') })
})

const TyporaBlockMathRule = new InputRule({
  find: /^\$\$([\s\S]+?)\$\$$/,
  handler: ({ state, range, match }) => {
    const blockMathNode = state.schema.nodes.blockMath
    if (!blockMathNode) return

    state.tr.replaceWith(range.from, range.to, blockMathNode.create({ latex: match[1].trim() }))
  }
})

export const TyporaInlineMath = InlineMath.extend({
  addInputRules() {
    return [TyporaInlineMathRule]
  }
})

export const TyporaBlockMath = BlockMath.extend({
  addInputRules() {
    return [TyporaBlockMathRule]
  }
})

const TyporaTaskListRule = new InputRule({
  find: /^[-+*]\s\[( |x|X)\]\s$/,
  handler: ({ state, range, match }) => {
    const checked = match[1].toLowerCase() === 'x'
    const taskItem = state.schema.nodes.taskItem.create(
      { checked },
      state.schema.nodes.paragraph.create()
    )
    const taskList = state.schema.nodes.taskList.create({}, [taskItem])
    state.tr.replaceWith(range.from, range.to, taskList)
  }
})

export const TyporaTaskList = Extension.create({
  name: 'typoraTaskListRule',
  addInputRules() {
    return [TyporaTaskListRule]
  }
})

const TyporaBoldUnderscoreRule = createWrappedMarkRule({
  find: /(^|[^\w_])__((?:[^_\n]+))__([ \t])$/,
  markName: 'bold',
  prefixGroup: 1,
  innerGroup: 2,
  suffixGroup: 3,
  blockedMarks: ['code']
})

export const TyporaBold = Bold.extend({
  addInputRules() {
    return [
      markInputRule({
        find: boldStarInputRegex,
        type: this.type
      }),
      TyporaBoldUnderscoreRule
    ]
  },
  addPasteRules() {
    return [
      markPasteRule({
        find: boldStarPasteRegex,
        type: this.type
      }),
      markPasteRule({
        find: /(?:^|\s)(__(?!\s+__)((?:[^_]+))__(?!_))/g,
        type: this.type
      })
    ]
  }
})

export const TyporaItalic = Italic.extend({
  addInputRules() {
    return [
      markInputRule({
        find: italicStarInputRegex,
        type: this.type
      })
    ]
  },
  addPasteRules() {
    return [
      markPasteRule({
        find: italicStarPasteRegex,
        type: this.type
      })
    ]
  }
})

const TyporaCodeRule = createWrappedMarkRule({
  find: /(^|[\s(>])(`([^`\n]+)`)([\s),.!?;:])?$/,
  markName: 'code',
  prefixGroup: 1,
  innerGroup: 3,
  suffixGroup: 4,
  blockedMarks: ['code'],
  removeMarks: ['link']
})

export const StrictCode = Code.extend({
  addInputRules() {
    return [TyporaCodeRule]
  }
})

export const NoLinkInsideCode = Extension.create({
  name: 'noLinkInsideCode',
  addProseMirrorPlugins() {
    return [
      new Plugin({
        appendTransaction: (_transactions, oldState, newState) => {
          if (oldState.doc.eq(newState.doc)) return null
          const linkMark = newState.schema.marks.link
          const codeMark = newState.schema.marks.code
          if (!linkMark || !codeMark) return null

          const tr = newState.tr
          let changed = false

          newState.doc.descendants((node, pos) => {
            if (!node.isText) return
            const hasCode = node.marks.some((mark) => mark.type === codeMark)
            if (!hasCode) return
            const hasLink = node.marks.some((mark) => mark.type === linkMark)
            if (!hasLink) return
            tr.removeMark(pos, pos + node.nodeSize, linkMark)
            changed = true
          })

          return changed ? tr : null
        }
      })
    ]
  }
})