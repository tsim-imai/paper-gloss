-- Schema v2 for glossary: drop POS/tags, add aliases and definition_meta
-- Date: 2025-10-24

PRAGMA foreign_keys = OFF;
BEGIN TRANSACTION;

-- Recreate terms without pos/tags
CREATE TABLE IF NOT EXISTS terms_new (
    id TEXT PRIMARY KEY NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    lemma_en TEXT NOT NULL,
    lemma_ja TEXT NOT NULL,
    reading_kana TEXT,
    note TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO terms_new (id, slug, lemma_en, lemma_ja, reading_kana, note, created_at, updated_at)
SELECT id, slug, lemma_en, lemma_ja, reading_kana, note, created_at, updated_at
FROM terms;

DROP TABLE terms;
ALTER TABLE terms_new RENAME TO terms;

CREATE INDEX IF NOT EXISTS idx_terms_slug ON terms(slug);
CREATE INDEX IF NOT EXISTS idx_terms_lemma_en ON terms(lemma_en);
CREATE INDEX IF NOT EXISTS idx_terms_lemma_ja ON terms(lemma_ja);

-- New: term_aliases (semantic synonyms/abbreviations)
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

-- New: definition_meta (generation metadata/quality)
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

COMMIT;
PRAGMA foreign_keys = ON;

