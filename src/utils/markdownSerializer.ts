import type { JSONContent } from '@tiptap/core'

const escapeInlineText = (text: string): string => {
  return text
    .replace(/\\/g, '\\\\')
    .replace(/([*_`~\[\]#>+!|])/g, '\\$1')
}

const escapeCode = (text: string): string => text.replace(/`/g, '\\`')

const escapeLinkText = (text: string): string => text.replace(/[\[\]]/g, '\\$&')

const serializeInline = (node: JSONContent): string => {
  if (node.type === 'text') {
    const raw = node.text ?? ''
    const marks = node.marks ?? []
    let result = escapeInlineText(raw)

    if (marks.some((mark) => mark.type === 'code')) {
      return `\`${escapeCode(raw)}\``
    }

    for (const mark of marks) {
      if (mark.type === 'bold') result = `**${result}**`
      if (mark.type === 'italic') result = `*${result}*`
      if (mark.type === 'strike') result = `~~${result}~~`
      if (mark.type === 'link') {
        const href = mark.attrs?.href ?? ''
        result = `[${escapeLinkText(result)}](${href})`
      }
    }

    return result
  }

  if (node.type === 'hardBreak') return '  \n'
  if (node.type === 'inlineMath') return `$${node.attrs?.latex ?? ''}$`

  return (node.content ?? []).map(serializeInline).join('')
}

const serializeListItem = (node: JSONContent, marker: string, indent: string): string => {
  const lines = (node.content ?? []).map((child) => serializeBlock(child, `${indent}  `)).filter(Boolean)
  if (lines.length === 0) return `${indent}${marker} `

  const [first, ...rest] = lines
  const mappedRest = rest.map((line) => `${indent}  ${line}`)
  return [`${indent}${marker} ${first}`, ...mappedRest].join('\n')
}

const serializeTaskItem = (node: JSONContent, indent: string): string => {
  const checked = node.attrs?.checked ? 'x' : ' '
  const lines = (node.content ?? []).map((child) => serializeBlock(child, `${indent}  `)).filter(Boolean)
  if (lines.length === 0) return `${indent}- [${checked}] `

  const [first, ...rest] = lines
  const mappedRest = rest.map((line) => `${indent}  ${line}`)
  return [`${indent}- [${checked}] ${first}`, ...mappedRest].join('\n')
}

const serializeTable = (node: JSONContent): string => {
  const rows = (node.content ?? []).filter((row) => row.type === 'tableRow')
  if (rows.length === 0) return ''

  const matrix = rows.map((row) =>
    (row.content ?? []).map((cell) =>
      (cell.content ?? [])
        .map((child) => (child.type === 'paragraph' ? (child.content ?? []).map(serializeInline).join('') : serializeInline(child)))
        .join(' ')
        .trim()
    )
  )

  const columnCount = Math.max(...matrix.map((row) => row.length), 1)
  const normalized = matrix.map((row) => Array.from({ length: columnCount }, (_, index) => row[index] ?? ''))
  const header = normalized[0]
  const separator = Array.from({ length: columnCount }, () => '---')
  const body = normalized.slice(1)

  return [
    `| ${header.join(' | ')} |`,
    `| ${separator.join(' | ')} |`,
    ...body.map((row) => `| ${row.join(' | ')} |`)
  ].join('\n')
}

const serializeBlock = (node: JSONContent, indent = ''): string => {
  const content = node.content ?? []

  if (node.type === 'paragraph') return content.map(serializeInline).join('')

  if (node.type === 'heading') {
    const level = Math.max(1, Math.min(6, Number(node.attrs?.level ?? 1)))
    return `${'#'.repeat(level)} ${content.map(serializeInline).join('')}`
  }

  if (node.type === 'blockquote') {
    const inner = content.map((child) => serializeBlock(child, indent)).join('\n\n')
    return inner
      .split('\n')
      .map((line) => `> ${line}`)
      .join('\n')
  }

  if (node.type === 'codeBlock') {
    const language = node.attrs?.language ? String(node.attrs.language) : ''
    const code = content.map((child) => child.text ?? '').join('')
    return `\`\`\`${language}\n${code}\n\`\`\``
  }

  if (node.type === 'bulletList') return content.map((item) => serializeListItem(item, '-', indent)).join('\n')

  if (node.type === 'orderedList') {
    let index = Number(node.attrs?.start ?? 1)
    return content
      .map((item) => {
        const line = serializeListItem(item, `${index}.`, indent)
        index += 1
        return line
      })
      .join('\n')
  }

  if (node.type === 'taskList') return content.map((item) => serializeTaskItem(item, indent)).join('\n')
  if (node.type === 'horizontalRule') return '---'
  if (node.type === 'table') return serializeTable(node)
  if (node.type === 'blockMath') return `$$\n${node.attrs?.latex ?? ''}\n$$`

  return content.map((child) => serializeBlock(child, indent)).join('\n\n')
}

export const editorJsonToMarkdown = (json: JSONContent | null | undefined): string => {
  if (!json || json.type !== 'doc') return ''
  const blocks = (json.content ?? []).map((node) => serializeBlock(node)).filter(Boolean)
  return blocks.join('\n\n').trimEnd()
}