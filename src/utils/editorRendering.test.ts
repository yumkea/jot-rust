import { Editor } from '@tiptap/core'
import StarterKit from '@tiptap/starter-kit'
import TaskItem from '@tiptap/extension-task-item'
import TaskList from '@tiptap/extension-task-list'
import { describe, expect, it } from 'vitest'
import { StrictCode, TyporaBlockMath, TyporaBold, TyporaInlineMath, TyporaItalic } from './editorRendering'

const waitForInputRules = async (): Promise<void> => {
  await new Promise((resolve) => setTimeout(resolve, 10))
}

const createEditor = (): Editor => {
  return new Editor({
    element: document.createElement('div'),
    extensions: [
      StarterKit.configure({
        code: false,
        bold: false,
        italic: false
      }),
      TyporaBold,
      TyporaItalic,
      StrictCode,
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
      TaskList,
      TaskItem.configure({ nested: true })
    ],
    content: ''
  })
}

describe('editor rendering rules', () => {
  it('keeps trailing space outside inline code marks', async () => {
    const editor = createEditor()

    editor.commands.insertContent('`aa` ', { applyInputRules: true })
    await waitForInputRules()

    expect(editor.getJSON()).toEqual({
      type: 'doc',
      content: [
        {
          type: 'paragraph',
          content: [
            { type: 'text', text: 'aa', marks: [{ type: 'code' }] },
            { type: 'text', text: ' ' }
          ]
        }
      ]
    })

    editor.destroy()
  })

  it('keeps trailing space after inline math nodes', async () => {
    const editor = createEditor()

    editor.commands.insertContent('$x+y$ ', { applyInputRules: true })
    await waitForInputRules()

    expect(editor.getJSON()).toEqual({
      type: 'doc',
      content: [
        {
          type: 'paragraph',
          content: [
            { type: 'inlineMath', attrs: { latex: 'x+y' } },
            { type: 'text', text: ' ' }
          ]
        }
      ]
    })

    editor.destroy()
  })

  it('keeps trailing space after double-underscore bold and ignores single underscore italic', async () => {
    const editor = createEditor()

    editor.commands.insertContent('__aa__ ', { applyInputRules: true })
    await waitForInputRules()

    expect(editor.getJSON()).toEqual({
      type: 'doc',
      content: [
        {
          type: 'paragraph',
          content: [
            { type: 'text', text: 'aa', marks: [{ type: 'bold' }] },
            { type: 'text', text: ' ' }
          ]
        }
      ]
    })

    editor.commands.setContent('')
    editor.commands.insertContent('_aa_ ', { applyInputRules: true })
    await waitForInputRules()

    expect(editor.getJSON()).toEqual({
      type: 'doc',
      content: [
        {
          type: 'paragraph',
          content: [{ type: 'text', text: '_aa_ ' }]
        }
      ]
    })

    editor.destroy()
  })
})