/** Metadata for the EPUB document. */
export interface EpubInfo {
  title: string
  description: string
  publisher: string
  author: string
  /** Label used for the Table of Contents page. */
  tocTitle: string
  lang: string
  fonts: string[]
  /** Raw CSS injected as `styles.css`. Omit for an empty stylesheet. */
  css?: string
  /** EPUB spec version (use `3`). */
  version: number
  /** Raw XML written to `META-INF/encryption.xml`. Omit to skip. */
  encryption?: string
  /** Raw XML written to `META-INF/metadata.xml`. Omit to skip. */
  metadataXml?: string
  /** Raw XML written to `META-INF/manifest.xml`. Omit to skip. */
  manifestXml?: string
}
