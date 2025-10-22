# Feature Specification: Paper Translation & Glossary System

**Feature Branch**: `001-paper-translation-glossary`
**Created**: 2025-10-19
**Status**: Draft
**Input**: User description: "@docs/requirements.md を基に仕様を作成してください"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Read Translated Papers (Priority: P1)

A researcher downloads an English machine learning paper PDF and wants to read it in Japanese to understand the content more easily, while keeping the original PDF visible for reference (especially for figures, equations, and tables).

**Why this priority**: This is the core value proposition. Without translation viewing, the product has no purpose. This is the minimum viable feature that delivers immediate value.

**Independent Test**: Can be fully tested by uploading a PDF, waiting for translation to complete, and verifying that the translated text is displayed in a readable format with the option to view the original PDF side-by-side.

**Acceptance Scenarios**:

1. **Given** a user has a PDF file on their local machine, **When** they upload the PDF via the interface, **Then** the system accepts the file, stores it, and displays a processing status indicator
2. **Given** a user has an arXiv abstract URL (e.g., `https://arxiv.org/abs/2212.14578`), **When** they paste the URL and provide a paper title into the import form, **Then** the system validates the arXiv URL format, downloads the PDF from arXiv, stores it, and displays a processing status indicator
3. **Given** a PDF has been imported and is being processed, **When** the user views the processing status, **Then** they see granular progress indicators (extraction → chunking → translation) with percentage completion
4. **Given** translation has completed for a paper, **When** the user navigates to the translation view, **Then** they see the Japanese translated text organized by sections/paragraphs matching the original structure
5. **Given** a user is viewing translated text, **When** they want to reference the original PDF, **Then** they can open the original PDF in a side-by-side or tabbed view

---

### User Story 2 - Understand ML Terms via Tooltips (Priority: P2)

A researcher is reading a translated paper and encounters highlighted machine learning terms. They want to quickly understand what each term means without interrupting their reading flow by hovering over or clicking the highlighted terms.

**Why this priority**: This is the key differentiator that makes the tool more than just a translation service. Term comprehension support is the second most critical feature after basic translation viewing.

**Independent Test**: Can be tested by viewing a translated paper with extracted terms, hovering over highlighted terms, and verifying that tooltips appear instantly with Japanese definitions, English original forms, and category tags.

**Acceptance Scenarios**:

1. **Given** a translated paper with extracted terms, **When** the user views the translation, **Then** all extracted machine learning terms are visually highlighted (distinct but non-intrusive styling)
2. **Given** a term is highlighted in the translation, **When** the user hovers their cursor over it, **Then** a tooltip appears within 100 milliseconds showing the Japanese definition (2-3 sentences), English original form, and category tags
3. **Given** a tooltip is displayed, **When** the user clicks on the highlighted term, **Then** the tooltip remains open (sticky) until the user clicks outside or presses Escape
4. **Given** multiple occurrences of the same term in the text, **When** the user hovers over any occurrence, **Then** the same definition tooltip appears for all instances
5. **Given** a term has variant spellings or Japanese expressions in the text, **When** the user hovers over any variant, **Then** the tooltip shows the canonical form with all known variants listed

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
6. **Given** a term exists in the glossary, **When** the user requests a regenerated AI definition, **Then** the system calls the LLM to create a new 2-3 sentence Japanese explanation and updates the definition

---

### Edge Cases

- **What happens when a PDF cannot be parsed** (scanned images, corrupted files)? System should detect extraction failure, warn the user which pages failed, and allow partial processing to continue for extractable pages. User should be able to retry failed pages individually.

- **What happens when LLM calls fail or timeout** during translation or term extraction? System should preserve partial results (translated chunks, extracted terms up to the failure point), display an error indicator for failed chunks, and allow the user to retry only the failed chunks without reprocessing the entire paper.

- **What happens when the same term has multiple conflicting definitions** from different papers? System should store all definitions with source paper references and allow the user to choose a preferred definition or merge them manually.

- **What happens when users search for terms with ambiguous spellings** (e.g., "NN" could mean "neural network" or "nearest neighbor")? System should show all matching terms and allow users to refine the search or disambiguate by selecting the correct term from a list.

- **What happens when translation produces awkward or incorrect Japanese** for a specific chunk? User should be able to flag the chunk for manual review, optionally provide a corrected translation, and trigger a regeneration using a different LLM prompt.

- **What happens when a term appears hundreds of times in a long paper?** Tooltip performance must remain under 100ms. Occurrence tracking should be efficient enough to handle high-frequency terms without degradation.

- **What happens when a user deletes a paper that is still being processed?** System should stop the background processing tasks, clean up any partial results (chunks, translations in progress), remove the paper from the database, delete the PDF file from storage, and remove associated occurrences. If terms become orphaned (no occurrences in any remaining papers), they should be automatically cleaned up or flagged for user review.

- **What happens when deletion fails midway through?** (e.g., database deletion succeeds but file deletion fails) System should handle partial deletion gracefully, log the failure, and either retry or allow manual cleanup. The UI should show an appropriate error message.

## Requirements *(mandatory)*

### Functional Requirements

#### Paper Import & Storage

- **FR-001**: System MUST accept PDF file uploads from the user's local filesystem
- **FR-002**: System MUST accept arXiv abstract URLs (format: `https://arxiv.org/abs/{arxiv_id}`) and automatically download PDF files from arXiv (MVP: arXiv-only, other sources out of scope)
- **FR-003**: System MUST store uploaded/downloaded PDFs in a persistent local file structure (`artifacts/papers/{paper_id}/source.pdf`)
- **FR-004**: System MUST store paper metadata (title, source URL, import timestamp, processing status) in persistent storage
- **FR-005**: System MUST detect when a PDF cannot be parsed or is corrupted, warn the user, and allow partial processing

#### Text Extraction & Chunking

- **FR-006**: System MUST extract text content from PDF files without requiring OCR (text-based PDFs only)
- **FR-007**: System MUST handle extraction failures gracefully by warning users about unparseable pages and continuing with extractable pages
- **FR-008**: System MUST divide extracted text into chunks for translation, prioritizing natural boundaries (paragraphs, sections) when possible
- **FR-009**: System MUST use chunk sizes between 800-1200 words (or equivalent character count) with 10-15% overlap to preserve context across chunk boundaries
- **FR-010**: System MUST cache chunk content using a hash to avoid reprocessing unchanged chunks

#### Translation

- **FR-011**: System MUST translate each text chunk into Japanese using an external LLM service via OpenAI-compatible API endpoints
- **FR-012**: System MUST preserve mathematical notation, symbols, reference labels, and formatting in translated output
- **FR-013**: System MUST handle translation failures by preserving partial results and allowing retry of failed chunks individually
- **FR-014**: System MUST limit concurrent LLM requests to a maximum of 10 parallel calls to respect rate limits
- **FR-015**: System MUST implement exponential backoff retry logic for transient LLM API failures
- **FR-016**: System MUST log all LLM prompts and responses to disk for reproducibility and debugging

#### Term Extraction & Glossary

- **FR-017**: System MUST extract machine learning-related terms from text using LLM-based analysis
- **FR-018**: System MUST store extracted terms with English lemma, Japanese translation, reading (kana), part of speech, category tags, and notes
- **FR-019**: System MUST support term variants (spelling variations, hyphenation differences, singular/plural) and link them to canonical term entries
- **FR-020**: System MUST detect and present duplicate term candidates to users, requiring explicit user confirmation before merging
- **FR-021**: System MUST generate Japanese definitions (2-3 sentences) for extracted terms using LLM calls
- **FR-022**: System MUST allow users to manually add, edit, and delete terms in the glossary
- **FR-023**: System MUST support bilingual search (English and Japanese) with normalization for katakana, hyphens, spaces, and basic singular/plural forms

#### Translation Viewing & Term Highlighting

- **FR-024**: System MUST display translated text in a readable page view organized by sections/paragraphs
- **FR-025**: System MUST highlight all extracted machine learning terms in the translated text with visually distinct but non-intrusive styling
- **FR-026**: System MUST display tooltips when users hover over or click highlighted terms
- **FR-027**: Tooltips MUST appear within 100 milliseconds of user interaction
- **FR-028**: Tooltips MUST show Japanese definition, English original form, and category tags
- **FR-029**: System MUST allow users to view the original PDF side-by-side with the translated text or in a separate tab
- **FR-030**: System MUST synchronize term highlighting across all occurrences of the same term or its variants

Clarification (API behavior for not-yet-translated papers):
- When a paper exists but translation has not yet been produced (extraction/translation pending), `GET /api/papers/{id}/translation` MUST return 200 with payload `{ paper_id, chunks: [] }`.
- 404 MUST only indicate that the paper resource itself does not exist.

#### Progress Tracking & Error Recovery

- **FR-031**: System MUST display granular progress indicators during paper processing (import → extraction → translation → term processing) with percentage completion
- **FR-032**: System MUST allow users to retry failed processing steps (chunks, term extraction, definition generation) without reprocessing successful steps
- **FR-033**: System MUST preserve all partial results when errors occur to prevent data loss

#### Data Persistence & Export

- **FR-034**: System MUST persist all user data (papers, translations, terms, definitions) in local storage that survives application restarts
- **FR-035**: System MUST support glossary export to standard formats (JSON, CSV)
- **FR-036**: System MUST use versioned, reversible database migrations to manage schema changes

#### Paper Management

- **FR-037**: System MUST allow users to delete imported papers along with all associated data (chunks, translations, occurrences, and stored PDF files)
- **FR-038**: System MUST require explicit user confirmation before deleting a paper to prevent accidental data loss
- **FR-039**: System MUST clean up orphaned terms (terms with no remaining occurrences across all papers) after paper deletion

### Key Entities *(include if feature involves data)*

- **Paper**: Represents a single imported scientific paper. Key attributes: unique identifier, title, source URL, file path to stored PDF, processing status (pending/in-progress/completed/failed), import timestamp. Relationships: contains multiple Chunks; terms from this paper have Occurrences linked to it.

- **Chunk**: A segment of extracted text from a paper for translation purposes. Key attributes: unique identifier, parent paper reference, sequential index, source text (English), translated HTML/text (Japanese), content hash for caching. Relationships: belongs to one Paper; may contain multiple Term Occurrences.

- **Term**: A canonical concept/vocabulary entry in the glossary. Key attributes: unique identifier, slug for URL-safe lookup, English lemma (canonical form), Japanese lemma (canonical translation), reading (kana), part of speech, category tags, user notes, creation timestamp. Relationships: has multiple Term Variants (spelling variations); has one Definition; appears in multiple Occurrences across papers.

- **Term Variant**: A spelling or expression variation of a canonical term. Key attributes: unique identifier, parent term reference, language (English/Japanese), surface form (the actual string that appears in text). Relationships: belongs to one Term. Purpose: enables fuzzy matching and normalization (e.g., "neural network" vs "neural-network" vs "NN" all link to the same Term).

- **Definition**: A human-readable explanation of a term in Japanese. Key attributes: unique identifier, parent term reference, language (always Japanese for MVP), definition text (2-3 sentences), provider (LLM model name or "manual"), last updated timestamp. Relationships: belongs to one Term.

- **Occurrence**: A specific instance where a term appears in a translated paper. Key attributes: unique identifier, term reference, paper reference, chunk reference, start/end position or HTML marker ID. Relationships: links a Term to a specific location in a Paper's Chunk. Purpose: enables highlighting and occurrence tracking.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can successfully upload a PDF or provide a URL, and view the translated Japanese text within 5 minutes for a typical 10-page paper (assuming LLM response times of ~5-10 seconds per chunk)

- **SC-002**: Translated text preserves the original structure (paragraphs, sections) such that users can navigate the translation and original PDF in parallel without losing context

- **SC-003**: At least 80% of machine learning-related technical terms in a paper are automatically extracted and highlighted in the translation

- **SC-004**: Tooltips appear within 100 milliseconds of hover interaction, providing immediate term comprehension without interrupting reading flow

- **SC-005**: Users can search the glossary in both English and Japanese and find matching terms within 2 seconds, including fuzzy matches (katakana variants, hyphenation differences, singular/plural)

- **SC-006**: System successfully processes papers up to 50 pages (~400 chunks) without data loss or unrecoverable errors, preserving partial results if failures occur

- **SC-007**: Users can retry failed processing steps (individual chunks or term extraction) without needing to restart the entire pipeline from scratch

- **SC-008**: Glossary data persists across application restarts, and users can export their glossary to standard formats for backup or external use

- **SC-009**: Users can manually add, edit, and merge terms in the glossary, and changes are immediately reflected in all future tooltip displays

- **SC-010**: System respects the 10 concurrent LLM request limit, processes papers within reasonable time (not timing out), and handles API failures gracefully with automatic retry

## Assumptions

- **A-001**: PDFs are text-based (not scanned images requiring OCR). OCR is explicitly out of scope for MVP.

- **A-002**: Users have access to a local or remote OpenAI-compatible LLM endpoint (e.g., localhost:8000/v1/chat/completions) with a maximum token context of 30,000 tokens and support for at least 10 concurrent requests.

- **A-003**: Papers are primarily in English. Translation is unidirectional (English → Japanese). Reverse translation or multi-language support is out of scope.

- **A-004**: Users are comfortable with command-line or simple web interface for initial setup (e.g., configuring LLM API base URL and key). No advanced authentication or multi-user support is required.

- **A-005**: Local disk storage is sufficient for storing PDFs, translations, and glossary data (single-user, personal use). Cloud sync or distributed storage is out of scope.

- **A-006**: Users are researchers, students, or professionals familiar with machine learning terminology who want comprehension support, not beginners needing extensive educational scaffolding.

- **A-007**: LLM-generated definitions and translations are treated as drafts that users can review and edit. Perfect accuracy is not guaranteed; user control and editability are prioritized.

- **A-008**: Figures, tables, and complex mathematical equations are referenced from the original PDF side-by-side view. Translation focuses on body text, not rendering of visual content.

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
