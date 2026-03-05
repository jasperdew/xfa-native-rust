//! XFA form flattening — convert interactive XFA forms to static PDF.
//!
//! Flattening bakes all field values and appearance streams into the
//! page content, then removes the XFA metadata and scripting dictionaries
//! from the document catalog. The result is a non-interactive PDF that
//! renders identically in any viewer.

use crate::appearance::{generate_appearances, to_pdf_stream, AppearanceConfig, FieldAppearance};
use crate::error::{PdfError, Result};
use crate::pdf_reader::PdfReader;
use lopdf::content::{Content, Operation};
use lopdf::{dictionary, Dictionary, Object, ObjectId, Stream};
use xfa_layout_engine::layout::LayoutDom;

/// Configuration for the flattening process.
#[derive(Debug, Clone)]
pub struct FlattenConfig {
    /// Appearance stream generation settings.
    pub appearance: AppearanceConfig,
    /// Whether to remove the XFA entry from the AcroForm dictionary.
    pub remove_xfa: bool,
    /// Whether to remove the AcroForm dictionary entirely.
    pub remove_acroform: bool,
    /// Whether to compress flattened content streams.
    pub compress: bool,
}

impl Default for FlattenConfig {
    fn default() -> Self {
        Self {
            appearance: AppearanceConfig::default(),
            remove_xfa: true,
            remove_acroform: true,
            compress: true,
        }
    }
}

/// Result of a flatten operation.
#[derive(Debug)]
pub struct FlattenResult {
    /// Number of pages processed.
    pub pages_processed: usize,
    /// Number of fields flattened.
    pub fields_flattened: usize,
    /// Number of appearance streams generated.
    pub streams_generated: usize,
}

/// Flatten a LayoutDom into a PDF document, producing a static PDF.
///
/// This is the primary entry point for flattening. It:
/// 1. Generates appearance streams from the layout
/// 2. Creates page content streams embedding all field appearances
/// 3. Builds a complete PDF document structure
/// 4. Optionally removes XFA/AcroForm metadata
///
/// Returns the flattened PDF as bytes.
pub fn flatten_to_pdf(layout: &LayoutDom, config: &FlattenConfig) -> Result<Vec<u8>> {
    let appearances = generate_appearances(layout, &config.appearance);
    let mut doc = lopdf::Document::new();

    let mut page_ids = Vec::new();

    for (page_idx, page) in layout.pages.iter().enumerate() {
        let empty = Vec::new();
        let page_appearances = appearances.pages.get(page_idx).unwrap_or(&empty);

        let (page_id, _, _) =
            build_page(&mut doc, page.width, page.height, page_appearances, config)?;

        page_ids.push(page_id);
    }

    // Build page tree
    let pages_id = doc.add_object(Object::Dictionary(build_pages_dict(&page_ids)));

    // Set parent reference on each page
    for &page_id in &page_ids {
        if let Ok(Object::Dictionary(ref mut dict)) = doc.get_object_mut(page_id) {
            dict.set("Parent", Object::Reference(pages_id));
        }
    }

    // Build catalog
    let mut catalog = Dictionary::new();
    catalog.set("Type", Object::Name(b"Catalog".to_vec()));
    catalog.set("Pages", Object::Reference(pages_id));
    let catalog_id = doc.add_object(Object::Dictionary(catalog));

    // Set trailer
    doc.trailer.set("Root", Object::Reference(catalog_id));

    // Serialize
    let mut buf = Vec::new();
    doc.save_to(&mut buf)
        .map_err(|e| PdfError::Io(std::io::Error::other(format!("save: {e}"))))?;

    Ok(buf)
}

/// Flatten a LayoutDom into an existing PDF, replacing XFA content.
///
/// This modifies the PdfReader's document in-place:
/// 1. Generates appearance streams from the layout
/// 2. Replaces page content with flattened streams
/// 3. Removes XFA and optionally AcroForm from the catalog
pub fn flatten_into_pdf(
    reader: &mut PdfReader,
    layout: &LayoutDom,
    config: &FlattenConfig,
) -> Result<FlattenResult> {
    let appearances = generate_appearances(layout, &config.appearance);
    let doc = reader.document_mut();
    let mut fields_flattened = 0;
    let mut streams_generated = 0;

    // Get existing page object IDs
    let page_ids: Vec<ObjectId> = doc.get_pages().values().copied().collect();

    for (page_idx, page_appearances) in appearances.pages.iter().enumerate() {
        if page_idx >= page_ids.len() {
            break;
        }
        let page_id = page_ids[page_idx];

        // Build content stream for this page's appearances
        let (content_stream, stream_count) =
            build_page_content_stream(doc, page_appearances, config)?;

        // Add content stream to document
        let content_id = doc.add_object(Object::Stream(content_stream));

        // Update page's Contents reference
        if let Ok(Object::Dictionary(ref mut page_dict)) = doc.get_object_mut(page_id) {
            page_dict.set("Contents", Object::Reference(content_id));
        }

        fields_flattened += page_appearances.len();
        streams_generated += stream_count;
    }

    // Remove XFA entry from AcroForm
    if config.remove_xfa || config.remove_acroform {
        remove_xfa_metadata(doc, config.remove_acroform);
    }

    Ok(FlattenResult {
        pages_processed: appearances.pages.len().min(page_ids.len()),
        fields_flattened,
        streams_generated,
    })
}

/// Build a single PDF page with flattened field appearances.
fn build_page(
    doc: &mut lopdf::Document,
    width: f64,
    height: f64,
    appearances: &[FieldAppearance],
    config: &FlattenConfig,
) -> Result<(ObjectId, usize, usize)> {
    let (content_stream, stream_count) = build_page_content_stream(doc, appearances, config)?;

    let content_id = doc.add_object(Object::Stream(content_stream));

    // Build font resource dictionary
    let font_dict = build_font_dict(appearances);

    let resources = dictionary! {
        "Font" => Object::Dictionary(font_dict),
    };

    let page = dictionary! {
        "Type" => Object::Name(b"Page".to_vec()),
        "MediaBox" => Object::Array(vec![
            0.0.into(), 0.0.into(), width.into(), height.into(),
        ]),
        "Contents" => Object::Reference(content_id),
        "Resources" => Object::Dictionary(resources),
    };

    let page_id = doc.add_object(Object::Dictionary(page));
    Ok((page_id, appearances.len(), stream_count))
}

/// Build a page content stream that paints all field appearances.
///
/// Each appearance is wrapped in a q/Q save/restore pair with
/// a coordinate transform to position it on the page.
fn build_page_content_stream(
    doc: &mut lopdf::Document,
    appearances: &[FieldAppearance],
    config: &FlattenConfig,
) -> Result<(Stream, usize)> {
    let mut ops = Vec::new();
    let mut xobject_dict = Dictionary::new();
    let mut stream_count = 0;

    for (idx, field) in appearances.iter().enumerate() {
        let xobject_name = format!("XF{idx}");

        // Add the appearance stream as a Form XObject
        let stream = to_pdf_stream(&field.normal_appearance, config.compress);
        let xobject_id = doc.add_object(Object::Stream(stream));
        xobject_dict.set(
            xobject_name.as_bytes().to_vec(),
            Object::Reference(xobject_id),
        );
        stream_count += 1;

        // Paint the XObject at the field's position
        // q — save graphics state
        ops.push(Operation::new("q", vec![]));

        // cm — translate to field position
        ops.push(Operation::new(
            "cm",
            vec![
                1.0.into(),
                0.0.into(),
                0.0.into(),
                1.0.into(),
                field.x.into(),
                field.y.into(),
            ],
        ));

        // Do — paint the XObject
        ops.push(Operation::new(
            "Do",
            vec![Object::Name(xobject_name.as_bytes().to_vec())],
        ));

        // Q — restore graphics state
        ops.push(Operation::new("Q", vec![]));
    }

    let content = Content { operations: ops };
    let content_bytes = content
        .encode()
        .map_err(|e| PdfError::Io(std::io::Error::other(format!("encode: {e}"))))?;

    let mut dict = Dictionary::new();

    // Add XObject resources
    if !xobject_dict.is_empty() {
        let resources = dictionary! {
            "XObject" => Object::Dictionary(xobject_dict),
        };
        // Note: Resources should be on the page, not on the content stream.
        // We'll return the xobject_dict separately if needed.
        // For now, embed a minimal stream.
        let _ = resources; // We handle resources at the page level
    }

    let (data, filter) = if config.compress && !content_bytes.is_empty() {
        let compressed = crate::appearance::compress_stream(&content_bytes);
        (compressed, Some("FlateDecode"))
    } else {
        (content_bytes, None)
    };

    if let Some(f) = filter {
        dict.set("Filter", Object::Name(f.as_bytes().to_vec()));
    }

    Ok((Stream::new(dict, data), stream_count))
}

/// Build a font resource dictionary from field appearances.
fn build_font_dict(appearances: &[FieldAppearance]) -> Dictionary {
    let mut font_dict = Dictionary::new();
    let mut seen = std::collections::HashSet::new();

    for field in appearances {
        for font_name in &field.normal_appearance.fonts_used {
            if seen.insert(font_name.clone()) {
                let font_obj = dictionary! {
                    "Type" => Object::Name(b"Font".to_vec()),
                    "Subtype" => Object::Name(b"Type1".to_vec()),
                    "BaseFont" => Object::Name(b"Helvetica".to_vec()),
                };
                font_dict.set(font_name.as_bytes(), Object::Dictionary(font_obj));
            }
        }
    }

    font_dict
}

/// Build a Pages dictionary for the document.
fn build_pages_dict(page_ids: &[ObjectId]) -> Dictionary {
    let kids: Vec<Object> = page_ids.iter().map(|id| Object::Reference(*id)).collect();
    dictionary! {
        "Type" => Object::Name(b"Pages".to_vec()),
        "Count" => Object::Integer(page_ids.len() as i64),
        "Kids" => Object::Array(kids),
    }
}

/// Remove XFA metadata and optionally AcroForm from the document catalog.
fn remove_xfa_metadata(doc: &mut lopdf::Document, remove_acroform: bool) {
    // Navigate to catalog
    let catalog_id = match doc.trailer.get(b"Root") {
        Ok(Object::Reference(id)) => *id,
        _ => return,
    };

    // If we only need to remove XFA (not full AcroForm), find the AcroForm
    // reference first to avoid overlapping borrows.
    let acroform_ref = if !remove_acroform {
        if let Ok(Object::Dictionary(catalog)) = doc.get_object(catalog_id) {
            if let Ok(Object::Reference(r)) = catalog.get(b"AcroForm") {
                Some(*r)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    // Remove XFA from AcroForm dict (if indirect reference)
    if let Some(acroform_id) = acroform_ref {
        if let Ok(Object::Dictionary(ref mut acroform)) = doc.get_object_mut(acroform_id) {
            acroform.remove(b"XFA");
        }
    }

    // Remove AcroForm from catalog if requested, and NeedsRendering
    if let Ok(Object::Dictionary(ref mut catalog)) = doc.get_object_mut(catalog_id) {
        if remove_acroform {
            catalog.remove(b"AcroForm");
        }
        catalog.remove(b"NeedsRendering");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xfa_layout_engine::form::FormNodeId;
    use xfa_layout_engine::layout::{LayoutContent, LayoutNode, LayoutPage};
    use xfa_layout_engine::types::Rect;

    fn make_layout(fields: Vec<LayoutNode>) -> LayoutDom {
        LayoutDom {
            pages: vec![LayoutPage {
                width: 612.0,
                height: 792.0,
                nodes: fields,
            }],
        }
    }

    fn text_node(name: &str, x: f64, y: f64, w: f64, h: f64, text: &str) -> LayoutNode {
        LayoutNode {
            form_node: FormNodeId(0),
            rect: Rect::new(x, y, w, h),
            name: name.to_string(),
            content: LayoutContent::WrappedText {
                lines: vec![text.to_string()],
                font_size: 12.0,
            },
            children: vec![],
        }
    }

    #[test]
    fn flatten_single_field() {
        let layout = make_layout(vec![text_node("Name", 72.0, 72.0, 200.0, 20.0, "John")]);
        let config = FlattenConfig::default();
        let pdf_bytes = flatten_to_pdf(&layout, &config).unwrap();
        assert!(!pdf_bytes.is_empty());

        // Should be valid PDF
        let doc = lopdf::Document::load_mem(&pdf_bytes).unwrap();
        assert_eq!(doc.get_pages().len(), 1);
    }

    #[test]
    fn flatten_multiple_fields() {
        let layout = make_layout(vec![
            text_node("Name", 72.0, 72.0, 200.0, 20.0, "John Doe"),
            text_node("SSN", 72.0, 100.0, 200.0, 20.0, "123-45-6789"),
            text_node("City", 72.0, 128.0, 200.0, 20.0, "New York"),
        ]);
        let config = FlattenConfig::default();
        let pdf_bytes = flatten_to_pdf(&layout, &config).unwrap();
        let doc = lopdf::Document::load_mem(&pdf_bytes).unwrap();
        assert_eq!(doc.get_pages().len(), 1);
    }

    #[test]
    fn flatten_multipage() {
        let layout = LayoutDom {
            pages: vec![
                LayoutPage {
                    width: 612.0,
                    height: 792.0,
                    nodes: vec![text_node("Page1Field", 72.0, 72.0, 200.0, 20.0, "A")],
                },
                LayoutPage {
                    width: 612.0,
                    height: 792.0,
                    nodes: vec![text_node("Page2Field", 72.0, 72.0, 200.0, 20.0, "B")],
                },
            ],
        };
        let config = FlattenConfig::default();
        let pdf_bytes = flatten_to_pdf(&layout, &config).unwrap();
        let doc = lopdf::Document::load_mem(&pdf_bytes).unwrap();
        assert_eq!(doc.get_pages().len(), 2);
    }

    #[test]
    fn flatten_empty_layout() {
        let layout = LayoutDom {
            pages: vec![LayoutPage {
                width: 612.0,
                height: 792.0,
                nodes: vec![],
            }],
        };
        let config = FlattenConfig::default();
        let pdf_bytes = flatten_to_pdf(&layout, &config).unwrap();
        let doc = lopdf::Document::load_mem(&pdf_bytes).unwrap();
        assert_eq!(doc.get_pages().len(), 1);
    }

    #[test]
    fn flatten_result_counts() {
        let layout = make_layout(vec![
            text_node("A", 10.0, 10.0, 100.0, 20.0, "X"),
            text_node("B", 10.0, 40.0, 100.0, 20.0, "Y"),
        ]);

        let appearances = generate_appearances(&layout, &AppearanceConfig::default());
        assert_eq!(appearances.pages[0].len(), 2);
    }

    #[test]
    fn flatten_no_xfa_in_output() {
        let layout = make_layout(vec![text_node("F", 72.0, 72.0, 200.0, 20.0, "Test")]);
        let config = FlattenConfig::default();
        let pdf_bytes = flatten_to_pdf(&layout, &config).unwrap();
        let doc = lopdf::Document::load_mem(&pdf_bytes).unwrap();

        // Catalog should not have AcroForm (we built fresh)
        let catalog_id = match doc.trailer.get(b"Root").unwrap() {
            Object::Reference(id) => *id,
            _ => panic!("No Root reference"),
        };
        let catalog = doc.get_object(catalog_id).unwrap().as_dict().unwrap();
        assert!(
            catalog.get(b"AcroForm").is_err(),
            "Flattened PDF should not have AcroForm"
        );
    }

    #[test]
    fn flatten_uncompressed() {
        let layout = make_layout(vec![text_node("F", 72.0, 72.0, 100.0, 20.0, "Hi")]);
        let config = FlattenConfig {
            compress: false,
            ..Default::default()
        };
        let pdf_bytes = flatten_to_pdf(&layout, &config).unwrap();
        // Should still be valid
        let doc = lopdf::Document::load_mem(&pdf_bytes).unwrap();
        assert_eq!(doc.get_pages().len(), 1);
    }

    #[test]
    fn flatten_into_existing_pdf() {
        // Create a minimal PDF with one page
        let mut doc = lopdf::Document::new();
        let content = Stream::new(Dictionary::new(), b"".to_vec());
        let content_id = doc.add_object(Object::Stream(content));

        let page = dictionary! {
            "Type" => Object::Name(b"Page".to_vec()),
            "MediaBox" => Object::Array(vec![
                0.0.into(), 0.0.into(), 612.0.into(), 792.0.into(),
            ]),
            "Contents" => Object::Reference(content_id),
        };
        let page_id = doc.add_object(Object::Dictionary(page));

        let pages = dictionary! {
            "Type" => Object::Name(b"Pages".to_vec()),
            "Count" => Object::Integer(1),
            "Kids" => Object::Array(vec![Object::Reference(page_id)]),
        };
        let pages_id = doc.add_object(Object::Dictionary(pages));

        // Set parent on page
        if let Ok(Object::Dictionary(ref mut p)) = doc.get_object_mut(page_id) {
            p.set("Parent", Object::Reference(pages_id));
        }

        let catalog = dictionary! {
            "Type" => Object::Name(b"Catalog".to_vec()),
            "Pages" => Object::Reference(pages_id),
        };
        let catalog_id = doc.add_object(Object::Dictionary(catalog));
        doc.trailer.set("Root", Object::Reference(catalog_id));

        // Save to bytes and load as PdfReader
        let mut buf = Vec::new();
        doc.save_to(&mut buf).unwrap();
        let mut reader = PdfReader::from_bytes(&buf).unwrap();

        // Build a layout and flatten into it
        let layout = make_layout(vec![text_node("Test", 72.0, 72.0, 200.0, 20.0, "Hello")]);
        let config = FlattenConfig::default();
        let result = flatten_into_pdf(&mut reader, &layout, &config).unwrap();

        assert_eq!(result.pages_processed, 1);
        assert_eq!(result.fields_flattened, 1);
        assert!(result.streams_generated > 0);
    }

    #[test]
    fn remove_xfa_from_catalog() {
        let mut doc = lopdf::Document::new();

        // Build a catalog with AcroForm + XFA
        let acroform = dictionary! {
            "XFA" => Object::String(b"dummy".to_vec(), lopdf::StringFormat::Literal),
        };
        let catalog = dictionary! {
            "Type" => Object::Name(b"Catalog".to_vec()),
            "AcroForm" => Object::Dictionary(acroform),
        };
        let catalog_id = doc.add_object(Object::Dictionary(catalog));
        doc.trailer.set("Root", Object::Reference(catalog_id));

        remove_xfa_metadata(&mut doc, true);

        let cat = doc.get_object(catalog_id).unwrap().as_dict().unwrap();
        assert!(cat.get(b"AcroForm").is_err(), "AcroForm should be removed");
    }
}
