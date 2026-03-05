//! Golden render tests — verify layout engine output is visually stable.
//!
//! These tests create form trees, run layout, render to images, and compare
//! against golden reference images. On first run, golden images are created.
//! On subsequent runs, the test verifies the output matches.

use std::path::PathBuf;

use xfa_golden_tests::compare_images;
use xfa_golden_tests::render::render_layout;
use xfa_layout_engine::form::{FormNode, FormNodeId, FormNodeType, FormTree, Occur};
use xfa_layout_engine::layout::LayoutEngine;
use xfa_layout_engine::text::FontMetrics;
use xfa_layout_engine::types::{BoxModel, LayoutStrategy};

fn golden_dir() -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("golden");
    std::fs::create_dir_all(&dir).ok();
    dir
}

/// Run layout, render, and compare against golden image.
/// If the golden file doesn't exist, creates it (first run).
/// Returns true if the test passed or if a new golden was created.
fn assert_golden(name: &str, tree: &FormTree, root: FormNodeId) {
    let engine = LayoutEngine::new(tree);
    let layout = engine.layout(root).unwrap();
    let images = render_layout(&layout);

    for (i, img) in images.iter().enumerate() {
        let golden_path = golden_dir().join(format!("{name}_page{i}.png"));

        if golden_path.exists() {
            let expected = image::open(&golden_path).unwrap();
            let result = compare_images(img, &expected, 0.0, 0).unwrap();
            assert!(
                result.passed,
                "Golden test '{name}' page {i} failed: {:.2}% pixels differ ({} of {})",
                result.diff_percentage, result.differing_pixels, result.total_pixels
            );
        } else {
            // First run: save golden image
            img.save(&golden_path).unwrap();
            eprintln!(
                "Created golden image: {}",
                golden_path.display()
            );
        }
    }
}

fn make_field(tree: &mut FormTree, name: &str, w: f64, h: f64) -> FormNodeId {
    tree.add_node(FormNode {
        name: name.to_string(),
        node_type: FormNodeType::Field {
            value: name.to_string(),
        },
        box_model: BoxModel {
            width: Some(w),
            height: Some(h),
            max_width: f64::MAX,
            max_height: f64::MAX,
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
    })
}

fn make_draw(tree: &mut FormTree, name: &str, content: &str, w: f64, h: f64) -> FormNodeId {
    tree.add_node(FormNode {
        name: name.to_string(),
        node_type: FormNodeType::Draw {
            content: content.to_string(),
        },
        box_model: BoxModel {
            width: Some(w),
            height: Some(h),
            max_width: f64::MAX,
            max_height: f64::MAX,
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
    })
}

fn make_subform(
    tree: &mut FormTree,
    name: &str,
    strategy: LayoutStrategy,
    w: Option<f64>,
    h: Option<f64>,
    children: Vec<FormNodeId>,
) -> FormNodeId {
    tree.add_node(FormNode {
        name: name.to_string(),
        node_type: FormNodeType::Subform,
        box_model: BoxModel {
            width: w,
            height: h,
            max_width: f64::MAX,
            max_height: f64::MAX,
            ..Default::default()
        },
        layout: strategy,
        children,
        occur: Occur::once(),
        font: FontMetrics::default(),
        calculate: None,
        validate: None,
        column_widths: vec![],
        col_span: 1,
    })
}

// ── Test cases ──────────────────────────────────────────────────────

#[test]
fn golden_simple_positioned() {
    let mut tree = FormTree::new();
    let f1 = tree.add_node(FormNode {
        name: "Name".to_string(),
        node_type: FormNodeType::Field {
            value: "John".to_string(),
        },
        box_model: BoxModel {
            width: Some(150.0),
            height: Some(25.0),
            x: 50.0,
            y: 50.0,
            max_width: f64::MAX,
            max_height: f64::MAX,
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
    let f2 = tree.add_node(FormNode {
        name: "Email".to_string(),
        node_type: FormNodeType::Field {
            value: "john@example.com".to_string(),
        },
        box_model: BoxModel {
            width: Some(200.0),
            height: Some(25.0),
            x: 50.0,
            y: 90.0,
            max_width: f64::MAX,
            max_height: f64::MAX,
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
        name: "Root".to_string(),
        node_type: FormNodeType::Root,
        box_model: BoxModel {
            width: Some(300.0),
            height: Some(200.0),
            max_width: f64::MAX,
            max_height: f64::MAX,
            ..Default::default()
        },
        layout: LayoutStrategy::Positioned,
        children: vec![f1, f2],
        occur: Occur::once(),
        font: FontMetrics::default(),
        calculate: None,
        validate: None,
        column_widths: vec![],
        col_span: 1,
    });

    assert_golden("simple_positioned", &tree, root);
}

#[test]
fn golden_tb_layout() {
    let mut tree = FormTree::new();
    let f1 = make_field(&mut tree, "Item1", 200.0, 30.0);
    let f2 = make_field(&mut tree, "Item2", 200.0, 30.0);
    let f3 = make_field(&mut tree, "Item3", 200.0, 30.0);
    let root = make_subform(
        &mut tree,
        "Root",
        LayoutStrategy::TopToBottom,
        Some(300.0),
        Some(200.0),
        vec![f1, f2, f3],
    );

    assert_golden("tb_layout", &tree, root);
}

#[test]
fn golden_lr_tb_wrapping() {
    let mut tree = FormTree::new();
    let items: Vec<FormNodeId> = (0..6)
        .map(|i| make_field(&mut tree, &format!("Box{i}"), 80.0, 40.0))
        .collect();
    let root = make_subform(
        &mut tree,
        "Root",
        LayoutStrategy::LeftToRightTB,
        Some(250.0),
        Some(200.0),
        items,
    );

    assert_golden("lr_tb_wrapping", &tree, root);
}

#[test]
fn golden_nested_subforms() {
    let mut tree = FormTree::new();
    let header_label = make_draw(&mut tree, "Title", "Invoice", 200.0, 20.0);
    let header = make_subform(
        &mut tree,
        "Header",
        LayoutStrategy::TopToBottom,
        Some(250.0),
        Some(30.0),
        vec![header_label],
    );
    let f1 = make_field(&mut tree, "Amount", 100.0, 25.0);
    let f2 = make_field(&mut tree, "Tax", 100.0, 25.0);
    let body = make_subform(
        &mut tree,
        "Body",
        LayoutStrategy::TopToBottom,
        Some(250.0),
        None,
        vec![f1, f2],
    );
    let root = make_subform(
        &mut tree,
        "Root",
        LayoutStrategy::TopToBottom,
        Some(300.0),
        Some(200.0),
        vec![header, body],
    );

    assert_golden("nested_subforms", &tree, root);
}

#[test]
fn golden_multipage_overflow() {
    let mut tree = FormTree::new();
    // Create many items that overflow a small page
    let items: Vec<FormNodeId> = (0..10)
        .map(|i| make_field(&mut tree, &format!("Row{i}"), 150.0, 30.0))
        .collect();
    let root = make_subform(
        &mut tree,
        "Root",
        LayoutStrategy::TopToBottom,
        Some(200.0),
        Some(100.0), // Small page: fits ~3 items
        items,
    );

    let engine = LayoutEngine::new(&tree);
    let layout = engine.layout(root).unwrap();
    let images = render_layout(&layout);

    // Should produce multiple pages
    assert!(images.len() > 1, "Expected multiple pages, got {}", images.len());

    // Compare each page against golden
    for (i, img) in images.iter().enumerate() {
        let golden_path = golden_dir().join(format!("multipage_overflow_page{i}.png"));
        if golden_path.exists() {
            let expected = image::open(&golden_path).unwrap();
            let result = compare_images(img, &expected, 0.0, 0).unwrap();
            assert!(result.passed, "Page {i} differs");
        } else {
            img.save(&golden_path).unwrap();
        }
    }
}
