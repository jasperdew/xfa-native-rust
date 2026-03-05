//! XFA template XML parser — converts template XML into a `FormTree`.
//!
//! Parses the `<template>` packet from an XFA PDF and produces a
//! `FormTree` ready for data merge, scripting, and layout.
//!
//! Supports the core XFA 3.3 §2 template elements:
//! - `<subform>` with layout, dimensions, and occur rules
//! - `<field>` with value, caption, calculate/validate scripts
//! - `<draw>` for static content
//! - `<pageSet>` / `<pageArea>` / `<contentArea>` for pagination
//! - `<exclGroup>` (exclusive groups treated as subforms)

use crate::error::{PdfError, Result};
use xfa_layout_engine::form::{ContentArea, FormNode, FormNodeId, FormNodeType, FormTree, Occur};
use xfa_layout_engine::text::FontMetrics;
use xfa_layout_engine::types::{BoxModel, Caption, CaptionPlacement, LayoutStrategy, Measurement};

/// Parse XFA template XML into a `FormTree`.
///
/// Returns the tree and the root node ID. The template XML should be the
/// raw `<template ...>` element extracted from the XFA packets.
pub fn parse_template(template_xml: &str) -> Result<(FormTree, FormNodeId)> {
    let doc = roxmltree::Document::parse(template_xml)
        .map_err(|e| PdfError::XmlParse(format!("template parse: {e}")))?;

    let root_elem = doc.root_element();

    // The root element should be <template>, containing one top-level <subform>
    if !root_elem.has_tag_name("template")
        && !root_elem.tag_name().name().eq_ignore_ascii_case("template")
    {
        return Err(PdfError::XmlParse(
            "root element is not <template>".to_string(),
        ));
    }

    let mut tree = FormTree::new();

    // Find the top-level subform(s)
    let children = parse_children(&mut tree, &root_elem)?;

    if children.is_empty() {
        return Err(PdfError::XmlParse(
            "template contains no subforms".to_string(),
        ));
    }

    // Always wrap in a Root node so the top-level subform name
    // (e.g., "form1") is preserved as part of the SOM path.
    let root_id = tree.add_node(FormNode {
        name: "Root".to_string(),
        node_type: FormNodeType::Root,
        box_model: BoxModel::default(),
        layout: LayoutStrategy::TopToBottom,
        children,
        occur: Occur::once(),
        font: FontMetrics::default(),
        calculate: None,
        validate: None,
        column_widths: vec![],
        col_span: 1,
    });
    Ok((tree, root_id))
}

/// Parse children of a container element (template, subform, exclGroup).
fn parse_children(tree: &mut FormTree, parent: &roxmltree::Node) -> Result<Vec<FormNodeId>> {
    let mut children = Vec::new();

    for child in parent.children().filter(|n| n.is_element()) {
        let tag = child.tag_name().name();
        match tag {
            "subform" => {
                let id = parse_subform(tree, &child)?;
                children.push(id);
            }
            "field" => {
                let id = parse_field(tree, &child)?;
                children.push(id);
            }
            "draw" => {
                let id = parse_draw(tree, &child)?;
                children.push(id);
            }
            "exclGroup" => {
                // Treat exclusive group as a subform container
                let id = parse_exclgroup(tree, &child)?;
                children.push(id);
            }
            "area" => {
                // <area> is a positioned container, treated like a subform
                let id = parse_area(tree, &child)?;
                children.push(id);
            }
            // Skip structural elements we handle internally
            "pageSet" | "occur" | "value" | "caption" | "ui" | "calculate" | "validate"
            | "margin" | "border" | "font" | "para" | "items" | "variables" | "proto" | "desc"
            | "extras" | "keep" | "break" | "breakBefore" | "breakAfter" | "overflow"
            | "assist" | "bind" | "connect" | "event" | "setProperty" | "traverse" | "speak"
            | "toolTip" | "bookend" => {}
            _ => {
                // Unknown elements: skip silently
            }
        }
    }

    Ok(children)
}

/// Parse a `<subform>` element.
fn parse_subform(tree: &mut FormTree, elem: &roxmltree::Node) -> Result<FormNodeId> {
    let name = elem.attribute("name").unwrap_or("").to_string();
    let layout = parse_layout_strategy(elem);
    let box_model = parse_box_model(elem);
    let occur = parse_occur(elem);
    let column_widths = parse_column_widths(elem);

    // Check for pageSet inside this subform
    let mut children = Vec::new();
    let mut page_set_id = None;

    for child in elem.children().filter(|n| n.is_element()) {
        let tag = child.tag_name().name();
        match tag {
            "pageSet" => {
                let id = parse_page_set(tree, &child)?;
                page_set_id = Some(id);
            }
            "subform" => {
                let id = parse_subform(tree, &child)?;
                children.push(id);
            }
            "field" => {
                let id = parse_field(tree, &child)?;
                children.push(id);
            }
            "draw" => {
                let id = parse_draw(tree, &child)?;
                children.push(id);
            }
            "exclGroup" => {
                let id = parse_exclgroup(tree, &child)?;
                children.push(id);
            }
            "area" => {
                let id = parse_area(tree, &child)?;
                children.push(id);
            }
            _ => {}
        }
    }

    // If there's a pageSet, insert it before the content children
    if let Some(ps) = page_set_id {
        children.insert(0, ps);
    }

    let id = tree.add_node(FormNode {
        name,
        node_type: FormNodeType::Subform,
        box_model,
        layout,
        children,
        occur,
        font: parse_font_metrics(elem),
        calculate: extract_script(elem, "calculate"),
        validate: extract_script(elem, "validate"),
        column_widths,
        col_span: parse_col_span(elem),
    });

    Ok(id)
}

/// Parse a `<field>` element.
fn parse_field(tree: &mut FormTree, elem: &roxmltree::Node) -> Result<FormNodeId> {
    let name = elem.attribute("name").unwrap_or("").to_string();
    let box_model = parse_box_model(elem);
    let value = extract_value(elem);

    let id = tree.add_node(FormNode {
        name,
        node_type: FormNodeType::Field { value },
        box_model,
        layout: LayoutStrategy::Positioned,
        children: vec![],
        occur: parse_occur(elem),
        font: parse_font_metrics(elem),
        calculate: extract_script(elem, "calculate"),
        validate: extract_script(elem, "validate"),
        column_widths: vec![],
        col_span: parse_col_span(elem),
    });

    Ok(id)
}

/// Parse a `<draw>` element.
fn parse_draw(tree: &mut FormTree, elem: &roxmltree::Node) -> Result<FormNodeId> {
    let name = elem.attribute("name").unwrap_or("").to_string();
    let box_model = parse_box_model(elem);
    let content = extract_value(elem);

    let id = tree.add_node(FormNode {
        name,
        node_type: FormNodeType::Draw { content },
        box_model,
        layout: LayoutStrategy::Positioned,
        children: vec![],
        occur: Occur::once(),
        font: parse_font_metrics(elem),
        calculate: None,
        validate: None,
        column_widths: vec![],
        col_span: parse_col_span(elem),
    });

    Ok(id)
}

/// Parse an `<exclGroup>` element as a subform.
fn parse_exclgroup(tree: &mut FormTree, elem: &roxmltree::Node) -> Result<FormNodeId> {
    let name = elem.attribute("name").unwrap_or("").to_string();
    let box_model = parse_box_model(elem);
    let children = parse_children(tree, elem)?;

    let id = tree.add_node(FormNode {
        name,
        node_type: FormNodeType::Subform,
        box_model,
        layout: parse_layout_strategy(elem),
        children,
        occur: parse_occur(elem),
        font: FontMetrics::default(),
        calculate: extract_script(elem, "calculate"),
        validate: extract_script(elem, "validate"),
        column_widths: vec![],
        col_span: parse_col_span(elem),
    });

    Ok(id)
}

/// Parse an `<area>` element as a positioned subform.
fn parse_area(tree: &mut FormTree, elem: &roxmltree::Node) -> Result<FormNodeId> {
    let name = elem.attribute("name").unwrap_or("").to_string();
    let box_model = parse_box_model(elem);
    let children = parse_children(tree, elem)?;

    let id = tree.add_node(FormNode {
        name,
        node_type: FormNodeType::Subform,
        box_model,
        layout: LayoutStrategy::Positioned,
        children,
        occur: Occur::once(),
        font: FontMetrics::default(),
        calculate: None,
        validate: None,
        column_widths: vec![],
        col_span: parse_col_span(elem),
    });

    Ok(id)
}

/// Parse a `<pageSet>` element.
fn parse_page_set(tree: &mut FormTree, elem: &roxmltree::Node) -> Result<FormNodeId> {
    let mut children = Vec::new();

    for child in elem.children().filter(|n| n.is_element()) {
        if child.tag_name().name() == "pageArea" {
            let id = parse_page_area(tree, &child)?;
            children.push(id);
        }
    }

    let id = tree.add_node(FormNode {
        name: "pageSet".to_string(),
        node_type: FormNodeType::PageSet,
        box_model: BoxModel::default(),
        layout: LayoutStrategy::Positioned,
        children,
        occur: Occur::once(),
        font: FontMetrics::default(),
        calculate: None,
        validate: None,
        column_widths: vec![],
        col_span: 1,
    });

    Ok(id)
}

/// Parse a `<pageArea>` element.
fn parse_page_area(tree: &mut FormTree, elem: &roxmltree::Node) -> Result<FormNodeId> {
    let name = elem.attribute("name").unwrap_or("Page1").to_string();
    let mut content_areas = Vec::new();

    for child in elem.children().filter(|n| n.is_element()) {
        if child.tag_name().name() == "contentArea" {
            content_areas.push(parse_content_area(&child));
        }
    }

    // If no content area specified, use a default US Letter area
    if content_areas.is_empty() {
        content_areas.push(ContentArea::default());
    }

    let id = tree.add_node(FormNode {
        name,
        node_type: FormNodeType::PageArea { content_areas },
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

    Ok(id)
}

/// Parse a `<contentArea>` element.
fn parse_content_area(elem: &roxmltree::Node) -> ContentArea {
    let name = elem.attribute("name").unwrap_or("").to_string();
    let x = parse_measurement_attr(elem, "x").unwrap_or(0.0);
    let y = parse_measurement_attr(elem, "y").unwrap_or(0.0);
    let width = parse_measurement_attr(elem, "w").unwrap_or(612.0);
    let height = parse_measurement_attr(elem, "h").unwrap_or(792.0);

    ContentArea {
        name,
        x,
        y,
        width,
        height,
        leader: None,
        trailer: None,
    }
}

// --- Attribute Parsing Helpers ---

/// Parse a measurement attribute (e.g., w="8in", h="10.5in") to points.
fn parse_measurement_attr(elem: &roxmltree::Node, attr: &str) -> Option<f64> {
    elem.attribute(attr)
        .and_then(Measurement::parse)
        .map(|m| m.to_points())
}

/// Parse the `layout` attribute into a `LayoutStrategy`.
fn parse_layout_strategy(elem: &roxmltree::Node) -> LayoutStrategy {
    match elem.attribute("layout") {
        Some("tb") => LayoutStrategy::TopToBottom,
        Some("lr-tb") => LayoutStrategy::LeftToRightTB,
        Some("rl-tb") => LayoutStrategy::RightToLeftTB,
        Some("table") => LayoutStrategy::Table,
        Some("row") => LayoutStrategy::Row,
        Some("position") => LayoutStrategy::Positioned,
        _ => LayoutStrategy::Positioned,
    }
}

/// Parse a `BoxModel` from element attributes.
fn parse_box_model(elem: &roxmltree::Node) -> BoxModel {
    let width = parse_measurement_attr(elem, "w");
    let height = parse_measurement_attr(elem, "h");
    let x = parse_measurement_attr(elem, "x").unwrap_or(0.0);
    let y = parse_measurement_attr(elem, "y").unwrap_or(0.0);
    let min_w = parse_measurement_attr(elem, "minW").unwrap_or(0.0);
    let max_w = parse_measurement_attr(elem, "maxW").unwrap_or(f64::MAX);
    let min_h = parse_measurement_attr(elem, "minH").unwrap_or(0.0);
    let max_h = parse_measurement_attr(elem, "maxH").unwrap_or(f64::MAX);

    let margins = parse_margins(elem);
    let border_width = parse_border_width(elem);
    let caption = parse_caption(elem);

    BoxModel {
        width,
        height,
        x,
        y,
        margins,
        border_width,
        min_width: min_w,
        max_width: max_w,
        min_height: min_h,
        max_height: max_h,
        caption,
    }
}

/// Parse `<margin>` child element for insets.
fn parse_margins(elem: &roxmltree::Node) -> xfa_layout_engine::types::Insets {
    let margin = elem
        .children()
        .find(|n| n.is_element() && n.tag_name().name() == "margin");

    match margin {
        Some(m) => {
            let top = parse_measurement_attr(&m, "topInset").unwrap_or(0.0);
            let right = parse_measurement_attr(&m, "rightInset").unwrap_or(0.0);
            let bottom = parse_measurement_attr(&m, "bottomInset").unwrap_or(0.0);
            let left = parse_measurement_attr(&m, "leftInset").unwrap_or(0.0);
            xfa_layout_engine::types::Insets {
                top,
                right,
                bottom,
                left,
            }
        }
        None => xfa_layout_engine::types::Insets::default(),
    }
}

/// Parse `<border>` child for border width.
fn parse_border_width(elem: &roxmltree::Node) -> f64 {
    let border = elem
        .children()
        .find(|n| n.is_element() && n.tag_name().name() == "border");

    match border {
        Some(b) => {
            // Look for <edge> child with thickness attribute
            let edge = b
                .children()
                .find(|n| n.is_element() && n.tag_name().name() == "edge");
            match edge {
                Some(e) => parse_measurement_attr(&e, "thickness").unwrap_or(0.0),
                None => 0.0,
            }
        }
        None => 0.0,
    }
}

/// Parse `<caption>` child element.
fn parse_caption(elem: &roxmltree::Node) -> Option<Caption> {
    let cap = elem
        .children()
        .find(|n| n.is_element() && n.tag_name().name() == "caption")?;

    let placement = match cap.attribute("placement") {
        Some("top") => CaptionPlacement::Top,
        Some("right") => CaptionPlacement::Right,
        Some("bottom") => CaptionPlacement::Bottom,
        Some("inline") => CaptionPlacement::Inline,
        _ => CaptionPlacement::Left,
    };

    let reserve = parse_measurement_attr(&cap, "reserve");
    let text = extract_value(&cap);

    Some(Caption {
        placement,
        reserve,
        text,
    })
}

/// Parse `<occur>` child element.
fn parse_occur(elem: &roxmltree::Node) -> Occur {
    let occur = elem
        .children()
        .find(|n| n.is_element() && n.tag_name().name() == "occur");

    match occur {
        Some(o) => {
            let min: u32 = o.attribute("min").and_then(|s| s.parse().ok()).unwrap_or(1);
            let max: Option<u32> = o.attribute("max").and_then(|s| {
                let v: i32 = s.parse().ok()?;
                if v < 0 {
                    None
                } else {
                    Some(v as u32)
                }
            });
            let initial: u32 = o
                .attribute("initial")
                .and_then(|s| s.parse().ok())
                .unwrap_or(min);
            Occur::repeating(min, max, initial)
        }
        None => Occur::once(),
    }
}

/// Parse `columnWidths` attribute for table layouts.
fn parse_column_widths(elem: &roxmltree::Node) -> Vec<f64> {
    match elem.attribute("columnWidths") {
        Some(s) => s
            .split_whitespace()
            .filter_map(|w| {
                if w == "0" || w == "0pt" {
                    Some(-1.0) // auto-size
                } else {
                    Measurement::parse(w).map(|m| m.to_points())
                }
            })
            .collect(),
        None => vec![],
    }
}

/// Parse `colSpan` attribute.
fn parse_col_span(elem: &roxmltree::Node) -> i32 {
    elem.attribute("colSpan")
        .and_then(|s| s.parse().ok())
        .unwrap_or(1)
}

/// Parse font metrics from `<font>` child element.
fn parse_font_metrics(elem: &roxmltree::Node) -> FontMetrics {
    let font = elem
        .children()
        .find(|n| n.is_element() && n.tag_name().name() == "font");

    match font {
        Some(f) => {
            let size = parse_measurement_attr(&f, "size").unwrap_or(10.0);
            FontMetrics {
                size,
                ..FontMetrics::default()
            }
        }
        None => FontMetrics::default(),
    }
}

/// Extract a `<value>` child's text content.
///
/// Handles the XFA value model: `<value><text>content</text></value>`
/// or `<value><float>42.5</float></value>`, etc.
fn extract_value(elem: &roxmltree::Node) -> String {
    let value_elem = elem
        .children()
        .find(|n| n.is_element() && n.tag_name().name() == "value");

    match value_elem {
        Some(v) => {
            // Look for typed children: text, float, integer, decimal, etc.
            for child in v.children().filter(|n| n.is_element()) {
                let tag = child.tag_name().name();
                match tag {
                    "text" | "float" | "integer" | "decimal" | "date" | "time" | "dateTime"
                    | "exData" => {
                        return child.text().unwrap_or("").trim().to_string();
                    }
                    _ => {}
                }
            }
            // Fallback: direct text content
            v.text().unwrap_or("").trim().to_string()
        }
        None => String::new(),
    }
}

/// Extract a script from a named event child.
///
/// Looks for `<calculate><script>...</script></calculate>` or
/// `<validate><script>...</script></validate>`.
fn extract_script(elem: &roxmltree::Node, event_name: &str) -> Option<String> {
    let event_elem = elem
        .children()
        .find(|n| n.is_element() && n.tag_name().name() == event_name)?;

    let script = event_elem
        .children()
        .find(|n| n.is_element() && n.tag_name().name() == "script")?;

    let text = script.text()?.trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_template() {
        let xml = r#"<template xmlns="http://www.xfa.org/schema/xfa-template/3.3/">
            <subform name="form1" layout="tb">
                <field name="Name" w="200pt" h="25pt">
                    <value><text>John</text></value>
                </field>
                <field name="Amount" w="100pt" h="25pt">
                    <value><float>42.50</float></value>
                </field>
            </subform>
        </template>"#;

        let (tree, root) = parse_template(xml).unwrap();
        let root_node = tree.get(root);
        assert_eq!(root_node.name, "Root");
        assert!(matches!(root_node.node_type, FormNodeType::Root));
        assert_eq!(root_node.children.len(), 1);

        // Top-level subform
        let form1 = tree.get(root_node.children[0]);
        assert_eq!(form1.name, "form1");
        assert!(matches!(form1.node_type, FormNodeType::Subform));
        assert_eq!(form1.children.len(), 2);

        // First field
        let name_field = tree.get(form1.children[0]);
        assert_eq!(name_field.name, "Name");
        if let FormNodeType::Field { value } = &name_field.node_type {
            assert_eq!(value, "John");
        } else {
            panic!("Expected Field");
        }
        assert_eq!(name_field.box_model.width, Some(200.0));
        assert_eq!(name_field.box_model.height, Some(25.0));

        // Second field
        let amount_field = tree.get(form1.children[1]);
        assert_eq!(amount_field.name, "Amount");
        if let FormNodeType::Field { value } = &amount_field.node_type {
            assert_eq!(value, "42.50");
        } else {
            panic!("Expected Field");
        }
    }

    #[test]
    fn parse_nested_subforms() {
        let xml = r#"<template xmlns="http://www.xfa.org/schema/xfa-template/3.3/">
            <subform name="form1" layout="tb">
                <subform name="Customer">
                    <field name="Name"><value><text>Acme</text></value></field>
                </subform>
            </subform>
        </template>"#;

        let (tree, root) = parse_template(xml).unwrap();
        let form1 = tree.get(tree.get(root).children[0]);
        assert_eq!(form1.children.len(), 1);

        let customer = tree.get(form1.children[0]);
        assert_eq!(customer.name, "Customer");
        assert!(matches!(customer.node_type, FormNodeType::Subform));
        assert_eq!(customer.children.len(), 1);
    }

    /// Helper to get the form1 subform inside the Root wrapper.
    fn get_form1(tree: &FormTree, root: FormNodeId) -> FormNodeId {
        tree.get(root).children[0]
    }

    #[test]
    fn parse_occur_rules() {
        let xml = r#"<template xmlns="http://www.xfa.org/schema/xfa-template/3.3/">
            <subform name="form1">
                <subform name="Item">
                    <occur min="0" max="-1" initial="2"/>
                    <field name="Desc"/>
                </subform>
            </subform>
        </template>"#;

        let (tree, root) = parse_template(xml).unwrap();
        let form1 = get_form1(&tree, root);
        let item = tree.get(tree.get(form1).children[0]);
        assert_eq!(item.name, "Item");
        assert!(item.occur.is_repeating());
        assert_eq!(item.occur.min, 0);
        assert_eq!(item.occur.max, None); // unlimited
        assert_eq!(item.occur.initial, 2);
    }

    #[test]
    fn parse_scripts() {
        let xml = r#"<template xmlns="http://www.xfa.org/schema/xfa-template/3.3/">
            <subform name="form1">
                <field name="Tax">
                    <calculate><script>Subtotal * 0.21</script></calculate>
                    <validate><script>Tax >= 0</script></validate>
                </field>
            </subform>
        </template>"#;

        let (tree, root) = parse_template(xml).unwrap();
        let form1 = get_form1(&tree, root);
        let tax = tree.get(tree.get(form1).children[0]);
        assert_eq!(tax.calculate, Some("Subtotal * 0.21".to_string()));
        assert_eq!(tax.validate, Some("Tax >= 0".to_string()));
    }

    #[test]
    fn parse_draw_element() {
        let xml = r#"<template xmlns="http://www.xfa.org/schema/xfa-template/3.3/">
            <subform name="form1">
                <draw name="Header">
                    <value><text>Tax Form 2024</text></value>
                </draw>
            </subform>
        </template>"#;

        let (tree, root) = parse_template(xml).unwrap();
        let form1 = get_form1(&tree, root);
        let draw = tree.get(tree.get(form1).children[0]);
        assert_eq!(draw.name, "Header");
        if let FormNodeType::Draw { content } = &draw.node_type {
            assert_eq!(content, "Tax Form 2024");
        } else {
            panic!("Expected Draw");
        }
    }

    #[test]
    fn parse_page_set() {
        let xml = r#"<template xmlns="http://www.xfa.org/schema/xfa-template/3.3/">
            <subform name="form1">
                <pageSet>
                    <pageArea name="Page1">
                        <contentArea name="body" x="0.25in" y="0.25in" w="8in" h="10.5in"/>
                    </pageArea>
                </pageSet>
                <field name="F1"/>
            </subform>
        </template>"#;

        let (tree, root) = parse_template(xml).unwrap();
        let form1 = tree.get(get_form1(&tree, root));
        // First child should be pageSet, second should be field
        assert_eq!(form1.children.len(), 2);

        let page_set = tree.get(form1.children[0]);
        assert!(matches!(page_set.node_type, FormNodeType::PageSet));

        let page_area = tree.get(page_set.children[0]);
        if let FormNodeType::PageArea { content_areas } = &page_area.node_type {
            assert_eq!(content_areas.len(), 1);
            assert_eq!(content_areas[0].name, "body");
            assert!((content_areas[0].x - 18.0).abs() < 0.1); // 0.25in = 18pt
            assert!((content_areas[0].width - 576.0).abs() < 0.1); // 8in = 576pt
        } else {
            panic!("Expected PageArea");
        }
    }

    #[test]
    fn parse_measurement_units() {
        let xml = r#"<template xmlns="http://www.xfa.org/schema/xfa-template/3.3/">
            <subform name="form1">
                <field name="F1" w="2in" h="10mm" x="1cm" y="36pt"/>
            </subform>
        </template>"#;

        let (tree, root) = parse_template(xml).unwrap();
        let form1 = get_form1(&tree, root);
        let f1 = tree.get(tree.get(form1).children[0]);
        assert!((f1.box_model.width.unwrap() - 144.0).abs() < 0.1); // 2in
        assert!((f1.box_model.height.unwrap() - 28.35).abs() < 0.1); // 10mm
        assert!((f1.box_model.x - 28.35).abs() < 0.1); // 1cm
        assert!((f1.box_model.y - 36.0).abs() < 0.1); // 36pt
    }

    #[test]
    fn parse_caption_element() {
        let xml = r#"<template xmlns="http://www.xfa.org/schema/xfa-template/3.3/">
            <subform name="form1">
                <field name="Name">
                    <caption placement="top" reserve="20pt">
                        <value><text>Full Name:</text></value>
                    </caption>
                </field>
            </subform>
        </template>"#;

        let (tree, root) = parse_template(xml).unwrap();
        let form1 = get_form1(&tree, root);
        let f = tree.get(tree.get(form1).children[0]);
        let cap = f.box_model.caption.as_ref().unwrap();
        assert_eq!(cap.placement, CaptionPlacement::Top);
        assert_eq!(cap.reserve, Some(20.0));
        assert_eq!(cap.text, "Full Name:");
    }

    #[test]
    fn parse_layout_strategies() {
        let xml = r#"<template xmlns="http://www.xfa.org/schema/xfa-template/3.3/">
            <subform name="form1" layout="tb">
                <subform name="a" layout="lr-tb"/>
                <subform name="b" layout="table"/>
                <subform name="c" layout="row"/>
                <subform name="d" layout="position"/>
                <subform name="e" layout="rl-tb"/>
            </subform>
        </template>"#;

        let (tree, root) = parse_template(xml).unwrap();
        let form1 = tree.get(get_form1(&tree, root));
        assert_eq!(form1.layout, LayoutStrategy::TopToBottom);

        let cases = [
            (0, LayoutStrategy::LeftToRightTB),
            (1, LayoutStrategy::Table),
            (2, LayoutStrategy::Row),
            (3, LayoutStrategy::Positioned),
            (4, LayoutStrategy::RightToLeftTB),
        ];
        for (i, expected) in &cases {
            let child = tree.get(form1.children[*i]);
            assert_eq!(child.layout, *expected, "child {} layout mismatch", i);
        }
    }

    #[test]
    fn empty_template_fails() {
        let xml = r#"<template xmlns="http://www.xfa.org/schema/xfa-template/3.3/">
        </template>"#;

        let result = parse_template(xml);
        assert!(result.is_err());
    }

    #[test]
    fn invalid_xml_fails() {
        let result = parse_template("<not valid xml");
        assert!(result.is_err());
    }
}
