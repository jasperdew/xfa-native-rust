//! Accuracy Report — measure field conversion accuracy across the test corpus.
//!
//! Processes each PDF in the corpus through the XFA pipeline and reports
//! per-form and aggregate accuracy metrics.
//!
//! Usage:
//!   cargo run --bin accuracy-report -- --corpus corpus/ --output reports/accuracy/

use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use pdfium_ffi_bridge::pdf_reader::PdfReader;
use pdfium_ffi_bridge::xfa_extract::XfaPackets;

#[derive(Parser)]
#[command(name = "accuracy-report", about = "XFA field conversion accuracy dashboard")]
struct Cli {
    /// Path to the PDF corpus directory.
    #[arg(long, default_value = "corpus")]
    corpus: PathBuf,

    /// Output directory for reports.
    #[arg(long, default_value = "reports/accuracy")]
    output: PathBuf,

    /// Generate HTML report in addition to JSON.
    #[arg(long)]
    html: bool,

    /// Only process these PDFs (comma-separated basenames, e.g. "f1040,fw9").
    #[arg(long)]
    filter: Option<String>,
}

/// Error category for field conversion failures.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorCategory {
    /// XFA extraction failed entirely.
    ExtractionFailure,
    /// Template packet missing or unparseable.
    TemplateParsing,
    /// Datasets packet missing.
    DatasetsMissing,
    /// Data binding: field has no matching data node.
    DataBinding,
    /// Scripting: FormCalc script present but not yet executable.
    Scripting,
    /// Layout: field has unsupported layout attributes.
    Layout,
}

impl std::fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExtractionFailure => write!(f, "extraction"),
            Self::TemplateParsing => write!(f, "template"),
            Self::DatasetsMissing => write!(f, "datasets"),
            Self::DataBinding => write!(f, "data-binding"),
            Self::Scripting => write!(f, "scripting"),
            Self::Layout => write!(f, "layout"),
        }
    }
}

/// Per-field analysis result.
#[derive(Debug, Serialize, Deserialize)]
pub struct FieldAnalysis {
    pub name: String,
    pub has_data_value: bool,
    pub has_calculate_script: bool,
    pub has_validate_script: bool,
    pub has_event_script: bool,
    pub supported: bool,
    pub error_category: Option<ErrorCategory>,
}

/// Per-form accuracy result.
#[derive(Debug, Serialize, Deserialize)]
pub struct FormAccuracy {
    pub filename: String,
    pub xfa_type: String,
    pub total_fields: usize,
    pub supported_fields: usize,
    pub unsupported_fields: usize,
    pub accuracy_percent: f64,
    pub extraction_ok: bool,
    pub template_ok: bool,
    pub datasets_ok: bool,
    pub has_formcalc: bool,
    pub page_count: usize,
    pub errors_by_category: BTreeMap<String, usize>,
    pub fields: Vec<FieldAnalysis>,
}

/// Aggregate accuracy report.
#[derive(Debug, Serialize, Deserialize)]
pub struct AccuracyReport {
    pub generated_at: String,
    pub corpus_dir: String,
    pub forms: Vec<FormAccuracy>,
    pub summary: AccuracySummary,
}

/// Dashboard summary.
#[derive(Debug, Serialize, Deserialize)]
pub struct AccuracySummary {
    pub total_forms: usize,
    pub total_fields: usize,
    pub supported_fields: usize,
    pub unsupported_fields: usize,
    pub overall_accuracy: f64,
    pub extraction_success_rate: f64,
    pub forms_with_errors: usize,
    pub errors_by_category: BTreeMap<String, usize>,
    pub per_type: BTreeMap<String, TypeSummary>,
}

/// Per XFA type (static/dynamic) summary.
#[derive(Debug, Serialize, Deserialize)]
pub struct TypeSummary {
    pub forms: usize,
    pub fields: usize,
    pub supported: usize,
    pub accuracy: f64,
}

fn main() {
    let cli = Cli::parse();

    println!("XFA Accuracy Report");
    println!("===================");
    println!("Corpus: {}", cli.corpus.display());
    println!();

    let filter: Option<Vec<String>> = cli.filter.map(|f| {
        f.split(',').map(|s| s.trim().to_string()).collect()
    });

    let forms = process_corpus(&cli.corpus, filter.as_deref());

    let summary = build_summary(&forms);
    let report = AccuracyReport {
        generated_at: chrono_free_date(),
        corpus_dir: cli.corpus.display().to_string(),
        forms,
        summary,
    };

    // Print console summary
    print_summary(&report.summary);

    // Write JSON report
    fs::create_dir_all(&cli.output).expect("Failed to create output directory");
    let json_path = cli.output.join("accuracy.json");
    let json = serde_json::to_string_pretty(&report).expect("JSON serialization failed");
    fs::write(&json_path, &json).expect("Failed to write JSON report");
    println!("\nJSON report: {}", json_path.display());

    // Write timestamped copy for trending
    let trend_path = cli.output.join(format!("accuracy_{}.json", report.generated_at));
    fs::write(&trend_path, &json).ok();

    // Write HTML report
    if cli.html {
        let html_path = cli.output.join("accuracy.html");
        let html = generate_html_report(&report);
        fs::write(&html_path, html).expect("Failed to write HTML report");
        println!("HTML report: {}", html_path.display());
    }
}

fn process_corpus(corpus_dir: &Path, filter: Option<&[String]>) -> Vec<FormAccuracy> {
    let mut entries: Vec<PathBuf> = match fs::read_dir(corpus_dir) {
        Ok(dir) => dir
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("pdf"))
            })
            .collect(),
        Err(e) => {
            eprintln!("Error reading corpus directory: {e}");
            return Vec::new();
        }
    };
    entries.sort();

    if let Some(names) = filter {
        entries.retain(|p| {
            let stem = p.file_stem().unwrap_or_default().to_string_lossy();
            names.iter().any(|n| stem.contains(n.as_str()))
        });
    }

    println!("Processing {} PDFs...", entries.len());

    let mut results = Vec::with_capacity(entries.len());
    for (i, path) in entries.iter().enumerate() {
        let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        print!("  [{}/{}] {}... ", i + 1, entries.len(), filename);

        let result = analyze_pdf(path);
        println!(
            "{}/{} fields supported ({:.1}%)",
            result.supported_fields, result.total_fields, result.accuracy_percent
        );
        results.push(result);
    }

    results
}

fn analyze_pdf(path: &Path) -> FormAccuracy {
    let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();

    let bytes = match fs::read(path) {
        Ok(b) => b,
        Err(e) => {
            return form_error(&filename, ErrorCategory::ExtractionFailure, &format!("read: {e}"));
        }
    };

    let reader = match PdfReader::from_bytes(&bytes) {
        Ok(r) => r,
        Err(e) => {
            return form_error(&filename, ErrorCategory::ExtractionFailure, &format!("PDF: {e}"));
        }
    };

    let page_count = reader.page_count();

    let packets = match reader.extract_xfa() {
        Ok(p) if !p.packets.is_empty() => p,
        Ok(_) => {
            return form_error(&filename, ErrorCategory::ExtractionFailure, "no XFA packets");
        }
        Err(e) => {
            return form_error(&filename, ErrorCategory::ExtractionFailure, &format!("XFA: {e}"));
        }
    };

    let template_xml = match packets.template() {
        Some(t) => t,
        None => {
            return form_error(&filename, ErrorCategory::TemplateParsing, "no template packet");
        }
    };

    let datasets_ok = packets.datasets().is_some();

    let xfa_type = detect_xfa_type(template_xml);
    let has_formcalc = detect_formcalc(template_xml);

    let fields = analyze_fields(template_xml, &packets);

    let total_fields = fields.len();
    let supported_fields = fields.iter().filter(|f| f.supported).count();
    let unsupported_fields = total_fields - supported_fields;
    let accuracy_percent = if total_fields > 0 {
        (supported_fields as f64 / total_fields as f64) * 100.0
    } else {
        100.0 // No fields = no errors
    };

    let mut errors_by_category: BTreeMap<String, usize> = BTreeMap::new();
    for f in &fields {
        if let Some(cat) = &f.error_category {
            *errors_by_category.entry(cat.to_string()).or_insert(0) += 1;
        }
    }

    FormAccuracy {
        filename,
        xfa_type,
        total_fields,
        supported_fields,
        unsupported_fields,
        accuracy_percent,
        extraction_ok: true,
        template_ok: true,
        datasets_ok,
        has_formcalc,
        page_count,
        errors_by_category,
        fields,
    }
}

fn form_error(filename: &str, category: ErrorCategory, _detail: &str) -> FormAccuracy {
    let mut errors_by_category = BTreeMap::new();
    errors_by_category.insert(category.to_string(), 1);

    FormAccuracy {
        filename: filename.to_string(),
        xfa_type: "unknown".to_string(),
        total_fields: 0,
        supported_fields: 0,
        unsupported_fields: 0,
        accuracy_percent: 0.0,
        extraction_ok: category != ErrorCategory::ExtractionFailure,
        template_ok: category != ErrorCategory::TemplateParsing,
        datasets_ok: false,
        has_formcalc: false,
        page_count: 0,
        errors_by_category,
        fields: vec![],
    }
}

/// Analyze fields in the template XML.
///
/// Extracts `<field` elements and checks what pipeline stages can handle them.
/// Uses a depth-tracking parser to correctly handle nested elements.
fn analyze_fields(template_xml: &str, packets: &XfaPackets) -> Vec<FieldAnalysis> {
    let mut fields = Vec::new();

    let datasets_xml = packets.datasets().unwrap_or("");
    let has_datasets = !datasets_xml.is_empty();

    // Find each <field ...> opening tag and extract its content up to the
    // matching </field> (handling nesting depth).
    let mut search_pos = 0;
    while let Some(rel_start) = template_xml[search_pos..].find("<field") {
        let abs_start = search_pos + rel_start;
        let after_tag = abs_start + 6; // len("<field")

        // Ensure it's <field followed by space, >, or / (not <fields or <fieldset)
        if after_tag < template_xml.len() {
            let next_char = template_xml.as_bytes()[after_tag];
            if next_char != b' ' && next_char != b'>' && next_char != b'/' {
                search_pos = after_tag;
                continue;
            }
        }

        // Extract field name from attributes
        let name = extract_attr(&template_xml[abs_start..], "name")
            .unwrap_or_else(|| format!("unnamed_{}", fields.len()));

        // Find the end of this field element (depth-aware)
        let field_end = find_element_end_nested(&template_xml[abs_start..], "field")
            .map(|e| abs_start + e)
            .unwrap_or_else(|| {
                // Fallback: skip to the next <field or end of string
                template_xml[after_tag..]
                    .find("<field")
                    .map(|p| after_tag + p)
                    .unwrap_or(template_xml.len())
            });

        let field_content = &template_xml[abs_start..field_end];

        // Check for scripts within this field's content
        let has_calculate = field_content.contains("<calculate");
        let has_validate = field_content.contains("<validate");
        let has_event = field_content.contains("<event");

        // Check data binding
        let has_data = if has_datasets {
            datasets_xml.contains(&format!("<{name}>"))
                || datasets_xml.contains(&format!("<{name} "))
                || datasets_xml.contains(&format!("<{name}/>"))
        } else {
            false
        };

        // Determine support status
        let (supported, error_category) = assess_field_support(
            field_content,
            has_calculate,
            has_validate,
            has_event,
            has_data,
            has_datasets,
        );

        fields.push(FieldAnalysis {
            name,
            has_data_value: has_data,
            has_calculate_script: has_calculate,
            has_validate_script: has_validate,
            has_event_script: has_event,
            supported,
            error_category,
        });

        search_pos = field_end;
    }

    fields
}

/// Assess whether a field is currently supported by the engine.
fn assess_field_support(
    field_content: &str,
    has_calculate: bool,
    _has_validate: bool,
    has_event: bool,
    has_data: bool,
    has_datasets: bool,
) -> (bool, Option<ErrorCategory>) {
    // Check for missing datasets packet — fields cannot be bound without data
    if !has_datasets {
        return (false, Some(ErrorCategory::DatasetsMissing));
    }

    // Check for unsupported layout features on the field itself
    // Note: <draw> inside a field is a caption, which is supported
    let has_unsupported_layout = field_content.contains("rotate=\"")
        || field_content.contains("presence=\"inactive\"");

    if has_unsupported_layout {
        return (false, Some(ErrorCategory::Layout));
    }

    // Check scripting support
    // We support basic calculate scripts but not event scripts or complex validation
    if has_event && field_content.contains("contentType=\"application/x-javascript\"") {
        return (false, Some(ErrorCategory::Scripting));
    }

    // We support FormCalc calculate and validate scripts
    let has_js_calculate = has_calculate
        && field_content.contains("contentType=\"application/x-javascript\"");
    if has_js_calculate {
        return (false, Some(ErrorCategory::Scripting));
    }

    // Check if field has data binding — fields without data context
    // may not render correctly
    if !has_data && !has_calculate {
        return (false, Some(ErrorCategory::DataBinding));
    }

    // Basic field types we support
    (true, None)
}

/// Extract an XML attribute value.
fn extract_attr(xml: &str, attr: &str) -> Option<String> {
    let pattern = format!("{attr}=\"");
    let start = xml.find(&pattern)?;
    let value_start = start + pattern.len();
    let end = xml[value_start..].find('"')?;
    Some(xml[value_start..value_start + end].to_string())
}

/// Find the end of an XML element with nesting depth tracking.
///
/// Starting from the opening `<tag`, finds the matching `</tag>`.
fn find_element_end_nested(xml: &str, tag: &str) -> Option<usize> {
    let open_tag = format!("<{tag}");
    let close_tag = format!("</{tag}>");

    // First, find the end of the opening tag
    let gt_pos = xml.find('>')?;

    // Check for self-closing: <field ... />
    if gt_pos > 0 && xml.as_bytes()[gt_pos - 1] == b'/' {
        return Some(gt_pos + 1);
    }

    // Track depth: we start at depth 1 (inside the first <tag>)
    let mut depth = 1;
    let mut pos = gt_pos + 1;

    while pos < xml.len() && depth > 0 {
        if let Some(next_lt) = xml[pos..].find('<') {
            let abs_lt = pos + next_lt;

            if xml[abs_lt..].starts_with(&close_tag) {
                depth -= 1;
                if depth == 0 {
                    return Some(abs_lt + close_tag.len());
                }
                pos = abs_lt + close_tag.len();
            } else if xml[abs_lt..].starts_with(&open_tag) {
                // Check it's a real open tag (followed by space, >, or /)
                let after = abs_lt + open_tag.len();
                if after < xml.len() {
                    let ch = xml.as_bytes()[after];
                    if ch == b' ' || ch == b'>' || ch == b'/' {
                        // Check for self-closing nested element
                        if let Some(nested_gt) = xml[abs_lt..].find('>') {
                            let nested_gt_abs = abs_lt + nested_gt;
                            if xml.as_bytes()[nested_gt_abs - 1] == b'/' {
                                // Self-closing nested, don't change depth
                                pos = nested_gt_abs + 1;
                            } else {
                                depth += 1;
                                pos = nested_gt_abs + 1;
                            }
                        } else {
                            pos = abs_lt + 1;
                        }
                    } else {
                        pos = abs_lt + 1;
                    }
                } else {
                    pos = abs_lt + 1;
                }
            } else {
                pos = abs_lt + 1;
            }
        } else {
            break;
        }
    }

    None
}

fn detect_xfa_type(template_xml: &str) -> String {
    if template_xml.contains("layout=\"tb\"")
        || template_xml.contains("layout=\"lr-tb\"")
        || template_xml.contains("layout=\"rl-tb\"")
    {
        "dynamic".to_string()
    } else {
        "static".to_string()
    }
}

fn detect_formcalc(template_xml: &str) -> bool {
    template_xml.contains("<script")
        || template_xml.contains("<calculate")
        || template_xml.contains("contentType=\"application/x-formcalc\"")
}

fn build_summary(forms: &[FormAccuracy]) -> AccuracySummary {
    let total_forms = forms.len();
    let total_fields: usize = forms.iter().map(|f| f.total_fields).sum();
    let supported_fields: usize = forms.iter().map(|f| f.supported_fields).sum();
    let unsupported_fields = total_fields - supported_fields;
    let overall_accuracy = if total_fields > 0 {
        (supported_fields as f64 / total_fields as f64) * 100.0
    } else {
        0.0
    };

    let extraction_ok_count = forms.iter().filter(|f| f.extraction_ok).count();
    let extraction_success_rate = if total_forms > 0 {
        (extraction_ok_count as f64 / total_forms as f64) * 100.0
    } else {
        0.0
    };

    let forms_with_errors = forms.iter().filter(|f| f.accuracy_percent < 100.0).count();

    let mut errors_by_category: BTreeMap<String, usize> = BTreeMap::new();
    for form in forms {
        for (cat, count) in &form.errors_by_category {
            *errors_by_category.entry(cat.clone()).or_insert(0) += count;
        }
    }

    let mut per_type: BTreeMap<String, TypeSummary> = BTreeMap::new();
    for form in forms {
        let entry = per_type.entry(form.xfa_type.clone()).or_insert(TypeSummary {
            forms: 0,
            fields: 0,
            supported: 0,
            accuracy: 0.0,
        });
        entry.forms += 1;
        entry.fields += form.total_fields;
        entry.supported += form.supported_fields;
    }
    for ts in per_type.values_mut() {
        ts.accuracy = if ts.fields > 0 {
            (ts.supported as f64 / ts.fields as f64) * 100.0
        } else {
            0.0
        };
    }

    AccuracySummary {
        total_forms,
        total_fields,
        supported_fields,
        unsupported_fields,
        overall_accuracy,
        extraction_success_rate,
        forms_with_errors,
        errors_by_category,
        per_type,
    }
}

fn print_summary(s: &AccuracySummary) {
    println!();
    println!("=== Accuracy Dashboard ===");
    println!("  Total forms:          {}", s.total_forms);
    println!("  Total fields:         {}", s.total_fields);
    println!("  Supported fields:     {}", s.supported_fields);
    println!("  Unsupported fields:   {}", s.unsupported_fields);
    println!("  Overall accuracy:     {:.1}%", s.overall_accuracy);
    println!("  Extraction success:   {:.1}%", s.extraction_success_rate);
    println!("  Forms with errors:    {}", s.forms_with_errors);

    if !s.errors_by_category.is_empty() {
        println!();
        println!("  Errors by category:");
        for (cat, count) in &s.errors_by_category {
            println!("    {cat}: {count}");
        }
    }

    if !s.per_type.is_empty() {
        println!();
        println!("  Per XFA type:");
        for (xfa_type, ts) in &s.per_type {
            println!(
                "    {xfa_type}: {} forms, {}/{} fields ({:.1}%)",
                ts.forms, ts.supported, ts.fields, ts.accuracy
            );
        }
    }
}

fn generate_html_report(report: &AccuracyReport) -> String {
    let s = &report.summary;
    let mut html = String::new();

    html.push_str(&format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<title>XFA Accuracy Dashboard</title>
<style>
body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; margin: 20px; background: #f5f5f5; }}
h1 {{ color: #333; }}
.dashboard {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 15px; margin-bottom: 20px; }}
.card {{ background: white; padding: 20px; border-radius: 8px; text-align: center; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }}
.card-value {{ font-size: 2.5em; font-weight: bold; }}
.card-label {{ color: #666; font-size: 0.9em; margin-top: 5px; }}
.pass {{ color: #28a745; }}
.warn {{ color: #ffc107; }}
.fail {{ color: #dc3545; }}
.section {{ background: white; padding: 20px; border-radius: 8px; margin-bottom: 20px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }}
table {{ width: 100%; border-collapse: collapse; }}
th {{ background: #333; color: white; padding: 10px; text-align: left; }}
td {{ padding: 8px 10px; border-bottom: 1px solid #eee; }}
tr.low {{ background: #fff5f5; }}
.bar {{ height: 20px; border-radius: 3px; background: #e0e0e0; overflow: hidden; }}
.bar-fill {{ height: 100%; border-radius: 3px; }}
.target-line {{ border-left: 2px dashed #dc3545; position: absolute; }}
.generated {{ color: #999; font-size: 0.8em; margin-top: 20px; }}
</style>
</head>
<body>
<h1>XFA Accuracy Dashboard</h1>
<p class="generated">Generated: {} | Corpus: {}</p>
"#,
        report.generated_at, report.corpus_dir
    ));

    // Dashboard cards
    let accuracy_class = if s.overall_accuracy >= 95.0 {
        "pass"
    } else if s.overall_accuracy >= 90.0 {
        "warn"
    } else {
        "fail"
    };

    html.push_str(&format!(
        r#"<div class="dashboard">
<div class="card"><div class="card-value {accuracy_class}">{:.1}%</div><div class="card-label">Overall Accuracy</div></div>
<div class="card"><div class="card-value">{}</div><div class="card-label">Total Forms</div></div>
<div class="card"><div class="card-value">{}</div><div class="card-label">Total Fields</div></div>
<div class="card"><div class="card-value pass">{}</div><div class="card-label">Supported</div></div>
<div class="card"><div class="card-value fail">{}</div><div class="card-label">Unsupported</div></div>
<div class="card"><div class="card-value">{:.1}%</div><div class="card-label">Extraction Rate</div></div>
</div>
"#,
        s.overall_accuracy,
        s.total_forms,
        s.total_fields,
        s.supported_fields,
        s.unsupported_fields,
        s.extraction_success_rate,
    ));

    // Target indicator
    html.push_str(r#"<div class="section">
<h2>Target Progress</h2>
<p>Phase 1 target: <strong>90%</strong> | Phase 2 target: <strong>95%+</strong></p>
<div class="bar" style="position:relative; height:30px;">
"#);
    let fill_color = if s.overall_accuracy >= 95.0 {
        "#28a745"
    } else if s.overall_accuracy >= 90.0 {
        "#ffc107"
    } else {
        "#dc3545"
    };
    html.push_str(&format!(
        r#"<div class="bar-fill" style="width:{:.1}%; background:{fill_color};"></div>
</div></div>
"#,
        s.overall_accuracy.min(100.0)
    ));

    // Error breakdown
    if !s.errors_by_category.is_empty() {
        html.push_str(r#"<div class="section"><h2>Errors by Category</h2><table>
<tr><th>Category</th><th>Count</th><th>% of Unsupported</th></tr>"#);
        for (cat, count) in &s.errors_by_category {
            let pct = if s.unsupported_fields > 0 {
                (*count as f64 / s.unsupported_fields as f64) * 100.0
            } else {
                0.0
            };
            html.push_str(&format!(
                "<tr><td>{cat}</td><td>{count}</td><td>{pct:.1}%</td></tr>"
            ));
        }
        html.push_str("</table></div>\n");
    }

    // Per-form table
    html.push_str(r#"<div class="section"><h2>Per-Form Results</h2><table>
<tr><th>Form</th><th>Type</th><th>Fields</th><th>Supported</th><th>Accuracy</th><th>Pages</th><th>FormCalc</th></tr>"#);

    let mut sorted_forms: Vec<&FormAccuracy> = report.forms.iter().collect();
    sorted_forms.sort_by(|a, b| {
        a.accuracy_percent
            .partial_cmp(&b.accuracy_percent)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    for form in &sorted_forms {
        let row_class = if form.accuracy_percent < 90.0 { " class=\"low\"" } else { "" };
        let acc_class = if form.accuracy_percent >= 95.0 {
            "pass"
        } else if form.accuracy_percent >= 90.0 {
            "warn"
        } else {
            "fail"
        };
        let formcalc = if form.has_formcalc { "yes" } else { "-" };
        html.push_str(&format!(
            r#"<tr{row_class}><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td class="{acc_class}">{:.1}%</td><td>{}</td><td>{formcalc}</td></tr>
"#,
            form.filename,
            form.xfa_type,
            form.total_fields,
            form.supported_fields,
            form.accuracy_percent,
            form.page_count,
        ));
    }

    html.push_str("</table></div>\n</body>\n</html>\n");
    html
}

fn chrono_free_date() -> String {
    std::process::Command::new("date")
        .args(["+%Y-%m-%d"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}
