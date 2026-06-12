// ESM wrapper for the napi-rs CJS platform binding loader.
// The loader (index.js) does platform detection and loads the correct .node binary.
import { createRequire } from 'node:module'

const _require = createRequire(import.meta.url)
const binding = _require('../index.js') as typeof import('../index.js')

export const { Epub } = binding
export type { EpubInfo, EpubImage } from '../index.js'
