//! End-to-end processing pipeline — PDF to JSON and pixel output (pure Rust).
//!
//! Connects the full chain: PDF → XFA extraction → template parse →
//! data merge → scripting → JSON / layout → rendering.

use crate::data_merge;
use crate::error::{PdfError, Result};
use crate::native_renderer::{render_layout, RenderConfig};
use crate::pdf_reader::PdfReader;
use crate::template_parser;
use crate::xfa_extract::XfaPackets;
use image::DynamicImage;
use std::path::Path;
use xfa_layout_engine::form::{FormNodeId, FormTree};
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
}
