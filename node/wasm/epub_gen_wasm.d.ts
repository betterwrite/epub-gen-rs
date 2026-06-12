/* tslint:disable */
/* eslint-disable */

/**
 * EPUB builder for browser environments.
 *
 * ```js
 * import init, { Epub } from 'epub-gen3/wasm/epub_gen_wasm.js'
 * await init()
 *
 * const epub = new Epub(
 *   { title: 'My Book', description: '...', publisher: '...', author: '...',
 *     tocTitle: 'Contents', lang: 'en', fonts: [], version: 3 },
 *   [['Chapter One', 'First paragraph.', 'Second paragraph.']]
 * )
 * const bytes = epub.archive()          // Uint8Array
 * const blob  = new Blob([bytes], { type: 'application/epub+zip' })
 * const url   = URL.createObjectURL(blob)
 * ```
 */
export class Epub {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Build the EPUB and return its bytes as a `Uint8Array`.
     *
     * No filesystem access is performed — it is up to the caller to write or
     * stream the returned bytes (e.g. via `Blob` + `URL.createObjectURL`).
     */
    archive(): Uint8Array;
    /**
     * Create a new EPUB builder.
     *
     * `info`     — plain JS object matching `EpubInfo`.
     * `chapters` — `string[][]` where index 0 of each inner array is the
     *              chapter title and remaining entries become `<p>` elements.
     */
    constructor(info: any, chapters: any);
    /**
     * Attach images. Pass an array of `{ id, path, data, cover }` objects
     * where `data` is a `Uint8Array` of the raw image bytes. Flag exactly one
     * with `cover: true` to set the book cover.
     */
    setImages(images: any): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_epub_free: (a: number, b: number) => void;
    readonly epub_archive: (a: number) => [number, number, number, number];
    readonly epub_new: (a: any, b: any) => [number, number, number];
    readonly epub_setImages: (a: number, b: any) => [number, number];
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
