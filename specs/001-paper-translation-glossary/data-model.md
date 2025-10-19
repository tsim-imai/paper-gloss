# Data Model: Paper Translation & Glossary System

**Date**: 2025-10-19
**Branch**: `001-paper-translation-glossary`
**Source**: Extracted from [spec.md](./spec.md) Key Entities section

---

## Overview

This document defines the database schema, entity relationships, validation rules, and state transitions for the paper translation and glossary system. The data model supports SQLite (via SQLx) with compile-time query checking and reversible migrations.

---

## Entity Relationship Diagram

```
┌─────────────┐
│   Paper     │
└──────┬──────┘
       │ 1
       │
       │ N
┌──────▼──────┐         ┌──────────────┐
│   Chunk     │ N ────1 │  Occurrence  │ N ───┐
└─────────────┘         └──────────────┘      │
                                               │
                                               │ 1
                                         ┌─────▼──────┐
                                         │    Term    │
                                         └──────┬─────┘
                                                │ 1
                              ┌─────────────────┼─────────────────┐
                              │ N               │ N               │ 1
                        ┌─────▼────────┐  ┌────▼────────┐  ┌─────▼──────────┐
                        │ TermVariant  │  │ Occurrence  │  │  Definition    │
                        └──────────────┘  └─────────────┘  └────────────────┘
```

---

## Entities

### 1. Paper

**Purpose**: Represents a single imported scientific paper (PDF).

**Fields**:

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | UUID | PRIMARY KEY | Unique identifier (UUID v4) |
| `title` | TEXT | NOT NULL | Paper title (extracted from PDF metadata or user input) |
| `source_url` | TEXT | NULLABLE | Original download URL if imported via URL |
| `file_path` | TEXT | NOT NULL, UNIQUE | Relative path to stored PDF (`artifacts/papers/{id}/source.pdf`) |
| `status` | TEXT | NOT NULL, CHECK | Processing status: `pending`, `processing`, `completed`, `failed` |
| `created_at` | TIMESTAMP | NOT NULL, DEFAULT CURRENT_TIMESTAMP | Import timestamp |
| `updated_at` | TIMESTAMP | NOT NULL, DEFAULT CURRENT_TIMESTAMP | Last modification timestamp |

**Relationships**:
- **1:N** with `Chunk` (one paper has many chunks)
- **1:N** with `Occurrence` (via chunks) (terms appear in this paper)

**State Transitions**:
```
pending → processing → completed
                    ↘ failed
```

**Validation Rules**:
- `status` MUST be one of: `pending`, `processing`, `completed`, `failed`
- `file_path` MUST be unique (prevent duplicate imports of same file)
- `title` SHOULD be trimmed and non-empty (validated at application layer)

**Indexes**:
- PRIMARY KEY on `id`
- UNIQUE INDEX on `file_path`
- INDEX on `status` (for filtering papers by processing state)

---

### 2. Chunk

**Purpose**: A segment of extracted text from a paper for translation purposes.

**Fields**:

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | UUID | PRIMARY KEY | Unique identifier (UUID v4) |
| `paper_id` | UUID | NOT NULL, FOREIGN KEY | References `Paper.id` (ON DELETE CASCADE) |
| `index` | INTEGER | NOT NULL | Sequential index within the paper (0-based, for ordering) |
| `src_text` | TEXT | NOT NULL | Source text (English) extracted from PDF |
| `trans_html` | TEXT | NULLABLE | Translated HTML/text (Japanese), NULL if not yet translated |
| `content_hash` | TEXT | NOT NULL, UNIQUE | SHA-256 hash of `src_text` for caching (avoid retranslation) |
| `token_count` | INTEGER | NULLABLE | Estimated token count for LLM API rate limiting |
| `created_at` | TIMESTAMP | NOT NULL, DEFAULT CURRENT_TIMESTAMP | Chunk creation timestamp |
| `updated_at` | TIMESTAMP | NOT NULL, DEFAULT CURRENT_TIMESTAMP | Last modification timestamp |

**Relationships**:
- **N:1** with `Paper` (many chunks belong to one paper)
- **1:N** with `Occurrence` (terms appear in this chunk)

**Validation Rules**:
- `index` MUST be >= 0 and unique per `paper_id` (composite unique constraint)
- `content_hash` MUST be unique globally (prevents duplicate chunks across papers)
- `src_text` SHOULD be non-empty (validated at application layer)
- `trans_html` SHOULD be valid HTML if present (basic tag validation)

**Indexes**:
- PRIMARY KEY on `id`
- UNIQUE INDEX on `(paper_id, index)` (ordering within paper)
- UNIQUE INDEX on `content_hash` (caching lookup)
- FOREIGN KEY index on `paper_id`

---

### 3. Term

**Purpose**: A canonical concept/vocabulary entry in the glossary.

**Fields**:

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | UUID | PRIMARY KEY | Unique identifier (UUID v4) |
| `slug` | TEXT | NOT NULL, UNIQUE | URL-safe lookup key (kebab-case English lemma) |
| `lemma_en` | TEXT | NOT NULL | English canonical form (e.g., "neural network") |
| `lemma_ja` | TEXT | NOT NULL | Japanese canonical translation (e.g., "ニューラルネットワーク") |
| `reading_kana` | TEXT | NULLABLE | Japanese reading in hiragana (e.g., "にゅーらるねっとわーく") |
| `pos` | TEXT | NULLABLE | Part of speech (e.g., "noun", "verb", "adjective") |
| `tags` | TEXT | NULLABLE | Comma-separated category tags (e.g., "machine-learning,deep-learning") |
| `note` | TEXT | NULLABLE | User-editable notes |
| `created_at` | TIMESTAMP | NOT NULL, DEFAULT CURRENT_TIMESTAMP | Term creation timestamp |
| `updated_at` | TIMESTAMP | NOT NULL, DEFAULT CURRENT_TIMESTAMP | Last modification timestamp |

**Relationships**:
- **1:N** with `TermVariant` (one term has many spelling variations)
- **1:1** with `Definition` (one term has one definition)
- **1:N** with `Occurrence` (term appears in multiple locations)

**Validation Rules**:
- `slug` MUST be unique, lowercase, alphanumeric with hyphens only
- `lemma_en` SHOULD be non-empty and trimmed
- `lemma_ja` SHOULD be non-empty and trimmed
- `tags` SHOULD be comma-separated lowercase keywords (validated at application layer)

**Indexes**:
- PRIMARY KEY on `id`
- UNIQUE INDEX on `slug`
- INDEX on `lemma_en` (for English search)
- INDEX on `lemma_ja` (for Japanese search)
- FULLTEXT INDEX on `tags` (for category filtering, if SQLite FTS enabled)

---

### 4. TermVariant

**Purpose**: A spelling or expression variation of a canonical term.

**Fields**:

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | UUID | PRIMARY KEY | Unique identifier (UUID v4) |
| `term_id` | UUID | NOT NULL, FOREIGN KEY | References `Term.id` (ON DELETE CASCADE) |
| `lang` | TEXT | NOT NULL, CHECK | Language code: `en` (English) or `ja` (Japanese) |
| `surface` | TEXT | NOT NULL | Surface form as it appears in text (e.g., "neural-network", "NN", "neural net") |
| `created_at` | TIMESTAMP | NOT NULL, DEFAULT CURRENT_TIMESTAMP | Variant creation timestamp |

**Relationships**:
- **N:1** with `Term` (many variants belong to one canonical term)

**Validation Rules**:
- `lang` MUST be one of: `en`, `ja`
- `surface` MUST be non-empty
- Composite UNIQUE constraint on `(term_id, lang, surface)` (prevent duplicate variants)

**Indexes**:
- PRIMARY KEY on `id`
- UNIQUE INDEX on `(term_id, lang, surface)`
- INDEX on `surface` (for fast variant lookup during highlighting)
- FOREIGN KEY index on `term_id`

**Purpose**: Enables fuzzy matching and normalization (e.g., "neural network" vs "neural-network" vs "NN" all link to the same Term).

---

### 5. Definition

**Purpose**: A human-readable explanation of a term in Japanese.

**Fields**:

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | UUID | PRIMARY KEY | Unique identifier (UUID v4) |
| `term_id` | UUID | NOT NULL, UNIQUE, FOREIGN KEY | References `Term.id` (ON DELETE CASCADE) - one definition per term |
| `lang` | TEXT | NOT NULL, CHECK, DEFAULT 'ja' | Language code (always `ja` for MVP) |
| `text` | TEXT | NOT NULL | Definition text (2-3 sentences, Japanese) |
| `provider` | TEXT | NOT NULL | Source of definition: LLM model name (e.g., "gpt-4") or "manual" |
| `created_at` | TIMESTAMP | NOT NULL, DEFAULT CURRENT_TIMESTAMP | Definition creation timestamp |
| `updated_at` | TIMESTAMP | NOT NULL, DEFAULT CURRENT_TIMESTAMP | Last modification timestamp |

**Relationships**:
- **1:1** with `Term` (one definition per term, enforced by UNIQUE constraint on `term_id`)

**Validation Rules**:
- `lang` MUST be `ja` (Japanese-only for MVP)
- `text` MUST be non-empty and trimmed
- `provider` SHOULD indicate LLM model or "manual"

**Indexes**:
- PRIMARY KEY on `id`
- UNIQUE INDEX on `term_id` (enforces 1:1 relationship)
- FOREIGN KEY index on `term_id`

---

### 6. Occurrence

**Purpose**: A specific instance where a term appears in a translated paper.

**Fields**:

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | UUID | PRIMARY KEY | Unique identifier (UUID v4) |
| `term_id` | UUID | NOT NULL, FOREIGN KEY | References `Term.id` (ON DELETE CASCADE) |
| `paper_id` | UUID | NOT NULL, FOREIGN KEY | References `Paper.id` (ON DELETE CASCADE) |
| `chunk_id` | UUID | NOT NULL, FOREIGN KEY | References `Chunk.id` (ON DELETE CASCADE) |
| `start_pos` | INTEGER | NOT NULL | Start position in `trans_html` (character offset or HTML marker ID) |
| `end_pos` | INTEGER | NOT NULL | End position in `trans_html` (character offset or HTML marker ID) |
| `created_at` | TIMESTAMP | NOT NULL, DEFAULT CURRENT_TIMESTAMP | Occurrence detection timestamp |

**Relationships**:
- **N:1** with `Term` (many occurrences of one term)
- **N:1** with `Paper` (occurrences in one paper)
- **N:1** with `Chunk` (occurrences in one chunk)

**Validation Rules**:
- `start_pos` MUST be < `end_pos`
- Composite UNIQUE constraint on `(chunk_id, start_pos, end_pos)` (prevent duplicate highlights in same position)

**Indexes**:
- PRIMARY KEY on `id`
- INDEX on `term_id` (for finding all occurrences of a term)
- INDEX on `paper_id` (for finding all occurrences in a paper)
- INDEX on `chunk_id` (for finding all occurrences in a chunk)
- UNIQUE INDEX on `(chunk_id, start_pos, end_pos)`
- FOREIGN KEY indexes on `term_id`, `paper_id`, `chunk_id`

**Purpose**: Enables highlighting and occurrence tracking. Supports navigation from glossary to specific paper locations.

---

## Module Dependency Graph (Acyclic Verification)

```
┌─────────────────────────┐
│  Database Layer         │  ← SQLx connection pool, schema
└────────────┬────────────┘
             ↑
┌────────────┴────────────┐
│  Persistence Service    │  ← CRUD operations for all entities
└────────────┬────────────┘
             ↑
┌────────────┴────────────────────────────────┐
│  Business Logic Services (independent)      │
├─────────────────────────────────────────────┤
│  • PDF Processing    (src_text → chunks)    │
│  • Term Extraction   (src_text → terms)     │
│  • Term Normalization (fuzzy matching)      │
└────────────┬────────────────────────────────┘
             ↑                    ↑
             │                    │
             │              ┌─────┴──────────┐
             │              │  LLM Client    │  ← reqwest, no internal deps
             │              └────────────────┘
             │
┌────────────┴────────────┐
│  API Layer              │  ← Axum routes + utoipa schemas
└─────────────────────────┘
```

**Validation**:
- ✅ No circular dependencies
- ✅ Each layer depends only on layers below
- ✅ LLM Client is independent (only external HTTP deps)
- ✅ Business logic services can be tested in isolation

---

## Normalization & Search Strategy

### English Term Normalization

**Goal**: Match variants like "neural network", "neural-network", "NN" to same canonical term.

**Approach**:
1. Store all variants in `TermVariant` table with `lang='en'`
2. Search query normalization:
   - Lowercase conversion
   - Hyphen/space normalization (treat as equivalent)
   - Basic stemming (optional: "networks" → "network")
3. Query `TermVariant.surface` with normalized search term
4. Return parent `Term`

### Japanese Term Normalization

**Goal**: Match variants like "ニューラルネットワーク", "ニューラル・ネットワーク", "ニューラルﾈｯﾄﾜｰｸ" to same canonical term.

**Approach**:
1. Store all variants in `TermVariant` table with `lang='ja'`
2. Search query normalization:
   - Full-width/half-width katakana conversion
   - Middle dot (・) removal
   - Long vowel mark (ー) normalization
3. Query `TermVariant.surface` with normalized search term
4. Return parent `Term`

**Implementation**: Normalization logic in `backend/src/services/terms/normalization.rs`

---

## Migration Strategy

### Initial Schema Migration

**File**: `backend/migrations/001_initial_schema.sql`

**Up Migration**:
- CREATE TABLES for all 6 entities
- CREATE INDEXES and FOREIGN KEYS
- ADD CHECK constraints for enums (`status`, `lang`)
- CREATE UUIDs extension if needed (SQLite built-in via `randomblob(16)`)

**Down Migration**:
- DROP TABLES in reverse dependency order (Occurrence → TermVariant/Definition → Term → Chunk → Paper)

### Future Migrations

- Versioned and reversible (up/down)
- Applied via `sqlx migrate run`
- Compile-time verified via `sqlx::query!` macros

**Example**:
```bash
sqlx migrate add add_term_frequency_column
# Edit up/down SQL
sqlx migrate run
```

---

## Data Integrity Constraints (Constitution Principle V)

1. **Cascade Deletes**:
   - Deleting a `Paper` cascades to `Chunk` and `Occurrence`
   - Deleting a `Term` cascades to `TermVariant`, `Definition`, and `Occurrence`
   - Prevents orphaned records

2. **Unique Constraints**:
   - `Paper.file_path` prevents duplicate imports
   - `Chunk.content_hash` enables caching (avoid retranslation)
   - `(Chunk.paper_id, Chunk.index)` ensures ordered chunks
   - `Term.slug` ensures unique URLs
   - `Definition.term_id` enforces 1:1 relationship

3. **Timestamps**:
   - `created_at` and `updated_at` on all entities for audit trail
   - `updated_at` auto-updated via trigger or application layer

4. **Reversible Migrations**:
   - All migrations MUST have up/down SQL
   - Enables rollback if schema change causes issues

---

## Example Queries

### Find Term by English or Japanese Search

```sql
-- Normalized search (application layer normalizes query first)
SELECT t.* FROM terms t
JOIN term_variants tv ON tv.term_id = t.id
WHERE tv.surface = ? AND tv.lang = ?
LIMIT 1;
```

### Get All Occurrences of a Term in a Paper

```sql
SELECT o.*, c.index AS chunk_index, c.trans_html
FROM occurrences o
JOIN chunks c ON o.chunk_id = c.id
WHERE o.term_id = ? AND o.paper_id = ?
ORDER BY c.index, o.start_pos;
```

### List Papers by Status

```sql
SELECT * FROM papers
WHERE status = 'completed'
ORDER BY created_at DESC
LIMIT 10;
```

### Get Term with Definition and Variants

```sql
SELECT
  t.*,
  d.text AS definition,
  d.provider AS definition_provider,
  GROUP_CONCAT(tv.surface, ', ') AS variants
FROM terms t
LEFT JOIN definitions d ON d.term_id = t.id
LEFT JOIN term_variants tv ON tv.term_id = t.id
WHERE t.id = ?
GROUP BY t.id;
```

---

## Next Steps

1. **Generate OpenAPI Contracts**: Define API endpoints in `contracts/` directory
2. **Implement Schema in SQLx**: Create migration files in `backend/migrations/`
3. **Verify Compile-Time Queries**: Use `sqlx::query!` macros for type safety
4. **Generate quickstart.md**: Document setup, migration commands, and database backup procedures

---

**Status**: ✅ Complete - Ready for Phase 1 continuation (API contracts, quickstart)
