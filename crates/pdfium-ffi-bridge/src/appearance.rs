//! Appearance stream generation — LayoutNode to PDF Form XObjects.
//!
//! Converts positioned layout nodes into PDF appearance streams that can
//! be embedded in form field widgets. Each stream contains PDF content
//! operators for rendering text, borders, and background fills.

use lopdf::content::{Content, Operation};
use lopdf::{dictionary, Dictionary, Object, ObjectId, Stream};
use xfa_layout_engine::layout::{LayoutContent, LayoutDom, LayoutNode, LayoutPage};

/// Configuration for appearance stream generation.
#[derive(Debug, Clone)]
pub struct AppearanceConfig {
    /// Default font name used in PDF resources (e.g., "Helv").
    pub default_font_name: String,
    /// Whether to compress streams with FlateDecode.
    pub compress: bool,
    /// Border width in points.
    pub border_width: f64,
    /// Border color (RGB, each 0.0-1.0).
    pub border_color: [f64; 3],
    /// Field background color (RGB, each 0.0-1.0). None = transparent.
    pub field_bg_color: Option<[f64; 3]>,
    /// Text color (RGB, each 0.0-1.0).
    pub text_color: [f64; 3],
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            default_font_name: "Helv".to_string(),
            compress: true,
            border_width: 0.5,
            border_color: [0.0, 0.0, 0.0],
            field_bg_color: Some([1.0, 1.0, 1.0]),
            text_color: [0.0, 0.0, 0.0],
        }
    }
}

/// A generated appearance stream with its resource requirements.
#[derive(Debug)]
pub struct AppearanceStream {
    /// The raw PDF content stream bytes.
    pub stream_data: Vec<u8>,
    /// Bounding box [x, y, width, height] in points.
    pub bbox: [f64; 4],
    /// Font names referenced in this stream.
    pub fonts_used: Vec<String>,
    /// Whether the stream is compressed.
    pub compressed: bool,
}

/// Result of generating appearances for an entire layout.
#[derive(Debug)]
pub struct PageAppearances {
    /// One entry per page, each containing field appearances.
    pub pages: Vec<Vec<FieldAppearance>>,
}

/// A single field's appearance data.
#[derive(Debug)]
pub struct FieldAppearance {
    /// Name of the source form node.
    pub field_name: String,
    /// Position on page (absolute coordinates in points).
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    /// The appearance stream for the normal state.
    pub normal_appearance: AppearanceStream,
}

/// Generate appearance streams for all fields in a layout.
pub fn generate_appearances(layout: &LayoutDom, config: &AppearanceConfig) -> PageAppearances {
    let pages = layout
        .pages
        .iter()
        .map(|page| generate_page_appearances(page, config))
        .collect();
    PageAppearances { pages }
}

/// Generate appearance streams for all fields on a single page.
fn generate_page_appearances(page: &LayoutPage, config: &AppearanceConfig) -> Vec<FieldAppearance> {
    let mut appearances = Vec::new();
    for node in &page.nodes {
        collect_field_appearances(node, config, 0.0, 0.0, page.height, &mut appearances);
    }
    appearances
}

/// Recursively collect field appearances from a layout tree.
///
/// PDF coordinate system has origin at bottom-left, while layout uses top-left.
/// We convert Y coordinates: pdf_y = page_height - layout_y - node_height.
fn collect_field_appearances(
    node: &LayoutNode,
    config: &AppearanceConfig,
    parent_x: f64,
    parent_y: f64,
    page_height: f64,
    appearances: &mut Vec<FieldAppearance>,
) {
    let abs_x = node.rect.x + parent_x;
    let abs_y = node.rect.y + parent_y;
    let w = node.rect.width;
    let h = node.rect.height;

    match &node.content {
        LayoutContent::WrappedText { lines, font_size } => {
            let stream = build_text_appearance(w, h, lines, *font_size, config);
            // Convert to PDF coordinates (bottom-left origin)
            let pdf_y = page_height - abs_y - h;
            appearances.push(FieldAppearance {
                field_name: node.name.clone(),
                x: abs_x,
                y: pdf_y,
                width: w,
                height: h,
                normal_appearance: stream,
            });
        }
        LayoutContent::Field { value } => {
            let lines = if value.is_empty() {
                vec![]
            } else {
                vec![value.clone()]
            };
            let stream = build_text_appearance(w, h, &lines, 10.0, config);
            let pdf_y = page_height - abs_y - h;
            appearances.push(FieldAppearance {
                field_name: node.name.clone(),
                x: abs_x,
                y: pdf_y,
                width: w,
                height: h,
                normal_appearance: stream,
            });
        }
        LayoutContent::Text(content) => {
            let lines = if content.is_empty() {
                vec![]
            } else {
                vec![content.clone()]
            };
            let stream = build_text_appearance(w, h, &lines, 10.0, config);
            let pdf_y = page_height - abs_y - h;
            appearances.push(FieldAppearance {
                field_name: node.name.clone(),
                x: abs_x,
                y: pdf_y,
                width: w,
                height: h,
                normal_appearance: stream,
            });
        }
        LayoutContent::None => {
            // Container — no appearance stream needed
        }
    }

    for child in &node.children {
        collect_field_appearances(child, config, abs_x, abs_y, page_height, appearances);
    }
}

/// Build a text field appearance stream.
///
/// Generates PDF content operators for:
/// 1. Background fill (optional)
/// 2. Border stroke
/// 3. Text lines with proper positioning
fn build_text_appearance(
    width: f64,
    height: f64,
    lines: &[String],
    font_size: f64,
    config: &AppearanceConfig,
) -> AppearanceStream {
    let mut ops = Vec::new();

    // Save graphics state
    ops.push(Operation::new("q", vec![]));

    // Background fill
    if let Some(bg) = config.field_bg_color {
        ops.push(Operation::new(
            "rg",
            vec![bg[0].into(), bg[1].into(), bg[2].into()],
        ));
        ops.push(Operation::new(
            "re",
            vec![0.0.into(), 0.0.into(), width.into(), height.into()],
        ));
        ops.push(Operation::new("f", vec![]));
    }

    // Border
    if config.border_width > 0.0 {
        ops.push(Operation::new("w", vec![config.border_width.into()]));
        ops.push(Operation::new(
            "RG",
            vec![
                config.border_color[0].into(),
                config.border_color[1].into(),
                config.border_color[2].into(),
            ],
        ));
        let inset = config.border_width / 2.0;
        ops.push(Operation::new(
            "re",
            vec![
                inset.into(),
                inset.into(),
                (width - config.border_width).into(),
                (height - config.border_width).into(),
            ],
        ));
        ops.push(Operation::new("S", vec![]));
    }

    // Text content
    if !lines.is_empty() {
        ops.push(Operation::new("BT", vec![]));

        // Set font
        ops.push(Operation::new(
            "Tf",
            vec![
                Object::Name(config.default_font_name.as_bytes().to_vec()),
                font_size.into(),
            ],
        ));

        // Set text color
        ops.push(Operation::new(
            "rg",
            vec![
                config.text_color[0].into(),
                config.text_color[1].into(),
                config.text_color[2].into(),
            ],
        ));

        // Position first line: left margin, top of field minus ascent
        let line_height = font_size * 1.2;
        let text_x = config.border_width + 2.0;
        let text_y = height - config.border_width - font_size;

        ops.push(Operation::new("Td", vec![text_x.into(), text_y.into()]));

        // First line
        if let Some(first) = lines.first() {
            ops.push(Operation::new(
                "Tj",
                vec![Object::String(
                    first.as_bytes().to_vec(),
                    lopdf::StringFormat::Literal,
                )],
            ));
        }

        // Subsequent lines: move down by line height
        for line in lines.iter().skip(1) {
            ops.push(Operation::new(
                "Td",
                vec![0.0.into(), (-line_height).into()],
            ));
            ops.push(Operation::new(
                "Tj",
                vec![Object::String(
                    line.as_bytes().to_vec(),
                    lopdf::StringFormat::Literal,
                )],
            ));
        }

        ops.push(Operation::new("ET", vec![]));
    }

    // Restore graphics state
    ops.push(Operation::new("Q", vec![]));

    let content = Content { operations: ops };
    let stream_data = content.encode().unwrap_or_default();

    AppearanceStream {
        stream_data,
        bbox: [0.0, 0.0, width, height],
        fonts_used: vec![config.default_font_name.clone()],
        compressed: false,
    }
}

/// Build a checkbox appearance stream for the "on" state.
pub fn build_checkbox_on(width: f64, height: f64, config: &AppearanceConfig) -> AppearanceStream {
    let mut ops = Vec::new();

    ops.push(Operation::new("q", vec![]));

    // Background
    if let Some(bg) = config.field_bg_color {
        ops.push(Operation::new(
            "rg",
            vec![bg[0].into(), bg[1].into(), bg[2].into()],
        ));
        ops.push(Operation::new(
            "re",
            vec![0.0.into(), 0.0.into(), width.into(), height.into()],
        ));
        ops.push(Operation::new("f", vec![]));
    }

    // Border
    ops.push(Operation::new("w", vec![config.border_width.into()]));
    ops.push(Operation::new(
        "RG",
        vec![
            config.border_color[0].into(),
            config.border_color[1].into(),
            config.border_color[2].into(),
        ],
    ));
    ops.push(Operation::new(
        "re",
        vec![0.0.into(), 0.0.into(), width.into(), height.into()],
    ));
    ops.push(Operation::new("S", vec![]));

    // Checkmark (diagonal cross)
    let margin = width * 0.15;
    ops.push(Operation::new(
        "w",
        vec![(config.border_width * 2.0).into()],
    ));
    // Line from bottom-left to top-right
    ops.push(Operation::new("m", vec![margin.into(), margin.into()]));
    ops.push(Operation::new(
        "l",
        vec![(width - margin).into(), (height - margin).into()],
    ));
    ops.push(Operation::new("S", vec![]));
    // Line from top-left to bottom-right
    ops.push(Operation::new(
        "m",
        vec![margin.into(), (height - margin).into()],
    ));
    ops.push(Operation::new(
        "l",
        vec![(width - margin).into(), margin.into()],
    ));
    ops.push(Operation::new("S", vec![]));

    ops.push(Operation::new("Q", vec![]));

    let content = Content { operations: ops };
    let stream_data = content.encode().unwrap_or_default();

    AppearanceStream {
        stream_data,
        bbox: [0.0, 0.0, width, height],
        fonts_used: vec![],
        compressed: false,
    }
}

/// Build a checkbox appearance stream for the "off" state (empty box).
pub fn build_checkbox_off(width: f64, height: f64, config: &AppearanceConfig) -> AppearanceStream {
    let mut ops = Vec::new();

    ops.push(Operation::new("q", vec![]));

    // Background
    if let Some(bg) = config.field_bg_color {
        ops.push(Operation::new(
            "rg",
            vec![bg[0].into(), bg[1].into(), bg[2].into()],
        ));
        ops.push(Operation::new(
            "re",
            vec![0.0.into(), 0.0.into(), width.into(), height.into()],
        ));
        ops.push(Operation::new("f", vec![]));
    }

    // Border
    ops.push(Operation::new("w", vec![config.border_width.into()]));
    ops.push(Operation::new(
        "RG",
        vec![
            config.border_color[0].into(),
            config.border_color[1].into(),
            config.border_color[2].into(),
        ],
    ));
    ops.push(Operation::new(
        "re",
        vec![0.0.into(), 0.0.into(), width.into(), height.into()],
    ));
    ops.push(Operation::new("S", vec![]));

    ops.push(Operation::new("Q", vec![]));

    let content = Content { operations: ops };
    let stream_data = content.encode().unwrap_or_default();

    AppearanceStream {
        stream_data,
        bbox: [0.0, 0.0, width, height],
        fonts_used: vec![],
        compressed: false,
    }
}

/// Build a radio button appearance for the "on" state (filled circle).
pub fn build_radio_on(width: f64, height: f64, config: &AppearanceConfig) -> AppearanceStream {
    let mut ops = Vec::new();
    let cx = width / 2.0;
    let cy = height / 2.0;
    let r = (width.min(height) / 2.0) - config.border_width;

    ops.push(Operation::new("q", vec![]));

    // Outer circle (border) — approximated with Bézier curves
    let k = 0.5523; // Bézier approximation constant for circles
    append_circle_ops(&mut ops, cx, cy, r, k);
    ops.push(Operation::new("w", vec![config.border_width.into()]));
    ops.push(Operation::new(
        "RG",
        vec![
            config.border_color[0].into(),
            config.border_color[1].into(),
            config.border_color[2].into(),
        ],
    ));
    ops.push(Operation::new("S", vec![]));

    // Inner filled circle
    let inner_r = r * 0.5;
    append_circle_ops(&mut ops, cx, cy, inner_r, k);
    ops.push(Operation::new(
        "rg",
        vec![
            config.text_color[0].into(),
            config.text_color[1].into(),
            config.text_color[2].into(),
        ],
    ));
    ops.push(Operation::new("f", vec![]));

    ops.push(Operation::new("Q", vec![]));

    let content = Content { operations: ops };
    let stream_data = content.encode().unwrap_or_default();

    AppearanceStream {
        stream_data,
        bbox: [0.0, 0.0, width, height],
        fonts_used: vec![],
        compressed: false,
    }
}

/// Build a radio button appearance for the "off" state (empty circle).
pub fn build_radio_off(width: f64, height: f64, config: &AppearanceConfig) -> AppearanceStream {
    let mut ops = Vec::new();
    let cx = width / 2.0;
    let cy = height / 2.0;
    let r = (width.min(height) / 2.0) - config.border_width;

    ops.push(Operation::new("q", vec![]));

    append_circle_ops(&mut ops, cx, cy, r, 0.5523);
    ops.push(Operation::new("w", vec![config.border_width.into()]));
    ops.push(Operation::new(
        "RG",
        vec![
            config.border_color[0].into(),
            config.border_color[1].into(),
            config.border_color[2].into(),
        ],
    ));
    ops.push(Operation::new("S", vec![]));

    ops.push(Operation::new("Q", vec![]));

    let content = Content { operations: ops };
    let stream_data = content.encode().unwrap_or_default();

    AppearanceStream {
        stream_data,
        bbox: [0.0, 0.0, width, height],
        fonts_used: vec![],
        compressed: false,
    }
}

/// Append circle path operations (Bézier approximation) to the ops list.
fn append_circle_ops(ops: &mut Vec<Operation>, cx: f64, cy: f64, r: f64, k: f64) {
    let kr = k * r;
    // Move to rightmost point
    ops.push(Operation::new("m", vec![(cx + r).into(), cy.into()]));
    // Top-right quadrant
    ops.push(Operation::new(
        "c",
        vec![
            (cx + r).into(),
            (cy + kr).into(),
            (cx + kr).into(),
            (cy + r).into(),
            cx.into(),
            (cy + r).into(),
        ],
    ));
    // Top-left quadrant
    ops.push(Operation::new(
        "c",
        vec![
            (cx - kr).into(),
            (cy + r).into(),
            (cx - r).into(),
            (cy + kr).into(),
            (cx - r).into(),
            cy.into(),
        ],
    ));
    // Bottom-left quadrant
    ops.push(Operation::new(
        "c",
        vec![
            (cx - r).into(),
            (cy - kr).into(),
            (cx - kr).into(),
            (cy - r).into(),
            cx.into(),
            (cy - r).into(),
        ],
    ));
    // Bottom-right quadrant
    ops.push(Operation::new(
        "c",
        vec![
            (cx + kr).into(),
            (cy - r).into(),
            (cx + r).into(),
            (cy - kr).into(),
            (cx + r).into(),
            cy.into(),
        ],
    ));
}

/// Compress a stream with FlateDecode if configured.
pub fn compress_stream(data: &[u8]) -> Vec<u8> {
    use std::io::Write;
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(data).unwrap_or_default();
    encoder.finish().unwrap_or_default()
}

/// Convert an AppearanceStream into a lopdf Stream object.
pub fn to_pdf_stream(appearance: &AppearanceStream, compress: bool) -> Stream {
    let (data, filter) = if compress && !appearance.stream_data.is_empty() {
        let compressed = compress_stream(&appearance.stream_data);
        (compressed, Some("FlateDecode"))
    } else {
        (appearance.stream_data.clone(), None)
    };

    let mut dict = Dictionary::new();
    dict.set("Type", Object::Name(b"XObject".to_vec()));
    dict.set("Subtype", Object::Name(b"Form".to_vec()));
    dict.set(
        "BBox",
        Object::Array(vec![
            appearance.bbox[0].into(),
            appearance.bbox[1].into(),
            appearance.bbox[2].into(),
            appearance.bbox[3].into(),
        ]),
    );

    // Resources: font dictionary
    if !appearance.fonts_used.is_empty() {
        let mut font_dict = Dictionary::new();
        for font_name in &appearance.fonts_used {
            // Reference a standard PDF base font
            let font_obj = dictionary! {
                "Type" => Object::Name(b"Font".to_vec()),
                "Subtype" => Object::Name(b"Type1".to_vec()),
                "BaseFont" => Object::Name(b"Helvetica".to_vec()),
            };
            font_dict.set(font_name.as_bytes(), Object::Dictionary(font_obj));
        }
        let resources = dictionary! {
            "Font" => Object::Dictionary(font_dict),
        };
        dict.set("Resources", Object::Dictionary(resources));
    }

    if let Some(f) = filter {
        dict.set("Filter", Object::Name(f.as_bytes().to_vec()));
    }

    Stream::new(dict, data)
}

/// Create a PDF appearance dictionary entry for a field widget annotation.
///
/// Returns the normal appearance stream as a lopdf Object suitable for
/// embedding in an annotation's /AP dictionary.
pub fn create_appearance_dict(
    doc: &mut lopdf::Document,
    appearance: &AppearanceStream,
    compress: bool,
) -> ObjectId {
    let stream = to_pdf_stream(appearance, compress);
    doc.add_object(Object::Stream(stream))
}

#[cfg(test)]
mod tests {
    use super::*;
    use xfa_layout_engine::form::FormNodeId;
    use xfa_layout_engine::types::Rect;

    fn make_text_node(name: &str, x: f64, y: f64, w: f64, h: f64, text: &str) -> LayoutNode {
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

    fn make_field_node(name: &str, x: f64, y: f64, w: f64, h: f64, value: &str) -> LayoutNode {
        LayoutNode {
            form_node: FormNodeId(0),
            rect: Rect::new(x, y, w, h),
            name: name.to_string(),
            content: LayoutContent::Field {
                value: value.to_string(),
            },
            children: vec![],
        }
    }

    #[test]
    fn text_appearance_generates_content() {
        let config = AppearanceConfig::default();
        let stream = build_text_appearance(100.0, 20.0, &["Hello".to_string()], 12.0, &config);
        assert!(!stream.stream_data.is_empty());
        assert_eq!(stream.bbox, [0.0, 0.0, 100.0, 20.0]);
        assert_eq!(stream.fonts_used, vec!["Helv"]);
    }

    #[test]
    fn text_appearance_contains_operators() {
        let config = AppearanceConfig::default();
        let stream = build_text_appearance(100.0, 20.0, &["Hello".to_string()], 12.0, &config);
        let text = String::from_utf8_lossy(&stream.stream_data);
        // Should contain text operators
        assert!(text.contains("BT"), "Missing BT operator");
        assert!(text.contains("ET"), "Missing ET operator");
        assert!(text.contains("Tf"), "Missing Tf operator");
        assert!(text.contains("Tj"), "Missing Tj operator");
        // Should contain graphics state
        assert!(text.contains("q"), "Missing q operator");
        assert!(text.contains("Q"), "Missing Q operator");
    }

    #[test]
    fn empty_text_no_text_operators() {
        let config = AppearanceConfig::default();
        let stream = build_text_appearance(100.0, 20.0, &[], 12.0, &config);
        let text = String::from_utf8_lossy(&stream.stream_data);
        assert!(!text.contains("BT"), "Should not have BT with no text");
    }

    #[test]
    fn multiline_text_appearance() {
        let config = AppearanceConfig::default();
        let lines = vec![
            "Line 1".to_string(),
            "Line 2".to_string(),
            "Line 3".to_string(),
        ];
        let stream = build_text_appearance(200.0, 60.0, &lines, 10.0, &config);
        let text = String::from_utf8_lossy(&stream.stream_data);
        // Should contain multiple Tj operators
        let tj_count = text.matches("Tj").count();
        assert_eq!(tj_count, 3, "Should have 3 Tj operators for 3 lines");
    }

    #[test]
    fn checkbox_on_appearance() {
        let config = AppearanceConfig::default();
        let stream = build_checkbox_on(12.0, 12.0, &config);
        assert!(!stream.stream_data.is_empty());
        let text = String::from_utf8_lossy(&stream.stream_data);
        // Checkmark uses line operators
        assert!(text.contains("m"), "Missing moveto");
        assert!(text.contains("l"), "Missing lineto");
        assert!(text.contains("S"), "Missing stroke");
    }

    #[test]
    fn checkbox_off_appearance() {
        let config = AppearanceConfig::default();
        let stream = build_checkbox_off(12.0, 12.0, &config);
        let text = String::from_utf8_lossy(&stream.stream_data);
        // Should have border but no checkmark lines
        assert!(text.contains("re"), "Missing rectangle");
        assert!(text.contains("S"), "Missing stroke");
    }

    #[test]
    fn radio_on_appearance() {
        let config = AppearanceConfig::default();
        let stream = build_radio_on(14.0, 14.0, &config);
        let text = String::from_utf8_lossy(&stream.stream_data);
        // Circle uses curve operators
        assert!(text.contains("c"), "Missing curve operator");
        assert!(text.contains("f"), "Missing fill for inner circle");
    }

    #[test]
    fn radio_off_appearance() {
        let config = AppearanceConfig::default();
        let stream = build_radio_off(14.0, 14.0, &config);
        let text = String::from_utf8_lossy(&stream.stream_data);
        assert!(text.contains("c"), "Missing curve operator");
        assert!(text.contains("S"), "Missing stroke");
    }

    #[test]
    fn to_pdf_stream_uncompressed() {
        let config = AppearanceConfig::default();
        let appearance = build_text_appearance(100.0, 20.0, &["Test".to_string()], 12.0, &config);
        let stream = to_pdf_stream(&appearance, false);
        assert_eq!(stream.content, appearance.stream_data);
        let dict = &stream.dict;
        assert_eq!(
            dict.get(b"Type").unwrap(),
            &Object::Name(b"XObject".to_vec())
        );
        assert_eq!(
            dict.get(b"Subtype").unwrap(),
            &Object::Name(b"Form".to_vec())
        );
    }

    #[test]
    fn to_pdf_stream_compressed() {
        let config = AppearanceConfig::default();
        let appearance = build_text_appearance(100.0, 20.0, &["Test".to_string()], 12.0, &config);
        let stream = to_pdf_stream(&appearance, true);
        // Compressed data should be smaller or different
        assert_ne!(stream.content, appearance.stream_data);
        assert_eq!(
            stream.dict.get(b"Filter").unwrap(),
            &Object::Name(b"FlateDecode".to_vec())
        );
    }

    #[test]
    fn generate_appearances_collects_fields() {
        let layout = LayoutDom {
            pages: vec![LayoutPage {
                width: 612.0,
                height: 792.0,
                nodes: vec![
                    make_text_node("Name", 10.0, 10.0, 200.0, 20.0, "John Doe"),
                    make_field_node("SSN", 10.0, 40.0, 200.0, 20.0, "123-45-6789"),
                ],
            }],
        };
        let config = AppearanceConfig::default();
        let result = generate_appearances(&layout, &config);
        assert_eq!(result.pages.len(), 1);
        assert_eq!(result.pages[0].len(), 2);
        assert_eq!(result.pages[0][0].field_name, "Name");
        assert_eq!(result.pages[0][1].field_name, "SSN");
    }

    #[test]
    fn y_coordinate_conversion() {
        let layout = LayoutDom {
            pages: vec![LayoutPage {
                width: 612.0,
                height: 792.0,
                nodes: vec![make_text_node("Top", 10.0, 10.0, 200.0, 20.0, "Hi")],
            }],
        };
        let config = AppearanceConfig::default();
        let result = generate_appearances(&layout, &config);
        let field = &result.pages[0][0];
        // Layout Y=10, height=20, page_height=792
        // PDF Y = 792 - 10 - 20 = 762
        assert_eq!(field.y, 762.0);
    }

    #[test]
    fn nested_nodes_collected() {
        let layout = LayoutDom {
            pages: vec![LayoutPage {
                width: 612.0,
                height: 792.0,
                nodes: vec![LayoutNode {
                    form_node: FormNodeId(0),
                    rect: Rect::new(0.0, 0.0, 612.0, 792.0),
                    name: "Container".to_string(),
                    content: LayoutContent::None,
                    children: vec![
                        make_text_node("Field1", 10.0, 10.0, 100.0, 20.0, "A"),
                        make_text_node("Field2", 10.0, 40.0, 100.0, 20.0, "B"),
                    ],
                }],
            }],
        };
        let config = AppearanceConfig::default();
        let result = generate_appearances(&layout, &config);
        assert_eq!(result.pages[0].len(), 2);
    }

    #[test]
    fn create_appearance_dict_returns_valid_id() {
        let mut doc = lopdf::Document::new();
        let config = AppearanceConfig::default();
        let appearance = build_text_appearance(100.0, 20.0, &["Test".to_string()], 12.0, &config);
        let id = create_appearance_dict(&mut doc, &appearance, false);
        // Should be a valid object ID
        assert!(id.0 > 0);
        // Should be retrievable
        let obj = doc.get_object(id).unwrap();
        assert!(matches!(obj, Object::Stream(_)));
    }

    #[test]
    fn background_fill_can_be_disabled() {
        let config = AppearanceConfig {
            field_bg_color: None,
            ..Default::default()
        };
        let stream = build_text_appearance(100.0, 20.0, &["Test".to_string()], 12.0, &config);
        let text = String::from_utf8_lossy(&stream.stream_data);
        // Should not have fill operator before the text block
        // (only the text color 'rg' inside BT..ET, not a fill rectangle)
        let before_bt = text.split("BT").next().unwrap_or("");
        assert!(
            !before_bt.contains(" f\n"),
            "Should not fill when bg is None"
        );
    }
}
