import type MarkdownIt from 'markdown-it'

type MarkdownToken = {
  type: string
  tag: string
  nesting: number
  markup: string
  content: string
}

type MarkdownDelimiter = {
  marker: number
  length: number
  token: number
  end: number
  open: boolean
  close: boolean
}

type MarkdownState = {
  pos: number
  src: string
  delimiters: MarkdownDelimiter[]
  tokens: MarkdownToken[]
  tokens_meta: Array<{ delimiters?: MarkdownDelimiter[] } | null>
  scanDelims: (start: number, canSplitWord: boolean) => {
    can_open: boolean
    can_close: boolean
    length: number
  }
  push: (type: string, tag: string, nesting: number) => MarkdownToken
}

const tokenizeWithoutSingleUnderscoreEmphasis = (state: any, silent: boolean): boolean => {
  const marker = state.src.charCodeAt(state.pos)
  const isAsterisk = marker === 0x2a
  const isUnderscore = marker === 0x5f

  if (silent || (!isAsterisk && !isUnderscore)) return false

  const scanned = state.scanDelims(state.pos, isAsterisk)
  const delimiterLength = isUnderscore && scanned.length % 2 !== 0 ? 0 : scanned.length

  if (delimiterLength === 0) {
    for (let index = 0; index < scanned.length; index += 1) {
      const token = state.push('text', '', 0)
      token.content = String.fromCharCode(marker)
    }

    state.pos += scanned.length
    return true
  }

  for (let index = 0; index < delimiterLength; index += 1) {
    const token = state.push('text', '', 0)
    token.content = String.fromCharCode(marker)

    state.delimiters.push({
      marker,
      length: delimiterLength,
      token: state.tokens.length - 1,
      end: -1,
      open: scanned.can_open,
      close: scanned.can_close
    })
  }

  state.pos += scanned.length
  return true
}

const postProcessWithoutSingleUnderscoreEmphasis = (state: MarkdownState, delimiters: MarkdownDelimiter[]): void => {
  for (let index = delimiters.length - 1; index >= 0; index -= 1) {
    const startDelimiter = delimiters[index]

    if (startDelimiter.end === -1) continue
    if (startDelimiter.marker !== 0x2a && startDelimiter.marker !== 0x5f) continue

    const endDelimiter = delimiters[startDelimiter.end]
    const isStrong = index > 0
      && delimiters[index - 1].end === startDelimiter.end + 1
      && delimiters[index - 1].marker === startDelimiter.marker
      && delimiters[index - 1].token === startDelimiter.token - 1
      && delimiters[startDelimiter.end + 1].token === endDelimiter.token + 1

    const openingToken = state.tokens[startDelimiter.token]
    openingToken.type = isStrong ? 'strong_open' : 'em_open'
    openingToken.tag = isStrong ? 'strong' : 'em'
    openingToken.nesting = 1
    openingToken.markup = isStrong ? '**' : '*'
    openingToken.content = ''

    const closingToken = state.tokens[endDelimiter.token]
    closingToken.type = isStrong ? 'strong_close' : 'em_close'
    closingToken.tag = isStrong ? 'strong' : 'em'
    closingToken.nesting = -1
    closingToken.markup = isStrong ? '**' : '*'
    closingToken.content = ''

    if (!isStrong) continue

    state.tokens[delimiters[index - 1].token].content = ''
    state.tokens[delimiters[startDelimiter.end + 1].token].content = ''
    index -= 1
  }
}

const applyWithoutSingleUnderscoreEmphasis = (state: any): boolean => {
  postProcessWithoutSingleUnderscoreEmphasis(state, state.delimiters)

  for (const meta of state.tokens_meta) {
    if (!meta?.delimiters) continue
    postProcessWithoutSingleUnderscoreEmphasis(state, meta.delimiters)
  }

  return true
}

export const installMarkdownEmphasisRules = (md: MarkdownIt): void => {
  md.inline.ruler.at('emphasis', tokenizeWithoutSingleUnderscoreEmphasis)
  md.inline.ruler2.at('emphasis', applyWithoutSingleUnderscoreEmphasis)
}