//! End-to-end processing pipeline — PDF to JSON and pixel output (pure Rust).
//!
//! Connects the full chain: PDF → XFA extraction → template parse →
//! data merge → scripting → JSON / layout → rendering.

use crate::data_merge;
use crate::dataset_sync;
use crate::error::{PdfError, Result};
use crate::native_renderer::{render_layout, RenderConfig};
use crate::pdf_reader::PdfReader;
use crate::template_parser;
use crate::xfa_extract::XfaPackets;
use image::DynamicImage;
use std::path::Path;
use xfa_json::types::FormData;
use xfa_layout_engine::form::{FormNodeId, FormNodeType, FormTree};
use xfa_layout_engine::layout::{LayoutDom, LayoutEngine};
use xfa_layout_engine::scripting;

/// Render a pre-built `LayoutDom` to page images.
///
/// This is the primary entry point when the form tree is already constructed
/// and laid out. Returns one image per page.
pub fn render_layout_dom(layout: &LayoutDom, config: &RenderConfig) -> Vec<DynamicImage> {
    render_layout(layout, config)
}

/// Render a `FormTree` to page images.
///
/// Runs scripting (calculate/validate), then performs layout and rendering.
pub fn render_form_tree(
    form: &mut FormTree,
    root: FormNodeId,
    config: &RenderConfig,
) -> Result<Vec<DynamicImage>> {
    // Run calculate scripts to populate computed field values.
    scripting::run_calculations(form)
        .map_err(|e| PdfError::RenderError(format!("scripting: {e}")))?;

    // Layout the form tree into pages.
    let engine = LayoutEngine::new(form);
    let layout = engine
        .layout(root)
        .map_err(|e| PdfError::RenderError(format!("layout: {e}")))?;

    // Render layout to images.
    Ok(render_layout(&layout, config))
}

/// Extract XFA packets from a PDF file.
pub fn extract_xfa_from_file(path: &Path) -> Result<XfaPackets> {
    let reader = PdfReader::from_file(path)?;
    reader.extract_xfa()
}

/// Extract XFA packets from PDF bytes.
pub fn extract_xfa_from_bytes(bytes: &[u8]) -> Result<XfaPackets> {
    let reader = PdfReader::from_bytes(bytes)?;
    reader.extract_xfa()
}

/// Extract XFA from PDF bytes and convert to JSON.
///
/// Full pipeline: PDF → XFA extraction → template parse → data merge →
/// FormCalc scripting → JSON output with typed field values and metadata.
///
/// Returns a JSON object with:
/// - `fields`: field values keyed by SOM-style dotted path
/// - `schema`: field metadata (type, required, scripts, etc.)
pub fn pdf_to_json(bytes: &[u8]) -> Result<serde_json::Value> {
    // Step 1: Extract XFA packets from PDF
    let packets = extract_xfa_from_bytes(bytes)?;

    let template_xml = packets.template().ok_or_else(|| {
        PdfError::XfaPacketNotFound("no template packet — this PDF may not contain XFA".to_string())
    })?;

    // Step 2: Parse template XML into FormTree
    let (mut tree, root) = template_parser::parse_template(template_xml)?;

    // Step 3: Merge datasets (if present) into FormTree
    if let Some(datasets_xml) = packets.datasets() {
        data_merge::merge_data(&mut tree, root, datasets_xml)?;
    }

    // Step 4: Run FormCalc calculate scripts
    let _ = scripting::run_calculations(&mut tree)
        .map_err(|e| PdfError::RenderError(format!("scripting: {e}")))?;

    // Step 5: Export to JSON
    let data = xfa_json::form_tree_to_json(&tree, root);
    let schema = xfa_json::export_schema(&tree, root);

    let result = serde_json::json!({
        "fields": serde_json::to_value(&data.fields).unwrap_or_default(),
        "schema": serde_json::to_value(&schema.fields).unwrap_or_default(),
    });

    Ok(result)
}

/// Extract XFA from a PDF file and convert to JSON.
///
/// File-based convenience wrapper around [`pdf_to_json`].
pub fn pdf_file_to_json(path: &Path) -> Result<serde_json::Value> {
    let bytes = std::fs::read(path)?;
    pdf_to_json(&bytes)
}

/// Merge JSON data into an XFA PDF template, returning a new PDF.
///
/// Full pipeline: PDF template → XFA extraction → template parse →
/// data merge → validation → JSON import → scripting → PDF output.
///
/// The `data` JSON must have a `fields` key with SOM-style dotted paths
/// (e.g., `"form1.Name"`) mapped to values. Unknown field names produce
/// a validation error.
pub fn json_to_pdf(template: &[u8], data: &serde_json::Value) -> Result<Vec<u8>> {
    // Step 1: Extract XFA packets from the template PDF
    let packets = extract_xfa_from_bytes(template)?;

    let template_xml = packets.template().ok_or_else(|| {
        PdfError::XfaPacketNotFound("no template packet — this PDF may not contain XFA".to_string())
    })?;

    // Step 2: Parse template XML into FormTree
    let (mut tree, root) = template_parser::parse_template(template_xml)?;

    // Step 3: Merge existing datasets (if present) as defaults
    if let Some(datasets_xml) = packets.datasets() {
        data_merge::merge_data(&mut tree, root, datasets_xml)?;
    }

    // Step 4: Parse input JSON into FormData
    let form_data = parse_json_input(data)?;

    // Step 5: Validate input fields against the form schema
    let schema = xfa_json::export_schema(&tree, root);
    validate_fields(&form_data, &schema)?;

    // Step 6: Import JSON data into FormTree
    xfa_json::json_to_form_tree(&form_data, &mut tree, root);

    // Step 7: Run FormCalc calculate scripts
    let _ = scripting::run_calculations(&mut tree)
        .map_err(|e| PdfError::RenderError(format!("scripting: {e}")))?;

    // Step 8: Convert FormTree back to datasets XML
    let datasets_xml = form_tree_to_datasets_xml(&tree, root);

    // Step 9: Write updated datasets into the PDF
    let mut reader = PdfReader::from_bytes(template)?;
    dataset_sync::sync_datasets_xml(&mut reader, &datasets_xml)?;

    // Step 10: Save and return the modified PDF
    reader.save_to_bytes()
}

/// File-based convenience wrapper around [`json_to_pdf`].
pub fn json_file_to_pdf(template_path: &Path, data: &serde_json::Value) -> Result<Vec<u8>> {
    let bytes = std::fs::read(template_path)?;
    json_to_pdf(&bytes, data)
}

/// Parse a `serde_json::Value` into a `FormData` structure.
///
/// Accepts either `{"fields": {...}}` or a flat `{...}` object.
fn parse_json_input(data: &serde_json::Value) -> Result<FormData> {
    // If data has a "fields" key, use that; otherwise treat the whole object as fields
    let fields_value = data.get("fields").unwrap_or(data);

    let form_data: FormData = serde_json::from_value(serde_json::json!({
        "fields": fields_value,
    }))
    .map_err(|e| PdfError::Validation(format!("invalid JSON input: {e}")))?;

    Ok(form_data)
}

/// Validate that all field paths in the input exist in the form schema.
fn validate_fields(data: &FormData, schema: &xfa_json::FormSchema) -> Result<()> {
    let mut unknown = Vec::new();

    for key in data.fields.keys() {
        if !schema.fields.contains_key(key) {
            // For array fields, check if the base path (without array index) exists
            // Array data is stored at the parent subform path, not as individual fields
            if !is_repeating_path(key, schema) {
                unknown.push(key.clone());
            }
        }
    }

    if !unknown.is_empty() {
        return Err(PdfError::Validation(format!(
            "unknown field(s): {}",
            unknown.join(", ")
        )));
    }

    Ok(())
}

/// Check if a field path corresponds to a repeating section in the schema.
///
/// Repeating sections appear as array values in FormData at the subform path,
/// but individual fields inside them are in the schema with the subform prefix.
fn is_repeating_path(path: &str, schema: &xfa_json::FormSchema) -> bool {
    // Check if any schema field starts with this path followed by a dot
    // (meaning this path is a repeating subform containing known fields)
    let prefix = format!("{path}.");
    schema.fields.keys().any(|k| k.starts_with(&prefix))
}

/// Convert a FormTree's field values to XFA datasets XML.
///
/// Produces a complete `<xfa:datasets>` element suitable for writing
/// back into a PDF.
fn form_tree_to_datasets_xml(tree: &FormTree, root: FormNodeId) -> String {
    let mut data_xml = String::new();
    let node = tree.get(root);

    match &node.node_type {
        FormNodeType::Root => {
            // Root node: emit children (the top-level subform)
            for &child_id in &node.children {
                write_data_node(tree, child_id, &mut data_xml, 2);
            }
        }
        _ => {
            write_data_node(tree, root, &mut data_xml, 2);
        }
    }

    format!(
        "<xfa:datasets xmlns:xfa=\"http://www.xfa.org/schema/xfa-data/1.0/\">\n\
         <xfa:data>\n\
         {data_xml}\
         </xfa:data>\n\
         </xfa:datasets>"
    )
}

/// Recursively write a FormTree node as XML data elements.
fn write_data_node(tree: &FormTree, node_id: FormNodeId, out: &mut String, depth: usize) {
    let node = tree.get(node_id);
    let indent = "  ".repeat(depth);
    let name = &node.name;

    // Skip nodes with empty names to avoid emitting invalid XML like <>...</>
    if name.is_empty() {
        for &child_id in &node.children {
            write_data_node(tree, child_id, out, depth);
        }
        return;
    }

    match &node.node_type {
        FormNodeType::Subform => {
            if node.children.is_empty() {
                out.push_str(&format!("{indent}<{name}/>\n"));
            } else {
                out.push_str(&format!("{indent}<{name}>\n"));
                for &child_id in &node.children {
                    write_data_node(tree, child_id, out, depth + 1);
                }
                out.push_str(&format!("{indent}</{name}>\n"));
            }
        }
        FormNodeType::Field { value } => {
            if value.is_empty() {
                out.push_str(&format!("{indent}<{name}/>\n"));
            } else {
                let escaped = xml_escape(value);
                out.push_str(&format!("{indent}<{name}>{escaped}</{name}>\n"));
            }
        }
        FormNodeType::Draw { .. } => {
            // Draw nodes are presentation-only; not written to datasets
        }
        FormNodeType::Root => {
            for &child_id in &node.children {
                write_data_node(tree, child_id, out, depth);
            }
        }
        FormNodeType::PageSet | FormNodeType::PageArea { .. } => {
            // Structural nodes: skip
        }
    }
}

/// Escape special XML characters.
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Save rendered page images to PNG files.
///
/// Files are written to `{dir}/{prefix}_page_{n}.png`.
/// Returns the list of written file paths.
pub fn save_pages_as_png(
    images: &[DynamicImage],
    dir: &Path,
    prefix: &str,
) -> Result<Vec<std::path::PathBuf>> {
    std::fs::create_dir_all(dir)?;
    let mut paths = Vec::with_capacity(images.len());
    for (i, img) in images.iter().enumerate() {
        let path = dir.join(format!("{prefix}_page_{i}.png"));
        img.save(&path)
            .map_err(|e| PdfError::RenderError(format!("save PNG: {e}")))?;
        paths.push(path);
    }
    Ok(paths)
}

#[cfg(test)]
mod tests {
    use super::*;
    use xfa_layout_engine::form::*;
    use xfa_layout_engine::text::FontMetrics;
    use xfa_layout_engine::types::*;

    fn simple_form() -> (FormTree, FormNodeId) {
        let mut tree = FormTree::new();
        let field = tree.add_node(FormNode {
            name: "Name".to_string(),
            node_type: FormNodeType::Field {
                value: "John".to_string(),
            },
            box_model: BoxModel {
                width: Some(200.0),
                height: Some(25.0),
                x: 20.0,
                y: 20.0,
                ..Default::default()
            },
            layout: LayoutStrategy::Positioned,
            children: vec![],
            occur: Occur::once(),
            font: FontMetrics::default(),
            calculate: None,
            validate: None,
            column_widths: vec![],
            col_span: 1,
        });
        let root = tree.add_node(FormNode {
            name: "form1".to_string(),
            node_type: FormNodeType::Subform,
            box_model: BoxModel {
                width: Some(612.0),
                height: Some(792.0),
                ..Default::default()
            },
            layout: LayoutStrategy::Positioned,
            children: vec![field],
            occur: Occur::once(),
            font: FontMetrics::default(),
            calculate: None,
            validate: None,
            column_widths: vec![],
            col_span: 1,
        });
        (tree, root)
    }

    #[test]
    fn render_form_tree_produces_images() {
        let (mut tree, root) = simple_form();
        let config = RenderConfig::default();
        let images = render_form_tree(&mut tree, root, &config).unwrap();
        assert_eq!(images.len(), 1);
        assert_eq!(images[0].width(), 612);
        assert_eq!(images[0].height(), 792);
    }

    #[test]
    fn render_form_tree_with_dpi_scaling() {
        let (mut tree, root) = simple_form();
        let config = RenderConfig::with_dpi(144.0);
        let images = render_form_tree(&mut tree, root, &config).unwrap();
        assert_eq!(images[0].width(), 1224); // 612 * 2
        assert_eq!(images[0].height(), 1584); // 792 * 2
    }

    #[test]
    fn render_form_tree_with_calculate_script() {
        let mut tree = FormTree::new();
        let field = tree.add_node(FormNode {
            name: "Total".to_string(),
            node_type: FormNodeType::Field {
                value: String::new(),
            },
            box_model: BoxModel {
                width: Some(100.0),
                height: Some(25.0),
                x: 10.0,
                y: 10.0,
                ..Default::default()
            },
            layout: LayoutStrategy::Positioned,
            children: vec![],
            occur: Occur::once(),
            font: FontMetrics::default(),
            calculate: Some("10 + 20".to_string()),
            validate: None,
            column_widths: vec![],
            col_span: 1,
        });
        let root = tree.add_node(FormNode {
            name: "form1".to_string(),
            node_type: FormNodeType::Subform,
            box_model: BoxModel {
                width: Some(200.0),
                height: Some(100.0),
                ..Default::default()
            },
            layout: LayoutStrategy::Positioned,
            children: vec![field],
            occur: Occur::once(),
            font: FontMetrics::default(),
            calculate: None,
            validate: None,
            column_widths: vec![],
            col_span: 1,
        });
        let config = RenderConfig::default();
        let images = render_form_tree(&mut tree, root, &config).unwrap();
        assert_eq!(images.len(), 1);
        // The field value should have been computed by the calculate script
        if let FormNodeType::Field { value } = &tree.get(field).node_type {
            assert_eq!(value, "30");
        } else {
            panic!("expected Field node");
        }
    }

    #[test]
    fn save_pages_creates_files() {
        let (mut tree, root) = simple_form();
        let config = RenderConfig::default();
        let images = render_form_tree(&mut tree, root, &config).unwrap();

        let dir = std::env::temp_dir().join("xfa_pipeline_test");
        let paths = save_pages_as_png(&images, &dir, "test").unwrap();
        assert_eq!(paths.len(), 1);
        assert!(paths[0].exists());
        // Cleanup
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn extract_xfa_from_invalid_bytes_fails() {
        let result = extract_xfa_from_bytes(b"not a pdf");
        assert!(result.is_err());
    }

    #[test]
    fn form_tree_to_datasets_xml_produces_valid_xml() {
        let (tree, root) = simple_form();
        let xml = form_tree_to_datasets_xml(&tree, root);

        assert!(xml.contains("<xfa:datasets"));
        assert!(xml.contains("<xfa:data>"));
        assert!(xml.contains("<form1>"));
        assert!(xml.contains("<Name>John</Name>"));
        assert!(xml.contains("</form1>"));
        assert!(xml.contains("</xfa:data>"));
        assert!(xml.contains("</xfa:datasets>"));
    }

    #[test]
    fn form_tree_to_datasets_xml_with_root_node() {
        let mut tree = FormTree::new();
        let name = tree.add_node(FormNode {
            name: "Name".to_string(),
            node_type: FormNodeType::Field {
                value: "Alice".to_string(),
            },
            box_model: BoxModel::default(),
            layout: LayoutStrategy::Positioned,
            children: vec![],
            occur: Occur::once(),
            font: FontMetrics::default(),
            calculate: None,
            validate: None,
            column_widths: vec![],
            col_span: 1,
        });
        let form = tree.add_node(FormNode {
            name: "form1".to_string(),
            node_type: FormNodeType::Subform,
            box_model: BoxModel::default(),
            layout: LayoutStrategy::TopToBottom,
            children: vec![name],
            occur: Occur::once(),
            font: FontMetrics::default(),
            calculate: None,
            validate: None,
            column_widths: vec![],
            col_span: 1,
        });
        let root = tree.add_node(FormNode {
            name: "Root".to_string(),
            node_type: FormNodeType::Root,
            box_model: BoxModel::default(),
            layout: LayoutStrategy::TopToBottom,
            children: vec![form],
            occur: Occur::once(),
            font: FontMetrics::default(),
            calculate: None,
            validate: None,
            column_widths: vec![],
            col_span: 1,
        });

        let xml = form_tree_to_datasets_xml(&tree, root);
        assert!(xml.contains("<form1>"));
        assert!(xml.contains("<Name>Alice</Name>"));
    }

    #[test]
    fn form_tree_to_datasets_xml_escapes_special_chars() {
        let mut tree = FormTree::new();
        let field = tree.add_node(FormNode {
            name: "Note".to_string(),
            node_type: FormNodeType::Field {
                value: "A & B < C".to_string(),
            },
            box_model: BoxModel::default(),
            layout: LayoutStrategy::Positioned,
            children: vec![],
            occur: Occur::once(),
            font: FontMetrics::default(),
            calculate: None,
            validate: None,
            column_widths: vec![],
            col_span: 1,
        });
        let form = tree.add_node(FormNode {
            name: "form1".to_string(),
            node_type: FormNodeType::Subform,
            box_model: BoxModel::default(),
            layout: LayoutStrategy::TopToBottom,
            children: vec![field],
            occur: Occur::once(),
            font: FontMetrics::default(),
            calculate: None,
            validate: None,
            column_widths: vec![],
            col_span: 1,
        });
        let xml = form_tree_to_datasets_xml(&tree, form);
        assert!(xml.contains("A &amp; B &lt; C"));
    }

    #[test]
    fn validate_fields_rejects_unknown() {
        let mut tree = FormTree::new();
        let name = tree.add_node(FormNode {
            name: "Name".to_string(),
            node_type: FormNodeType::Field {
                value: "x".to_string(),
            },
            box_model: BoxModel::default(),
            layout: LayoutStrategy::Positioned,
            children: vec![],
            occur: Occur::once(),
            font: FontMetrics::default(),
            calculate: None,
            validate: None,
            column_widths: vec![],
            col_span: 1,
        });
        let form = tree.add_node(FormNode {
            name: "form1".to_string(),
            node_type: FormNodeType::Subform,
            box_model: BoxModel::default(),
            layout: LayoutStrategy::TopToBottom,
            children: vec![name],
            occur: Occur::once(),
            font: FontMetrics::default(),
            calculate: None,
            validate: None,
            column_widths: vec![],
            col_span: 1,
        });
        let root = tree.add_node(FormNode {
            name: "Root".to_string(),
            node_type: FormNodeType::Root,
            box_model: BoxModel::default(),
            layout: LayoutStrategy::TopToBottom,
            children: vec![form],
            occur: Occur::once(),
            font: FontMetrics::default(),
            calculate: None,
            validate: None,
            column_widths: vec![],
            col_span: 1,
        });

        let schema = xfa_json::export_schema(&tree, root);

        // Valid field
        let mut valid_fields = indexmap::IndexMap::new();
        valid_fields.insert(
            "form1.Name".to_string(),
            xfa_json::FieldValue::Text("ok".to_string()),
        );
        let valid = FormData {
            fields: valid_fields,
        };
        assert!(validate_fields(&valid, &schema).is_ok());

        // Unknown field
        let mut bad_fields = indexmap::IndexMap::new();
        bad_fields.insert(
            "form1.DoesNotExist".to_string(),
            xfa_json::FieldValue::Text("x".to_string()),
        );
        let bad = FormData { fields: bad_fields };
        let err = validate_fields(&bad, &schema);
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("DoesNotExist"));
    }

    #[test]
    fn parse_json_input_with_fields_key() {
        let json = serde_json::json!({
            "fields": {
                "form1.Name": "Alice"
            }
        });
        let data = parse_json_input(&json).unwrap();
        assert!(data.fields.contains_key("form1.Name"));
    }

    #[test]
    fn parse_json_input_flat() {
        let json = serde_json::json!({
            "form1.Name": "Bob"
        });
        let data = parse_json_input(&json).unwrap();
        assert!(data.fields.contains_key("form1.Name"));
    }
}
