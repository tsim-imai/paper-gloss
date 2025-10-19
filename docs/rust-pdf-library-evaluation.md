# Rust PDF Text Extraction Libraries Evaluation

**Date**: 2025-10-19
**Purpose**: Extract text from English scientific papers (ML domain) in text-based PDFs
**Requirements**: No OCR, handle extraction failures gracefully, chunk by natural boundaries

---

## Decision: Recommended Library

### Primary Recommendation: `pdf-extract`

**Rationale**:
- **Simplicity**: Designed specifically for straightforward text extraction with minimal boilerplate
- **Actively maintained**: Latest version 0.9.0 (April 2025), with consistent updates throughout 2024-2025
- **Permissive license**: MIT licensed with no GPL restrictions
- **Pure Rust**: Minimal external dependencies, builds on `lopdf` for PDF parsing
- **Cross-platform**: Works on macOS, Linux, and Windows without platform-specific dependencies

**Basic usage**:
```rust
let bytes = std::fs::read("paper.pdf").unwrap();
let text = pdf_extract::extract_text_from_mem(&bytes).unwrap();
```

---

## Alternative Options Evaluated

### 1. `lopdf` - Low-Level PDF Manipulation

**Strengths**:
- Most popular Rust PDF library for general PDF manipulation
- Latest version 0.37.0 (August 2025), actively maintained
- MIT licensed
- Two parser options: `nom_parser` (faster) and `pom_parser`
- Excellent for complex PDF operations beyond text extraction

**Weaknesses**:
- Requires understanding of PDF specification for advanced use
- More verbose for simple text extraction compared to `pdf-extract`
- Overkill if you only need text extraction

**Use case**: Choose this if you need fine-grained control over PDF structure or plan to manipulate PDFs beyond text extraction.

**License**: MIT

---

### 2. `pdfium-render` - Google Chromium's PDF Library

**Strengths**:
- Based on Google Chromium's battle-tested PDFium library
- High-quality rendering and text extraction
- Comprehensive feature set (rendering to images, text search, etc.)
- Apache 2.0 / BSD-3-Clause licensed (permissive)

**Weaknesses**:
- **Complex setup**: Requires pre-built PDFium binaries or building PDFium yourself
- **External C++ dependency**: Not pure Rust, must link against libpdfium
- **Platform-specific configuration**: Requires proper library naming and paths
- May need to link against C++ standard library (`libstdc++` or `libc++`)
- macOS may require linking against CoreGraphics framework

**Setup complexity**:
- Download pre-built binaries from https://github.com/bblanchon/pdfium-binaries
- Configure `PDFIUM_DYNAMIC_LIB_PATH` or `PDFIUM_STATIC_LIB_PATH`
- Ensure correct library naming for your platform (`libpdfium.so`, `libpdfium.a`, etc.)

**Use case**: Only choose this if you need advanced PDF rendering features beyond text extraction.

**License**: Apache 2.0 / BSD-3-Clause (permissive)

---

### 3. `poppler-rs` - Poppler Bindings

**Strengths**:
- Bindings to the well-established Poppler library
- Proven text extraction quality

**Critical Weakness**:
- **GPL licensed**: Poppler is based on GPL-licensed xpdf-3.0
- **License contamination**: Any program linking against this must be GPL licensed
- **Blocks commercial use**: Cannot use in closed-source commercial applications without GPL compliance

**Verdict**: ❌ **Avoid** unless your project is GPL-compatible.

**License**: GPL (incompatible with commercial closed-source use)

---

### 4. `mupdf-rs` / `mupdf-sys` - MuPDF Bindings

**Strengths**:
- MuPDF is known for excellent handling of complex documents

**Weaknesses**:
- Only raw `bindgen` bindings available (`mupdf-sys`)
- No high-level Rust abstractions
- Requires manual low-level work
- Less documented than alternatives

**Verdict**: Requires too much low-level work compared to `pdf-extract`.

---

### 5. `oxidize-pdf` - New Pure Rust Alternative (2024)

**Strengths**:
- 100% pure Rust with zero C dependencies
- Optimized for document content extraction and batch processing
- Tested on 759 real-world PDFs with 98.8% corruption recovery
- OCR support (Tesseract integration)
- AI/RAG integration features

**Critical Weakness**:
- **AGPL-3.0 licensed**: Strong copyleft, requires open-sourcing your code
- **SaaS restriction**: AGPL specifically targets SaaS applications
- Relatively new, less battle-tested than alternatives

**Verdict**: ❌ **Avoid** due to AGPL license restrictions unless your project is AGPL-compatible.

**License**: AGPL-3.0 (incompatible with commercial closed-source use)

---

## Evaluation Matrix

| Library | License | Pure Rust | Setup Complexity | Text Extraction | Maintenance | Commercial Use |
|---------|---------|-----------|------------------|-----------------|-------------|----------------|
| **pdf-extract** | MIT | ✅ Yes | ⭐ Simple | ⭐⭐⭐ Excellent | ⭐⭐⭐ Active | ✅ Yes |
| lopdf | MIT | ✅ Yes | ⭐⭐ Moderate | ⭐⭐ Good | ⭐⭐⭐ Active | ✅ Yes |
| pdfium-render | Apache/BSD | ❌ No (C++) | ⭐⭐⭐ Complex | ⭐⭐⭐ Excellent | ⭐⭐ Moderate | ✅ Yes |
| poppler-rs | **GPL** | ❌ No (C) | ⭐⭐ Moderate | ⭐⭐⭐ Excellent | ⭐ Low | ❌ No |
| oxidize-pdf | **AGPL** | ✅ Yes | ⭐ Simple | ⭐⭐ Good | ⭐⭐ Growing | ❌ No |
| mupdf-rs | Various | ❌ No (C) | ⭐⭐⭐ Complex | ⭐⭐ Good | ⭐ Low | ⚠️ Check |

---

## Implementation Recommendation

### For Scientific Paper Text Extraction

```rust
// Cargo.toml
[dependencies]
pdf-extract = "0.9"

// Example implementation
use std::fs;
use pdf_extract::extract_text_from_mem;

fn extract_paper_text(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let bytes = fs::read(path)?;
    let text = extract_text_from_mem(&bytes)?;
    Ok(text)
}

// With error handling
fn extract_with_fallback(path: &str) -> Option<String> {
    match fs::read(path) {
        Ok(bytes) => extract_text_from_mem(&bytes).ok(),
        Err(_) => None
    }
}
```

### Chunking Strategy

`pdf-extract` returns plain text. For chunking by natural boundaries:

1. **Post-processing approach**: Parse the extracted text to identify paragraphs and sections
2. Use regex patterns to detect section headers (e.g., "Abstract", "Introduction", numbered sections)
3. Split on double newlines for paragraph boundaries
4. Consider additional NLP libraries for more sophisticated boundary detection

### Fallback Strategy

```rust
// Graceful degradation
fn try_extract_text(path: &str) -> Result<String, ExtractionError> {
    // Try pdf-extract first
    if let Ok(bytes) = fs::read(path) {
        if let Ok(text) = extract_text_from_mem(&bytes) {
            return Ok(text);
        }
    }

    // If pdf-extract fails, could fall back to lopdf for more control
    // or return structured error for logging
    Err(ExtractionError::ExtractionFailed)
}
```

---

## Scientific Paper Considerations

### Known Challenges

Scientific papers often have:
- **Multi-column layouts**: Text extraction may not preserve reading order
- **Equations and symbols**: May not extract correctly as plain text
- **Tables and figures**: May produce garbled text
- **Headers/footers**: Will be included in extraction

### Mitigation Strategies

1. **Layout awareness**: Consider post-processing to detect and reorder multi-column text
2. **Filtering**: Remove headers, footers, and page numbers via heuristics
3. **Quality checks**: Validate extracted text (character ratio, length, etc.)
4. **Specialized parsers**: For critical metadata (title, authors), consider dedicated scientific PDF parsers like GROBID (external tool, not Rust)

---

## Final Recommendation

**Use `pdf-extract` (MIT licensed)** as your primary library for the following reasons:

1. ✅ **Fit for purpose**: Designed specifically for text extraction
2. ✅ **License compatible**: MIT allows commercial use
3. ✅ **Low maintenance**: Pure Rust, no external dependencies
4. ✅ **Cross-platform**: Works on macOS, Linux, Windows out of the box
5. ✅ **Actively maintained**: Regular updates throughout 2024-2025
6. ✅ **Simple API**: Minimal boilerplate code
7. ✅ **Community support**: Most recommended for simple text extraction

**Backup option**: Keep `lopdf` in mind if you need more control or encounter PDFs that `pdf-extract` can't handle. Both are MIT licensed and work well together.

**Avoid**: `poppler-rs` (GPL), `oxidize-pdf` (AGPL), and `pdfium-render` (too complex for simple text extraction).

---

## References

- pdf-extract: https://crates.io/crates/pdf-extract
- lopdf: https://crates.io/crates/lopdf
- pdfium-render: https://crates.io/crates/pdfium-render
- Scientific PDF extraction challenges: https://arxiv.org/abs/2010.12647
