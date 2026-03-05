//! Integration tests for the AVRT pipeline.

use image::{DynamicImage, Rgba, RgbaImage};
use std::path::PathBuf;
use xfa_golden_tests::avrt::{
    discover_masters, generate_html_report, run_avrt, AvrtConfig, CategoryConfig,
};

fn unique_test_dir(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("avrt_test_{name}_{}", std::process::id()))
}

fn save_test_image(path: &std::path::Path, color: Rgba<u8>) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    let img = RgbaImage::from_pixel(50, 50, color);
    DynamicImage::ImageRgba8(img).save(path).unwrap();
}

#[test]
fn avrt_discovers_masters_by_category() {
    let base = unique_test_dir("discover");
    let _ = std::fs::remove_dir_all(&base);
    let masters = base.join("masters");

    save_test_image(
        &masters.join("tax-individual/f1040_page1.png"),
        Rgba([255, 255, 255, 255]),
    );
    save_test_image(
        &masters.join("tax-individual/f1040_page2.png"),
        Rgba([255, 255, 255, 255]),
    );
    save_test_image(
        &masters.join("immigration/i130_page1.png"),
        Rgba([255, 255, 255, 255]),
    );

    let entries = discover_masters(&masters).unwrap();
    assert_eq!(entries.len(), 3);

    assert_eq!(entries[0].0, "immigration");
    assert_eq!(entries[0].1, "i130");
    assert_eq!(entries[1].0, "tax-individual");
    assert_eq!(entries[1].1, "f1040");
    assert_eq!(entries[1].2, 1);
    assert_eq!(entries[2].2, 2);

    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn avrt_all_pass_identical_images() {
    let base = unique_test_dir("allpass");
    let _ = std::fs::remove_dir_all(&base);
    let masters = base.join("masters");
    let actuals = base.join("actuals");
    let diffs = base.join("diffs");

    let white = Rgba([255, 255, 255, 255]);
    save_test_image(&masters.join("tax/fw9_page1.png"), white);
    save_test_image(&actuals.join("tax/fw9_page1.png"), white);

    let config = AvrtConfig::default();
    let report = run_avrt(&masters, &actuals, Some(&diffs), &config).unwrap();

    assert_eq!(report.results.len(), 1);
    assert!(report.results[0].passed);
    assert_eq!(report.summary.total_pages, 1);
    assert_eq!(report.summary.passed_pages, 1);
    assert_eq!(report.summary.failed_pages, 0);
    assert_eq!(report.summary.pass_rate, 100.0);

    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn avrt_detects_pixel_differences() {
    let base = unique_test_dir("pixeldiff");
    let _ = std::fs::remove_dir_all(&base);
    let masters = base.join("masters");
    let actuals = base.join("actuals");
    let diffs = base.join("diffs");

    save_test_image(
        &masters.join("tax/form_page1.png"),
        Rgba([255, 255, 255, 255]),
    );
    save_test_image(
        &actuals.join("tax/form_page1.png"),
        Rgba([200, 200, 200, 255]),
    );

    let config = AvrtConfig {
        default_threshold: 0.0,
        default_channel_tolerance: 0,
        categories: Default::default(),
    };
    let report = run_avrt(&masters, &actuals, Some(&diffs), &config).unwrap();

    assert_eq!(report.results.len(), 1);
    assert!(!report.results[0].passed);
    assert_eq!(report.summary.failed_pages, 1);

    let diff_file = diffs.join("tax/form_page1.png");
    assert!(diff_file.exists(), "Diff image should be generated");

    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn avrt_per_category_threshold() {
    let base = unique_test_dir("catthreshold");
    let _ = std::fs::remove_dir_all(&base);
    let masters = base.join("masters");
    let actuals = base.join("actuals");
    let diffs = base.join("diffs");

    save_test_image(
        &masters.join("strict/form_page1.png"),
        Rgba([255, 255, 255, 255]),
    );
    save_test_image(
        &actuals.join("strict/form_page1.png"),
        Rgba([250, 250, 250, 255]),
    );
    save_test_image(
        &masters.join("relaxed/form_page1.png"),
        Rgba([255, 255, 255, 255]),
    );
    save_test_image(
        &actuals.join("relaxed/form_page1.png"),
        Rgba([250, 250, 250, 255]),
    );

    let mut categories = std::collections::BTreeMap::new();
    categories.insert(
        "strict".to_string(),
        CategoryConfig {
            threshold: Some(0.0),
            channel_tolerance: Some(0),
        },
    );
    categories.insert(
        "relaxed".to_string(),
        CategoryConfig {
            threshold: Some(100.0),
            channel_tolerance: Some(10),
        },
    );

    let config = AvrtConfig {
        default_threshold: 1.0,
        default_channel_tolerance: 5,
        categories,
    };
    let report = run_avrt(&masters, &actuals, Some(&diffs), &config).unwrap();

    assert_eq!(report.results.len(), 2);

    let strict_result = report.results.iter().find(|r| r.category == "strict").unwrap();
    let relaxed_result = report.results.iter().find(|r| r.category == "relaxed").unwrap();

    assert!(!strict_result.passed, "Strict should fail");
    assert!(relaxed_result.passed, "Relaxed should pass");

    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn avrt_skips_missing_actuals() {
    let base = unique_test_dir("skipmissing");
    let _ = std::fs::remove_dir_all(&base);
    let masters = base.join("masters");
    let actuals = base.join("actuals");

    save_test_image(
        &masters.join("tax/form_page1.png"),
        Rgba([255, 255, 255, 255]),
    );
    std::fs::create_dir_all(&actuals).unwrap();

    let config = AvrtConfig::default();
    let report = run_avrt(&masters, &actuals, None, &config).unwrap();

    assert_eq!(report.results.len(), 0);
    assert_eq!(report.summary.skipped_forms, 1);

    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn avrt_html_report_contains_results() {
    let base = unique_test_dir("htmlreport");
    let _ = std::fs::remove_dir_all(&base);
    let masters = base.join("masters");
    let actuals = base.join("actuals");
    let diffs = base.join("diffs");

    save_test_image(
        &masters.join("tax/fw9_page1.png"),
        Rgba([255, 255, 255, 255]),
    );
    save_test_image(
        &actuals.join("tax/fw9_page1.png"),
        Rgba([255, 255, 255, 255]),
    );

    let config = AvrtConfig::default();
    let report = run_avrt(&masters, &actuals, Some(&diffs), &config).unwrap();
    let html = generate_html_report(&report);

    assert!(html.contains("AVRT Conformance Report"));
    assert!(html.contains("fw9"));
    assert!(html.contains("PASS"));
    assert!(html.contains("100.0%"));

    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn avrt_json_report_serializable() {
    let base = unique_test_dir("jsonreport");
    let _ = std::fs::remove_dir_all(&base);
    let masters = base.join("masters");
    let actuals = base.join("actuals");

    save_test_image(
        &masters.join("tax/fw9_page1.png"),
        Rgba([255, 255, 255, 255]),
    );
    save_test_image(
        &actuals.join("tax/fw9_page1.png"),
        Rgba([255, 255, 255, 255]),
    );

    let config = AvrtConfig::default();
    let report = run_avrt(&masters, &actuals, None, &config).unwrap();

    let json = serde_json::to_string_pretty(&report).unwrap();
    assert!(json.contains("\"pass_rate\""));
    assert!(json.contains("\"fw9\""));

    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn avrt_multi_page_form() {
    let base = unique_test_dir("multipage");
    let _ = std::fs::remove_dir_all(&base);
    let masters = base.join("masters");
    let actuals = base.join("actuals");
    let diffs = base.join("diffs");

    let white = Rgba([255, 255, 255, 255]);
    for page in 1..=4 {
        save_test_image(
            &masters.join(format!("tax/f1040_page{page}.png")),
            white,
        );
        save_test_image(
            &actuals.join(format!("tax/f1040_page{page}.png")),
            white,
        );
    }

    let config = AvrtConfig::default();
    let report = run_avrt(&masters, &actuals, Some(&diffs), &config).unwrap();

    assert_eq!(report.results.len(), 4);
    assert!(report.results.iter().all(|r| r.passed));
    assert_eq!(report.summary.total_pages, 4);
    assert_eq!(report.summary.pass_rate, 100.0);

    let _ = std::fs::remove_dir_all(&base);
}
