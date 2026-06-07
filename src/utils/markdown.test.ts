import { describe, expect, it } from 'vitest'
import { editorJsonToMarkdown, normalizeContentForEditor } from './markdown'

describe('markdown pipeline', () => {
  it('serializes editor json through the extracted serializer', () => {
    const markdown = editorJsonToMarkdown({
      type: 'doc',
      content: [
        {
          type: 'heading',
          attrs: { level: 2 },
          content: [{ type: 'text', text: 'Title' }]
        },
        {
          type: 'paragraph',
          content: [
            { type: 'text', text: 'bold', marks: [{ type: 'bold' }] },
            { type: 'text', text: ' ' },
            { type: 'text', text: 'code', marks: [{ type: 'code' }] },
            { type: 'hardBreak' },
            { type: 'text', text: 'link', marks: [{ type: 'link', attrs: { href: 'https://example.com' } }] }
          ]
        },
        {
          type: 'taskList',
          content: [
            {
              type: 'taskItem',
              attrs: { checked: true },
              content: [
                {
                  type: 'paragraph',
                  content: [{ type: 'text', text: 'done' }]
                }
              ]
            }
          ]
        },
        {
          type: 'table',
          content: [
            {
              type: 'tableRow',
              content: [
                { type: 'tableHeader', content: [{ type: 'paragraph', content: [{ type: 'text', text: 'A' }] }] },
                { type: 'tableHeader', content: [{ type: 'paragraph', content: [{ type: 'text', text: 'B' }] }] }
              ]
            },
            {
              type: 'tableRow',
              content: [
                { type: 'tableCell', content: [{ type: 'paragraph', content: [{ type: 'text', text: '1' }] }] },
                { type: 'tableCell', content: [{ type: 'paragraph', content: [{ type: 'text', text: '2' }] }] }
              ]
            }
          ]
        },
        {
          type: 'blockMath',
          attrs: { latex: 'x^2' }
        }
      ]
    })

    expect(markdown).toBe([
      '## Title',
      '',
      '**bold** `code`  ',
      '[link](https://example.com)',
      '',
      '- [x] done',
      '',
      '| A | B |',
      '| --- | --- |',
      '| 1 | 2 |',
      '',
      '$$',
      'x^2',
      '$$'
    ].join('\n'))
  })

  it('keeps single underscores literal while still parsing double underscores and task lists', () => {
    const html = normalizeContentForEditor('_aa_ __bb__\n\n- [x] done\n\n__aaa___')

    expect(html).toContain('_aa_ <strong>bb</strong>')
    expect(html).toContain('data-type="taskList"')
    expect(html).toContain('data-checked="true"')
    expect(html).toContain('__aaa___')
  })

  it('keeps bare urls and file-like names as plain text', () => {
    const html = normalizeContentForEditor('pingkey.zip\n\ngithub.com\n\nhttps://pingkey.zip')

    expect(html).toContain('<p>pingkey.zip</p>')
    expect(html).toContain('<p>github.com</p>')
    expect(html).toContain('<p>https://pingkey.zip</p>')
    expect(html).not.toContain('<a href=')
  })

  it('unwraps markdown links and existing anchor tags into plain text', () => {
    const markdownHtml = normalizeContentForEditor('[example](https://example.com)')
    const rawHtml = normalizeContentForEditor('<p><a href="https://example.com">example.com</a></p>')

    expect(markdownHtml).toContain('<p>example</p>')
    expect(markdownHtml).not.toContain('<a href=')
    expect(rawHtml).toContain('<p>example.com</p>')
    expect(rawHtml).not.toContain('<a href=')
  })
})