# Test Expansion Plan — SDK Completeness & Desktop App

> Uitbreiding van de test-infrastructuur ter ondersteuning van de volledige SDK
> (write APIs, content engine, compliance conversie, bindings) en de Tauri desktop app.
>
> Context: huidige setup dekt parsing, rendering, text extraction en compliance
> validatie via corpus runner (50K+ PDFs) en veraPDF oracle. Issues #349–#359
> verbeteren die bestaande runner. Dit plan beschrijft **wat er nog ontbreekt**.

---

## Huidige dekking vs. gaten

| Gebied | Huidige dekking | Gat |
|--------|----------------|-----|
| PDF parsing | Corpus (50K), fuzz (8 targets) | — |
| Rendering | Corpus render test + SSIM oracle | Geen pixel-baseline voor specifieke constructen |
| Text extraction | Corpus + Poppler oracle | — |
| Compliance validatie | Corpus + veraPDF oracle | — |
| AcroForm lezen | form_fields test (minimaal) | Geen interactie (calculate, validate scripts) |
| **AcroForm schrijven** | — | Volledig ontbrekend |
| **Annotatie creatie** | — | Volledig ontbrekend |
| **PDF signing** | Detectie alleen | Geen sign + verify roundtrip |
| **Content editor** | Unit tests in pdf-manip | Geen corpus-level test |
| **Text replacement** | Unit tests | Geen corpus-level test |
| **Search-and-redact** | Unit tests | Geen corpus-level verificatie |
| **PDF/A conversie** | — | Geen convert → validate roundtrip |
| **DOCX/XLSX/PPTX** | — | Volledig ontbrekend |
| **Bindings (C/Py/Node/WASM)** | — | Geen geautomatiseerde tests |
| **Desktop app** | — | Geen UI/integratie tests |
| **Performance regressie** | Criterion bestaat, geen baseline | Geen CI vergelijking |

---

## 1. Corpus Runner Uitbreidingen (xfa-test-runner)

### 1.1 Nieuwe tests toevoegen aan de test matrix

De huidige matrix heeft 15 tests. Hieronder 8 nieuwe tests die de SDK write-functionaliteit
en content intelligence dekken, zonder de bestaande runner-architectuur te wijzigen.

| Test | Tier | Wat het test | Pass-criterium |
|------|------|-------------|----------------|
| `form_write` | standard | Vul eerste text field, save, heropen, lees waarde | Waarde roundtripped |
| `annot_create` | standard | Voeg highlight annotatie toe op pagina 1, save, heropen | Annotatie bestaat in output |
| `content_roundtrip` | standard | Decode content stream → encode → decode, vergelijk ops | Operaties identiek |
| `text_replace` | full | Zoek eerste woord op pagina 1, vervang, extraheer tekst | Vervangen woord aanwezig |
| `redact` | full | Redact eerste woord pagina 1, extraheer tekst | Woord niet meer aanwezig |
| `pdfa_convert` | full | Converteer naar PDF/A-2b, valideer met veraPDF | 0 compliance failures |
| `image_extract_verify` | full | Extraheer images, check dimensies > 0 en bytes > 0 | Alle images valide |
| `sign_roundtrip` | full | Sign met test cert, verify signature | Signature valid |

**Implementatie-impact:** Elke test is een nieuwe `impl PdfTest` in `tests/`. De runner, database en reporting hoeven niet te veranderen — de bestaande `TestRegistry` pikt ze automatisch op.

**Skip-logica:** Tests skippen wanneer de feature niet van toepassing is (bijv. `form_write` skipt als er geen AcroForm fields zijn). Dit houdt de applicable pass rate zuiver.

### 1.2 Write-operatie validatie: roundtrip patroon

Alle write-tests volgen hetzelfde patroon:

```
Input PDF → mutatie (lopdf) → save naar tempfile → heropen → assertions
```

Dit garandeert dat:
1. De mutatie geen corrupt PDF oplevert (heropen slaagt)
2. De mutatie het gewenste effect heeft (assertion op resultaat)
3. Bestaande content niet beschadigd wordt (parse test slaagt nog)

### 1.3 PDF/A conversie roundtrip

Dit is de hoogst-waardevolle nieuwe test:

```
Input PDF
  → detect_pdfa_level() (of forceer A-2b)
  → pdfa_convert pipeline (xmp, fonts, colorspace, cleanup)
  → save naar tempfile
  → validate_pdfa() met onze eigen checker
  → validate met veraPDF oracle (als beschikbaar)
  → vergelijk resultaten
```

**Metric:** percentage PDFs dat na conversie door veraPDF komt. Dit is een
directe kwaliteitsmeting van onze conversie-pipeline.

### 1.4 Content engine corpus test

De content editor, text runs en text replacement worden nu alleen met unit tests
gedekt (synthetische PDFs). Een corpus-level test vangt edge cases:

```
Voor elke PDF in corpus:
  1. content_roundtrip: decode → encode → decode, vergelijk operaties
  2. Als tekst gevonden: text_replace eerste woord, verify in output
  3. Als tekst gevonden: redact eerste woord, verify afwezig in output
```

**Verwachte skip-rate:** ~30% (PDFs zonder tekst of met niet-decodeerbare streams).

---

## 2. Document Conversie Tests (DOCX/XLSX/PPTX)

### 2.1 Roundtrip kwaliteitstest

Document conversies hebben geen oracle (geen referentie-implementatie om tegen te
vergelijken). In plaats daarvan gebruiken we structurele validatie:

| Conversie | Validatie |
|-----------|-----------|
| PDF → DOCX | ZIP valid, `word/document.xml` parseable, tekst aanwezig, images embedded |
| PDF → XLSX | ZIP valid, `xl/sharedStrings.xml` parseable, minstens 1 cell met data |
| PDF → PPTX | ZIP valid, `ppt/slides/slide1.xml` parseable, content aanwezig |

### 2.2 Tekst-behoud metric

```
Input PDF → extract text → convert to DOCX → extract text from DOCX
→ Levenshtein similarity score
```

**Drempel:** ≥ 0.80 similarity voor PDFs met ≥ 100 karakters tekst.
Dit wordt een nieuwe metric in de runner database: `conversion_text_similarity`.

### 2.3 Corpus subset

Niet alle 50K PDFs hoeven geconverteerd te worden. Een representatieve subset:
- 500 PDFs met tekst (willekeurig)
- 100 PDFs met tabellen (gedetecteerd via tabel-heuristiek in pdf-extract)
- 50 PDFs met images

Selectie via `corpus-categorize.py` uitbreiden met nieuwe tags.

---

## 3. Binding Tests

### 3.1 Architectuur: per-binding test suite

Elke binding krijgt een eigen test suite in de taal van de binding:

```
crates/pdf-capi/tests/         → C tests (compile + run via cc crate of Makefile)
crates/pdf-python/tests/       → pytest
crates/pdf-node/tests/         → Jest/Vitest
crates/xfa-wasm/tests/         → wasm-pack test (headless browser)
```

### 3.2 Gemeenschappelijke test-scenario's

Elke binding test suite dekt dezelfde 12 kernscenario's:

| # | Scenario | Valideert |
|---|----------|-----------|
| 1 | Open PDF, tel pagina's | Basic document loading |
| 2 | Render pagina 1 naar PNG | Rendering pipeline |
| 3 | Extraheer tekst pagina 1 | Text extraction |
| 4 | Lees metadata (titel, auteur) | Metadata API |
| 5 | Lees AcroForm fields | Forms API |
| 6 | Vul text field, save | Forms write API |
| 7 | Lees annotaties | Annotations API |
| 8 | Voeg highlight toe, save | Annotations write API |
| 9 | Valideer PDF/A | Compliance API |
| 10 | Merge 2 PDFs | Manipulation API |
| 11 | Verifieer handtekening | Signatures API |
| 12 | Extraheer images | Image extraction API |

### 3.3 C API (pdf-capi) — smoke tests

```c
// test_open.c
#include "xfa_pdf.h"
int main() {
    XfaDocument* doc = xfa_open_file("test.pdf");
    assert(doc != NULL);
    assert(xfa_page_count(doc) > 0);
    xfa_close(doc);
    return 0;
}
```

Gebouwd en gerund via `build.rs` of een `Makefile` in de test directory.
CI draait deze na `cargo build --release -p pdf-capi`.

### 3.4 Python (pdf-python) — pytest

```python
# tests/test_basic.py
import xfa_pdf

def test_open():
    doc = xfa_pdf.open("fixtures/sample.pdf")
    assert doc.page_count > 0

def test_render():
    doc = xfa_pdf.open("fixtures/sample.pdf")
    png = doc.render_page(0, dpi=72)
    assert len(png) > 0
```

CI: `maturin develop && pytest tests/`

### 3.5 Node.js (pdf-node) — Jest

```javascript
// tests/basic.test.js
const { openPdf } = require('xfa-pdf-node');

test('open PDF and count pages', () => {
  const doc = openPdf('fixtures/sample.pdf');
  expect(doc.pageCount).toBeGreaterThan(0);
});
```

CI: `npm run build && npm test`

### 3.6 WASM (xfa-wasm) — wasm-pack test

```rust
// tests/web.rs
#[wasm_bindgen_test]
fn test_parse_pdf() {
    let bytes = include_bytes!("fixtures/sample.pdf");
    let doc = XfaDocument::from_bytes(bytes).unwrap();
    assert!(doc.page_count() > 0);
}
```

CI: `wasm-pack test --headless --chrome`

### 3.7 Geplande bindings (Java, .NET, Swift, Kotlin)

Zodra #335–#338 geïmplementeerd zijn, volgen dezelfde 12 scenario's:

| Binding | Test framework | Build |
|---------|---------------|-------|
| Java | JUnit 5 | Gradle + JNI load |
| .NET | xUnit | dotnet test + P/Invoke |
| Swift | XCTest | xcodebuild + C API |
| Kotlin | JUnit 5 | Gradle + JNI (Android) |

---

## 4. Desktop App Tests (pdf-desktop)

### 4.1 Backend unit tests (Rust)

De Tauri backend commands zijn gewone Rust functies. Test deze los van de UI:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmd_open_pdf() {
        let result = open_pdf_command("fixtures/sample.pdf".into());
        assert!(result.is_ok());
        let info = result.unwrap();
        assert!(info.page_count > 0);
    }

    #[test]
    fn cmd_render_page() {
        let doc = open_pdf_command("fixtures/sample.pdf".into()).unwrap();
        let png = render_page_command(doc.id, 0, 72).unwrap();
        assert!(!png.is_empty());
    }

    #[test]
    fn cmd_get_thumbnails() {
        let doc = open_pdf_command("fixtures/sample.pdf".into()).unwrap();
        let thumbs = get_thumbnails_command(doc.id, 48).unwrap();
        assert_eq!(thumbs.len(), doc.page_count);
    }
}
```

**Dekking:** Alle Tauri `#[tauri::command]` functies worden unit-getest.

### 4.2 Frontend component tests

Voor de WebView frontend (React/Svelte):

```
pdf-desktop/
  src-tauri/        ← Rust backend (unit tests via cargo test)
  src/              ← Frontend
    components/
      __tests__/    ← Component tests (Vitest + Testing Library)
        Viewport.test.tsx
        Sidebar.test.tsx
        Toolbar.test.tsx
```

| Component | Test |
|-----------|------|
| Viewport | Rendert canvas, reageert op zoom/scroll |
| ThumbnailSidebar | Toont thumbnails, click navigeert |
| AnnotationToolbar | Toolbar toggle, tool selectie |
| FileManager | Open/save/recent files |
| PrintDialog | Print configuratie |

### 4.3 Integration tests (Tauri test driver)

Tauri v2 ondersteunt `tauri-driver` voor headless testing:

```rust
// tests/integration.rs
use tauri_test::{mock_builder, MockRuntime};

#[test]
fn open_pdf_renders_first_page() {
    let app = mock_builder().build().unwrap();
    let window = app.get_window("main").unwrap();
    // Invoke command
    let result: serde_json::Value = window.invoke("open_pdf", json!({"path": "test.pdf"}));
    assert!(result["pageCount"].as_u64().unwrap() > 0);
}
```

### 4.4 Screenshot regression tests

Voor visuele stabiliteit van de desktop app:

```
1. Start app headless (Tauri test mode)
2. Open referentie-PDF
3. Screenshot viewport
4. Vergelijk met golden screenshot (SSIM ≥ 0.98)
5. Fail bij visuele regressie
```

**Goldens:** Opgeslagen in `pdf-desktop/golden/` per platform (macOS, Linux, Windows).

### 4.5 Undo/Redo tests (#333)

```
Open PDF → add annotation → undo → verify gone → redo → verify present
Open PDF → fill field → save → undo → verify unsaved state
```

### 4.6 Keyboard shortcut tests (#334)

Tabel-driven tests voor alle shortcuts:

```rust
#[test]
fn keyboard_shortcuts() {
    let cases = vec![
        ("Ctrl+O", Action::OpenFile),
        ("Ctrl+S", Action::Save),
        ("Ctrl+Z", Action::Undo),
        ("Ctrl+Shift+Z", Action::Redo),
        ("Ctrl+P", Action::Print),
        ("+", Action::ZoomIn),
        ("-", Action::ZoomOut),
    ];
    for (key, expected_action) in cases {
        assert_eq!(parse_shortcut(key).action(), expected_action);
    }
}
```

---

## 5. Performance Regressie Detectie

### 5.1 Benchmark baseline in CI

Huidige Criterion benchmarks bestaan maar worden niet vergeleken.

**Aanpak:**
1. `cargo bench` resultaten opslaan als JSON (`--message-format=json`)
2. Vergelijk met baseline uit `main` branch
3. Fail bij >10% regressie in kritieke paden

**Kritieke benchmarks:**

| Benchmark | Drempel | Meetpunt |
|-----------|---------|----------|
| `pdf_parse` (1MB PDF) | ≤ 5ms | Parsing latency |
| `render_page` (letter-size) | ≤ 200ms | Rendering throughput |
| `text_extract` (10 pagina's) | ≤ 50ms | Extraction speed |
| `compliance_check` (PDF/A) | ≤ 500ms | Compliance overhead |
| `pdfa_convert` | ≤ 2s | Conversion pipeline |

### 5.2 Memory profiling

Nieuwe benchmark: piek-RSS meting voor grote PDFs.

```rust
#[bench]
fn memory_large_pdf(b: &mut Bencher) {
    let data = std::fs::read("fixtures/large-100-pages.pdf").unwrap();
    b.iter(|| {
        let pdf = Pdf::new(data.clone()).unwrap();
        // Access all pages to trigger lazy loading
        for page in pdf.pages().iter() {
            let _ = page.page_stream();
        }
    });
}
```

Drempel: ≤ 3× bestandsgrootte (bijv. 10 MB PDF → ≤ 30 MB RSS).

---

## 6. Signing Roundtrip Tests

### 6.1 Test-certificaat generatie

Eenmalig: genereer een self-signed test certificate voor CI:

```bash
openssl req -x509 -newkey rsa:2048 -keyout test-key.pem -out test-cert.pem \
  -days 3650 -nodes -subj "/CN=XFA Test"
openssl pkcs12 -export -out test.p12 -inkey test-key.pem -in test-cert.pem -password pass:test
```

Opslaan in `fixtures/certs/test.p12`.

### 6.2 Sign → verify roundtrip

```rust
#[test]
fn sign_and_verify_roundtrip() {
    let mut doc = lopdf::Document::load("fixtures/sample.pdf").unwrap();
    let cert = load_test_cert("fixtures/certs/test.p12", "test");

    // Sign
    let signed = pdf_sign::sign(&mut doc, &cert, SignLevel::PadesB).unwrap();

    // Verify
    let pdf = Pdf::new(signed).unwrap();
    let sigs = pdf_sign::extract_signatures(&pdf);
    assert_eq!(sigs.len(), 1);
    assert!(sigs[0].is_valid());
}
```

### 6.3 Corpus signing test

In de corpus runner: neem 100 willekeurige PDFs, sign ze, verify, check dat de
rest van de PDF intact is (parse, text, metadata).

---

## 7. Fuzz Target Uitbreidingen

### 7.1 Nieuwe fuzz targets

| Target | Input | Fuzz wat |
|--------|-------|----------|
| `fuzz_content_editor` | Raw content stream bytes | ContentEditor::from_stream() roundtrip |
| `fuzz_text_replace` | PDF bytes + zoekterm | replace_text() crash-freedom |
| `fuzz_redact` | PDF bytes + pattern | search_and_redact() crash-freedom |
| `fuzz_pdfa_convert` | PDF bytes | pdfa_convert pipeline crash-freedom |
| `fuzz_lopdf_roundtrip` | PDF bytes | lopdf load → save → load roundtrip |
| `fuzz_form_fill` | PDF bytes + field values | form value persistence |
| `fuzz_annot_create` | PDF bytes + annot params | annotation creation |
| `fuzz_sign` | PDF bytes | signing pipeline |

### 7.2 Corpus seeding

Gebruik de 7 bekende failure-PDFs uit run3 plus 50 diverse PDFs uit het corpus als seeds.

---

## 8. CI Pipeline Integratie

### 8.1 Huidige CI

```
cargo fmt --check
cargo clippy -- -D warnings
cargo test --workspace
```

### 8.2 Uitgebreide CI matrix

```yaml
jobs:
  lint:
    - cargo fmt --check
    - cargo clippy --workspace -- -D warnings

  unit-tests:
    - cargo test --workspace --features image-insert,font-subset

  binding-tests:
    - cargo build --release -p pdf-capi
    - cd crates/pdf-capi && make test      # C smoke tests
    - cd crates/pdf-python && maturin develop && pytest
    - cd crates/pdf-node && npm run build && npm test
    - cd crates/xfa-wasm && wasm-pack test --headless --chrome

  desktop-tests:
    - cd crates/pdf-desktop && cargo test  # Backend unit tests
    - cd crates/pdf-desktop && npm test    # Frontend component tests

  benchmarks:
    - cargo bench --package pdf-bench -- --output-format=json > bench.json
    - python scripts/bench-compare.py baseline.json bench.json --threshold 10

  fuzz-smoke:
    - cargo +nightly fuzz run fuzz_pdf_parser -- -max_total_time=60
    - cargo +nightly fuzz run fuzz_content_editor -- -max_total_time=60
    - cargo +nightly fuzz run fuzz_lopdf_roundtrip -- -max_total_time=60
```

### 8.3 VPS nightly run

De VPS corpus run wordt een nightly job:

```bash
# Nightly: full corpus, all tests, all oracles
xfa-test-runner run \
  --corpus /opt/xfa-corpus/general \
  --db /opt/xfa-results/nightly-$(date +%Y%m%d).sqlite \
  --tier full \
  --workers auto \
  --timeout 30

# Compare met vorige nightly
xfa-test-runner check-regression \
  --db-old /opt/xfa-results/nightly-$(date -d yesterday +%Y%m%d).sqlite \
  --db-new /opt/xfa-results/nightly-$(date +%Y%m%d).sqlite
```

---

## 9. Prioritering

### Fase 1: Meteen (bij huidige SDK work)

| Actie | Effort | Impact |
|-------|--------|--------|
| 4 nieuwe corpus tests (form_write, content_roundtrip, redact, pdfa_convert) | 2 dagen | Hoog — dekt alle write APIs |
| Fuzz targets voor content editor en lopdf roundtrip | 1 dag | Hoog — crash-freedom |
| Desktop backend unit tests | 1 dag | Medium — Tauri commands |

### Fase 2: Bij binding implementatie (#335–#340)

| Actie | Effort | Impact |
|-------|--------|--------|
| Per-binding test suite (12 scenario's) | 1 dag per binding | Hoog — API contract |
| C API smoke tests | 0.5 dag | Hoog — foundation voor .NET/Swift/Java |
| WASM browser tests | 1 dag | Medium |

### Fase 3: Kwaliteitsverdieping

| Actie | Effort | Impact |
|-------|--------|--------|
| Signing roundtrip test | 1 dag | Hoog — security-critical |
| Document conversie tests (DOCX/XLSX/PPTX) | 2 dagen | Medium |
| Performance baseline in CI | 1 dag | Medium — regressie-detectie |
| Desktop screenshot regression | 2 dagen | Medium — visuele stabiliteit |

### Fase 4: Continu

| Actie | Effort | Impact |
|-------|--------|--------|
| Nightly corpus run met nieuwe tests | Setup 0.5 dag | Hoog |
| Fuzz corpus maintenance | Doorlopend | Medium |
| Benchmark trend tracking | Doorlopend | Medium |

---

## 10. Metrics Dashboard Uitbreiding

De bestaande runner dashboard (`xfa-test-runner dashboard`) wordt uitgebreid met:

| Metric | Bron | Doel |
|--------|------|------|
| **Applicable pass rate** | runner DB | ≥ 99.9% |
| **Write roundtrip success** | form_write + annot_create | ≥ 99% |
| **Content roundtrip fidelity** | content_roundtrip | ≥ 99.5% |
| **PDF/A conversion rate** | pdfa_convert | ≥ 95% |
| **Redaction completeness** | redact test | 100% (tekst weg) |
| **Signing success** | sign_roundtrip | ≥ 99% |
| **Conversion text similarity** | DOCX/XLSX tekst | ≥ 80% |
| **Binding smoke pass** | per binding | 12/12 |
| **Benchmark delta** | bench-compare | ≤ 10% regressie |
| **Crash count** | runner + fuzz | 0 |

---

## Samenvatting

De uitbreiding rust op 4 pijlers:

1. **Corpus runner +8 tests** — write APIs, content engine, compliance conversie, redactie
   worden getest op dezelfde 50K+ PDFs die nu alleen gelezen worden. Roundtrip patroon
   (muteer → save → heropen → verify) garandeert dat writes geen corrupt PDF opleveren.

2. **Binding test suites** — 12 gemeenschappelijke scenario's per taal garanderen API
   contract-naleving. Dezelfde test cases in C, Python, Node, WASM, en later Java/.NET/Swift/Kotlin.

3. **Desktop app tests** — Tauri backend unit tests + frontend component tests + screenshot
   regression. Dekt undo/redo, shortcuts, annotation toolbar, print dialoog.

4. **Fuzz + performance** — 8 nieuwe fuzz targets voor crash-freedom van write paths.
   Benchmark baselines in CI voorkomen performance regressie.
