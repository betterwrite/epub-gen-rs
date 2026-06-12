use std::fmt::Write as FmtWrite;
use std::fs;
use std::io::{Cursor, Write};
use zip::write::FileOptions;
use slugify::slugify;
use chrono::Utc;
use uuid::Uuid;

/// A CSS stylesheet included in the EPUB.
///
/// Each stylesheet is written to `OEBPS/<path>`, added to the OPF manifest
/// with `media-type="text/css"`, and linked from every content document via
/// `<link rel="stylesheet">` in declaration order.
pub struct Stylesheet {
  /// Unique manifest id (valid XML id — letters, digits, `-`, `_`).
  pub id: String,
  /// Path relative to `OEBPS/`, e.g. `"css/base.css"` or `"typography.css"`.
  pub path: String,
  /// Raw CSS content.
  pub content: String,
}

pub struct Info {
  pub title: String,
  pub description: String,
  pub publisher: String,
  pub author: String,
  pub toc_title: String,
  pub lang: String,
  pub fonts: Vec<String>,
  /// Convenience shorthand: raw CSS written to `OEBPS/styles.css` and linked
  /// first in every chapter. Use `stylesheets` when you need multiple files or
  /// custom paths.
  pub css: Option<String>,
  pub version: i8,
  /// Additional CSS stylesheets. Linked after `css` (if any), in order.
  pub stylesheets: Vec<Stylesheet>,
  /// Raw XML written to `META-INF/encryption.xml` when `Some`.
  /// Describes encryption applied to container resources (EPUB spec §4.3.3).
  pub encryption: Option<String>,
  /// Raw XML written to `META-INF/metadata.xml` when `Some`.
  /// Container-level metadata supplement (EPUB spec §4.3.4).
  pub metadata_xml: Option<String>,
  /// Raw XML written to `META-INF/manifest.xml` when `Some`.
  /// Lists files in the container beyond what the OPF covers (EPUB spec §4.3.5).
  pub manifest_xml: Option<String>,
}

/// An image embedded in the EPUB.
///
/// Images are written to `OEBPS/images/<path>` and can be referenced from
/// chapter paragraphs with raw markup, e.g.
/// `<img src="images/diagram.png" alt="Diagram" />`.
pub struct Image {
  /// Unique manifest id (must be a valid XML id — letters, digits, `-`, `_`).
  pub id: String,
  /// File name written under `OEBPS/images/`, e.g. `cover.png`.
  /// The extension determines the media-type.
  pub path: String,
  /// Raw image bytes.
  pub data: Vec<u8>,
  /// When `true`, this image is the book cover: it receives
  /// `properties="cover-image"` in the manifest, the EPUB 2
  /// `<meta name="cover">` hint, and an auto-generated cover page placed
  /// first in the spine. At most one image should set this.
  pub cover: bool,
}

pub struct EPUB {
  info: Info,
  chapters: Vec<Vec<String>>,
  images: Vec<Image>,
}

/// Maps a file extension to an EPUB 3 core media-type.
/// Falls back to `application/octet-stream` for unknown extensions.
fn media_type_for(path: &str) -> &'static str {
  let ext = path
    .rsplit('.')
    .next()
    .map(|e| e.to_ascii_lowercase())
    .unwrap_or_default();
  match ext.as_str() {
    "png" => "image/png",
    "jpg" | "jpeg" => "image/jpeg",
    "gif" => "image/gif",
    "svg" => "image/svg+xml",
    "webp" => "image/webp",
    _ => "application/octet-stream",
  }
}

impl EPUB {
  pub fn new(info: Info, chapters: Vec<Vec<String>>) -> EPUB {
    EPUB { info, chapters, images: Vec::new() }
  }

  /// Attach images to the EPUB. Returns `self` for chaining.
  pub fn with_images(mut self, images: Vec<Image>) -> EPUB {
    self.images = images;
    self
  }

  /// Replace the EPUB's images in place.
  pub fn set_images(&mut self, images: Vec<Image>) {
    self.images = images;
  }

  /// Replace the extra stylesheets in place.
  pub fn set_stylesheets(&mut self, stylesheets: Vec<Stylesheet>) {
    self.info.stylesheets = stylesheets;
  }

  /// The cover image, if any image was flagged `cover: true`.
  fn cover(&self) -> Option<&Image> {
    self.images.iter().find(|img| img.cover)
  }

  /// All stylesheets in link order: the legacy `css` field first (as
  /// `styles.css`), then every entry in `info.stylesheets`.
  fn all_stylesheets(&self) -> Vec<(&str, &str, &str)> {
    // (manifest-id, href, content)
    let mut list: Vec<(&str, &str, &str)> = Vec::new();
    if self.info.css.is_some() {
      list.push(("css", "styles.css", self.info.css.as_deref().unwrap_or("")));
    }
    for s in &self.info.stylesheets {
      list.push((&s.id, &s.path, &s.content));
    }
    list
  }

  pub fn run(&mut self) {
    let bytes = self.archive().expect("failed to build EPUB archive");
    self.write(bytes);
  }

  fn write_chapters(&self) -> Vec<(&String, String)> {
    self.chapters.iter().map(|chapter| {
      let title = &chapter[0];
      let content: String = chapter
        .iter()
        .skip(1)
        .map(|p| format!("      <p>{p}</p>\n"))
        .collect();

      // Built with write! to avoid Rust 2021 reserved-prefix errors on
      // identifier" patterns (e.g. bodymatter", chapter", css") inside
      // format!(r#"..."#).
      let mut s = String::new();
      s.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
      s.push_str("<!DOCTYPE html>\n");
      write!(s, "<html xmlns=\"http://www.w3.org/1999/xhtml\"\n").unwrap();
      write!(s, "      xmlns:epub=\"http://www.idpf.org/2007/ops\"\n").unwrap();
      write!(s, "      xml:lang=\"{0}\" lang=\"{0}\">\n", self.info.lang).unwrap();
      s.push_str("  <head>\n");
      s.push_str("    <meta charset=\"UTF-8\" />\n");
      write!(s, "    <title>{title}</title>\n").unwrap();
      for (_, href, _) in self.all_stylesheets() {
        write!(s, "    <link rel=\"stylesheet\" type=\"text/css\" href=\"{href}\" />\n").unwrap();
      }
      s.push_str("  </head>\n");
      s.push_str("  <body epub:type=\"bodymatter\">\n");
      s.push_str("    <section epub:type=\"chapter\">\n");
      write!(s, "      <h1>{title}</h1>\n").unwrap();
      s.push_str(&content);
      s.push_str("    </section>\n");
      s.push_str("  </body>\n");
      s.push_str("</html>\n");

      (title, s)
    }).collect()
  }

  fn manifest(&self) -> String {
    let items: String = self.chapters
      .iter()
      .map(|ch| {
        let id = slugify!(&ch[0]);
        let href = slugify!(&ch[0], separator = "_");
        format!("    <item id=\"{id}\" href=\"{href}.xhtml\" media-type=\"application/xhtml+xml\" />")
      })
      .collect::<Vec<_>>()
      .join("\n");

    // Image resources. The cover image carries properties="cover-image".
    let image_items: String = self.images
      .iter()
      .map(|img| {
        let mtype = media_type_for(&img.path);
        let props = if img.cover { " properties=\"cover-image\"" } else { "" };
        format!(
          "    <item id=\"{}\" href=\"images/{}\" media-type=\"{}\"{} />",
          img.id, img.path, mtype, props
        )
      })
      .collect::<Vec<_>>()
      .join("\n");

    // Auto-generated cover page (only when a cover image exists).
    let cover_page = if self.cover().is_some() {
      "\n    <item id=\"cover\" href=\"cover.xhtml\" media-type=\"application/xhtml+xml\" />"
    } else {
      ""
    };

    let css_items: String = self
      .all_stylesheets()
      .into_iter()
      .map(|(id, href, _)| {
        format!("    <item id=\"{id}\" href=\"{href}\" media-type=\"text/css\" />")
      })
      .collect::<Vec<_>>()
      .join("\n");

    let mut s = String::new();
    s.push_str("    <item id=\"ncx\" href=\"toc.ncx\" media-type=\"application/x-dtbncx+xml\" />\n");
    s.push_str("    <item id=\"toc\" href=\"toc.xhtml\" media-type=\"application/xhtml+xml\" properties=\"nav\" />\n");
    if !css_items.is_empty() {
      s.push_str(&css_items);
    }
    s.push_str(cover_page);
    if !items.is_empty() {
      s.push('\n');
      s.push_str(&items);
    }
    if !image_items.is_empty() {
      s.push('\n');
      s.push_str(&image_items);
    }
    s
  }

  // spine idrefs must match the manifest item ids exactly (both use slugify default separator)
  fn spine(&self) -> String {
    let items: String = self.chapters
      .iter()
      .map(|ch| format!("    <itemref idref=\"{}\" />", slugify!(&ch[0])))
      .collect::<Vec<_>>()
      .join("\n");

    // Cover page leads the spine when present.
    let cover_ref = if self.cover().is_some() {
      "    <itemref idref=\"cover\" />\n"
    } else {
      ""
    };

    format!("<spine toc=\"ncx\">\n{cover_ref}    <itemref idref=\"toc\" />\n{items}\n  </spine>")
  }

  // EPUB 3 cover page displaying the cover image. Only emitted when a cover exists.
  fn cover_xhtml(&self) -> Option<String> {
    let cover = self.cover()?;
    let mut s = String::new();
    s.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    s.push_str("<!DOCTYPE html>\n");
    s.push_str("<html xmlns=\"http://www.w3.org/1999/xhtml\"\n");
    s.push_str("      xmlns:epub=\"http://www.idpf.org/2007/ops\"\n");
    write!(s, "      xml:lang=\"{0}\" lang=\"{0}\">\n", self.info.lang).unwrap();
    s.push_str("  <head>\n");
    s.push_str("    <meta charset=\"UTF-8\" />\n");
    write!(s, "    <title>{}</title>\n", self.info.title).unwrap();
    s.push_str("    <style>img { max-width: 100%; height: auto; }</style>\n");
    s.push_str("  </head>\n");
    s.push_str("  <body epub:type=\"cover\">\n");
    s.push_str("    <section epub:type=\"cover\">\n");
    write!(
      s,
      "      <img src=\"images/{}\" alt=\"{}\" />\n",
      cover.path, self.info.title
    ).unwrap();
    s.push_str("    </section>\n");
    s.push_str("  </body>\n");
    s.push_str("</html>\n");
    Some(s)
  }

  fn toc_xhtml(&self) -> String {
    self.chapters
      .iter()
      .map(|ch| {
        let title = &ch[0];
        let href = format!("{}.xhtml", slugify!(title, separator = "_"));
        format!("        <li><a href=\"{href}\">{title}</a></li>")
      })
      .collect::<Vec<_>>()
      .join("\n")
  }

  fn toc_ncx(&self, uuid: &Uuid) -> String {
    let nav_points: String = self.chapters
      .iter()
      .enumerate()
      .map(|(i, ch)| {
        let id = slugify!(&ch[0]);
        let title = &ch[0];
        let src = format!("{}.xhtml", slugify!(&ch[0], separator = "_"));
        let order = i + 1;
        format!(
          "    <navPoint id=\"{id}\" playOrder=\"{order}\" class=\"chapter\">\n\
                 <navLabel><text>{title}</text></navLabel>\n\
                 <content src=\"{src}\"/>\n\
               </navPoint>"
        )
      })
      .collect::<Vec<_>>()
      .join("\n");

    let title = &self.info.title;
    let author = &self.info.author;
    let toc_title = &self.info.toc_title;

    // write! used throughout to avoid identifier" reserved-prefix issue
    let mut s = String::new();
    s.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    write!(s, "<ncx xmlns=\"http://www.daisy.org/z3986/2005/ncx/\" version=\"2005-1\">\n").unwrap();
    s.push_str("  <head>\n");
    write!(s, "    <meta name=\"dtb:uid\" content=\"urn:uuid:{uuid}\" />\n").unwrap();
    s.push_str("    <meta name=\"dtb:depth\" content=\"1\" />\n");
    s.push_str("    <meta name=\"dtb:totalPageCount\" content=\"0\" />\n");
    s.push_str("    <meta name=\"dtb:maxPageNumber\" content=\"0\" />\n");
    s.push_str("  </head>\n");
    write!(s, "  <docTitle><text>{title}</text></docTitle>\n").unwrap();
    write!(s, "  <docAuthor><text>{author}</text></docAuthor>\n").unwrap();
    s.push_str("  <navMap>\n");
    write!(s, "    <navPoint id=\"toc\" playOrder=\"0\" class=\"chapter\">\n").unwrap();
    write!(s, "      <navLabel><text>{toc_title}</text></navLabel>\n").unwrap();
    s.push_str("      <content src=\"toc.xhtml\"/>\n");
    s.push_str("    </navPoint>\n");
    write!(s, "{nav_points}\n").unwrap();
    s.push_str("  </navMap>\n");
    s.push_str("</ncx>\n");
    s
  }

  // Builds content.opf using write! to avoid Rust 2021 reserved-prefix errors
  // on identifier" sequences (relators", modified", fonts") inside format!(r#"..."#).
  fn opf(&self, uuid: &Uuid, modified: &str, today: &str, year: &str) -> String {
    let mut s = String::new();
    s.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    s.push_str("<package xmlns=\"http://www.idpf.org/2007/opf\"\n");
    s.push_str("         version=\"3.0\"\n");
    s.push_str("         unique-identifier=\"BookId\"\n");
    write!(s, "         xml:lang=\"{}\"\n", self.info.lang).unwrap();
    s.push_str("         prefix=\"ibooks: http://vocabulary.itunes.apple.com/rdf/ibooks/vocabulary-extensions-1.0/\">\n\n");
    s.push_str("  <metadata xmlns:dc=\"http://purl.org/dc/elements/1.1/\">\n");
    write!(s, "    <dc:identifier id=\"BookId\">urn:uuid:{uuid}</dc:identifier>\n").unwrap();
    write!(s, "    <dc:title>{}</dc:title>\n", self.info.title).unwrap();
    write!(s, "    <dc:language>{}</dc:language>\n", self.info.lang).unwrap();
    write!(s, "    <dc:creator id=\"creator\">{}</dc:creator>\n", self.info.author).unwrap();
    s.push_str("    <meta refines=\"#creator\" property=\"role\" scheme=\"marc:relators\">aut</meta>\n");
    write!(s, "    <dc:publisher>{}</dc:publisher>\n", self.info.publisher).unwrap();
    write!(s, "    <dc:description>{}</dc:description>\n", self.info.description).unwrap();
    write!(s, "    <dc:date>{today}</dc:date>\n").unwrap();
    write!(s, "    <dc:rights>Copyright &#x00A9; {year} {}</dc:rights>\n", self.info.publisher).unwrap();
    write!(s, "    <meta property=\"dcterms:modified\">{modified}</meta>\n").unwrap();
    s.push_str("    <meta property=\"ibooks:specified-fonts\">false</meta>\n");
    // EPUB 2 cover hint for reading systems that don't read cover-image properties.
    if let Some(cover) = self.cover() {
      write!(s, "    <meta name=\"cover\" content=\"{}\" />\n", cover.id).unwrap();
    }
    s.push_str("  </metadata>\n\n");
    write!(s, "  <manifest>\n{}\n  </manifest>\n\n", self.manifest()).unwrap();
    write!(s, "  {}\n\n", self.spine()).unwrap();
    write!(s, "  <guide>\n    <reference type=\"toc\" title=\"{}\" href=\"toc.xhtml\"/>\n  </guide>\n\n", self.info.toc_title).unwrap();
    s.push_str("</package>\n");
    s
  }

  // Builds toc.xhtml (EPUB 3 nav document) using write! for the same reason.
  fn nav_xhtml(&self) -> String {
    let mut s = String::new();
    s.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    s.push_str("<!DOCTYPE html>\n");
    write!(s, "<html xmlns=\"http://www.w3.org/1999/xhtml\"\n").unwrap();
    s.push_str("      xmlns:epub=\"http://www.idpf.org/2007/ops\"\n");
    write!(s, "      xml:lang=\"{0}\" lang=\"{0}\">\n", self.info.lang).unwrap();
    s.push_str("  <head>\n");
    s.push_str("    <meta charset=\"UTF-8\" />\n");
    write!(s, "    <title>{}</title>\n", self.info.title).unwrap();
    for (_, href, _) in self.all_stylesheets() {
      write!(s, "    <link rel=\"stylesheet\" type=\"text/css\" href=\"{href}\" />\n").unwrap();
    }
    s.push_str("  </head>\n");
    s.push_str("  <body epub:type=\"frontmatter\">\n");
    s.push_str("    <nav id=\"toc\" epub:type=\"toc\">\n");
    write!(s, "      <h1>{}</h1>\n", self.info.toc_title).unwrap();
    s.push_str("      <ol>\n");
    write!(s, "{}\n", self.toc_xhtml()).unwrap();
    s.push_str("      </ol>\n");
    s.push_str("    </nav>\n");
    s.push_str("  </body>\n");
    s.push_str("</html>\n");
    s
  }

  pub fn archive(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut buf = Cursor::new(Vec::new());
    let mut zip = zip::ZipWriter::new(&mut buf);

    // mimetype: Stored, NO extra fields (EPUB spec §3.4)
    let mimetype_opts = FileOptions::default()
      .compression_method(zip::CompressionMethod::Stored);

    let stored = FileOptions::default()
      .compression_method(zip::CompressionMethod::Stored)
      .unix_permissions(0o644);

    let deflated = FileOptions::default()
      .compression_method(zip::CompressionMethod::Deflated)
      .unix_permissions(0o644);

    // single UUID shared between content.opf and toc.ncx
    let uuid = Uuid::new_v4();
    // dcterms:modified requires UTC ISO 8601 with Z suffix
    let modified = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let today = Utc::now().format("%Y-%m-%d").to_string();
    let year = Utc::now().format("%Y").to_string();

    let chapters = self.write_chapters();

    // ── mimetype (must be first entry in ZIP) ───────────────────────────────
    zip.start_file("mimetype", mimetype_opts)?;
    zip.write_all(b"application/epub+zip")?;

    // ── META-INF (hyphen, not underscore) ───────────────────────────────────
    zip.add_directory("META-INF/", stored)?;
    zip.start_file("META-INF/container.xml", stored)?;
    zip.write_all(
      b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
        <container version=\"1.0\" xmlns=\"urn:oasis:names:tc:opendocument:xmlns:container\">\n\
          <rootfiles>\n\
            <rootfile full-path=\"OEBPS/content.opf\"\n\
                      media-type=\"application/oebps-package+xml\"/>\n\
          </rootfiles>\n\
        </container>",
    )?;

    if let Some(xml) = &self.info.encryption {
      zip.start_file("META-INF/encryption.xml", deflated)?;
      zip.write_all(xml.as_bytes())?;
    }
    if let Some(xml) = &self.info.metadata_xml {
      zip.start_file("META-INF/metadata.xml", deflated)?;
      zip.write_all(xml.as_bytes())?;
    }
    if let Some(xml) = &self.info.manifest_xml {
      zip.start_file("META-INF/manifest.xml", deflated)?;
      zip.write_all(xml.as_bytes())?;
    }

    // ── OEBPS ───────────────────────────────────────────────────────────────
    zip.add_directory("OEBPS/", deflated)?;

    zip.start_file("OEBPS/content.opf", deflated)?;
    zip.write_all(self.opf(&uuid, &modified, &today, &year).as_bytes())?;

    zip.start_file("OEBPS/toc.ncx", deflated)?;
    zip.write_all(self.toc_ncx(&uuid).as_bytes())?;

    zip.start_file("OEBPS/toc.xhtml", deflated)?;
    zip.write_all(self.nav_xhtml().as_bytes())?;

    if let Some(cover_xhtml) = self.cover_xhtml() {
      zip.start_file("OEBPS/cover.xhtml", deflated)?;
      zip.write_all(cover_xhtml.as_bytes())?;
    }

    for (title, xhtml) in &chapters {
      zip.start_file(
        format!("OEBPS/{}.xhtml", slugify!(title, separator = "_")),
        stored,
      )?;
      zip.write_all(xhtml.as_bytes())?;
    }

    // ── images (already-compressed formats stored without re-compression) ────
    if !self.images.is_empty() {
      zip.add_directory("OEBPS/images/", deflated)?;
      for img in &self.images {
        // SVG is text and benefits from Deflate; raster formats are stored.
        let opts = if media_type_for(&img.path) == "image/svg+xml" {
          deflated
        } else {
          stored
        };
        zip.start_file(format!("OEBPS/images/{}", img.path), opts)?;
        zip.write_all(&img.data)?;
      }
    }

    for (_, href, content) in self.all_stylesheets() {
      zip.start_file(format!("OEBPS/{href}"), deflated)?;
      zip.write_all(content.as_bytes())?;
    }

    Ok(zip.finish()?.clone().into_inner())
  }

  pub fn write(&mut self, data: Vec<u8>) {
    fs::write(format!("{}.epub", self.info.title), data).ok();
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  macro_rules! chapter {
    ($($x:expr),*) => (vec![$($x.to_string()),*]);
  }

  #[test]
  fn it_build() {
    let mut epub = EPUB::new(
      Info {
        title: String::from("test"),
        description: String::from("test description"),
        publisher: String::from("test publisher"),
        author: String::from("test author"),
        toc_title: String::from("Table of Contents"),
        lang: String::from("en"),
        fonts: vec![],
        css: None,
        version: 3,
        stylesheets: vec![],
        encryption: None,
        metadata_xml: None,
        manifest_xml: None,
      },
      vec![
        chapter![
          "Chapter One",
          "Nullam tempor, metus vitae sagittis semper, massa nulla posuere ipsum, nec mollis tortor dui sed enim.",
          "Aliquam non posuere ex. Duis fermentum odio metus, quis ultrices nulla cursus vitae."
        ],
        chapter![
          "Chapter Two",
          "Pellentesque tempor, eros eu consectetur cursus, magna turpis lacinia nunc."
        ],
      ],
    );

    epub.run();
  }

  // A 1x1 transparent PNG.
  const PNG_1X1: [u8; 67] = [
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D,
    0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
    0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00, 0x00, 0x00,
    0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
    0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
    0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
  ];

  #[test]
  fn it_build_with_images() {
    let epub = EPUB::new(
      Info {
        title: String::from("test_images"),
        description: String::from("test description"),
        publisher: String::from("test publisher"),
        author: String::from("test author"),
        toc_title: String::from("Table of Contents"),
        lang: String::from("en"),
        fonts: vec![],
        css: None,
        version: 3,
        stylesheets: vec![],
        encryption: None,
        metadata_xml: None,
        manifest_xml: None,
      },
      vec![
        chapter![
          "Chapter One",
          "Some text before the image.",
          "<img src=\"images/inline.png\" alt=\"inline\" />"
        ],
      ],
    )
    .with_images(vec![
      Image {
        id: String::from("cover-img"),
        path: String::from("cover.png"),
        data: PNG_1X1.to_vec(),
        cover: true,
      },
      Image {
        id: String::from("inline-img"),
        path: String::from("inline.png"),
        data: PNG_1X1.to_vec(),
        cover: false,
      },
    ]);

    let bytes = epub.archive().expect("archive should build");
    assert!(!bytes.is_empty());

    // Verify the cover page, image entries, and manifest cover property exist.
    let reader = std::io::Cursor::new(bytes);
    let mut zip = zip::ZipArchive::new(reader).expect("valid zip");
    let names: Vec<String> = (0..zip.len())
      .map(|i| zip.by_index(i).unwrap().name().to_string())
      .collect();
    assert!(names.contains(&"OEBPS/cover.xhtml".to_string()));
    assert!(names.contains(&"OEBPS/images/cover.png".to_string()));
    assert!(names.contains(&"OEBPS/images/inline.png".to_string()));

    let mut opf = String::new();
    use std::io::Read;
    zip.by_name("OEBPS/content.opf").unwrap().read_to_string(&mut opf).unwrap();
    assert!(opf.contains("properties=\"cover-image\""));
    assert!(opf.contains("<meta name=\"cover\" content=\"cover-img\""));
    assert!(opf.contains("media-type=\"image/png\""));
    assert!(opf.contains("<itemref idref=\"cover\" />"));
  }
}
