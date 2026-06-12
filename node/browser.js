/**
 * Browser entry-point for epub-gen3.
 *
 * Re-exports the WASM Epub class and a ready promise that resolves once the
 * WASM module is initialised. Import this instead of the raw wasm glue:
 *
 *   import { Epub, ready } from 'epub-gen3/browser'
 *   await ready
 *   const epub = new Epub(info, chapters)
 */
import init, { Epub } from './wasm/epub_gen_wasm.js'

export { Epub }
export const ready = init()
