use std::fmt::Write as FmtWrite;
use std::fs;
use std::io::{Cursor, Write};
use zip::write::FileOptions;
use slugify::slugify;
use chrono::Utc;
use uuid::Uuid;

pub struct Info {
  pub title: String,
  pub description: String,
  pub publisher: String,
  pub author: String,
  pub toc_title: String,
  pub lang: String,
  pub fonts: Vec<String>,
  pub css: Option<String>,
  pub version: i8,
}

pub struct EPUB {
  info: Info,
  chapters: Vec<Vec<String>>,
}

impl EPUB {
  pub fn new(info: Info, chapters: Vec<Vec<String>>) -> EPUB {
    EPUB { info, chapters }
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
      s.push_str("    <link rel=\"stylesheet\" type=\"text/css\" href=\"styles.css\" />\n");
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

    format!(
      "    <item id=\"ncx\" href=\"toc.ncx\" media-type=\"application/x-dtbncx+xml\" />\n\
       <item id=\"toc\" href=\"toc.xhtml\" media-type=\"application/xhtml+xml\" properties=\"nav\" />\n\
       <item id=\"css\" href=\"styles.css\" media-type=\"text/css\" />\n\
       {items}"
    )
  }

  // spine idrefs must match the manifest item ids exactly (both use slugify default separator)
  fn spine(&self) -> String {
    let items: String = self.chapters
      .iter()
      .map(|ch| format!("    <itemref idref=\"{}\" />", slugify!(&ch[0])))
      .collect::<Vec<_>>()
      .join("\n");

    format!("<spine toc=\"ncx\">\n    <itemref idref=\"toc\" />\n{items}\n  </spine>")
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
    s.push_str("    <link rel=\"stylesheet\" type=\"text/css\" href=\"styles.css\" />\n");
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

    // ── OEBPS ───────────────────────────────────────────────────────────────
    zip.add_directory("OEBPS/", deflated)?;

    zip.start_file("OEBPS/content.opf", deflated)?;
    zip.write_all(self.opf(&uuid, &modified, &today, &year).as_bytes())?;

    zip.start_file("OEBPS/toc.ncx", deflated)?;
    zip.write_all(self.toc_ncx(&uuid).as_bytes())?;

    zip.start_file("OEBPS/toc.xhtml", deflated)?;
    zip.write_all(self.nav_xhtml().as_bytes())?;

    for (title, xhtml) in &chapters {
      zip.start_file(
        format!("OEBPS/{}.xhtml", slugify!(title, separator = "_")),
        stored,
      )?;
      zip.write_all(xhtml.as_bytes())?;
    }

    zip.start_file("OEBPS/styles.css", deflated)?;
    match &self.info.css {
      Some(css) => zip.write_all(css.as_bytes())?,
      None => zip.write_all(b"")?,
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
}
