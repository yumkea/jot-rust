import { readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'

const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), '../..')

const readProjectFile = (relativePath: string): string => {
  return readFileSync(resolve(projectRoot, relativePath), 'utf8')
}

describe('SideBar logo', () => {
  it('uses the shared jot svg asset for the bottom logo', () => {
    const source = readProjectFile('src/components/SideBar.vue')

    expect(source).toContain("import logoJot from '../../resources/jot.svg'")
    expect(source).toContain('<img :src="logoJot" class="logo-image" alt="Jot" draggable="false" />')
    expect(source).not.toContain('class="logo-text"')
  })

  it('keeps the jot logo asset available as an svg', () => {
    const svg = readProjectFile('resources/jot.svg')

    expect(svg).toContain('<svg')
    expect(svg).toContain('viewBox="0 0 1254 1254"')
  })
})
