#!/usr/bin/env python3
"""Curate a diverse 1K PDF corpus from the VPS corpus.

Selects 1,000 PDFs with maximum diversity across producers, features,
sizes, and categories. Uses the metadata.sqlite database for indexed
PDFs and supplements with random samples from the unindexed corpus.

Usage:
    python3 scripts/curate-corpus.py \
        --metadata /opt/xfa-corpus/metadata.sqlite \
        --corpus /opt/xfa-corpus \
        --output tests/curated-1k.txt \
        --target 1000
"""

import argparse
import os
import random
import sqlite3
import subprocess
import sys
from collections import defaultdict
from pathlib import Path


def classify_producer(producer: str | None) -> str:
    if not producer:
        return "unknown"
    p = producer.lower()
    if any(k in p for k in ("microsoft", "word", "powerpoint", "excel")):
        return "microsoft"
    if any(k in p for k in ("latex", "pdftex", "luatex", "xetex", "tex")):
        return "latex"
    if any(k in p for k in ("indesign", "illustrator", "acrobat", "adobe", "distiller", "pdfwriter")):
        return "adobe"
    if any(k in p for k in ("libreoffice", "openoffice")):
        return "libreoffice"
    if any(k in p for k in ("scan",)):
        return "scanner"
    if any(k in p for k in ("chrome", "chromium")):
        return "chrome"
    if any(k in p for k in ("itext",)):
        return "itext"
    if any(k in p for k in ("verapdf",)):
        return "verapdf"
    if any(k in p for k in ("quartz", "macos", "preview")):
        return "apple"
    if any(k in p for k in ("cairo",)):
        return "cairo"
    if any(k in p for k in ("wkhtmltopdf",)):
        return "wkhtmltopdf"
    if any(k in p for k in ("ghostscript", "gs")):
        return "ghostscript"
    if any(k in p for k in ("fpdf", "tcpdf", "mpdf")):
        return "php"
    return "other"


def classify_size(size: int) -> str:
    if size < 50_000:
        return "tiny"
    if size < 200_000:
        return "small"
    if size < 1_000_000:
        return "medium"
    if size < 10_000_000:
        return "large"
    return "xlarge"


def main():
    parser = argparse.ArgumentParser(description="Curate a diverse PDF corpus")
    parser.add_argument("--metadata", required=True, help="Path to metadata.sqlite")
    parser.add_argument("--corpus", required=True, help="Corpus root directory")
    parser.add_argument("--output", required=True, help="Output file (one path per line)")
    parser.add_argument("--target", type=int, default=1000, help="Target number of PDFs")
    parser.add_argument("--seed", type=int, default=42, help="Random seed for reproducibility")
    args = parser.parse_args()

    random.seed(args.seed)
    corpus_root = Path(args.corpus)

    # ── Step 1: Load indexed PDFs ──────────────────────────────────────

    conn = sqlite3.connect(args.metadata)
    conn.row_factory = sqlite3.Row
    rows = conn.execute("""
        SELECT path, size, category, producer,
               has_forms, has_xfa, has_signatures, has_annotations,
               has_encryption, claims_pdfa, claims_pdfua, page_count
        FROM pdfs
    """).fetchall()
    conn.close()

    print(f"Loaded {len(rows)} indexed PDFs from metadata DB")

    # ── Step 2: Classify and bucket ────────────────────────────────────

    # Feature buckets: ensure minimum representation
    feature_buckets = {
        "forms": [],
        "xfa": [],
        "signatures": [],
        "annotations": [],
        "encrypted": [],
        "pdfa": [],
        "pdfua": [],
        "multipage": [],  # 50+ pages
    }

    # Producer buckets
    producer_buckets = defaultdict(list)

    # Size buckets
    size_buckets = defaultdict(list)

    # Category buckets
    category_buckets = defaultdict(list)

    for row in rows:
        path = row["path"]
        # Verify the file exists
        full_path = corpus_root / path if not os.path.isabs(path) else Path(path)
        if not full_path.exists():
            continue

        entry = {
            "path": path,
            "full_path": str(full_path),
            "size": row["size"],
            "category": row["category"],
            "producer": row["producer"],
            "features": set(),
        }

        if row["has_forms"]:
            entry["features"].add("forms")
            feature_buckets["forms"].append(entry)
        if row["has_xfa"]:
            entry["features"].add("xfa")
            feature_buckets["xfa"].append(entry)
        if row["has_signatures"]:
            entry["features"].add("signatures")
            feature_buckets["signatures"].append(entry)
        if row["has_annotations"]:
            entry["features"].add("annotations")
            feature_buckets["annotations"].append(entry)
        if row["has_encryption"]:
            entry["features"].add("encrypted")
            feature_buckets["encrypted"].append(entry)
        if row["claims_pdfa"]:
            entry["features"].add("pdfa")
            feature_buckets["pdfa"].append(entry)
        if row["claims_pdfua"]:
            entry["features"].add("pdfua")
            feature_buckets["pdfua"].append(entry)
        if row["page_count"] and row["page_count"] >= 50:
            entry["features"].add("multipage")
            feature_buckets["multipage"].append(entry)

        prod_cat = classify_producer(row["producer"])
        producer_buckets[prod_cat].append(entry)

        size_cat = classify_size(row["size"])
        size_buckets[size_cat].append(entry)

        category_buckets[row["category"]].append(entry)

    # ── Step 3: Select with diversity quotas ───────────────────────────

    selected = {}  # path -> entry (deduplication)

    def add(entry):
        selected[entry["full_path"]] = entry

    def sample_from(bucket, n):
        available = [e for e in bucket if e["full_path"] not in selected]
        chosen = random.sample(available, min(n, len(available)))
        for e in chosen:
            add(e)
        return len(chosen)

    # Phase 1: Feature quotas (ensure minimum representation)
    feature_quotas = {
        "forms": 60,
        "xfa": 7,       # rare, take all
        "signatures": 50,
        "annotations": 60,
        "encrypted": 50,
        "pdfa": 80,
        "pdfua": 50,
        "multipage": 30,
    }
    print("\nPhase 1: Feature quotas")
    for feat, quota in feature_quotas.items():
        n = sample_from(feature_buckets[feat], quota)
        print(f"  {feat}: {n}/{quota} (available: {len(feature_buckets[feat])})")

    # Phase 2: Producer diversity
    producer_quotas = {
        "adobe": 120,
        "microsoft": 30,
        "latex": 40,
        "libreoffice": 40,
        "itext": 40,
        "verapdf": 50,
        "apple": 30,
        "ghostscript": 20,
        "chrome": 10,
        "scanner": 15,
        "cairo": 4,
        "other": 50,
        "unknown": 50,
    }
    print("\nPhase 2: Producer quotas")
    for prod, quota in producer_quotas.items():
        n = sample_from(producer_buckets[prod], quota)
        print(f"  {prod}: {n}/{quota} (available: {len(producer_buckets[prod])})")

    # Phase 3: Category coverage
    category_quotas = {
        "general": 100,
        "tagged": 50,
        "forms": 30,
        "signed": 30,
        "invoices": 10,
    }
    print("\nPhase 3: Category quotas")
    for cat, quota in category_quotas.items():
        n = sample_from(category_buckets[cat], quota)
        print(f"  {cat}: {n}/{quota} (available: {len(category_buckets[cat])})")

    # Phase 4: Size diversity
    size_quotas = {
        "tiny": 50,
        "small": 50,
        "medium": 50,
        "large": 20,
        "xlarge": 10,
    }
    print("\nPhase 4: Size quotas")
    for sz, quota in size_quotas.items():
        n = sample_from(size_buckets[sz], quota)
        print(f"  {sz}: {n}/{quota} (available: {len(size_buckets[sz])})")

    print(f"\nAfter quotas: {len(selected)} selected")

    # Phase 5: Fill remaining from stressful corpus (unindexed)
    remaining = args.target - len(selected)
    if remaining > 0:
        stressful_dir = corpus_root / "stressful"
        if stressful_dir.exists():
            stressful_pdfs = list(stressful_dir.rglob("*.pdf"))
            stressful_pdfs = [p for p in stressful_pdfs if str(p) not in selected]
            chosen = random.sample(stressful_pdfs, min(remaining // 2, len(stressful_pdfs)))
            for p in chosen:
                add({"path": str(p.relative_to(corpus_root)), "full_path": str(p), "size": p.stat().st_size})
            print(f"\nPhase 5: Added {len(chosen)} from stressful corpus")

    # Phase 6: Fill any remaining with random general PDFs
    remaining = args.target - len(selected)
    if remaining > 0:
        general_dir = corpus_root / "general"
        if general_dir.exists():
            general_pdfs = list(general_dir.rglob("*.pdf"))
            general_pdfs = [p for p in general_pdfs if str(p) not in selected]
            chosen = random.sample(general_pdfs, min(remaining, len(general_pdfs)))
            for p in chosen:
                add({"path": str(p.relative_to(corpus_root)), "full_path": str(p), "size": p.stat().st_size})
            print(f"Phase 6: Added {len(chosen)} from general corpus")

    # ── Step 4: Write output ───────────────────────────────────────────

    paths = sorted(selected.keys())
    output_path = Path(args.output)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with open(output_path, "w") as f:
        for p in paths:
            f.write(p + "\n")

    print(f"\nTotal selected: {len(paths)}")
    print(f"Written to: {output_path}")

    # Print summary statistics
    sizes = [selected[p].get("size", 0) for p in paths]
    print(f"\nSize distribution:")
    for cat in ("tiny", "small", "medium", "large", "xlarge"):
        if cat == "tiny":
            n = sum(1 for s in sizes if s < 50_000)
        elif cat == "small":
            n = sum(1 for s in sizes if 50_000 <= s < 200_000)
        elif cat == "medium":
            n = sum(1 for s in sizes if 200_000 <= s < 1_000_000)
        elif cat == "large":
            n = sum(1 for s in sizes if 1_000_000 <= s < 10_000_000)
        else:
            n = sum(1 for s in sizes if s >= 10_000_000)
        print(f"  {cat}: {n}")


if __name__ == "__main__":
    main()
