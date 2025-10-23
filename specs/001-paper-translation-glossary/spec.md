# Feature Specification: Paper Translation & Glossary System

**Feature Branch**: `001-paper-translation-glossary`
**Created**: 2025-10-19
**Last Updated**: 2025-10-23
**Status**: Implementation (JP-first Architecture)
**Input**: Based on `@docs/requirements.md` and `@docs/pipeline-jp-first.md`

## Architecture Overview

This system uses a **JP-first (Japanese-first) pipeline architecture** where:
1. Papers are translated from English to Japanese
2. Technical terms are extracted from the **Japanese translation** (not the English source)
3. Terms are mechanically scanned and matched against the dictionary
4. Definitions are generated independently

Each processing step is exposed as an **independent API endpoint** that can be executed separately and re-run as needed.

---

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Read Translated Papers (Priority: P1)

A researcher downloads an English machine learning paper PDF and wants to read it in Japanese to understand the content more easily, while keeping the original PDF visible for reference (especially for figures, equations, and tables).

**Why this priority**: This is the core value proposition. Without translation viewing, the product has no purpose. This is the minimum viable feature that delivers immediate value.

**Independent Test**: Can be fully tested by uploading a PDF, triggering the translation pipeline, waiting for completion, and verifying that the translated text is displayed in a readable format with the option to view the original PDF side-by-side.

**Acceptance Scenarios**:

1. **Given** a user has a PDF file on their local machine, **When** they upload the PDF via the interface, **Then** the system accepts the file, stores it, and displays it with status `pending`
2. **Given** a user has an arXiv abstract URL (e.g., `https://arxiv.org/abs/2212.14578`), **When** they paste the URL and provide a paper title into the import form, **Then** the system validates the arXiv URL format, downloads the PDF from arXiv, stores it, and displays it with status `pending`
3. **Given** a PDF has been imported with status `pending`, **When** the user triggers the translation pipeline (Pipeline A), **Then** the system extracts text, chunks it, and begins translating each chunk to Japanese
4. **Given** translation is in progress, **When** the user views the processing status, **Then** they see granular progress indicators (completed chunks / total chunks) with percentage completion
5. **Given** translation has completed for a paper, **When** the user navigates to the translation view, **Then** they see the Japanese translated text organized by chunks matching the original structure
6. **Given** a user is viewing translated text, **When** they want to reference the original PDF, **Then** they can open the original PDF in a side-by-side or tabbed view
7. **Given** some chunks failed during translation, **When** the user retries the translation pipeline, **Then** the system only retries failed chunks without reprocessing successful ones

---

### User Story 2 - Understand ML Terms via Tooltips (Priority: P2)

A researcher is reading a translated paper and wants to quickly understand machine learning terms by seeing them highlighted with instant tooltip definitions.

**Why this priority**: This is the key differentiator that makes the tool more than just a translation service. Term comprehension support is the second most critical feature after basic translation viewing.

**Independent Test**: Can be tested by viewing a translated paper, triggering term extraction (Pipeline B) and scanning (Pipeline C), then hovering over highlighted terms and verifying that tooltips appear instantly with Japanese definitions, English original forms, and category tags.

**Acceptance Scenarios**:

1. **Given** a translated paper exists, **When** the user triggers term extraction (Pipeline B), **Then** the system analyzes the Japanese translation using LLM and extracts machine learning terms with both Japanese and English lemmas
2. **Given** terms have been extracted, **When** the user triggers JP scanning (Pipeline C), **Then** the system mechanically scans the Japanese translation and creates occurrence records linking terms to their positions in the text
3. **Given** JP scanning has completed, **When** the user views the translation, **Then** all extracted machine learning terms are visually highlighted (distinct but non-intrusive styling)
4. **Given** a term is highlighted in the translation, **When** the user hovers their cursor over it, **Then** a tooltip appears within 100 milliseconds showing the Japanese definition (2-3 sentences), English original form, and category tags
5. **Given** a tooltip is displayed, **When** the user clicks on the highlighted term, **Then** the tooltip remains open (sticky) until the user clicks outside or presses Escape
6. **Given** multiple occurrences of the same term in the text, **When** the user hovers over any occurrence, **Then** the same definition tooltip appears for all instances
7. **Given** the user wants definitions for extracted terms, **When** they trigger definition generation (Pipeline D), **Then** the system generates concise Japanese definitions (2-3 sentences) for terms that don't have definitions yet

---

### User Story 3 - Manage and Review Glossary (Priority: P3)

A researcher wants to review all extracted terms in a dedicated glossary view, search for specific terms in both English and Japanese, manually add new terms they noticed, edit definitions for clarity, and merge duplicate entries that represent the same concept.

**Why this priority**: This enables long-term knowledge building beyond single-paper reading. It's valuable but not essential for the initial MVP core workflow.

**Independent Test**: Can be tested by opening the glossary view, searching for terms in both languages, adding a custom term, editing an existing definition, and merging two duplicate term entries.

**Acceptance Scenarios**:

1. **Given** the user is viewing a translated paper, **When** they open the glossary panel (right pane or dedicated view), **Then** they see a list of all extracted terms sorted by frequency or alphabetically
2. **Given** the glossary is displayed, **When** the user types a search query in English or Japanese, **Then** the glossary filters in real-time to show only matching terms (with normalization handling katakana, hyphens, singular/plural variants)
3. **Given** the user wants to add a new term, **When** they click "Add Term" and fill in the English lemma, Japanese lemma, and optional notes, **Then** the new term is saved and appears in the glossary
4. **Given** the user sees a term with an unclear or incorrect definition, **When** they edit the definition text and save, **Then** the updated definition is stored and shown in future tooltips
5. **Given** the glossary contains duplicate entries (e.g., "neural network" vs "neural-network"), **When** the user selects two entries and clicks "Merge", **Then** the system prompts for confirmation showing both entries, and upon confirmation, merges them into a single canonical term with all variants preserved
6. **Given** a term exists in the glossary without a definition, **When** the user requests a definition regeneration for that specific term, **Then** the system calls the LLM to create a new 2-3 sentence Japanese explanation and updates the definition

---

### Edge Cases

- **What happens when a PDF cannot be parsed** (scanned images, corrupted files)? System should detect extraction failure, warn the user which pages failed, and allow partial processing to continue for extractable pages. User should be able to retry extraction after fixing the PDF.

- **What happens when LLM calls fail or timeout** during translation or term extraction? System should preserve partial results (translated chunks, extracted terms up to the failure point), display an error indicator for failed chunks, and allow the user to retry only the failed steps without reprocessing successful ones. Each pipeline tracks its own status independently.

- **What happens when the same term has multiple conflicting definitions** from different papers? System should store all definitions with source paper references and allow the user to choose a preferred definition or merge them manually.

- **What happens when users search for terms with ambiguous spellings** (e.g., "NN" could mean "neural network" or "nearest neighbor")? System should show all matching terms and allow users to refine the search or disambiguate by selecting the correct term from a list.

- **What happens when translation produces awkward or incorrect Japanese** for a specific chunk? User should be able to flag the chunk for manual review, optionally provide a corrected translation, and trigger a re-translation of that specific chunk.

- **What happens when a term appears hundreds of times in a long paper?** Tooltip performance must remain under 100ms. Occurrence tracking should be efficient enough to handle high-frequency terms without degradation. The JP scanning pipeline creates occurrence records efficiently using mechanical matching.

- **What happens when a user deletes a paper that is still being processed?** System should check for active pipeline locks, prevent deletion if any pipeline is running, and require the user to wait for completion. Once no pipelines are running, deletion proceeds: stop any pending tasks, clean up partial results (chunks, translations in progress), remove the paper from the database, delete the PDF file from storage, and remove associated occurrences. If terms become orphaned (no occurrences in any remaining papers), they should remain in the glossary for future use (no automatic cleanup).

- **What happens when deletion fails midway through?** (e.g., database deletion succeeds but file deletion fails) System should handle partial deletion gracefully, log the failure, and either retry or allow manual cleanup. The UI should show an appropriate error message.

- **What happens when a pipeline is triggered but another pipeline is already running?** System should return HTTP 409 Conflict with a message indicating which pipeline is currently running. Pipelines are mutually exclusive per paper to prevent data corruption.

- **What happens when term extraction (Pipeline B) or scanning (Pipeline C) produces 0 results?** System should complete successfully with status `completed_empty` (not `failed`). This is a valid outcome, especially if the paper has no ML-related terms or if the dictionary doesn't match any terms in the translation.

---

## Requirements *(mandatory)*

### Functional Requirements

#### Paper Import & Storage

- **FR-001**: System MUST accept PDF file uploads from the user's local filesystem
- **FR-002**: System MUST accept arXiv abstract URLs (format: `https://arxiv.org/abs/{arxiv_id}`) and automatically download PDF files from arXiv (MVP: arXiv-only, other sources out of scope)
- **FR-003**: System MUST store uploaded/downloaded PDFs in a persistent local file structure (`artifacts/papers/{paper_id}/source.pdf`)
- **FR-004**: System MUST store paper metadata (title, source URL, import timestamp, status) in persistent storage with initial status `pending`
- **FR-005**: System MUST detect when a PDF cannot be parsed or is corrupted, warn the user, and allow partial processing
- **FR-006**: System MUST provide an API endpoint to retrieve the stored PDF file for display (`GET /papers/{id}/file`)

#### Text Extraction & Chunking

- **FR-007**: System MUST extract text content from PDF files without requiring OCR (text-based PDFs only)
- **FR-008**: System MUST handle extraction failures gracefully by warning users about unparseable pages and continuing with extractable pages
- **FR-009**: System MUST divide extracted text into chunks for translation, prioritizing natural boundaries (paragraphs, sections) when possible
- **FR-010**: System MUST use chunk sizes between 800-1200 words (or equivalent character count) with 10-15% overlap to preserve context across chunk boundaries
- **FR-011**: System MUST cache chunk content using a hash to avoid reprocessing unchanged chunks

#### Pipeline A: Translation

- **FR-012**: System MUST provide an independent API endpoint `POST /papers/{id}/translate` to trigger translation of all chunks
- **FR-013**: System MUST translate each text chunk into Japanese using an external LLM service via OpenAI-compatible API endpoints
- **FR-014**: System MUST preserve mathematical notation, symbols, reference labels, and formatting in translated output
- **FR-015**: System MUST handle translation failures by preserving partial results and tracking failed chunks with `chunks.status = 'failed'` and error messages
- **FR-016**: System MUST limit concurrent LLM requests to a maximum configurable value (`AI_MAX_CONCURRENCY`, default 5) to respect rate limits
- **FR-017**: System MUST implement exponential backoff retry logic for transient LLM API failures
- **FR-018**: System MUST log all LLM prompts and responses to disk for reproducibility and debugging (`artifacts/papers/{paper_id}/llm_logs/`)
- **FR-019**: System MUST return HTTP 202 Accepted when translation pipeline is triggered, with a message directing users to check progress via `GET /papers/{id}/status`
- **FR-020**: System MUST allow retry of translation pipeline, reprocessing only failed chunks
- **FR-021**: System MUST prevent concurrent pipeline execution for the same paper using pipeline locks (return HTTP 409 Conflict if another pipeline is running)

#### Pipeline B: Japanese Term Extraction

- **FR-022**: System MUST provide an independent API endpoint `POST /papers/{id}/extract-terms-jp` to trigger term extraction from Japanese translation
- **FR-023**: System MUST extract machine learning-related terms from **Japanese translated text** (not English source) using LLM-based analysis
- **FR-024**: System MUST extract terms with the following attributes: `lemma_ja` (Japanese canonical form), `lemma_en` (English canonical form), `reading_kana` (hiragana reading), `pos` (part of speech), `variants_ja` (variant spellings in Japanese)
- **FR-025**: System MUST register extracted terms in the `terms` and `term_variants` tables, using `lemma_ja` as the primary form
- **FR-026**: System MUST detect and skip duplicate terms during extraction (using both exact matching and normalization)
- **FR-027**: System MUST complete successfully with status `completed_empty` if 0 terms are extracted (not treat as failure)
- **FR-028**: System MUST return HTTP 202 Accepted when term extraction pipeline is triggered
- **FR-029**: System MUST prevent term extraction if translation is not complete (return HTTP 409 Conflict or 412 Precondition Failed)

#### Pipeline C: Japanese Mechanical Scanning

- **FR-030**: System MUST provide an independent API endpoint `POST /papers/{id}/scan-jp` to trigger mechanical scanning of Japanese translation
- **FR-031**: System MUST scan Japanese translated text using `term_variants` with `lang='ja'` to find term occurrences
- **FR-032**: System MUST create occurrence records linking terms to their positions in translated chunks with `method='jp-scan'`
- **FR-033**: System MUST store occurrence surface form (the actual text matched), start position, end position, and reference to the matched variant
- **FR-034**: System MUST complete successfully with status `completed_empty` if 0 occurrences are found (not treat as failure)
- **FR-035**: System MUST support re-running JP scanning, clearing previous `occurrences` with `method='jp-scan'` for the paper and regenerating them with the latest dictionary
- **FR-036**: System MUST return HTTP 202 Accepted when JP scanning pipeline is triggered
- **FR-037**: System MUST prevent JP scanning if translation is not complete (return HTTP 409 Conflict or 412 Precondition Failed)

#### Pipeline D: Definition Generation

- **FR-038**: System MUST provide an independent API endpoint `POST /papers/{id}/generate-definitions` to trigger definition generation for terms related to the paper
- **FR-039**: System MUST generate Japanese definitions (2-3 sentences) for extracted terms using LLM calls
- **FR-040**: System MUST only generate definitions for terms that don't already have definitions (preserve existing definitions)
- **FR-041**: System MUST use concise, accessible language suitable for non-specialists in the field
- **FR-042**: System MUST complete successfully with status `completed_empty` if 0 definitions are generated (not treat as failure)
- **FR-043**: System MUST track definition generation results in `papers.definitions_result_state` (completed_nonempty | completed_empty | failed)
- **FR-044**: System MUST return HTTP 202 Accepted when definition generation pipeline is triggered
- **FR-045**: System MUST provide a per-term definition regeneration endpoint `POST /terms/{id}/define` for manual definition refresh

#### Translation Viewing & Term Highlighting

- **FR-046**: System MUST display translated text in a readable page view organized by chunks
- **FR-047**: System MUST inject HTML span elements with term highlighting into translated chunks after JP scanning completes: `<span class="term" data-term-id="{id}" data-occurrence-id="{id}">{surface}</span>`
- **FR-048**: System MUST style highlighted terms with visually distinct but non-intrusive CSS (suggested: subtle background color, no bold/underline by default)
- **FR-049**: System MUST display tooltips when users hover over or click highlighted terms
- **FR-050**: Tooltips MUST appear within 100 milliseconds of user interaction
- **FR-051**: Tooltips MUST show Japanese definition, English original form, reading (kana), and category tags if available
- **FR-052**: System MUST allow users to view the original PDF side-by-side with the translated text or in a separate tab via `GET /papers/{id}/file`
- **FR-053**: System MUST synchronize term highlighting across all occurrences of the same term or its variants

#### Progress Tracking & Error Recovery

- **FR-054**: System MUST provide `GET /papers/{id}/status` endpoint returning detailed status for all pipelines
- **FR-055**: Status response MUST include for each pipeline: total count, completed count, failed count, last run timestamp, and current status (idle | processing | completed | completed_empty | failed)
- **FR-056**: System MUST display granular progress indicators in the UI during paper processing (translation, term extraction, scanning, definitions) with percentage completion
- **FR-057**: System MUST allow users to retry failed processing steps independently without reprocessing successful steps
- **FR-058**: System MUST preserve all partial results when errors occur to prevent data loss
- **FR-059**: System MUST use pipeline locks (`pipeline_locks` table) to ensure only one pipeline runs per paper at a time

#### Glossary Management

- **FR-060**: System MUST support manual term creation via `POST /terms` with fields: `lemma_en`, `lemma_ja`, `reading_kana`, `pos`, `tags`, `note`, and optional `variants`
- **FR-061**: System MUST support term editing via `PATCH /terms/{id}`
- **FR-062**: System MUST support term deletion via `DELETE /terms/{id}`
- **FR-063**: System MUST support term merging via `POST /terms/merge` requiring explicit user confirmation
- **FR-064**: System MUST support bilingual search via `GET /terms?q={query}&lang={en|ja}` with normalization for katakana, hyphens, spaces, and basic singular/plural forms
- **FR-065**: System MUST detect and present duplicate term candidates to users for manual review
- **FR-066**: System MUST preserve all term variants when merging duplicate terms

#### Data Persistence & Export

- **FR-067**: System MUST persist all user data (papers, translations, terms, definitions, occurrences) in local SQLite storage that survives application restarts
- **FR-068**: System MUST support glossary export to standard formats (JSON, CSV)
- **FR-069**: System MUST use versioned, reversible database migrations to manage schema changes

#### Paper Management

- **FR-070**: System MUST allow users to delete imported papers via `DELETE /papers/{id}` along with all associated data (chunks, translations, occurrences, and stored PDF files)
- **FR-071**: System MUST require explicit user confirmation before deleting a paper to prevent accidental data loss
- **FR-072**: System MUST prevent paper deletion if any pipeline is currently running for that paper (return HTTP 409 Conflict)
- **FR-073**: System MUST cascade delete chunks, occurrences when a paper is deleted
- **FR-074**: System MUST NOT automatically delete orphaned terms (terms with no occurrences) when a paper is deleted, as they may be useful for future papers

---

### Key Entities *(include if feature involves data)*

- **Paper**: Represents a single imported scientific paper. Key attributes: unique identifier, title, source URL, file path to stored PDF, processing status (pending/processing/completed/failed), import timestamp, pipeline execution timestamps (translation_last_run_at, terms_jp_last_run_at, scan_jp_last_run_at, definitions_last_run_at), definition result state. Relationships: contains multiple Chunks; terms from this paper have Occurrences linked to it.

- **Chunk**: A segment of extracted text from a paper for translation purposes. Key attributes: unique identifier, parent paper reference, sequential index, source text (English), translated HTML/text (Japanese), content hash for caching, processing status (pending/translated/failed), retry count, error message. Relationships: belongs to one Paper; may contain multiple Term Occurrences.

- **Term**: A canonical concept/vocabulary entry in the glossary. Key attributes: unique identifier, slug for URL-safe lookup, English lemma (canonical form), Japanese lemma (canonical translation), reading (kana), part of speech, category tags, user notes, creation timestamp. Relationships: has multiple Term Variants (spelling variations); has one Definition; appears in multiple Occurrences across papers.

- **Term Variant**: A spelling or expression variation of a canonical term. Key attributes: unique identifier, parent term reference, language (English/Japanese), surface form (the actual string that appears in text). Relationships: belongs to one Term. Purpose: enables fuzzy matching and normalization (e.g., "neural network" vs "neural-network" vs "NN" all link to the same Term).

- **Definition**: A human-readable explanation of a term in Japanese. Key attributes: unique identifier, parent term reference, language (always Japanese for MVP), definition text (2-3 sentences), provider (LLM model name or "manual"), last updated timestamp. Relationships: belongs to one Term.

- **Occurrence**: A specific instance where a term appears in a translated paper. Key attributes: unique identifier, term reference, paper reference, chunk reference, start/end position, surface form (matched text), method ('jp-scan'), variant reference. Relationships: links a Term to a specific location in a Paper's Chunk. Purpose: enables highlighting and occurrence tracking.

- **Pipeline Lock**: Prevents concurrent pipeline execution for a paper. Key attributes: paper_id (primary key), pipeline name, locked timestamp. Purpose: ensures data integrity by preventing multiple pipelines from modifying the same paper simultaneously.

---

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can successfully upload a PDF or provide a URL, trigger the translation pipeline, and view the translated Japanese text within 5 minutes for a typical 10-page paper (assuming LLM response times of ~5-10 seconds per chunk)

- **SC-002**: Translated text preserves the original structure (paragraphs, sections) such that users can navigate the translation and original PDF in parallel without losing context

- **SC-003**: Users can trigger term extraction (Pipeline B) and scanning (Pipeline C) independently, with at least 80% of machine learning-related technical terms in a typical ML paper being automatically extracted and highlighted in the translation

- **SC-004**: Tooltips appear within 100 milliseconds of hover interaction, providing immediate term comprehension without interrupting reading flow

- **SC-005**: Users can search the glossary in both English and Japanese and find matching terms within 2 seconds, including fuzzy matches (katakana variants, hyphenation differences, singular/plural)

- **SC-006**: System successfully processes papers up to 50 pages (~400 chunks) without data loss or unrecoverable errors, preserving partial results if failures occur

- **SC-007**: Users can retry failed processing steps (individual pipelines) without needing to restart the entire workflow from scratch

- **SC-008**: Glossary data persists across application restarts, and users can export their glossary to standard formats for backup or external use

- **SC-009**: Users can manually add, edit, and merge terms in the glossary, and changes are immediately reflected in all future tooltip displays and scanning operations

- **SC-010**: System respects the configurable concurrent LLM request limit (`AI_MAX_CONCURRENCY`), processes papers within reasonable time (not timing out), and handles API failures gracefully with automatic retry

- **SC-011**: Each pipeline can be executed independently and re-run as needed without interfering with other pipelines or corrupting data

- **SC-012**: Pipelines that produce 0 results complete successfully with `completed_empty` status, allowing users to understand that the operation succeeded but had no applicable items to process

---

## Assumptions

- **A-001**: PDFs are text-based (not scanned images requiring OCR). OCR is explicitly out of scope for MVP.

- **A-002**: Users have access to a local or remote OpenAI-compatible LLM endpoint (e.g., localhost:8000/v1/chat/completions) with a maximum token context of 30,000 tokens and support for at least 5-10 concurrent requests (configurable via `AI_MAX_CONCURRENCY`).

- **A-003**: Papers are primarily in English. Translation is unidirectional (English → Japanese). Reverse translation or multi-language support is out of scope.

- **A-004**: Users are comfortable with command-line or simple web interface for initial setup (e.g., configuring LLM API base URL and key). No advanced authentication or multi-user support is required.

- **A-005**: Local disk storage is sufficient for storing PDFs, translations, and glossary data (single-user, personal use). Cloud sync or distributed storage is out of scope.

- **A-006**: Users are researchers, students, or professionals familiar with machine learning terminology who want comprehension support, not beginners needing extensive educational scaffolding.

- **A-007**: LLM-generated definitions and translations are treated as drafts that users can review and edit. Perfect accuracy is not guaranteed; user control and editability are prioritized.

- **A-008**: Figures, tables, and complex mathematical equations are referenced from the original PDF side-by-side view. Translation focuses on body text, not rendering of visual content.

- **A-009**: Term extraction from Japanese translation (JP-first) is more effective for Japanese readers than extracting English terms and translating them, as it captures how terms are actually expressed in the translated context.

- **A-010**: Users understand that each pipeline must be triggered manually and independently. There is no automatic cascade (e.g., importing a paper does not automatically trigger translation).

---

## Out of Scope (for this feature/MVP)

- OCR for scanned/image-based PDFs
- LaTeX math reconstruction or advanced equation rendering in translation
- Automatic figure/table extraction or interpretation
- English definitions or multi-language glossary (Japanese-only for MVP)
- Spaced Repetition System (SRS) or Anki export (future feature)
- Vector embeddings, semantic search, or term clustering (future feature)
- Multi-user support, authentication, or authorization (local single-user tool)
- Cloud sync, backup, or cross-device glossary synchronization
- Advanced analytics (term frequency trends, co-occurrence graphs)
- Automatic pipeline cascade (user must manually trigger each pipeline)
- Batch processing of multiple papers simultaneously
