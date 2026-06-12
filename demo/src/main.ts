import './style.css'

document.querySelector<HTMLDivElement>('#app')!.innerHTML = `
  <p>foo</p>
`

import init, { Epub } from '../../node/wasm/epub_gen_wasm.js'
import wasmUrl from '../../node/wasm/epub_gen_wasm_bg.wasm?url'

await init(wasmUrl)   // passa a URL gerada pelo bundler

const epub = new Epub(
  { title: 'Meu Livro', description: '...', publisher: '...', author: '...',
    tocTitle: 'Sumário', lang: 'pt', fonts: [], version: 3 },
  [['Capítulo 1', 'Parágrafo um.']]
)

const bytes = epub.archive()
const blob  = new Blob([bytes as any], { type: 'application/epub+zip' })
const url   = URL.createObjectURL(blob)
const a     = Object.assign(document.createElement('a'), { href: url, download: 'livro.epub' })
a.click()