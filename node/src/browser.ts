import wasmInit, { Epub as WasmEpub } from '../wasm/epub_gen_wasm.js'
import type { EpubInfo } from './types.js'

export type { EpubInfo }

export interface EpubImageBrowser {
  id: string
  path: string
  /** Raw image bytes. */
  data: Uint8Array
  /** When `true`, this image is the book cover (at most one). */
  cover: boolean
}

/** Typed wrapper around the WASM Epub for browser environments. */
export class Epub {
  private inner: WasmEpub

  constructor(info: EpubInfo, chapters: string[][]) {
    this.inner = new WasmEpub(info, chapters)
  }

  /** Attach images. Flag exactly one with `cover: true` to set the book cover. */
  setImages(images: EpubImageBrowser[]): void {
    this.inner.setImages(images)
  }

  /**
   * Build the EPUB and return its bytes as a `Uint8Array`.
   * No filesystem access is performed — use `Blob` + `URL.createObjectURL` to download.
   */
  archive(): Uint8Array {
    return this.inner.archive()
  }
}

/**
 * Resolves once the WASM module is initialised.
 *
 * ```ts
 * import { Epub, ready } from 'epub-gen3/browser'
 * await ready
 * const epub = new Epub(info, chapters)
 * ```
 */
export const ready: Promise<void> = wasmInit().then(() => undefined)
