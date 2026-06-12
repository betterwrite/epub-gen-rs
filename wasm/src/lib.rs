use serde::Deserialize;
use wasm_bindgen::prelude::*;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct EpubInfoInput {
  title: String,
  description: String,
  publisher: String,
  author: String,
  toc_title: String,
  lang: String,
  fonts: Vec<String>,
  css: Option<String>,
  version: i8,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct EpubImageInput {
  id: String,
  path: String,
  /// Raw image bytes (a JS `Uint8Array` / number array).
  data: Vec<u8>,
  #[serde(default)]
  cover: bool,
}

/// EPUB builder for browser environments.
///
/// ```js
/// import init, { Epub } from 'epub-gen3/wasm/epub_gen_wasm.js'
/// await init()
///
/// const epub = new Epub(
///   { title: 'My Book', description: '...', publisher: '...', author: '...',
///     tocTitle: 'Contents', lang: 'en', fonts: [], version: 3 },
///   [['Chapter One', 'First paragraph.', 'Second paragraph.']]
/// )
/// const bytes = epub.archive()          // Uint8Array
/// const blob  = new Blob([bytes], { type: 'application/epub+zip' })
/// const url   = URL.createObjectURL(blob)
/// ```
#[wasm_bindgen]
pub struct Epub {
  title: String,
  description: String,
  publisher: String,
  author: String,
  toc_title: String,
  lang: String,
  fonts: Vec<String>,
  css: Option<String>,
  version: i8,
  chapters: Vec<Vec<String>>,
  images: Vec<EpubImageInput>,
}

#[wasm_bindgen]
impl Epub {
  /// Create a new EPUB builder.
  ///
  /// `info`     — plain JS object matching `EpubInfo`.
  /// `chapters` — `string[][]` where index 0 of each inner array is the
  ///              chapter title and remaining entries become `<p>` elements.
  #[wasm_bindgen(constructor)]
  pub fn new(info: JsValue, chapters: JsValue) -> Result<Epub, JsError> {
    let info: EpubInfoInput = serde_wasm_bindgen::from_value(info)
      .map_err(|e| JsError::new(&e.to_string()))?;

    let chapters: Vec<Vec<String>> = serde_wasm_bindgen::from_value(chapters)
      .map_err(|e| JsError::new(&e.to_string()))?;

    Ok(Epub {
      title: info.title,
      description: info.description,
      publisher: info.publisher,
      author: info.author,
      toc_title: info.toc_title,
      lang: info.lang,
      fonts: info.fonts,
      css: info.css,
      version: info.version,
      chapters,
      images: Vec::new(),
    })
  }

  /// Attach images. Pass an array of `{ id, path, data, cover }` objects
  /// where `data` is a `Uint8Array` of the raw image bytes. Flag exactly one
  /// with `cover: true` to set the book cover.
  #[wasm_bindgen(js_name = setImages)]
  pub fn set_images(&mut self, images: JsValue) -> Result<(), JsError> {
    self.images = serde_wasm_bindgen::from_value(images)
      .map_err(|e| JsError::new(&e.to_string()))?;
    Ok(())
  }

  /// Build the EPUB and return its bytes as a `Uint8Array`.
  ///
  /// No filesystem access is performed — it is up to the caller to write or
  /// stream the returned bytes (e.g. via `Blob` + `URL.createObjectURL`).
  pub fn archive(&self) -> Result<Vec<u8>, JsError> {
    let epub = epub_gen::EPUB::new(
      epub_gen::Info {
        title: self.title.clone(),
        description: self.description.clone(),
        publisher: self.publisher.clone(),
        author: self.author.clone(),
        toc_title: self.toc_title.clone(),
        lang: self.lang.clone(),
        fonts: self.fonts.clone(),
        css: self.css.clone(),
        version: self.version,
      },
      self.chapters.clone(),
    );

    let images = self
      .images
      .iter()
      .map(|img| epub_gen::Image {
        id: img.id.clone(),
        path: img.path.clone(),
        data: img.data.clone(),
        cover: img.cover,
      })
      .collect();
    let epub = epub.with_images(images);

    epub.archive().map_err(|e| JsError::new(&e.to_string()))
  }
}
