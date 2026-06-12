import './style.css'

document.querySelector<HTMLDivElement>('#app')!.innerHTML = `
  <p>foo</p>
`

import init, { Epub } from '../../node/wasm/epub_gen_wasm.js'
import wasmUrl from '../../node/wasm/epub_gen_wasm_bg.wasm?url'

await init(wasmUrl)   // passa a URL gerada pelo bundler

const epub = new Epub(
  { title: 'Book', description: '...', publisher: '...', author: '...',
    tocTitle: 'Book', lang: 'pt', fonts: [], version: 3 },
  [['Chapter 1', 'First paragraph.']]
)

const bytes = epub.archive()
const blob  = new Blob([bytes as any], { type: 'application/epub+zip' })
const url   = URL.createObjectURL(blob)
const a     = Object.assign(document.createElement('a'), { href: url, download: 'livro.epub' })
a.click()