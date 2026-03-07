#!/usr/bin/env python3
"""Categorize PDFs by features using binary inspection.

Scans all PDFs in the corpus directory, analyzes their features,
and stores metadata in a SQLite database.

Runs on VPS after download. Fills metadata.sqlite.

Usage:
    ./scripts/corpus-categorize.py [CORPUS_DIR]
    ./scripts/corpus-categorize.py --init-db [CORPUS_DIR]
    ./scripts/corpus-categorize.py --reindex [CORPUS_DIR]

Default corpus dir: /opt/xfa-corpus
"""

import argparse
import hashlib
import os
import re
import shutil
import sqlite3
import subprocess
import sys
from datetime import datetime
from pathlib import Path

SCHEMA = """
CREATE TABLE IF NOT EXISTS pdfs (
    hash TEXT PRIMARY KEY,
    path TEXT NOT NULL,
    size INTEGER NOT NULL,
    category TEXT NOT NULL,
    subcategory TEXT,
    source TEXT NOT NULL,
    pdf_version TEXT,
    page_count INTEGER,
    has_forms INTEGER DEFAULT 0,
    has_xfa INTEGER DEFAULT 0,
    has_signatures INTEGER DEFAULT 0,
    has_annotations INTEGER DEFAULT 0,
    has_encryption INTEGER DEFAULT 0,
    claims_pdfa INTEGER DEFAULT 0,
    claims_pdfua INTEGER DEFAULT 0,
    producer TEXT,
    file_size_category TEXT,
    added_at TEXT NOT NULL,
    notes TEXT
);

CREATE INDEX IF NOT EXISTS idx_pdfs_category ON pdfs(category);
CREATE INDEX IF NOT EXISTS idx_pdfs_source ON pdfs(source);
CREATE INDEX IF NOT EXISTS idx_pdfs_features ON pdfs(has_forms, has_xfa, has_signatures);
"""


def init_db(db_path: Path) -> sqlite3.Connection:
    """Create or open the metadata database."""
    conn = sqlite3.connect(str(db_path))
    conn.executescript(SCHEMA)
    conn.commit()
    return conn


def sha256_file(path: Path) -> str:
    """Compute SHA-256 hash of a file."""
    h = hashlib.sha256()
    with open(path, "rb") as f:
        while chunk := f.read(65536):
            h.update(chunk)
    return h.hexdigest()


def classify_size(size: int) -> str:
    """Classify file size into categories."""
    if size < 50_000:
        return "tiny"
    if size < 500_000:
        return "small"
    if size < 5_000_000:
        return "medium"
    if size < 50_000_000:
        return "large"
    return "huge"


def detect_category(path: Path, corpus_dir: Path) -> tuple[str, str | None]:
    """Determine category and subcategory from directory structure."""
    try:
        rel = path.relative_to(corpus_dir)
        parts = rel.parts
    except ValueError:
        return "general", None

    if len(parts) >= 2:
        return parts[0], parts[1]
    if len(parts) == 1:
        return "general", None
    return "general", None


def detect_source(path: Path) -> str:
    """Guess source from path or filename patterns."""
    name = path.name.lower()
    parent = path.parent.name.lower()

    if "govdocs" in parent or name.startswith(("000_", "001_", "002_")):
        return "govdocs"
    if "isartor" in parent or "isartor" in name:
        return "isartor"
    if "verapdf" in parent:
        return "verapdf"
    if "pdfbox" in parent:
        return "pdfbox"
    if "mustang" in name or "zugferd" in parent:
        return "mustang"
    if "pdf-assoc" in parent:
        return "pdf-association"
    if name.startswith(("f1", "fw", "f2", "f3", "f4", "f5", "f6", "f7", "f8", "f9")):
        return "irs"
    if name.startswith(("i-", "n-", "sf")):
        return "us-gov-forms"

    return "unknown"


def try_pdfinfo(path: Path) -> dict:
    """Try to extract metadata via pdfinfo (if available)."""
    result = {}
    if not shutil.which("pdfinfo"):
        return result

    try:
        proc = subprocess.run(
            ["pdfinfo", str(path)],
            capture_output=True,
            text=True,
            timeout=10,
        )
        for line in proc.stdout.splitlines():
            if ":" not in line:
                continue
            key, val = line.split(":", 1)
            key, val = key.strip(), val.strip()
            if key == "Pages":
                try:
                    result["page_count"] = int(val)
                except ValueError:
                    pass
            elif key == "PDF version":
                result["pdf_version"] = val
            elif key == "Producer":
                result["producer"] = val[:200]  # Truncate long producer strings
    except (subprocess.TimeoutExpired, OSError):
        pass

    return result


def analyze_pdf(pdf_path: Path, corpus_dir: Path) -> dict | None:
    """Analyze a single PDF for features and metadata."""
    try:
        size = pdf_path.stat().st_size
    except OSError:
        return None

    if size == 0:
        return None

    info = {
        "path": str(pdf_path),
        "size": size,
        "file_size_category": classify_size(size),
        "has_forms": 0,
        "has_xfa": 0,
        "has_signatures": 0,
        "has_annotations": 0,
        "has_encryption": 0,
        "claims_pdfa": 0,
        "claims_pdfua": 0,
        "pdf_version": None,
        "page_count": None,
        "producer": None,
    }

    # SHA-256 hash
    try:
        info["hash"] = sha256_file(pdf_path)
    except OSError:
        return None

    # Category from directory structure
    info["category"], info["subcategory"] = detect_category(pdf_path, corpus_dir)
    info["source"] = detect_source(pdf_path)

    # Binary inspection — read first 100KB for feature detection
    try:
        with open(pdf_path, "rb") as f:
            header_bytes = f.read(min(100_000, size))
            content = header_bytes.decode("latin-1", errors="ignore")

            # PDF version from header
            version_match = re.search(r"%PDF-(\d+\.\d+)", content[:20])
            if version_match:
                info["pdf_version"] = version_match.group(1)

            # Feature detection
            info["has_forms"] = 1 if "/AcroForm" in content else 0
            info["has_xfa"] = 1 if "/XFA" in content else 0
            info["has_signatures"] = (
                1 if "/Sig" in content and "/ByteRange" in content else 0
            )
            info["has_annotations"] = 1 if "/Annots" in content else 0
            info["has_encryption"] = 1 if "/Encrypt" in content else 0
            info["claims_pdfa"] = 1 if "pdfaid:part" in content.lower() else 0
            info["claims_pdfua"] = 1 if "pdfuaid:part" in content.lower() else 0
    except OSError:
        pass

    # Enrich with pdfinfo if available
    pdfinfo = try_pdfinfo(pdf_path)
    if "page_count" in pdfinfo:
        info["page_count"] = pdfinfo["page_count"]
    if "pdf_version" in pdfinfo:
        info["pdf_version"] = pdfinfo["pdf_version"]
    if "producer" in pdfinfo:
        info["producer"] = pdfinfo["producer"]

    return info


def categorize_corpus(corpus_dir: Path, db_path: Path, reindex: bool = False):
    """Walk corpus, analyze each PDF, insert into SQLite."""
    conn = init_db(db_path)

    if reindex:
        conn.execute("DELETE FROM pdfs")
        conn.commit()
        print("Cleared existing entries for reindex.")

    # Get already-indexed hashes to skip
    existing = set()
    if not reindex:
        cursor = conn.execute("SELECT hash FROM pdfs")
        existing = {row[0] for row in cursor}
        print(f"Skipping {len(existing)} already-indexed PDFs.")

    pdf_files = sorted(corpus_dir.rglob("*.pdf"))
    pdf_files.extend(sorted(corpus_dir.rglob("*.PDF")))
    total = len(pdf_files)
    inserted = 0
    skipped = 0
    errors = 0

    print(f"Found {total} PDF files in {corpus_dir}")
    print()

    for i, pdf_path in enumerate(pdf_files, 1):
        if i % 100 == 0 or i == total:
            print(f"  Progress: {i}/{total} (inserted={inserted}, skipped={skipped})")

        info = analyze_pdf(pdf_path, corpus_dir)
        if info is None:
            errors += 1
            continue

        if info["hash"] in existing:
            skipped += 1
            continue

        try:
            conn.execute(
                """INSERT OR REPLACE INTO pdfs
                (hash, path, size, category, subcategory, source,
                 pdf_version, page_count, has_forms, has_xfa,
                 has_signatures, has_annotations, has_encryption,
                 claims_pdfa, claims_pdfua, producer,
                 file_size_category, added_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
                (
                    info["hash"],
                    info["path"],
                    info["size"],
                    info["category"],
                    info.get("subcategory"),
                    info["source"],
                    info.get("pdf_version"),
                    info.get("page_count"),
                    info["has_forms"],
                    info["has_xfa"],
                    info["has_signatures"],
                    info["has_annotations"],
                    info["has_encryption"],
                    info["claims_pdfa"],
                    info["claims_pdfua"],
                    info.get("producer"),
                    info["file_size_category"],
                    datetime.utcnow().isoformat(),
                ),
            )
            inserted += 1
            existing.add(info["hash"])
        except sqlite3.Error as e:
            print(f"  ERROR: {pdf_path}: {e}", file=sys.stderr)
            errors += 1

    conn.commit()
    conn.close()

    print()
    print(f"Done: {inserted} inserted, {skipped} skipped, {errors} errors")
    print(f"Database: {db_path}")


def main():
    parser = argparse.ArgumentParser(description="Categorize PDF corpus into SQLite")
    parser.add_argument(
        "corpus_dir",
        nargs="?",
        default="/opt/xfa-corpus",
        help="Corpus directory (default: /opt/xfa-corpus)",
    )
    parser.add_argument(
        "--init-db", action="store_true", help="Only initialize the database schema"
    )
    parser.add_argument(
        "--reindex",
        action="store_true",
        help="Clear and rebuild the entire index",
    )

    args = parser.parse_args()
    corpus_dir = Path(args.corpus_dir)
    db_path = corpus_dir / "metadata.sqlite"

    if not corpus_dir.is_dir():
        print(f"ERROR: Directory not found: {corpus_dir}", file=sys.stderr)
        sys.exit(1)

    if args.init_db:
        conn = init_db(db_path)
        conn.close()
        print(f"Database initialized: {db_path}")
        return

    categorize_corpus(corpus_dir, db_path, reindex=args.reindex)


if __name__ == "__main__":
    main()
