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
        .map(|p| format!("      <p>{}</p>\n", p))
        .collect();

      let xhtml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml"
      xmlns:epub="http://www.idpf.org/2007/ops"
      xml:lang="{lang}" lang="{lang}">
  <head>
    <meta charset="UTF-8" />
    <title>{title}</title>
    <link rel="stylesheet" type="text/css" href="styles.css" />
  </head>
  <body epub:type="bodymatter">
    <section epub:type="chapter">
      <h1>{title}</h1>
{content}    </section>
  </body>
</html>"#,
        lang = self.info.lang,
        title = title,
        content = content,
      );

      (title, xhtml)
    }).collect()
  }

  fn manifest(&self) -> String {
    let items: String = self.chapters
      .iter()
      .map(|ch| {
        // id uses default slugify separator (-), href uses (_) for the filename
        let id = slugify!(&ch[0]);
        let href = slugify!(&ch[0], separator = "_");
        format!(
          r#"    <item id="{id}" href="{href}.xhtml" media-type="application/xhtml+xml" />"#,
        )
      })
      .collect::<Vec<_>>()
      .join("\n");

    format!(
      r#"    <item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml" />
    <item id="toc" href="toc.xhtml" media-type="application/xhtml+xml" properties="nav" />
    <item id="css" href="styles.css" media-type="text/css" />
{items}"#,
    )
  }

  // spine idrefs must match the manifest item ids exactly
  fn spine(&self) -> String {
    let items: String = self.chapters
      .iter()
      .map(|ch| format!(r#"    <itemref idref="{}" />"#, slugify!(&ch[0])))
      .collect::<Vec<_>>()
      .join("\n");

    format!(
      "<spine toc=\"ncx\">\n    <itemref idref=\"toc\" />\n{items}\n  </spine>",
    )
  }

  fn toc_xhtml(&self) -> String {
    self.chapters
      .iter()
      .map(|ch| {
        let title = &ch[0];
        let href = format!("{}.xhtml", slugify!(title, separator = "_"));
        format!(r#"        <li><a href="{href}">{title}</a></li>"#)
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
        format!(
          r#"    <navPoint id="{id}" playOrder="{order}" class="chapter">
      <navLabel><text>{title}</text></navLabel>
      <content src="{src}"/>
    </navPoint>"#,
          order = i + 1,
        )
      })
      .collect::<Vec<_>>()
      .join("\n");

    format!(
      r#"<?xml version="1.0" encoding="UTF-8"?>
<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/" version="2005-1">
  <head>
    <meta name="dtb:uid" content="urn:uuid:{uuid}" />
    <meta name="dtb:depth" content="1" />
    <meta name="dtb:totalPageCount" content="0" />
    <meta name="dtb:maxPageNumber" content="0" />
  </head>
  <docTitle><text>{title}</text></docTitle>
  <docAuthor><text>{author}</text></docAuthor>
  <navMap>
    <navPoint id="toc" playOrder="0" class="chapter">
      <navLabel><text>{toc_title}</text></navLabel>
      <content src="toc.xhtml"/>
    </navPoint>
{nav_points}
  </navMap>
</ncx>"#,
      title = self.info.title,
      author = self.info.author,
      toc_title = self.info.toc_title,
    )
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
      br#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf"
              media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#,
    )?;

    // ── OEBPS ───────────────────────────────────────────────────────────────
    zip.add_directory("OEBPS/", deflated)?;

    // content.opf: <spine> is a direct child of <package>, never inside <guide>
    zip.start_file("OEBPS/content.opf", deflated)?;
    let opf = format!(
      r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf"
         version="3.0"
         unique-identifier="BookId"
         xml:lang="{lang}"
         prefix="ibooks: http://vocabulary.itunes.apple.com/rdf/ibooks/vocabulary-extensions-1.0/">

  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:identifier id="BookId">urn:uuid:{uuid}</dc:identifier>
    <dc:title>{title}</dc:title>
    <dc:language>{lang}</dc:language>
    <dc:creator id="creator">{author}</dc:creator>
    <meta refines="#creator" property="role" scheme="marc:relators">aut</meta>
    <dc:publisher>{publisher}</dc:publisher>
    <dc:description>{description}</dc:description>
    <dc:date>{today}</dc:date>
    <dc:rights>Copyright &#x00A9; {year} {publisher}</dc:rights>
    <meta property="dcterms:modified">{modified}</meta>
    <meta property="ibooks:specified-fonts">false</meta>
  </metadata>

  <manifest>
{manifest}
  </manifest>

  {spine}

  <guide>
    <reference type="toc" title="{toc_title}" href="toc.xhtml"/>
  </guide>

</package>"#,
      lang = self.info.lang,
      uuid = uuid,
      title = self.info.title,
      author = self.info.author,
      publisher = self.info.publisher,
      description = self.info.description,
      today = today,
      year = year,
      modified = modified,
      toc_title = self.info.toc_title,
      manifest = self.manifest(),
      spine = self.spine(),
    );
    zip.write_all(opf.as_bytes())?;

    // toc.ncx — EPUB 2 backwards compat; shares the same UUID as content.opf
    zip.start_file("OEBPS/toc.ncx", deflated)?;
    zip.write_all(self.toc_ncx(&uuid).as_bytes())?;

    // toc.xhtml — EPUB 3 nav document
    zip.start_file("OEBPS/toc.xhtml", deflated)?;
    let nav = format!(
      r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml"
      xmlns:epub="http://www.idpf.org/2007/ops"
      xml:lang="{lang}" lang="{lang}">
  <head>
    <meta charset="UTF-8" />
    <title>{title}</title>
    <link rel="stylesheet" type="text/css" href="styles.css" />
  </head>
  <body epub:type="frontmatter">
    <nav id="toc" epub:type="toc">
      <h1>{toc_title}</h1>
      <ol>
{items}
      </ol>
    </nav>
  </body>
</html>"#,
      lang = self.info.lang,
      title = self.info.title,
      toc_title = self.info.toc_title,
      items = self.toc_xhtml(),
    );
    zip.write_all(nav.as_bytes())?;

    // chapter XHTML files
    for (title, xhtml) in &chapters {
      zip.start_file(
        format!("OEBPS/{}.xhtml", slugify!(title, separator = "_")),
        stored,
      )?;
      zip.write_all(xhtml.as_bytes())?;
    }

    // styles.css
    zip.start_file("OEBPS/styles.css", deflated)?;
    match &self.info.css {
      Some(css) => zip.write_all(css.as_bytes())?,
      None => zip.write_all(b"")?,
    }

    Ok(zip.finish()?.into_inner())
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
