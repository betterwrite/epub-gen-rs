import wasmInit, { Epub as WasmEpub } from '../wasm/epub_gen_wasm.js'
import type { EpubInfo } from './types.js'

export type { EpubInfo }

export interface EpubStylesheet {
  id: string
  path: string
  content: string
}

export interface EpubImageBrowser {
  id: string
  path: string
  /** Raw image bytes as a `Uint8Array`, or a base64-encoded string. */
  data: Uint8Array | string
  /** When `true`, this image is the book cover (at most one). */
  cover: boolean
}

function resolveData(data: Uint8Array | string): Uint8Array {
  if (typeof data !== 'string') return data
  const binary = atob(data)
  const out = new Uint8Array(binary.length)
  for (let i = 0; i < binary.length; i++) out[i] = binary.charCodeAt(i)
  return out
}

/** Typed wrapper around the WASM Epub for browser environments. */
export class Epub {
  private inner: WasmEpub

  constructor(info: EpubInfo, chapters: string[][]) {
    this.inner = new WasmEpub(info, chapters)
  }

  /** Attach additional CSS stylesheets. Linked after `css` (from info), in order. */
  setStylesheets(stylesheets: EpubStylesheet[]): void {
    ;(this.inner as unknown as Record<string, (v: unknown) => void>).setStylesheets(stylesheets)
  }

  /** Attach images. `data` may be a `Uint8Array` or a base64-encoded string. */
  setImages(images: EpubImageBrowser[]): void {
    this.inner.setImages(
      images.map(img => ({ ...img, data: resolveData(img.data) })),
    )
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
