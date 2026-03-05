//! Data merge — merge XFA datasets XML into a FormTree.
//!
//! In XFA, the `<datasets>` packet contains the form's data values.
//! This module parses the datasets XML and merges values into the
//! corresponding fields of a FormTree built from the template.

use crate::error::{PdfError, Result};
use xfa_layout_engine::form::{FormNodeId, FormNodeType, FormTree};

/// Merge datasets XML into a FormTree.
///
/// The datasets XML typically has the structure:
/// ```xml
/// <xfa:datasets xmlns:xfa="http://www.xfa.org/schema/xfa-data/1.0/">
///   <xfa:data>
///     <form1>
///       <FieldName>value</FieldName>
///       <SubformName>
///         <NestedField>value</NestedField>
///       </SubformName>
///     </form1>
///   </xfa:data>
/// </xfa:datasets>
/// ```
///
/// This function walks the FormTree and for each field, looks up the
/// corresponding data element by matching the tree structure.
pub fn merge_data(tree: &mut FormTree, root: FormNodeId, datasets_xml: &str) -> Result<()> {
    let doc = roxmltree::Document::parse(datasets_xml)
        .map_err(|e| PdfError::XmlParse(format!("datasets parse: {e}")))?;

    // Find the <xfa:data> element (or the root data element)
    let data_root = find_data_root(&doc)?;

    // Start merging from the root of the FormTree
    merge_node(tree, root, &data_root);

    Ok(())
}

/// Find the data root element within the datasets XML.
///
/// Looks for `<xfa:data>` inside `<xfa:datasets>`, or falls back
/// to the document root if the wrapper is absent.
fn find_data_root<'a>(doc: &'a roxmltree::Document) -> Result<roxmltree::Node<'a, 'a>> {
    let root = doc.root_element();

    // Case 1: <xfa:datasets><xfa:data>...</xfa:data></xfa:datasets>
    if root.tag_name().name() == "datasets" {
        for child in root.children().filter(|n| n.is_element()) {
            if child.tag_name().name() == "data" {
                // Return the first element child of <xfa:data> (the form root)
                if let Some(form_root) = child.children().find(|n| n.is_element()) {
                    return Ok(form_root);
                }
                return Ok(child);
            }
        }
        // No <xfa:data> found, try first element child
        if let Some(child) = root.children().find(|n| n.is_element()) {
            return Ok(child);
        }
    }

    // Case 2: Raw data XML without wrapper
    Ok(root)
}

/// Recursively merge data into a FormTree node.
fn merge_node(tree: &mut FormTree, node_id: FormNodeId, data_elem: &roxmltree::Node) {
    let node = tree.get(node_id);
    let node_name = node.name.clone();
    let node_type_is_field = matches!(node.node_type, FormNodeType::Field { .. });
    let children = node.children.clone();

    if node_type_is_field {
        // For Field nodes: look for a matching element or text
        if let Some(value) = find_element_text(data_elem, &node_name) {
            let node = tree.get_mut(node_id);
            if let FormNodeType::Field { value: ref mut v } = node.node_type {
                *v = value;
            }
        }
        return;
    }

    // For Root/Subform nodes: recurse into children
    // First, try to find a matching data element for the subform
    let subform_data = if matches!(
        tree.get(node_id).node_type,
        FormNodeType::Root | FormNodeType::Subform
    ) && !node_name.is_empty()
    {
        // If the data element's name matches the subform name, use it directly
        if data_elem.tag_name().name() == node_name {
            Some(*data_elem)
        } else {
            // Otherwise look for a child element with the subform name
            find_element(data_elem, &node_name)
        }
    } else {
        Some(*data_elem)
    };

    let data_ctx = subform_data.unwrap_or(*data_elem);

    // Track occurrences per name for repeating subforms
    let mut name_count: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for &child_id in &children {
        let child = tree.get(child_id);
        let child_name = child.name.clone();
        let is_repeating = child.occur.is_repeating();

        match &child.node_type {
            FormNodeType::Field { .. } => {
                // Field: merge value directly
                if let Some(value) = find_element_text(&data_ctx, &child_name) {
                    let child_mut = tree.get_mut(child_id);
                    if let FormNodeType::Field { value: ref mut v } = child_mut.node_type {
                        *v = value;
                    }
                }
            }
            FormNodeType::Subform if is_repeating => {
                // Repeating subform: find the Nth element with this name
                let idx = name_count.entry(child_name.clone()).or_insert(0);
                if let Some(data_child) = find_nth_element(&data_ctx, &child_name, *idx) {
                    merge_node(tree, child_id, &data_child);
                }
                *name_count.get_mut(&child_name).unwrap() += 1;
            }
            FormNodeType::Subform => {
                // Non-repeating subform: find matching element
                if let Some(data_child) = find_element(&data_ctx, &child_name) {
                    merge_node(tree, child_id, &data_child);
                } else {
                    // Try merging with current context (flat data)
                    merge_node(tree, child_id, &data_ctx);
                }
            }
            FormNodeType::PageSet | FormNodeType::PageArea { .. } => {
                // Skip structural nodes
            }
            FormNodeType::Root => {
                merge_node(tree, child_id, &data_ctx);
            }
            FormNodeType::Draw { .. } => {
                // Draw nodes are static, no data merge
            }
        }
    }
}

/// Find a child element by name and return its text content.
fn find_element_text(parent: &roxmltree::Node, name: &str) -> Option<String> {
    let elem = find_element(parent, name)?;
    // Get text content, handling elements with mixed content
    let text: String = elem
        .children()
        .filter(|n| n.is_text())
        .map(|n| n.text().unwrap_or(""))
        .collect();
    let trimmed = text.trim().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

/// Find the first child element with a given local name.
fn find_element<'a>(parent: &'a roxmltree::Node, name: &str) -> Option<roxmltree::Node<'a, 'a>> {
    parent
        .children()
        .find(|n| n.is_element() && n.tag_name().name() == name)
}

/// Find the Nth child element with a given local name.
fn find_nth_element<'a>(
    parent: &'a roxmltree::Node,
    name: &str,
    index: usize,
) -> Option<roxmltree::Node<'a, 'a>> {
    parent
        .children()
        .filter(|n| n.is_element() && n.tag_name().name() == name)
        .nth(index)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template_parser::parse_template;

    fn build_tree(template_xml: &str) -> (FormTree, FormNodeId) {
        parse_template(template_xml).unwrap()
    }

    fn get_field_value(tree: &FormTree, id: FormNodeId) -> String {
        match &tree.get(id).node_type {
            FormNodeType::Field { value } => value.clone(),
            _ => panic!("Not a field node"),
        }
    }

    /// Navigate through Root → form1 to get the top-level subform.
    fn get_form1(tree: &FormTree, root: FormNodeId) -> FormNodeId {
        tree.get(root).children[0]
    }

    #[test]
    fn merge_simple_data() {
        let template = r#"<template xmlns="http://www.xfa.org/schema/xfa-template/3.3/">
            <subform name="form1">
                <field name="Name"/>
                <field name="Amount"/>
            </subform>
        </template>"#;

        let datasets = r#"<xfa:datasets xmlns:xfa="http://www.xfa.org/schema/xfa-data/1.0/">
            <xfa:data>
                <form1>
                    <Name>John Doe</Name>
                    <Amount>42.50</Amount>
                </form1>
            </xfa:data>
        </xfa:datasets>"#;

        let (mut tree, root) = build_tree(template);
        merge_data(&mut tree, root, datasets).unwrap();

        let form1 = tree.get(get_form1(&tree, root));
        assert_eq!(get_field_value(&tree, form1.children[0]), "John Doe");
        assert_eq!(get_field_value(&tree, form1.children[1]), "42.50");
    }

    #[test]
    fn merge_nested_subforms() {
        let template = r#"<template xmlns="http://www.xfa.org/schema/xfa-template/3.3/">
            <subform name="form1">
                <subform name="Customer">
                    <field name="Name"/>
                    <field name="City"/>
                </subform>
            </subform>
        </template>"#;

        let datasets = r#"<xfa:datasets xmlns:xfa="http://www.xfa.org/schema/xfa-data/1.0/">
            <xfa:data>
                <form1>
                    <Customer>
                        <Name>Acme Corp</Name>
                        <City>Amsterdam</City>
                    </Customer>
                </form1>
            </xfa:data>
        </xfa:datasets>"#;

        let (mut tree, root) = build_tree(template);
        merge_data(&mut tree, root, datasets).unwrap();

        let form1 = get_form1(&tree, root);
        let customer = tree.get(tree.get(form1).children[0]);
        assert_eq!(get_field_value(&tree, customer.children[0]), "Acme Corp");
        assert_eq!(get_field_value(&tree, customer.children[1]), "Amsterdam");
    }

    #[test]
    fn merge_preserves_template_defaults() {
        let template = r#"<template xmlns="http://www.xfa.org/schema/xfa-template/3.3/">
            <subform name="form1">
                <field name="Name"><value><text>Default</text></value></field>
                <field name="Other"><value><text>Keep Me</text></value></field>
            </subform>
        </template>"#;

        let datasets = r#"<xfa:datasets xmlns:xfa="http://www.xfa.org/schema/xfa-data/1.0/">
            <xfa:data>
                <form1>
                    <Name>Override</Name>
                </form1>
            </xfa:data>
        </xfa:datasets>"#;

        let (mut tree, root) = build_tree(template);
        merge_data(&mut tree, root, datasets).unwrap();

        let form1 = tree.get(get_form1(&tree, root));
        assert_eq!(get_field_value(&tree, form1.children[0]), "Override");
        // Field not in data should keep its template default
        assert_eq!(get_field_value(&tree, form1.children[1]), "Keep Me");
    }

    #[test]
    fn merge_repeating_subforms() {
        let template = r#"<template xmlns="http://www.xfa.org/schema/xfa-template/3.3/">
            <subform name="form1">
                <subform name="Item">
                    <occur min="0" max="-1" initial="2"/>
                    <field name="Desc"/>
                </subform>
                <subform name="Item">
                    <occur min="0" max="-1" initial="2"/>
                    <field name="Desc"/>
                </subform>
            </subform>
        </template>"#;

        let datasets = r#"<xfa:datasets xmlns:xfa="http://www.xfa.org/schema/xfa-data/1.0/">
            <xfa:data>
                <form1>
                    <Item><Desc>Widget A</Desc></Item>
                    <Item><Desc>Widget B</Desc></Item>
                </form1>
            </xfa:data>
        </xfa:datasets>"#;

        let (mut tree, root) = build_tree(template);
        merge_data(&mut tree, root, datasets).unwrap();

        let form1 = tree.get(get_form1(&tree, root));
        let item0 = tree.get(form1.children[0]);
        let item1 = tree.get(form1.children[1]);

        assert_eq!(get_field_value(&tree, item0.children[0]), "Widget A");
        assert_eq!(get_field_value(&tree, item1.children[0]), "Widget B");
    }

    #[test]
    fn merge_empty_datasets_is_ok() {
        let template = r#"<template xmlns="http://www.xfa.org/schema/xfa-template/3.3/">
            <subform name="form1">
                <field name="Name"><value><text>Default</text></value></field>
            </subform>
        </template>"#;

        let datasets = r#"<xfa:datasets xmlns:xfa="http://www.xfa.org/schema/xfa-data/1.0/">
            <xfa:data/>
        </xfa:datasets>"#;

        let (mut tree, root) = build_tree(template);
        // Empty datasets should not fail
        let result = merge_data(&mut tree, root, datasets);
        assert!(result.is_ok());
    }

    #[test]
    fn merge_invalid_xml_fails() {
        let template = r#"<template xmlns="http://www.xfa.org/schema/xfa-template/3.3/">
            <subform name="form1"><field name="F1"/></subform>
        </template>"#;

        let (mut tree, root) = build_tree(template);
        let result = merge_data(&mut tree, root, "<not valid");
        assert!(result.is_err());
    }
}
