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
- **`EPUB`** — holds `Info` and `chapters: Vec<Vec<String>>`. Each chapter is a `Vec<String>` where index 0 is the title and subsequent entries become `<p>` elements.

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
OEBPS/<slug>.xhtml        (Stored — one file per chapter, slug via `slugify`)
OEBPS/styles.css          (Deflated — empty unless `Info::css` is Some)
```

Chapter filenames are derived with `slugify!(title, separator = "_")`. The same slug is used for manifest item `id` (with `-` separator) and `href` (with `_` separator) — keep both in sync when touching `manifest()`.

### Known gaps (from README)
UUID is only assigned per-book, not per-resource or per-path. `spine_ncx()` generates `idref` values like `content_N_item_N` that must match the manifest item ids — they currently do not (manifest uses slugified titles). This is a known bug.
