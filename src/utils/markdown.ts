import MarkdownIt from 'markdown-it'
import { installMarkdownEmphasisRules } from './markdownEmphasis'
export { editorJsonToMarkdown } from './markdownSerializer'

const md = new MarkdownIt({
  html: false,
  linkify: false,
  breaks: true,
  typographer: false
})

installMarkdownEmphasisRules(md)

export const isHtmlContent = (content: string): boolean => /<\/?[a-z][\s\S]*>/i.test(content)

const unwrapAnchors = (doc: Document): void => {
  for (const anchor of Array.from(doc.querySelectorAll('a'))) {
    anchor.replaceWith(doc.createTextNode(anchor.textContent ?? ''))
  }
}

const convertTaskListHtml = (html: string): string => {
  const parser = new DOMParser()
  const doc = parser.parseFromString(html, 'text/html')
  const taskPattern = /^\s*\[( |x|X)\]\s*/

  unwrapAnchors(doc)

  const listNodes = Array.from(doc.querySelectorAll('ul'))
  for (const ul of listNodes) {
    const items = Array.from(ul.children).filter((node) => node.tagName === 'LI') as HTMLLIElement[]
    if (items.length === 0) continue

    const allTask = items.every((li) => taskPattern.test((li.textContent ?? '').trimStart()))
    if (!allTask) continue

    ul.setAttribute('data-type', 'taskList')

    for (const li of items) {
      const source = li.innerHTML
      const match = source.match(taskPattern)
      if (!match) continue

      const checked = match[1].toLowerCase() === 'x'
      const itemContent = source.replace(taskPattern, '')
      li.setAttribute('data-type', 'taskItem')
      li.setAttribute('data-checked', checked ? 'true' : 'false')
      li.innerHTML = `<label contenteditable="false"><input type="checkbox" ${checked ? 'checked' : ''}></label><div><p>${itemContent || ''}</p></div>`
    }
  }

  return doc.body.innerHTML
}

export const normalizeContentForEditor = (content: string): string => {
  if (!content) return ''
  if (isHtmlContent(content)) {
    const parser = new DOMParser()
    const doc = parser.parseFromString(content, 'text/html')
    unwrapAnchors(doc)
    return doc.body.innerHTML
  }
  return convertTaskListHtml(md.render(content))
}
