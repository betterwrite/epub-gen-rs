# EPUB Implementation in Rust (WIP)

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
- [ ] Images, Checkbox and List's
- [ ] Custom Fonts

### Example

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
    description: 'Livro de exemplo gerado pelo addon nativo epub-gen-rs.',
    publisher: 'epub-gen-rs',
    author: 'Autor Exemplo',
    tocTitle: 'Sumário',
    lang: 'pt',
    fonts: [],
    css: undefined,
    version: 3,
  },
  [
    [
      'Capítulo Um',
      'Nullam tempor, metus vitae sagittis semper, massa nulla posuere ipsum.',
      'Aliquam non posuere ex. Duis fermentum odio metus, quis ultrices nulla cursus vitae.',
    ],
    [
      'Capítulo Dois',
      'Pellentesque tempor, eros eu consectetur cursus, magna turpis lacinia nunc.',
      'Integer iaculis arcu vitae elementum convallis. Praesent quam magna, maximus sed ullamcorper quis.',
    ],
    [
      'Capítulo Três',
      'Sed ac lobortis erat, id egestas tellus. Nullam velit turpis, maximus eget lacus quis.',
    ],
  ],
)

const buf = epub.archive()
const outPath = path.join(__dirname, 'exemplo.epub')
fs.writeFileSync(outPath, buf)
console.log(`EPUB gerado em: ${outPath} (${buf.length} bytes)`)
```