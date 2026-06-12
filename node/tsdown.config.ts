import { defineConfig } from 'tsdown'

export default defineConfig([
  {
    entry: { index: 'src/node.ts' },
    format: 'esm',
    outExtensions: () => ({ js: '.mjs' }),
    dts: true,
    deps: { neverBundle: [/\.node$/, /^node:/] },
    outDir: 'dist',
    clean: true,
  },
  {
    entry: { browser: 'src/browser.ts' },
    format: 'esm',
    dts: true,
    // Keep wasm glue as external — it loads its own .wasm at runtime.
    deps: { neverBundle: [/epub_gen_wasm/] },
    outDir: 'dist',
  },
])
