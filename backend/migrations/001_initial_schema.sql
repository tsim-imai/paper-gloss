-- Initial schema (v2 baseline) for paper-gloss
-- Created: 2025-10-24 (squashed)

-- =============================================================================
-- Papers table
-- =============================================================================
CREATE TABLE IF NOT EXISTS papers (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    source_url TEXT,
    file_path TEXT NOT NULL UNIQUE,
    status TEXT NOT NULL CHECK(status IN ('pending', 'processing', 'completed', 'failed')),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- pipeline timestamps & results
    translation_last_run_at TEXT,
    terms_jp_last_run_at TEXT,
    scan_jp_last_run_at TEXT,
    definitions_last_run_at TEXT,
    definitions_result_state TEXT,
    definitions_prompt_version TEXT,
    -- counts
    terms_jp_extracted_count INTEGER NOT NULL DEFAULT 0,
    definitions_generated_last INTEGER NOT NULL DEFAULT 0,
    definitions_failed_last INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_papers_status ON papers(status);

-- =============================================================================
-- Chunks table
-- =============================================================================
CREATE TABLE IF NOT EXISTS chunks (
    id TEXT PRIMARY KEY NOT NULL,
    paper_id TEXT NOT NULL,
    index_ INTEGER NOT NULL,
    src_text TEXT NOT NULL,
    trans_html TEXT,
    content_hash TEXT NOT NULL UNIQUE,
    token_count INTEGER,
    status TEXT NOT NULL DEFAULT 'pending' CHECK(status IN ('pending','translated','failed')),
    retry_count INTEGER NOT NULL DEFAULT 0,
    error_message TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (paper_id) REFERENCES papers(id) ON DELETE CASCADE,
    UNIQUE(paper_id, index_)
);

CREATE INDEX idx_chunks_paper_id ON chunks(paper_id);
CREATE INDEX idx_chunks_content_hash ON chunks(content_hash);

-- =============================================================================
-- Terms table (no pos/tags in v2)
-- =============================================================================
CREATE TABLE IF NOT EXISTS terms (
    id TEXT PRIMARY KEY NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    lemma_en TEXT NOT NULL,
    lemma_ja TEXT NOT NULL,
    reading_kana TEXT,
    note TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_terms_slug ON terms(slug);
CREATE INDEX idx_terms_lemma_en ON terms(lemma_en);
CREATE INDEX idx_terms_lemma_ja ON terms(lemma_ja);

-- =============================================================================
-- Term Variants table
-- =============================================================================
CREATE TABLE IF NOT EXISTS term_variants (
    id TEXT PRIMARY KEY NOT NULL,
    term_id TEXT NOT NULL,
    lang TEXT NOT NULL CHECK(lang IN ('en', 'ja')),
    surface TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (term_id) REFERENCES terms(id) ON DELETE CASCADE,
    UNIQUE(term_id, lang, surface)
);

CREATE INDEX idx_term_variants_term_id ON term_variants(term_id);
CREATE INDEX idx_term_variants_surface ON term_variants(surface);

-- =============================================================================
-- Definitions table
-- =============================================================================
CREATE TABLE IF NOT EXISTS definitions (
    id TEXT PRIMARY KEY NOT NULL,
    term_id TEXT NOT NULL UNIQUE,
    lang TEXT NOT NULL CHECK(lang IN ('ja')) DEFAULT 'ja',
    text TEXT NOT NULL,
    provider TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (term_id) REFERENCES terms(id) ON DELETE CASCADE
);

CREATE INDEX idx_definitions_term_id ON definitions(term_id);

-- =============================================================================
-- Occurrences table (extended for JP-first)
-- =============================================================================
CREATE TABLE IF NOT EXISTS occurrences (
    id TEXT PRIMARY KEY NOT NULL,
    term_id TEXT NOT NULL,
    paper_id TEXT NOT NULL,
    chunk_id TEXT NOT NULL,
    start_pos INTEGER NOT NULL,
    end_pos INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    surface TEXT,
    method TEXT NOT NULL DEFAULT 'jp-scan',
    variant_id TEXT,
    FOREIGN KEY (term_id) REFERENCES terms(id) ON DELETE CASCADE,
    FOREIGN KEY (paper_id) REFERENCES papers(id) ON DELETE CASCADE,
    FOREIGN KEY (chunk_id) REFERENCES chunks(id) ON DELETE CASCADE,
    UNIQUE(chunk_id, start_pos, end_pos),
    CHECK(start_pos < end_pos)
);

CREATE INDEX idx_occurrences_term_id ON occurrences(term_id);
CREATE INDEX idx_occurrences_paper_id ON occurrences(paper_id);
CREATE INDEX idx_occurrences_chunk_id ON occurrences(chunk_id);
CREATE INDEX idx_occurrences_method ON occurrences(method);

-- =============================================================================
-- Pipeline locks
-- =============================================================================
CREATE TABLE IF NOT EXISTS pipeline_locks (
    paper_id TEXT PRIMARY KEY,
    pipeline TEXT NOT NULL,
    locked_at DATETIME NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_pipeline_locks_locked_at ON pipeline_locks(locked_at);

-- =============================================================================
-- Aliases & Definition meta (v2)
-- =============================================================================
CREATE TABLE IF NOT EXISTS term_aliases (
    id TEXT PRIMARY KEY NOT NULL,
    term_id TEXT NOT NULL,
    surface TEXT NOT NULL,
    lang TEXT NOT NULL CHECK(lang IN ('en','ja')),
    kind TEXT NOT NULL CHECK(kind IN ('synonym','abbrev','alias')),
    confidence REAL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (term_id) REFERENCES terms(id) ON DELETE CASCADE,
    UNIQUE(term_id, lang, surface, kind)
);

CREATE INDEX IF NOT EXISTS idx_term_aliases_term_id ON term_aliases(term_id);

CREATE TABLE IF NOT EXISTS definition_meta (
    id TEXT PRIMARY KEY NOT NULL,
    term_id TEXT NOT NULL UNIQUE,
    provider TEXT,
    model TEXT,
    prompt_version TEXT,
    confidence REAL,
    flags TEXT,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (term_id) REFERENCES terms(id) ON DELETE CASCADE
);

-- =============================================================================
-- Triggers for updated_at timestamps
-- =============================================================================
CREATE TRIGGER update_papers_timestamp
AFTER UPDATE ON papers
FOR EACH ROW
BEGIN
    UPDATE papers SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

CREATE TRIGGER update_chunks_timestamp
AFTER UPDATE ON chunks
FOR EACH ROW
BEGIN
    UPDATE chunks SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

CREATE TRIGGER update_terms_timestamp
AFTER UPDATE ON terms
FOR EACH ROW
BEGIN
    UPDATE terms SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

CREATE TRIGGER update_definitions_timestamp
AFTER UPDATE ON definitions
FOR EACH ROW
BEGIN
    UPDATE definitions SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;
