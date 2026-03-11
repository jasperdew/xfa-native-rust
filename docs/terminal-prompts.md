# Terminal Prompts — Test Expansion

## Terminal 1: Fase 1 — Corpus Runner Tests + Fuzz Targets + Desktop Backend

```
Je werkt in /Users/jasperdewinter/Documents/XFA aan het uitbreiden van de test-infrastructuur.

Referentiedocument: docs/test-expansion-plan.md

Werk de volgende GitHub issues sequentieel af:

1. #366 — test-runner: write operation roundtrip tests (form_write, annot_create, content_roundtrip, text_replace)
2. #367 — test-runner: redaction verification corpus test
3. #368 — test-runner: PDF/A conversion roundtrip test met veraPDF validatie
4. #370 — fuzz: 8 nieuwe targets voor write paths
5. #374 — desktop app: backend unit tests voor Tauri commands

CONTEXT:

De corpus runner staat in crates/xfa-test-runner/. De test matrix staat in src/tests/mod.rs. Elke test is een struct die PdfTest implementeert. Bestaande tests (parse, render, compliance, etc.) staan in src/tests/*.rs als referentie.

De nieuwe tests #366-#368 volgen het roundtrip patroon:
- Input PDF → mutatie (lopdf) → save naar tempfile → heropen → assertions
- Elke test heeft skip-logica (skip als feature niet van toepassing is)
- Tests registreren zich in de TestRegistry, de runner pikt ze automatisch op

De fuzz targets (#370) komen in fuzz/fuzz_targets/. Bestaande targets (fuzz_pdf_parser, fuzz_content_stream, etc.) zijn de referentie. Gebruik 7 bekende failure-PDFs + 50 diverse PDFs als seeds in fuzz/corpus/.

De desktop backend tests (#374) gaan in crates/pdf-desktop/src-tauri/. Test alle #[tauri::command] functies als gewone Rust functies met #[cfg(test)] module.

BELANGRIJK:
- Werk NIET in crates/pdf-desktop/src/ (frontend) — dat is Terminal 3
- Werk NIET in binding crates (pdf-capi, pdf-python, pdf-node, xfa-wasm) — dat is Terminal 2
- cargo fmt + cargo clippy -- -D warnings voor elke commit
- Conventional commits in het Engels
- GEEN Co-Authored-By of referenties naar Claude in commits
- Haal voor elk issue eerst de GitHub issue description op met: gh issue view <nummer>
- Sluit elk issue na implementatie met: gh issue close <nummer> -c "Implemented in <commit>"
```

---

## Terminal 2: Fase 2 — Binding Test Suites

```
Je werkt in /Users/jasperdewinter/Documents/XFA aan het toevoegen van geautomatiseerde tests voor de language bindings.

Referentiedocument: docs/test-expansion-plan.md (§3)

Werk de volgende GitHub issues sequentieel af:

1. #371 — test: binding test suites — C API smoke tests
2. #372 — test: binding test suites — Python (pytest), Node.js (Jest), WASM (wasm-pack)
3. #373 — test: binding test suites — Java, .NET, Swift, Kotlin

CONTEXT:

Elke binding test suite implementeert dezelfde 12 kernscenario's:
1. Open PDF, tel pagina's
2. Render pagina 1 naar PNG
3. Extraheer tekst pagina 1
4. Lees metadata (titel, auteur)
5. Lees AcroForm fields
6. Vul text field, save
7. Lees annotaties
8. Voeg highlight toe, save
9. Valideer PDF/A
10. Merge 2 PDFs
11. Verifieer handtekening
12. Extraheer images

De bindings zelf bestaan al:
- C API: crates/pdf-capi/ (extern "C" functies)
- Python: crates/pdf-python/ (PyO3, gebouwd met maturin)
- Node.js: crates/pdf-node/ (napi-rs)
- WASM: crates/xfa-wasm/ (wasm-bindgen)

De geplande bindings (#335-#338: Java, .NET, Swift, Kotlin) bestaan nog niet.
Voor #373: maak alleen de test scaffolding en fixture bestanden klaar.
Implementeer de daadwerkelijke tests pas wanneer de binding crate bestaat.
Markeer met TODO comments welke tests nog niet kunnen draaien.

Test bestanden:
- C: crates/pdf-capi/tests/ (Makefile + .c bestanden)
- Python: crates/pdf-python/tests/ (pytest)
- Node.js: crates/pdf-node/tests/ (Jest of Vitest)
- WASM: crates/xfa-wasm/tests/ (wasm_bindgen_test)

Zorg voor een fixture PDF in een gedeelde locatie die alle tests gebruiken:
fixtures/sample.pdf (een simpele PDF met tekst, een form field en een annotatie).
Als deze nog niet bestaat, maak er een aan via lopdf in een build script of gebruik een bestaande test PDF.

BELANGRIJK:
- Werk NIET in crates/xfa-test-runner/ — dat is Terminal 1
- Werk NIET in crates/pdf-desktop/ — dat is Terminal 1 en 3
- Controleer eerst of de binding crate compileert voordat je tests schrijft
- cargo fmt + cargo clippy -- -D warnings voor Rust code
- Haal voor elk issue eerst de GitHub issue description op met: gh issue view <nummer>
- Sluit elk issue na implementatie met: gh issue close <nummer> -c "Implemented in <commit>"
- GEEN Co-Authored-By of referenties naar Claude in commits
```

---

## Terminal 3: Fase 3 — Signing, Desktop Frontend, Benchmarks, Conversies

```
Je werkt in /Users/jasperdewinter/Documents/XFA aan kwaliteitsverdieping van de test-infrastructuur.

Referentiedocument: docs/test-expansion-plan.md (§5, §6, §4.2-4.6, §2)

Werk de volgende GitHub issues sequentieel af:

1. #369 — test-runner: signing roundtrip test
2. #376 — test: performance regression detection — benchmark baselines in CI
3. #377 — test: document conversion quality tests (DOCX/XLSX/PPTX)
4. #375 — test: desktop app — frontend component tests + screenshot regression

VOLGORDE IS BELANGRIJK: #375 (desktop frontend) als laatste, omdat Terminal 1 eerst
#374 (desktop backend tests) moet afronden. De andere drie issues zijn onafhankelijk.

CONTEXT PER ISSUE:

#369 — Signing roundtrip:
- Genereer een self-signed test certificaat: fixtures/certs/test.p12
- Nieuwe test in crates/xfa-test-runner/src/tests/sign_roundtrip.rs
- Patroon: load PDF → sign met test cert → save → heropen → verify signature → check integriteit
- Gebruik een subset van 100 willekeurige PDFs (niet alle 50K)
- pdf-sign crate bevat de sign/verify logica

#376 — Performance baselines:
- Bestaande benchmarks: crates/pdf-bench/benches/pdf_parse.rs en xfa_operations.rs
- Voeg nieuwe benchmarks toe: render_page, text_extract, compliance_check, pdfa_convert
- Maak scripts/bench-compare.py die twee benchmark JSON outputs vergelijkt
- Exit code 1 bij >10% regressie
- Voeg memory profiling benchmark toe (piek-RSS voor grote PDFs)

#377 — Conversie quality tests:
- crates/pdf-docx/, crates/pdf-xlsx/, crates/pdf-pptx/
- Structurele validatie: output is valid ZIP, XML parseert, tekst aanwezig
- Tekst-behoud metric: Levenshtein similarity ≥ 0.80
- Gebruik een kleine subset (500 tekst-PDFs, 100 tabel-PDFs, 50 image-PDFs)
- Kan als unit tests in de conversie crates zelf, of als corpus runner test

#375 — Desktop frontend tests:
- WACHT tot Terminal 1 klaar is met #374 (backend tests)
- Frontend tests in crates/pdf-desktop/src/components/__tests__/
- Vitest + Testing Library voor component tests
- Screenshot regression met golden images in pdf-desktop/golden/
- Test undo/redo stack, keyboard shortcuts, annotation toolbar

BELANGRIJK:
- Werk NIET in crates/xfa-test-runner/src/tests/ voor #366-#368 — dat is Terminal 1
- Wel in crates/xfa-test-runner/src/tests/ voor #369 (signing roundtrip)
- Werk NIET in binding crates — dat is Terminal 2
- cargo fmt + cargo clippy -- -D warnings voor elke commit
- Conventional commits in het Engels
- GEEN Co-Authored-By of referenties naar Claude in commits
- Haal voor elk issue eerst de GitHub issue description op met: gh issue view <nummer>
- Sluit elk issue na implementatie met: gh issue close <nummer> -c "Implemented in <commit>"
```

---

## Terminal 4: Fase 4 — CI & Nightly (NA Fase 1-3)

Issues om af te werken nadat Terminal 1, 2 en 3 klaar zijn:

- **#378** — Nightly VPS corpus run met uitgebreide test matrix
- **#379** — CI pipeline uitbreiden met binding tests, desktop tests en benchmark vergelijking
