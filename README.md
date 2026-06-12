# EPUB 3.x Implementation in Rust

[https://www.w3.org/TR/epub-33/](https://www.w3.org/TR/epub-33/)

- [ ] Stable
- [x] Base Header and Text Flow
- [x] ToC
- [x] Deflated Zip
- [x] Stored for valid decrypted files
- [x] UUID for XHTML's items
- [ ] UUID for resources
- [ ] UUID for paths
- [x] Validade data entries
- [ ] Optional META_INF (encryption.xml, metadata.xml, manifest.xml, ...)
- [x] `container.xml` access
- [x] Images
- [ ] Checkbox and List's
- [ ] Custom Fonts

### Examples

`epub-gen-rs` supports rust, node and web browser (with wasm).

#### Rust

```rs
let mut epub = EPUB::new(Info {
  title: String::from("A Nice Title"),
  description: String::from("A some description..."),
  publisher: String::from("..."),
  author: String::from("..."),
  toc_title: String::from("Table of Contents"),
  lang: String::from("en"),
  fonts: vec![String::from("Roboto")],
  css: None,
  version: 3
}, vec![macro_for_this_please![
  "Title",
  "A some content...",
]]);

epub.run();
```

#### Node

`npm install epub-gen3`

```ts
'use strict'

const fs = require('fs')
const path = require('path')
const { Epub } = require('epub-gen3')

const epub = new Epub(
  {
    title: 'exemplo',
    description: 'epub-gen-rs example.',
    publisher: 'epub-gen-rs',
    author: 'Author',
    tocTitle: 'Toc Title',
    lang: 'en',
    fonts: [],
    css: undefined,
    version: 3,
  },
  [
    [
      'One',
      'Nullam tempor, metus vitae sagittis semper, massa nulla posuere ipsum.',
      'Aliquam non posuere ex. Duis fermentum odio metus, quis ultrices nulla cursus vitae.',
    ],
    [
      'Two',
      'Pellentesque tempor, eros eu consectetur cursus, magna turpis lacinia nunc.',
      'Integer iaculis arcu vitae elementum convallis. Praesent quam magna, maximus sed ullamcorper quis.',
    ]
  ],
)

const buf = epub.archive()
const outPath = path.join(__dirname, 'example.epub')
fs.writeFileSync(outPath, buf)
console.log(`${outPath} (${buf.length} bytes)`)
```

#### Browser

```ts
import init, { Epub } from 'epub-gen3/wasm/epub_gen_wasm.js'
import wasmUrl from 'epub-gen3/wasm/epub_gen_wasm_bg.wasm?url'

await init(wasmUrl)

const epub = new Epub(
  { title: 'Meu Livro', description: '...', publisher: '...', author: '...',
    tocTitle: 'Sumário', lang: 'en', fonts: [], version: 3 },
  [['1', 'Foo bar baz']]
)

const bytes = epub.archive()
const blob  = new Blob([bytes], { type: 'application/epub+zip' })
const url   = URL.createObjectURL(blob)
const a     = Object.assign(document.createElement('a'), { href: url, download: 'book.epub' })
a.click()
```