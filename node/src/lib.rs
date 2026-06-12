#![deny(clippy::all)]

use napi::bindgen_prelude::Buffer;
use napi_derive::napi;

/// Metadata for the EPUB document.
#[napi(object)]
pub struct EpubInfo {
  pub title: String,
  pub description: String,
  pub publisher: String,
  pub author: String,
  /// Label used for the Table of Contents page.
  pub toc_title: String,
  pub lang: String,
  pub fonts: Vec<String>,
  /// Raw CSS string injected as `styles.css`. Pass `null` for an empty stylesheet.
  pub css: Option<String>,
  /// EPUB spec version (use `3`).
  pub version: i32,
  /// Raw XML written to `META-INF/encryption.xml`. Pass `null` to omit.
  pub encryption: Option<String>,
  /// Raw XML written to `META-INF/metadata.xml`. Pass `null` to omit.
  pub metadata_xml: Option<String>,
  /// Raw XML written to `META-INF/manifest.xml`. Pass `null` to omit.
  pub manifest_xml: Option<String>,
}

/// A CSS stylesheet included in the EPUB.
#[napi(object)]
pub struct EpubStylesheet {
  /// Unique manifest id (letters, digits, `-`, `_`).
  pub id: String,
  /// Path relative to `OEBPS/`, e.g. `"css/typography.css"`.
  pub path: String,
  /// Raw CSS content.
  pub content: String,
}

/// An image embedded in the EPUB.
///
/// Images are written to `OEBPS/images/<path>` and can be referenced from
/// chapter paragraphs with raw markup, e.g.
/// `<img src="images/diagram.png" alt="Diagram" />`.
#[napi(object)]
pub struct EpubImage {
  /// Unique manifest id (letters, digits, `-`, `_`).
  pub id: String,
  /// File name written under `OEBPS/images/`, e.g. `cover.png`.
  /// The extension determines the media-type.
  pub path: String,
  /// Raw image bytes.
  pub data: Buffer,
  /// When `true`, this image is the book cover (at most one).
  pub cover: bool,
}

/// EPUB builder.
///
/// Each chapter is an array of strings: index 0 is the chapter title,
/// subsequent entries become `<p>` elements in the chapter body.
///
/// ```js
/// const epub = new Epub(info, [
///   ['Chapter One', 'First paragraph.', 'Second paragraph.'],
/// ]);
/// epub.setImages([
///   { id: 'cover-img', path: 'cover.png', data: coverBytes, cover: true },
/// ]);
/// ```
#[napi]
pub struct Epub {
  inner: epub_gen::EPUB,
}

#[napi]
impl Epub {
  /// Create a new EPUB builder.
  #[napi(constructor)]
  pub fn new(info: EpubInfo, chapters: Vec<Vec<String>>) -> Self {
    Epub {
      inner: epub_gen::EPUB::new(
        epub_gen::Info {
          title: info.title,
          description: info.description,
          publisher: info.publisher,
          author: info.author,
          toc_title: info.toc_title,
          lang: info.lang,
          fonts: info.fonts,
          css: info.css,
          version: info.version as i8,
          stylesheets: vec![],
          encryption: info.encryption,
          metadata_xml: info.metadata_xml,
          manifest_xml: info.manifest_xml,
        },
        chapters,
      ),
    }
  }

  /// Attach additional CSS stylesheets. Linked after `css` (from `EpubInfo`), in order.
  #[napi]
  pub fn set_stylesheets(&mut self, stylesheets: Vec<EpubStylesheet>) {
    let sheets = stylesheets
      .into_iter()
      .map(|s| epub_gen::Stylesheet { id: s.id, path: s.path, content: s.content })
      .collect();
    self.inner.set_stylesheets(sheets);
  }

  /// Attach images to the EPUB. Pass an array of `EpubImage` objects.
  /// Flag exactly one with `cover: true` to set the book cover.
  #[napi]
  pub fn set_images(&mut self, images: Vec<EpubImage>) {
    let images = images
      .into_iter()
      .map(|img| epub_gen::Image {
        id: img.id,
        path: img.path,
        data: img.data.to_vec(),
        cover: img.cover,
      })
      .collect();
    self.inner.set_images(images);
  }

  /// Build the EPUB and write it to `<title>.epub` in the current working directory.
  #[napi]
  pub fn run(&mut self) {
    self.inner.run();
  }

  /// Build the EPUB and return its contents as a `Buffer`.
  /// Useful when you want to stream, upload, or write the file yourself.
  ///
  /// ```js
  /// const buf = epub.archive();
  /// fs.writeFileSync('my-book.epub', buf);
  /// ```
  #[napi]
  pub fn archive(&self) -> napi::Result<Buffer> {
    let bytes = self
      .inner
      .archive()
      .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    Ok(bytes.into())
  }
}
