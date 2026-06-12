# EPUB 3.x Implementation in Rust

[https://www.w3.org/TR/epub-33/](https://www.w3.org/TR/epub-33/)

- [ ] Stable
- [x] Base Header and Text Flow
- [x] ToC
- [x] Deflated Zip
- [x] Stored for valid decrypted files
- [x] UUID for XHTML's items
- [x] Validade data entries
- [x] Optional META_INF (encryption.xml, metadata.xml, manifest.xml, ...)
- [x] `container.xml` access
- [x] Images
- [x] Multiple CSS stylesheets
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
  version: 3,
  stylesheets: vec![],
  encryption: None,
  metadata_xml: None,
  manifest_xml: None,
}, vec![macro_for_this_please![
  "Title",
  "A some content...",
]]);

epub.run();
```

With a cover image, an inline figure, and multiple stylesheets:

```rs
use std::fs;

let cover_bytes = fs::read("cover.jpg").unwrap();
let figure_bytes = fs::read("figure.png").unwrap();

let mut epub = EPUB::new(
  Info {
    title: String::from("A Nice Title"),
    description: String::from("..."),
    publisher: String::from("..."),
    author: String::from("..."),
    toc_title: String::from("Table of Contents"),
    lang: String::from("en"),
    fonts: vec![],
    css: Some(String::from("body { font-family: serif; }")),
    version: 3,
    stylesheets: vec![
      Stylesheet {
        id: String::from("typo"),
        path: String::from("css/typography.css"),
        content: String::from("h1 { font-size: 2rem; } p { line-height: 1.6; }"),
      },
    ],
    encryption: None,
    metadata_xml: None,
    manifest_xml: None,
  },
  vec![vec![
    String::from("Chapter One"),
    String::from("The chart below illustrates the data."),
    String::from(r#"<img src="images/figure.png" alt="Figure 1" />"#),
  ]],
)
.with_images(vec![
  Image {
    id: String::from("cover-img"),
    path: String::from("cover.jpg"),
    data: cover_bytes,
    cover: true,
  },
  Image {
    id: String::from("figure-img"),
    path: String::from("figure.png"),
    data: figure_bytes,
    cover: false,
  },
]);

// epub.run()  — writes <title>.epub to disk
// epub.archive()  — returns Vec<u8> for streaming
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

With a cover image, an inline figure, and multiple stylesheets:

```ts
const fs = require('fs')
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
    css: 'body { font-family: serif; }',
    version: 3,
  },
  [
    [
      'Chapter One',
      'The chart below illustrates the data.',
      '<img src="images/figure.png" alt="Figure 1" />',
    ],
  ],
)

epub.setStylesheets([
  { id: 'typo', path: 'css/typography.css', content: 'h1 { font-size: 2rem; } p { line-height: 1.6; }' },
])

epub.setImages([
  { id: 'cover-img', path: 'cover.jpg', data: fs.readFileSync('cover.jpg'), cover: true },
  { id: 'figure-img', path: 'figure.png', data: fs.readFileSync('figure.png'), cover: false },
])

fs.writeFileSync('example.epub', epub.archive())
```

#### Browser

```ts
import { Epub, ready } from 'epub-gen3/browser'

await ready   // waits for the WASM module to initialise

const epub = new Epub(
  { title: 'My Book', description: '...', publisher: '...', author: '...',
    tocTitle: 'Sumário', lang: 'pt', fonts: [], version: 3 },
  [['1', 'Foo bar baz']]
)

epub.setStylesheets([
  { id: 'base', path: 'css/base.css', content: 'body { font-family: serif; line-height: 1.6; }' },
])

const bytes = epub.archive()
const blob  = new Blob([bytes], { type: 'application/epub+zip' })
const url   = URL.createObjectURL(blob)
const a     = Object.assign(document.createElement('a'), { href: url, download: 'book.epub' })
a.click()
```

> Vite users: add `optimizeDeps.exclude: ['epub-gen3']` to `vite.config.ts` so Vite does not
> try to pre-bundle the WASM glue code.
>
> ```ts
> // vite.config.ts
> export default { optimizeDeps: { exclude: ['epub-gen3'] } }
> ```

With a cover image fetched from a URL:

```ts
import { Epub, ready } from 'epub-gen3/browser'

await ready

const coverRes = await fetch('/cover.jpg')
const coverData = new Uint8Array(await coverRes.arrayBuffer())

const epub = new Epub(
  { title: 'My Book', description: '...', publisher: '...', author: '...',
    tocTitle: 'Summary', lang: 'pt', fonts: [], version: 3 },
  [
    ['Chapter 1', 'Text.'],
    ['Chapter 2', '<img src="images/cover.jpg" alt="Cover" />'],
  ]
)

epub.setImages([
  { id: 'cover-img', path: 'cover.jpg', data: coverData, cover: true },
])

const bytes = epub.archive()
const blob  = new Blob([bytes], { type: 'application/epub+zip' })
const url   = URL.createObjectURL(blob)
Object.assign(document.createElement('a'), { href: url, download: 'livro.epub' }).click()
```