//! Font loading, glyph metrics, and subsetting.
//!
//! Provides real font measurement using TrueType/OpenType font data,
//! system font discovery, and font subsetting for embedding.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;

/// Errors related to font operations.
#[derive(Debug, Error)]
pub enum FontError {
    #[error("Font file not found: {0}")]
    FileNotFound(PathBuf),

    #[error("Failed to read font file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to parse font: {0}")]
    ParseError(String),

    #[error("Glyph not found for character: {0:?}")]
    GlyphNotFound(char),

    #[error("Subsetting failed: {0}")]
    SubsetError(String),

    #[error("No font found for family: {0}")]
    FamilyNotFound(String),
}

/// Font weight (XFA §8.3.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FontWeight {
    #[default]
    Normal,
    Bold,
}

/// Font style (XFA §8.3.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FontStyle {
    #[default]
    Normal,
    Italic,
}

/// A descriptor identifying a specific font variant.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FontDescriptor {
    pub family: String,
    pub weight: FontWeight,
    pub style: FontStyle,
}

impl FontDescriptor {
    pub fn new(family: impl Into<String>, weight: FontWeight, style: FontStyle) -> Self {
        Self {
            family: family.into(),
            weight,
            style,
        }
    }

    /// Create a descriptor for a regular weight, normal style font.
    pub fn regular(family: impl Into<String>) -> Self {
        Self::new(family, FontWeight::Normal, FontStyle::Normal)
    }
}

/// A loaded font with glyph metrics.
///
/// Wraps the parsed font data and provides measurement operations.
#[derive(Debug, Clone)]
pub struct LoadedFont {
    /// The font data (shared across clones).
    data: Arc<Vec<u8>>,
    /// Parsed font for glyph metrics.
    face: Arc<FontFace>,
    /// Family name as reported by the font.
    pub family_name: String,
    /// Whether this is a monospace font.
    pub is_monospace: bool,
}

/// Parsed font face data using ttf-parser.
#[derive(Debug)]
struct FontFace {
    units_per_em: f64,
    ascender: f64,
    descender: f64,
    line_gap: f64,
    /// Horizontal advance widths indexed by glyph ID.
    advances: Vec<u16>,
    /// Character-to-glyph mapping.
    cmap: HashMap<char, u16>,
    /// Kerning pairs: (left_glyph, right_glyph) -> adjustment.
    kern_pairs: HashMap<(u16, u16), i16>,
}

/// Extract a name string from a font face, trying all name records with the given ID.
fn extract_name(face: &ttf_parser::Face<'_>, name_id: u16) -> Option<String> {
    face.names()
        .into_iter()
        .filter(|n| n.name_id == name_id)
        .find_map(|n| n.to_string())
}

impl LoadedFont {
    /// Load a font from raw byte data.
    pub fn from_bytes(data: Vec<u8>) -> Result<Self, FontError> {
        let face =
            ttf_parser::Face::parse(&data, 0).map_err(|e| FontError::ParseError(format!("{e}")))?;

        let units_per_em = face.units_per_em() as f64;
        let ascender = face.ascender() as f64;
        let descender = face.descender() as f64;
        let line_gap = face.line_gap() as f64;
        let is_monospace = face.is_monospaced();

        // Extract family name — try multiple name IDs since some fonts
        // (especially TTC collections) may only have platform-specific encodings.
        let family_name = extract_name(&face, ttf_parser::name_id::TYPOGRAPHIC_FAMILY)
            .or_else(|| extract_name(&face, ttf_parser::name_id::FAMILY))
            .or_else(|| extract_name(&face, ttf_parser::name_id::FULL_NAME))
            .unwrap_or_default();

        // Build cmap and advance widths
        let num_glyphs = face.number_of_glyphs();
        let mut advances = Vec::with_capacity(num_glyphs as usize);
        for gid in 0..num_glyphs {
            let id = ttf_parser::GlyphId(gid);
            let advance = face.glyph_hor_advance(id).unwrap_or(0);
            advances.push(advance);
        }

        let mut cmap = HashMap::new();
        if let Some(subtable) = face.tables().cmap {
            for subtable in subtable.subtables {
                if !subtable.is_unicode() {
                    continue;
                }
                subtable.codepoints(|cp| {
                    if let Some(ch) = char::from_u32(cp) {
                        if let Some(gid) = subtable.glyph_index(cp) {
                            cmap.insert(ch, gid.0);
                        }
                    }
                });
            }
        }

        // Extract kerning pairs from kern table
        let mut kern_pairs = HashMap::new();
        if let Some(kern_table) = face.tables().kern {
            for subtable in kern_table.subtables {
                if !subtable.horizontal || subtable.variable {
                    continue;
                }
                if let ttf_parser::kern::Format::Format0(fmt0) = subtable.format {
                    for pair in fmt0.pairs {
                        kern_pairs.insert((pair.left().0, pair.right().0), pair.value);
                    }
                }
            }
        }

        let font_face = FontFace {
            units_per_em,
            ascender,
            descender,
            line_gap,
            advances,
            cmap,
            kern_pairs,
        };

        Ok(Self {
            data: Arc::new(data),
            face: Arc::new(font_face),
            family_name,
            is_monospace,
        })
    }

    /// Load a font from a file path.
    pub fn from_file(path: &Path) -> Result<Self, FontError> {
        if !path.exists() {
            return Err(FontError::FileNotFound(path.to_owned()));
        }
        let data = std::fs::read(path)?;
        Self::from_bytes(data)
    }

    /// Measure the width of a text string at a given font size (in points).
    ///
    /// Includes kerning adjustments between glyph pairs.
    pub fn measure_width(&self, text: &str, size_pt: f64) -> f64 {
        let scale = size_pt / self.face.units_per_em;
        let mut width = 0.0;
        let mut prev_gid: Option<u16> = None;

        for ch in text.chars() {
            let gid = self.glyph_id(ch);
            let advance = self.face.advances.get(gid as usize).copied().unwrap_or(0);
            width += advance as f64;

            // Apply kerning
            if let Some(prev) = prev_gid {
                if let Some(&kern) = self.face.kern_pairs.get(&(prev, gid)) {
                    width += kern as f64;
                }
            }
            prev_gid = Some(gid);
        }

        width * scale
    }

    /// Measure a single character's advance width at a given font size.
    pub fn char_width(&self, ch: char, size_pt: f64) -> f64 {
        let scale = size_pt / self.face.units_per_em;
        let gid = self.glyph_id(ch);
        let advance = self.face.advances.get(gid as usize).copied().unwrap_or(0);
        advance as f64 * scale
    }

    /// Get the ascender height (above baseline) in points.
    pub fn ascender(&self, size_pt: f64) -> f64 {
        self.face.ascender * size_pt / self.face.units_per_em
    }

    /// Get the descender depth (below baseline, typically negative) in points.
    pub fn descender(&self, size_pt: f64) -> f64 {
        self.face.descender * size_pt / self.face.units_per_em
    }

    /// Compute line height based on ascender, descender, and line gap.
    pub fn line_height(&self, size_pt: f64) -> f64 {
        let scale = size_pt / self.face.units_per_em;
        (self.face.ascender - self.face.descender + self.face.line_gap) * scale
    }

    /// Get the glyph ID for a character, falling back to .notdef (0) if not found.
    fn glyph_id(&self, ch: char) -> u16 {
        self.face.cmap.get(&ch).copied().unwrap_or(0)
    }

    /// Check if the font contains a glyph for the given character.
    pub fn has_glyph(&self, ch: char) -> bool {
        self.face.cmap.contains_key(&ch)
    }

    /// Collect all unique glyph IDs needed for a text string.
    pub fn glyphs_for_text(&self, text: &str) -> HashSet<u16> {
        text.chars().map(|ch| self.glyph_id(ch)).collect()
    }

    /// Return the raw font data bytes (for subsetting or embedding).
    pub fn font_data(&self) -> &[u8] {
        &self.data
    }

    /// Compute the average character width over a standard set of characters.
    ///
    /// Uses lowercase a-z and space to approximate average proportional width.
    pub fn avg_char_width(&self, size_pt: f64) -> f64 {
        let chars = "abcdefghijklmnopqrstuvwxyz ";
        let total: f64 = chars.chars().map(|c| self.char_width(c, size_pt)).sum();
        total / chars.len() as f64
    }
}

/// Subset a font to include only the glyphs needed for the given text.
///
/// Returns the subsetted font data as bytes.
pub fn subset_font(font_data: &[u8], text: &str) -> Result<Vec<u8>, FontError> {
    let mut glyph_ids = HashSet::new();
    // Always include .notdef (glyph 0)
    glyph_ids.insert(0u16);

    // Parse to get cmap for character→glyph mapping
    let face =
        ttf_parser::Face::parse(font_data, 0).map_err(|e| FontError::ParseError(e.to_string()))?;

    for ch in text.chars() {
        if let Some(gid) = face.glyph_index(ch) {
            glyph_ids.insert(gid.0);
        }
    }

    let mut remapper = subsetter::GlyphRemapper::new();
    for &gid in &glyph_ids {
        remapper.remap(gid);
    }
    let subsetted = subsetter::subset(font_data, 0, &remapper)
        .map_err(|e| FontError::SubsetError(format!("{e:?}")))?;

    Ok(subsetted)
}

/// System font resolver — discovers and loads fonts from OS directories.
pub struct FontResolver {
    /// Map from (family, weight, style) → file path.
    font_index: HashMap<FontDescriptor, PathBuf>,
    /// Cache of loaded fonts.
    loaded: HashMap<FontDescriptor, LoadedFont>,
    /// Fallback chain: tried in order when a requested family is not found.
    fallback_chain: Vec<String>,
}

impl std::fmt::Debug for FontResolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FontResolver")
            .field("indexed_fonts", &self.font_index.len())
            .field("loaded_fonts", &self.loaded.len())
            .field("fallback_chain", &self.fallback_chain)
            .finish()
    }
}

impl FontResolver {
    /// Create a new resolver that scans system font directories.
    pub fn new() -> Self {
        let mut resolver = Self {
            font_index: HashMap::new(),
            loaded: HashMap::new(),
            fallback_chain: default_fallback_chain(),
        };
        for dir in system_font_dirs() {
            resolver.scan_directory(&dir);
        }
        resolver
    }

    /// Create a resolver with no system fonts (useful for testing).
    pub fn empty() -> Self {
        Self {
            font_index: HashMap::new(),
            loaded: HashMap::new(),
            fallback_chain: Vec::new(),
        }
    }

    /// Scan a directory for font files and add them to the index.
    pub fn scan_directory(&mut self, dir: &Path) {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                self.scan_directory(&path);
                continue;
            }

            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase());

            if let Some("ttf" | "otf" | "ttc") = ext.as_deref() {
                self.index_font_file(&path);
            }
        }
    }

    /// Register a font from raw bytes with a given descriptor.
    pub fn register_font(
        &mut self,
        descriptor: FontDescriptor,
        data: Vec<u8>,
    ) -> Result<(), FontError> {
        let font = LoadedFont::from_bytes(data)?;
        self.loaded.insert(descriptor, font);
        Ok(())
    }

    /// Resolve a font descriptor to a loaded font.
    ///
    /// Tries exact match first, then falls back through the fallback chain.
    pub fn resolve(&mut self, descriptor: &FontDescriptor) -> Option<&LoadedFont> {
        // Check if already loaded
        if self.loaded.contains_key(descriptor) {
            return self.loaded.get(descriptor);
        }

        // Try loading from index
        if let Some(path) = self.font_index.get(descriptor).cloned() {
            if let Ok(font) = LoadedFont::from_file(&path) {
                self.loaded.insert(descriptor.clone(), font);
                return self.loaded.get(descriptor);
            }
        }

        // Try fallback chain
        for fallback_family in self.fallback_chain.clone() {
            let fallback_desc =
                FontDescriptor::new(fallback_family.clone(), descriptor.weight, descriptor.style);
            if self.loaded.contains_key(&fallback_desc) {
                return self.loaded.get(&fallback_desc);
            }
            if let Some(path) = self.font_index.get(&fallback_desc).cloned() {
                if let Ok(font) = LoadedFont::from_file(&path) {
                    self.loaded.insert(fallback_desc.clone(), font);
                    return self.loaded.get(&fallback_desc);
                }
            }
        }

        None
    }

    /// Get the number of indexed font files.
    pub fn indexed_count(&self) -> usize {
        self.font_index.len()
    }

    /// Get the number of loaded (cached) fonts.
    pub fn loaded_count(&self) -> usize {
        self.loaded.len()
    }

    /// Index a single font file: parse it to extract family name and style.
    fn index_font_file(&mut self, path: &Path) {
        let data = match std::fs::read(path) {
            Ok(d) => d,
            Err(_) => return,
        };

        let face = match ttf_parser::Face::parse(&data, 0) {
            Ok(f) => f,
            Err(_) => return,
        };

        let family = match extract_name(&face, ttf_parser::name_id::TYPOGRAPHIC_FAMILY)
            .or_else(|| extract_name(&face, ttf_parser::name_id::FAMILY))
            .or_else(|| extract_name(&face, ttf_parser::name_id::FULL_NAME))
        {
            Some(name) => name,
            None => return,
        };

        let weight = if face.is_bold() {
            FontWeight::Bold
        } else {
            FontWeight::Normal
        };

        let style = if face.is_italic() {
            FontStyle::Italic
        } else {
            FontStyle::Normal
        };

        let descriptor = FontDescriptor::new(family, weight, style);
        self.font_index.insert(descriptor, path.to_owned());
    }
}

impl Default for FontResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Return the default fallback font chain.
fn default_fallback_chain() -> Vec<String> {
    vec![
        "Helvetica".to_string(),
        "Arial".to_string(),
        "Liberation Sans".to_string(),
        "DejaVu Sans".to_string(),
        "Noto Sans".to_string(),
    ]
}

/// Return system font directories for the current OS.
fn system_font_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    #[cfg(target_os = "macos")]
    {
        dirs.push(PathBuf::from("/System/Library/Fonts"));
        dirs.push(PathBuf::from("/Library/Fonts"));
        if let Some(home) = std::env::var_os("HOME") {
            let mut user_fonts = PathBuf::from(home);
            user_fonts.push("Library/Fonts");
            dirs.push(user_fonts);
        }
    }

    #[cfg(target_os = "linux")]
    {
        dirs.push(PathBuf::from("/usr/share/fonts"));
        dirs.push(PathBuf::from("/usr/local/share/fonts"));
        if let Some(home) = std::env::var_os("HOME") {
            let mut user_fonts = PathBuf::from(home);
            user_fonts.push(".local/share/fonts");
            dirs.push(user_fonts);
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Some(windir) = std::env::var_os("WINDIR") {
            let mut fonts = PathBuf::from(windir);
            fonts.push("Fonts");
            dirs.push(fonts);
        }
        if let Some(localappdata) = std::env::var_os("LOCALAPPDATA") {
            let mut fonts = PathBuf::from(localappdata);
            fonts.push("Microsoft\\Windows\\Fonts");
            dirs.push(fonts);
        }
    }

    dirs
}

#[cfg(test)]
mod tests {
    use super::*;

    fn find_test_font() -> Option<PathBuf> {
        let candidates = [
            // macOS — prefer TTF over TTC (TTC name encoding may not decode)
            "/System/Library/Fonts/Supplemental/Arial.ttf",
            "/Library/Fonts/Arial.ttf",
            "/System/Library/Fonts/Helvetica.ttc",
            // Linux
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/TTF/DejaVuSans.ttf",
            "/usr/share/fonts/dejavu-sans-fonts/DejaVuSans.ttf",
        ];
        candidates.iter().map(PathBuf::from).find(|p| p.exists())
    }

    #[test]
    fn load_system_font() {
        let path = match find_test_font() {
            Some(p) => p,
            None => {
                eprintln!("No test font found, skipping");
                return;
            }
        };

        let font = LoadedFont::from_file(&path).expect("should load font");
        assert!(!font.family_name.is_empty());
    }

    #[test]
    fn measure_text_width() {
        let path = match find_test_font() {
            Some(p) => p,
            None => return,
        };

        let font = LoadedFont::from_file(&path).unwrap();

        // "W" should be wider than "i"
        let w_width = font.char_width('W', 12.0);
        let i_width = font.char_width('i', 12.0);
        assert!(
            w_width > i_width,
            "W ({w_width}) should be wider than i ({i_width})"
        );

        // Wider string should measure wider
        let short = font.measure_width("Hi", 12.0);
        let long = font.measure_width("Hello World", 12.0);
        assert!(long > short);
    }

    #[test]
    fn line_height_positive() {
        let path = match find_test_font() {
            Some(p) => p,
            None => return,
        };

        let font = LoadedFont::from_file(&path).unwrap();
        let lh = font.line_height(12.0);
        assert!(lh > 0.0);
        // Line height is typically >= font size (ascender - descender + line gap)
        assert!(lh >= 10.0, "Line height {lh} should be reasonable for 12pt");
    }

    #[test]
    fn ascender_descender() {
        let path = match find_test_font() {
            Some(p) => p,
            None => return,
        };

        let font = LoadedFont::from_file(&path).unwrap();
        let asc = font.ascender(12.0);
        let desc = font.descender(12.0);
        assert!(asc > 0.0, "Ascender should be positive");
        assert!(desc < 0.0, "Descender should be negative");
    }

    #[test]
    fn has_glyph_check() {
        let path = match find_test_font() {
            Some(p) => p,
            None => return,
        };

        let font = LoadedFont::from_file(&path).unwrap();
        assert!(font.has_glyph('A'));
        assert!(font.has_glyph('0'));
        // Most Latin fonts won't have rare CJK characters
    }

    #[test]
    fn glyphs_for_text_set() {
        let path = match find_test_font() {
            Some(p) => p,
            None => return,
        };

        let font = LoadedFont::from_file(&path).unwrap();
        let glyphs = font.glyphs_for_text("Hello");
        // H, e, l, o — 4 unique characters
        assert_eq!(glyphs.len(), 4);
    }

    #[test]
    fn avg_char_width_reasonable() {
        let path = match find_test_font() {
            Some(p) => p,
            None => return,
        };

        let font = LoadedFont::from_file(&path).unwrap();
        let avg = font.avg_char_width(12.0);
        // Average character width should be between 3pt and 10pt at 12pt size
        assert!(avg > 3.0 && avg < 10.0, "avg_char_width = {avg}");
    }

    #[test]
    fn subset_font_reduces_size() {
        let path = match find_test_font() {
            Some(p) => p,
            None => return,
        };

        let data = std::fs::read(&path).unwrap();
        // Skip .ttc files (font collections) — subsetter may not support them
        if path.extension().and_then(|e| e.to_str()) == Some("ttc") {
            eprintln!("Skipping .ttc for subsetting test");
            return;
        }

        let subsetted = subset_font(&data, "Hello").unwrap();
        assert!(
            subsetted.len() < data.len(),
            "Subset ({}) should be smaller than original ({})",
            subsetted.len(),
            data.len()
        );
    }

    #[test]
    fn font_resolver_empty() {
        let resolver = FontResolver::empty();
        assert_eq!(resolver.indexed_count(), 0);
        assert_eq!(resolver.loaded_count(), 0);
    }

    #[test]
    fn font_resolver_register() {
        let path = match find_test_font() {
            Some(p) => p,
            None => return,
        };

        let data = std::fs::read(&path).unwrap();
        let mut resolver = FontResolver::empty();
        let desc = FontDescriptor::regular("TestFont");
        resolver.register_font(desc.clone(), data).unwrap();
        assert_eq!(resolver.loaded_count(), 1);

        let font = resolver.resolve(&desc);
        assert!(font.is_some());
    }

    #[test]
    fn font_resolver_system_scan() {
        let resolver = FontResolver::new();
        // On any modern OS, there should be at least some fonts
        // (But CI environments may have none, so don't hard-fail)
        if resolver.indexed_count() > 0 {
            eprintln!("Found {} system fonts", resolver.indexed_count());
        }
    }

    #[test]
    fn font_descriptor_equality() {
        let d1 = FontDescriptor::new("Arial", FontWeight::Bold, FontStyle::Normal);
        let d2 = FontDescriptor::new("Arial", FontWeight::Bold, FontStyle::Normal);
        let d3 = FontDescriptor::new("Arial", FontWeight::Normal, FontStyle::Normal);
        assert_eq!(d1, d2);
        assert_ne!(d1, d3);
    }

    #[test]
    fn file_not_found_error() {
        let result = LoadedFont::from_file(Path::new("/nonexistent/font.ttf"));
        assert!(result.is_err());
    }
}
