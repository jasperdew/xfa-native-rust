//! Automated Visual Regression Testing (AVRT) pipeline.
//!
//! Batch-compares engine renders against Adobe Gold Master images,
//! applies per-category thresholds, and produces a conformance report.

use crate::{compare_images, GoldenTestError};
use image::DynamicImage;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// AVRT configuration with per-category thresholds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvrtConfig {
    /// Default threshold (max % differing pixels).
    pub default_threshold: f64,
    /// Default per-channel tolerance (0-255).
    pub default_channel_tolerance: u8,
    /// Per-category overrides.
    #[serde(default)]
    pub categories: BTreeMap<String, CategoryConfig>,
}

impl Default for AvrtConfig {
    fn default() -> Self {
        Self {
            default_threshold: 1.0,
            default_channel_tolerance: 5,
            categories: BTreeMap::new(),
        }
    }
}

/// Per-category threshold override.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryConfig {
    /// Max % differing pixels for this category.
    pub threshold: Option<f64>,
    /// Per-channel tolerance for this category.
    pub channel_tolerance: Option<u8>,
}

/// Result of a single form comparison.
#[derive(Debug, Serialize)]
pub struct FormResult {
    pub form_name: String,
    pub category: String,
    pub page: usize,
    pub threshold_used: f64,
    pub channel_tolerance_used: u8,
    pub total_pixels: u64,
    pub differing_pixels: u64,
    pub diff_percentage: f64,
    pub max_channel_diff: u8,
    pub passed: bool,
    pub master_path: PathBuf,
    pub actual_path: PathBuf,
    pub diff_path: Option<PathBuf>,
}

/// Overall AVRT conformance report.
#[derive(Debug, Serialize)]
pub struct AvrtReport {
    pub results: Vec<FormResult>,
    pub summary: AvrtSummary,
}

/// Summary metrics.
#[derive(Debug, Serialize)]
pub struct AvrtSummary {
    pub total_forms: usize,
    pub total_pages: usize,
    pub passed_pages: usize,
    pub failed_pages: usize,
    pub skipped_forms: usize,
    pub pass_rate: f64,
    pub per_category: BTreeMap<String, CategorySummary>,
}

/// Per-category summary.
#[derive(Debug, Serialize)]
pub struct CategorySummary {
    pub total_pages: usize,
    pub passed_pages: usize,
    pub pass_rate: f64,
    pub avg_diff_percentage: f64,
}

/// Discover forms in the gold masters directory.
///
/// Expects structure: `masters_dir/<category>/<form_name>_page<N>.png`
pub fn discover_masters(masters_dir: &Path) -> Result<Vec<(String, String, usize, PathBuf)>, GoldenTestError> {
    let mut entries = Vec::new();

    if !masters_dir.exists() {
        return Ok(entries);
    }

    let categories = std::fs::read_dir(masters_dir)?;
    for cat_entry in categories {
        let cat_entry = cat_entry?;
        let cat_path = cat_entry.path();
        if !cat_path.is_dir() {
            continue;
        }
        let category = cat_entry.file_name().to_string_lossy().to_string();

        let files = std::fs::read_dir(&cat_path)?;
        for file_entry in files {
            let file_entry = file_entry?;
            let file_path = file_entry.path();
            if file_path.extension().and_then(|e| e.to_str()) != Some("png") {
                continue;
            }

            let stem = file_path.file_stem().unwrap().to_string_lossy();
            if let Some((form_name, page_num)) = parse_master_filename(&stem) {
                entries.push((category.clone(), form_name, page_num, file_path));
            }
        }
    }

    entries.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
    Ok(entries)
}

/// Parse a master filename like `f1040_page1` into `("f1040", 1)`.
fn parse_master_filename(stem: &str) -> Option<(String, usize)> {
    let parts: Vec<&str> = stem.rsplitn(2, "_page").collect();
    if parts.len() == 2 {
        let page_num: usize = parts[0].parse().ok()?;
        Some((parts[1].to_string(), page_num))
    } else {
        // Single-page form: just the form name, page 1
        Some((stem.to_string(), 1))
    }
}

/// Run the AVRT pipeline comparing actuals against gold masters.
///
/// - `masters_dir`: directory with gold master PNGs (category subdirs)
/// - `actuals_dir`: directory with engine-rendered PNGs (same structure)
/// - `diffs_dir`: output directory for diff images (optional)
/// - `config`: threshold configuration
pub fn run_avrt(
    masters_dir: &Path,
    actuals_dir: &Path,
    diffs_dir: Option<&Path>,
    config: &AvrtConfig,
) -> Result<AvrtReport, GoldenTestError> {
    let masters = discover_masters(masters_dir)?;

    if let Some(dir) = diffs_dir {
        std::fs::create_dir_all(dir)?;
    }

    let mut results = Vec::new();
    let mut form_names = std::collections::HashSet::new();
    let mut skipped = 0usize;

    for (category, form_name, page, master_path) in &masters {
        form_names.insert(form_name.clone());

        // Find corresponding actual
        let actual_path = actuals_dir.join(category).join(master_path.file_name().unwrap());

        if !actual_path.exists() {
            skipped += 1;
            continue;
        }

        let cat_config = config.categories.get(category);
        let threshold = cat_config
            .and_then(|c| c.threshold)
            .unwrap_or(config.default_threshold);
        let channel_tolerance = cat_config
            .and_then(|c| c.channel_tolerance)
            .unwrap_or(config.default_channel_tolerance);

        let master_img = image::open(master_path)?;
        let actual_img = image::open(&actual_path)?;

        let comparison = compare_images(&actual_img, &master_img, threshold, channel_tolerance)?;

        let diff_path = if let Some(dir) = diffs_dir {
            let diff_file = dir.join(category).join(master_path.file_name().unwrap());
            if let Some(parent) = diff_file.parent() {
                std::fs::create_dir_all(parent)?;
            }
            if !comparison.passed {
                let diff_img = generate_diff_image_public(&actual_img, &master_img, channel_tolerance);
                diff_img.save(&diff_file)?;
            }
            Some(diff_file)
        } else {
            None
        };

        results.push(FormResult {
            form_name: form_name.clone(),
            category: category.clone(),
            page: *page,
            threshold_used: threshold,
            channel_tolerance_used: channel_tolerance,
            total_pixels: comparison.total_pixels,
            differing_pixels: comparison.differing_pixels,
            diff_percentage: comparison.diff_percentage,
            max_channel_diff: comparison.max_channel_diff,
            passed: comparison.passed,
            master_path: master_path.clone(),
            actual_path,
            diff_path,
        });
    }

    let summary = build_summary(&results, form_names.len(), skipped);
    Ok(AvrtReport { results, summary })
}

/// Build summary metrics from results.
fn build_summary(results: &[FormResult], total_forms: usize, skipped: usize) -> AvrtSummary {
    let total_pages = results.len();
    let passed_pages = results.iter().filter(|r| r.passed).count();
    let failed_pages = total_pages - passed_pages;
    let pass_rate = if total_pages > 0 {
        (passed_pages as f64 / total_pages as f64) * 100.0
    } else {
        0.0
    };

    let mut per_category: BTreeMap<String, (usize, usize, f64)> = BTreeMap::new();
    for r in results {
        let entry = per_category.entry(r.category.clone()).or_insert((0, 0, 0.0));
        entry.0 += 1;
        if r.passed {
            entry.1 += 1;
        }
        entry.2 += r.diff_percentage;
    }

    let per_category = per_category
        .into_iter()
        .map(|(cat, (total, passed, sum_diff))| {
            (
                cat,
                CategorySummary {
                    total_pages: total,
                    passed_pages: passed,
                    pass_rate: if total > 0 {
                        (passed as f64 / total as f64) * 100.0
                    } else {
                        0.0
                    },
                    avg_diff_percentage: if total > 0 {
                        sum_diff / total as f64
                    } else {
                        0.0
                    },
                },
            )
        })
        .collect();

    AvrtSummary {
        total_forms,
        total_pages,
        passed_pages,
        failed_pages,
        skipped_forms: skipped,
        pass_rate,
        per_category,
    }
}

/// Public diff image generator (re-exports internal logic).
fn generate_diff_image_public(
    actual: &DynamicImage,
    expected: &DynamicImage,
    channel_tolerance: u8,
) -> DynamicImage {
    use image::{GenericImageView, Rgba, RgbaImage};

    let (w, h) = actual.dimensions();
    let mut diff = RgbaImage::new(w, h);

    for y in 0..h {
        for x in 0..w {
            let Rgba(a) = actual.get_pixel(x, y);
            let Rgba(e) = expected.get_pixel(x, y);

            let differs = (0..4).any(|i| a[i].abs_diff(e[i]) > channel_tolerance);

            if differs {
                diff.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            } else {
                diff.put_pixel(x, y, Rgba([a[0] / 3, a[1] / 3, a[2] / 3, 255]));
            }
        }
    }

    DynamicImage::ImageRgba8(diff)
}

/// Generate an HTML visual diff report.
pub fn generate_html_report(report: &AvrtReport) -> String {
    let mut html = String::new();
    html.push_str(r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<title>AVRT Conformance Report</title>
<style>
body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; margin: 20px; background: #f5f5f5; }
h1 { color: #333; }
.summary { background: white; padding: 20px; border-radius: 8px; margin-bottom: 20px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }
.summary-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 15px; }
.metric { text-align: center; }
.metric-value { font-size: 2em; font-weight: bold; }
.metric-label { color: #666; font-size: 0.9em; }
.pass { color: #28a745; }
.fail { color: #dc3545; }
.category-section { margin-bottom: 20px; }
table { width: 100%; border-collapse: collapse; background: white; border-radius: 8px; overflow: hidden; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }
th { background: #333; color: white; padding: 10px; text-align: left; }
td { padding: 8px 10px; border-bottom: 1px solid #eee; }
tr.failed { background: #fff5f5; }
.comparison { display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 10px; margin: 10px 0; }
.comparison img { max-width: 100%; border: 1px solid #ddd; }
.comparison-label { text-align: center; font-weight: bold; color: #666; margin-bottom: 5px; }
details { margin: 5px 0; }
summary { cursor: pointer; }
</style>
</head>
<body>
<h1>AVRT Conformance Report</h1>
"#);

    // Summary section
    let s = &report.summary;
    html.push_str(&format!(
        r#"<div class="summary">
<h2>Summary</h2>
<div class="summary-grid">
<div class="metric">
  <div class="metric-value">{}</div>
  <div class="metric-label">Forms</div>
</div>
<div class="metric">
  <div class="metric-value">{}</div>
  <div class="metric-label">Pages Tested</div>
</div>
<div class="metric">
  <div class="metric-value {}">{}</div>
  <div class="metric-label">Passed</div>
</div>
<div class="metric">
  <div class="metric-value {}">{}</div>
  <div class="metric-label">Failed</div>
</div>
<div class="metric">
  <div class="metric-value">{}</div>
  <div class="metric-label">Skipped</div>
</div>
<div class="metric">
  <div class="metric-value {}">{:.1}%</div>
  <div class="metric-label">Pass Rate</div>
</div>
</div>
</div>
"#,
        s.total_forms,
        s.total_pages,
        if s.passed_pages == s.total_pages { "pass" } else { "" },
        s.passed_pages,
        if s.failed_pages > 0 { "fail" } else { "" },
        s.failed_pages,
        s.skipped_forms,
        if s.pass_rate >= 95.0 { "pass" } else { "fail" },
        s.pass_rate,
    ));

    // Per-category tables
    if !s.per_category.is_empty() {
        html.push_str(r#"<div class="summary"><h2>Per-Category Breakdown</h2><table>
<tr><th>Category</th><th>Pages</th><th>Passed</th><th>Pass Rate</th><th>Avg Diff %</th></tr>"#);
        for (cat, cs) in &s.per_category {
            html.push_str(&format!(
                r#"<tr><td>{cat}</td><td>{}</td><td>{}</td><td class="{}">{:.1}%</td><td>{:.3}%</td></tr>"#,
                cs.total_pages,
                cs.passed_pages,
                if cs.pass_rate >= 95.0 { "pass" } else { "fail" },
                cs.pass_rate,
                cs.avg_diff_percentage,
            ));
        }
        html.push_str("</table></div>\n");
    }

    // Detailed results (failed first)
    html.push_str(r#"<h2>Detailed Results</h2>
<table>
<tr><th>Form</th><th>Cat</th><th>Page</th><th>Status</th><th>Diff %</th><th>Threshold</th><th>Diff Pixels</th><th>Max Ch Diff</th></tr>"#);

    let mut sorted_results: Vec<&FormResult> = report.results.iter().collect();
    sorted_results.sort_by(|a, b| a.passed.cmp(&b.passed).then(b.diff_percentage.partial_cmp(&a.diff_percentage).unwrap_or(std::cmp::Ordering::Equal)));

    for r in &sorted_results {
        let status = if r.passed { "PASS" } else { "FAIL" };
        let class = if r.passed { "" } else { " class=\"failed\"" };
        html.push_str(&format!(
            r#"<tr{class}><td>{}</td><td>{}</td><td>{}</td><td class="{}">{status}</td><td>{:.3}%</td><td>{:.1}%</td><td>{}</td><td>{}</td></tr>
"#,
            r.form_name,
            r.category,
            r.page,
            if r.passed { "pass" } else { "fail" },
            r.diff_percentage,
            r.threshold_used,
            r.differing_pixels,
            r.max_channel_diff,
        ));
    }

    html.push_str("</table>\n</body>\n</html>\n");
    html
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_master_filename_with_page() {
        let (name, page) = parse_master_filename("f1040_page2").unwrap();
        assert_eq!(name, "f1040");
        assert_eq!(page, 2);
    }

    #[test]
    fn parse_master_filename_single_page() {
        let (name, page) = parse_master_filename("fw9").unwrap();
        assert_eq!(name, "fw9");
        assert_eq!(page, 1);
    }

    #[test]
    fn default_config() {
        let config = AvrtConfig::default();
        assert_eq!(config.default_threshold, 1.0);
        assert_eq!(config.default_channel_tolerance, 5);
        assert!(config.categories.is_empty());
    }

    #[test]
    fn config_serialization_roundtrip() {
        let mut config = AvrtConfig::default();
        config.categories.insert(
            "tax".to_string(),
            CategoryConfig {
                threshold: Some(0.5),
                channel_tolerance: Some(3),
            },
        );

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AvrtConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.default_threshold, 1.0);
        assert_eq!(
            deserialized.categories["tax"].threshold,
            Some(0.5)
        );
    }

    #[test]
    fn discover_masters_empty_dir() {
        let tmp = std::env::temp_dir().join("avrt_test_empty");
        let _ = std::fs::create_dir_all(&tmp);
        let masters = discover_masters(&tmp).unwrap();
        assert!(masters.is_empty());
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn discover_masters_nonexistent_dir() {
        let masters = discover_masters(Path::new("/nonexistent/path")).unwrap();
        assert!(masters.is_empty());
    }

    #[test]
    fn build_summary_computes_correctly() {
        let results = vec![
            FormResult {
                form_name: "f1040".to_string(),
                category: "tax".to_string(),
                page: 1,
                threshold_used: 1.0,
                channel_tolerance_used: 5,
                total_pixels: 10000,
                differing_pixels: 50,
                diff_percentage: 0.5,
                max_channel_diff: 10,
                passed: true,
                master_path: PathBuf::from("/m/tax/f1040_page1.png"),
                actual_path: PathBuf::from("/a/tax/f1040_page1.png"),
                diff_path: None,
            },
            FormResult {
                form_name: "f1040".to_string(),
                category: "tax".to_string(),
                page: 2,
                threshold_used: 1.0,
                channel_tolerance_used: 5,
                total_pixels: 10000,
                differing_pixels: 200,
                diff_percentage: 2.0,
                max_channel_diff: 30,
                passed: false,
                master_path: PathBuf::from("/m/tax/f1040_page2.png"),
                actual_path: PathBuf::from("/a/tax/f1040_page2.png"),
                diff_path: None,
            },
        ];

        let summary = build_summary(&results, 1, 0);
        assert_eq!(summary.total_forms, 1);
        assert_eq!(summary.total_pages, 2);
        assert_eq!(summary.passed_pages, 1);
        assert_eq!(summary.failed_pages, 1);
        assert_eq!(summary.pass_rate, 50.0);
        assert_eq!(summary.per_category["tax"].total_pages, 2);
        assert_eq!(summary.per_category["tax"].passed_pages, 1);
    }

    #[test]
    fn html_report_generation() {
        let report = AvrtReport {
            results: vec![],
            summary: AvrtSummary {
                total_forms: 0,
                total_pages: 0,
                passed_pages: 0,
                failed_pages: 0,
                skipped_forms: 0,
                pass_rate: 0.0,
                per_category: BTreeMap::new(),
            },
        };

        let html = generate_html_report(&report);
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("AVRT Conformance Report"));
    }

    #[test]
    fn run_avrt_with_matching_images() {
        let tmp = std::env::temp_dir().join("avrt_test_run");
        let masters = tmp.join("masters");
        let actuals = tmp.join("actuals");
        let diffs = tmp.join("diffs");

        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(masters.join("tax")).unwrap();
        std::fs::create_dir_all(actuals.join("tax")).unwrap();

        // Create identical test images
        let img = image::RgbaImage::from_pixel(10, 10, image::Rgba([128, 128, 128, 255]));
        let dyn_img = DynamicImage::ImageRgba8(img);
        dyn_img.save(masters.join("tax/fw9_page1.png")).unwrap();
        dyn_img.save(actuals.join("tax/fw9_page1.png")).unwrap();

        let config = AvrtConfig::default();
        let report = run_avrt(&masters, &actuals, Some(&diffs), &config).unwrap();

        assert_eq!(report.results.len(), 1);
        assert!(report.results[0].passed);
        assert_eq!(report.summary.pass_rate, 100.0);

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
