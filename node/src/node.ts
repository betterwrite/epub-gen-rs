// ESM wrapper for the napi-rs CJS platform binding loader.
// The loader (index.js) does platform detection and loads the correct .node binary.
import { createRequire } from 'node:module'
import type { EpubInfo } from './types.js'

const _require = createRequire(import.meta.url)
const binding = _require('../index.js') as typeof import('../index.js')
const NapiEpub = binding.Epub

export type { EpubInfo }

export interface EpubImage {
  id: string
  path: string
  /** Raw image bytes as a `Buffer`, or a base64-encoded string. */
  data: Buffer | string
  /** When `true`, this image is the book cover (at most one). */
  cover: boolean
}

function resolveData(data: Buffer | string): Buffer {
  return typeof data === 'string' ? Buffer.from(data, 'base64') : data
}

/** EPUB builder — Node.js native addon. */
export class Epub {
  private inner: InstanceType<typeof NapiEpub>

  constructor(info: EpubInfo, chapters: string[][]) {
    this.inner = new NapiEpub(info, chapters)
  }

  /** Attach images. `data` may be a `Buffer` or a base64-encoded string. */
  setImages(images: EpubImage[]): void {
    this.inner.setImages(
      images.map(img => ({ ...img, data: resolveData(img.data) })),
    )
  }

  /** Build the EPUB and write it to `<title>.epub` in the current working directory. */
  run(): void {
    this.inner.run()
  }

  /**
   * Build the EPUB and return its contents as a `Buffer`.
   *
   * ```ts
   * const buf = epub.archive()
   * fs.writeFileSync('book.epub', buf)
   * ```
   */
  archive(): Buffer {
    return this.inner.archive()
  }
}
