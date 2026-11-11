use std::fmt::Write;
use std::path::Path;

use lopdf::{Dictionary, Document, Object, Stream, text_string};
use pdfium_render::prelude::PdfDocumentMetadataTagType;

use super::PdfDate;
use crate::engine::PdfEngine;
use crate::error::PdfError;
use crate::files::{self, FileFacts};

const KEYWORD_SEPARATOR: &str = ", ";

/// What a PDF carries about itself in its document information and XMP: title, author (the
/// correspondent), subject (the category), keywords (the tags) and the document date.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PdfMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub keywords: Vec<String>,
    pub date: Option<PdfDate>,
}

impl PdfEngine {
    /// The document information of the PDF; blank values count as missing, the date is the UTC date of
    /// the creation date.
    ///
    /// # Errors
    /// [`PdfError::NotFound`] or [`PdfError::UnsupportedFormat`] if the file cannot be opened.
    pub fn read_metadata(self, path: &Path) -> Result<PdfMetadata, PdfError> {
        let document = self.open(path)?;
        let metadata = document.metadata();
        let value =
            |tag| metadata.get(tag).map(|entry| entry.value().trim().to_string()).filter(|value| !value.is_empty());
        Ok(PdfMetadata {
            title: value(PdfDocumentMetadataTagType::Title),
            author: value(PdfDocumentMetadataTagType::Author),
            subject: value(PdfDocumentMetadataTagType::Subject),
            keywords: value(PdfDocumentMetadataTagType::Keywords)
                .map(|keywords| {
                    keywords.split(',').map(str::trim).filter(|k| !k.is_empty()).map(String::from).collect()
                })
                .unwrap_or_default(),
            date: value(PdfDocumentMetadataTagType::CreationDate).and_then(|date| PdfDate::parse_utc(&date)),
        })
    }

    /// A copy of [`source`] with [`metadata`] in its document information and XMP, so other applications
    /// find them; values that are `None` or empty are removed. PDFium cannot write the document
    /// information, so `lopdf` rewrites the file, and PDFium checks the result before it is placed.
    ///
    /// # Errors
    /// [`PdfError::AlreadyExists`] if [`target`] exists; [`PdfError::UnsupportedFormat`] if the source
    /// cannot be read or the rewritten file is not the same document.
    pub fn write_metadata(self, source: &Path, metadata: &PdfMetadata, target: &Path) -> Result<FileFacts, PdfError> {
        let pages = self.page_count(source)?;
        let mut document = Document::load(source).map_err(PdfError::unsupported)?;
        set_info(&mut document, metadata)?;
        let xmp = Stream::new(
            Dictionary::from_iter([
                ("Type", Object::Name(b"Metadata".to_vec())),
                ("Subtype", Object::Name(b"XML".to_vec())),
            ]),
            xmp_of(metadata).into_bytes(),
        );
        let xmp_id = document.add_object(xmp);
        document.catalog_mut().map_err(PdfError::unsupported)?.set("Metadata", Object::Reference(xmp_id));
        files::write_new(target, |temp| {
            document.save(temp).map_err(PdfError::unsupported)?;
            // lopdf rewrites the whole file; a result PDFium reads differently never replaces anything.
            if self.page_count(temp)? == pages {
                Ok(())
            } else {
                Err(PdfError::unsupported("the rewritten PDF lost pages"))
            }
        })
    }
}

fn set_info(document: &mut Document, metadata: &PdfMetadata) -> Result<(), PdfError> {
    let info = match document.trailer.get(b"Info") {
        Ok(Object::Reference(id)) => document.get_dictionary_mut(*id).map_err(PdfError::unsupported)?,
        Ok(Object::Dictionary(_)) => {
            document.trailer.get_mut(b"Info").and_then(Object::as_dict_mut).map_err(PdfError::unsupported)?
        }
        _ => {
            let id = document.add_object(Dictionary::new());
            document.trailer.set("Info", Object::Reference(id));
            document.get_dictionary_mut(id).map_err(PdfError::unsupported)?
        }
    };
    let keywords = Some(metadata.keywords.join(KEYWORD_SEPARATOR)).filter(|k| !k.is_empty());
    let entries = [
        ("Title", metadata.title.clone()),
        ("Author", metadata.author.clone()),
        ("Subject", metadata.subject.clone()),
        ("Keywords", keywords),
    ];
    for (key, value) in entries {
        match value {
            Some(value) => info.set(key, text_string(&value)),
            None => {
                info.remove(key.as_bytes());
            }
        }
    }
    match metadata.date {
        Some(date) => info.set("CreationDate", Object::string_literal(date.to_pdf_string())),
        None => {
            info.remove(b"CreationDate");
        }
    }
    Ok(())
}

/// The Dublin Core and Adobe PDF schemas, as PDFBox's `XmpSerializer` writes them.
fn xmp_of(metadata: &PdfMetadata) -> String {
    let mut dublin_core = String::new();
    if let Some(title) = &metadata.title {
        let _ = write!(
            dublin_core,
            "<dc:title><rdf:Alt><rdf:li xml:lang=\"x-default\">{}</rdf:li></rdf:Alt></dc:title>",
            escaped(title)
        );
    }
    if let Some(author) = &metadata.author {
        let _ = write!(dublin_core, "<dc:creator><rdf:Seq><rdf:li>{}</rdf:li></rdf:Seq></dc:creator>", escaped(author));
    }
    if let Some(subject) = &metadata.subject {
        let _ = write!(
            dublin_core,
            "<dc:description><rdf:Alt><rdf:li xml:lang=\"x-default\">{}</rdf:li></rdf:Alt></dc:description>",
            escaped(subject)
        );
    }
    if !metadata.keywords.is_empty() {
        let items = metadata.keywords.iter().fold(String::new(), |mut items, k| {
            let _ = write!(items, "<rdf:li>{}</rdf:li>", escaped(k));
            items
        });
        let _ = write!(dublin_core, "<dc:subject><rdf:Bag>{items}</rdf:Bag></dc:subject>");
    }
    if let Some(date) = metadata.date {
        let _ = write!(dublin_core, "<dc:date><rdf:Seq><rdf:li>{}</rdf:li></rdf:Seq></dc:date>", date.to_xmp_string());
    }
    format!(
        "<?xpacket begin=\"\u{feff}\" id=\"W5M0MpCehiHzreSzNTczkc9d\"?>\n\
         <x:xmpmeta xmlns:x=\"adobe:ns:meta/\"><rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\">\
         <rdf:Description rdf:about=\"\" xmlns:dc=\"http://purl.org/dc/elements/1.1/\">{dublin_core}</rdf:Description>\
         <rdf:Description rdf:about=\"\" xmlns:pdf=\"http://ns.adobe.com/pdf/1.3/\"><pdf:Keywords>{}</pdf:Keywords>\
         </rdf:Description></rdf:RDF></x:xmpmeta>\n<?xpacket end=\"w\"?>",
        escaped(&metadata.keywords.join(KEYWORD_SEPARATOR))
    )
}

/// XML text: markup characters escaped, characters XML 1.0 forbids left out.
fn escaped(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\t' | '\n' | '\r' => result.push(character),
            c if c.is_control() => {}
            c => result.push(c),
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xmp_holds_every_field_escaped() {
        let metadata = PdfMetadata {
            title: Some("Bill <March> & more".into()),
            author: Some("Sample Energy".into()),
            subject: Some("Bills / Power".into()),
            keywords: vec!["Tax".into(), "Home".into()],
            date: Some(PdfDate { year: 2024, month: 3, day: 15 }),
        };

        let xmp = xmp_of(&metadata);

        assert!(xmp.contains("Bill &lt;March&gt; &amp; more"), "{xmp}");
        assert!(xmp.contains("<rdf:li>Sample Energy</rdf:li>"));
        assert!(xmp.contains("<rdf:Bag><rdf:li>Tax</rdf:li><rdf:li>Home</rdf:li></rdf:Bag>"));
        assert!(xmp.contains("<pdf:Keywords>Tax, Home</pdf:Keywords>"));
        assert!(xmp.contains("2024-03-15T00:00:00Z"));
    }
}
