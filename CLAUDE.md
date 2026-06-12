# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```sh
cargo build          # compile the library
cargo test           # run all tests
cargo test it_build  # run the single integration test (writes test.epub to disk)
cargo clippy         # lint
```

### Node.js addon (`node/`)

```sh
cd node
npm install          # install @napi-rs/cli
npm run build        # release build → produces epub-gen-rs.<platform>.node + index.js + index.d.ts
npm run build:debug  # debug build (faster, no optimisations)
```

`napi build` compiles the Rust crate and regenerates `index.js` / `index.d.ts`. The checked-in copies are the hand-written equivalents for reference only.

## Architecture

This is a single-file Rust library (`src/lib.rs`) with no binary. The entire public API is two structs:

- **`Info`** — metadata bag (title, author, publisher, lang, css, fonts, version).
- **`EPUB`** — holds `Info`, `chapters: Vec<Vec<String>>`, and `images: Vec<Image>`. Each chapter is a `Vec<String>` where index 0 is the title and subsequent entries become `<p>` elements. Images are attached via `with_images(...)` (builder) or `set_images(...)`.
- **`Image`** — embedded resource (`id`, `path`, `data: Vec<u8>`, `cover: bool`). Written to `OEBPS/images/<path>`; the extension drives the manifest media-type (`media_type_for`). One image may set `cover: true` to become the book cover (gets `properties="cover-image"`, the EPUB 2 `<meta name="cover">` hint, and an auto-generated `cover.xhtml` placed first in the spine). Reference non-cover images from chapter paragraphs with raw markup like `<img src="images/foo.png" alt="" />`.

The main entry points are:
- `EPUB::run()` — calls `archive()` then `write()`, panics on error, writes `<title>.epub` to the current directory.
- `EPUB::archive()` — builds the ZIP in memory and returns `Vec<u8>`. Use this when you need the bytes without writing a file.

### EPUB structure produced

```
mimetype                  (Stored, uncompressed — required by spec)
META_INF/container.xml    (Stored)
OEBPS/content.opf         (Deflated — package metadata + manifest + spine)
OEBPS/toc.ncx             (Deflated — NCX navigation for EPUB 2 compat)
OEBPS/toc.xhtml           (Deflated — EPUB 3 nav document)
OEBPS/cover.xhtml         (Deflated — only when an Image has cover: true; first in spine)
OEBPS/<slug>.xhtml        (Stored — one file per chapter, slug via `slugify`)
OEBPS/images/<path>       (Stored for raster, Deflated for SVG — only when images present)
OEBPS/styles.css          (Deflated — empty unless `Info::css` is Some)
```

Chapter filenames are derived with `slugify!(title, separator = "_")`. The same slug is used for manifest item `id` (with `-` separator) and `href` (with `_` separator) — keep both in sync when touching `manifest()`.

### Known gaps (from README)
UUID is only assigned per-book, not per-resource or per-path.

### Spine / manifest id contract
`spine()` itemrefs use `slugify!(title)` (hyphen separator). `manifest()` uses the same slug for `id` and `slugify!(title, separator = "_")` for `href`. Keep both helpers in sync — a mismatch breaks every reading system.
