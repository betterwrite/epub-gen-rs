import { defineConfig } from 'tsup'

export default defineConfig([
  {
    entry: { index: 'src/node.ts' },
    format: ['esm'],
    outExtension: () => ({ js: '.mjs' }),
    dts: true,
    external: [/\.node$/, /^node:/],
    outDir: 'dist',
    clean: true,
  },
  {
    entry: { browser: 'src/browser.ts' },
    format: ['esm'],
    dts: true,
    // Keep wasm glue as external — it loads its own .wasm at runtime.
    external: [/epub_gen_wasm/],
    outDir: 'dist',
  },
])
