<!--
  SYNC IMPACT REPORT

  Version change: (Initial creation) → 1.0.0
  Modified principles: N/A (initial version)
  Added sections:
    - Core Principles (5 principles defined)
    - Technical Constraints
    - Development Workflow
    - Governance
  Removed sections: N/A

  Templates requiring updates:
    ✅ plan-template.md - Constitution Check section aligned
    ✅ spec-template.md - User scenarios and requirements align with principles
    ✅ tasks-template.md - Task categorization reflects principle-driven development
    ✅ commands/*.md - Agent-agnostic guidance verified

  Follow-up TODOs: None
-->

# paper-gloss Constitution

## Core Principles

### I. Simplicity-First (YAGNI)

**Rule**: Every feature and implementation MUST serve an immediate, documented user need. Do not build for hypothetical future requirements.

**Rationale**: As an MVP for personal use, complexity is the primary risk. Focus on essential functionality first. Scale and extensibility are explicitly deprioritized in favor of working software that solves the core problem: reading English papers with ML term comprehension support.

**Guidelines**:
- Reject features not directly supporting core workflows (PDF import → translation → term highlighting → glossary review)
- Prefer straightforward solutions over flexible architectures
- Single-user, local-first design means authentication, multi-tenancy, and distributed systems are out of scope
- Document why simpler alternatives were rejected when complexity is unavoidable

### II. Test-First Development (NON-NEGOTIABLE)

**Rule**: Tests MUST be written before implementation. All tests MUST fail initially to verify they test the right behavior.

**Rationale**: Quality and regression prevention are critical for a personal-use tool with data persistence requirements. User's glossary data and translation history cannot be lost or corrupted. TDD ensures contracts are validated before code is written.

**Guidelines**:
- Follow Red-Green-Refactor cycle strictly: Write test → Verify failure → Implement → Verify success → Refactor
- Contract tests required for all public API endpoints (Rust backend)
- Integration tests required for critical paths: PDF processing, LLM calls, term extraction/matching, database persistence
- UI component tests required for term highlighting, tooltip display, glossary search
- Manual testing acceptable for UI polish and visual validation, but core interactions require automated tests

### III. UI/UX-Centric Design

**Rule**: User interactions MUST prioritize clarity, visual feedback, and low cognitive load. The interface is the product for this personal-use tool.

**Rationale**: The primary value is in reading comprehension support. If users cannot quickly scan highlighted terms, hover for definitions, or search the glossary, the tool fails regardless of backend sophistication.

**Guidelines**:
- Term highlighting MUST be visually distinct but non-intrusive
- Tooltips MUST appear instantly on hover (<100ms) with Japanese-only definitions
- Glossary view MUST support real-time bilingual search with normalization (katakana, hyphen, singular/plural fuzzy matching)
- Progress indicators MUST show granular status during PDF processing (import → extraction → translation → term processing)
- Error states MUST allow retry without data loss (failed chunks can be reprocessed without restarting pipeline)
- Design decisions default to usability over technical elegance

### IV. Modular Monolith Architecture

**Rule**: Code MUST be organized into independently testable, loosely coupled modules within a single repository. Deployment remains a single binary + single frontend build.

**Rationale**: Personal-use MVP benefits from deployment simplicity (monolith) while preserving maintainability through clear module boundaries (modular). This avoids microservice overhead while preventing "big ball of mud."

**Guidelines**:
- Backend modules: PDF processing, LLM client, term extraction, normalization/matching, database persistence, API layer
- Frontend modules: PDF viewer, translation display, term highlighting engine, tooltip renderer, glossary UI, API client
- Each module MUST have a clear interface and be testable in isolation (contract tests validate module boundaries)
- Shared types/schemas defined in Rust (utoipa OpenAPI generation) and consumed by TypeScript frontend
- SQLite database is the single source of truth; no inter-module state sharing except through DB
- Module dependencies MUST be acyclic (draw a dependency graph; cycles indicate design smell)

### V. Data Integrity and User Control

**Rule**: User data (glossary terms, translations, paper metadata) MUST be persistent, exportable, and recoverable. Users MUST have explicit control over LLM-generated content.

**Rationale**: This is a personal knowledge tool. Data loss or silent corruption breaks trust. Users need agency over which terms are glossary entries, which definitions are accepted, and how variants are merged.

**Guidelines**:
- All LLM prompts and responses MUST be logged to disk for reproducibility and debugging (`artifacts/papers/{paper_id}/llm_logs/`)
- Database schema migrations MUST be versioned and reversible (up/down migrations required)
- Term merging (duplicate resolution) MUST require explicit user confirmation; never auto-merge silently
- Glossary export to JSON, CSV, or Anki-compatible format MUST be available
- Failed operations (chunk translation errors, term extraction timeouts) MUST preserve partial results and allow selective retry
- Database backup MUST be documented in quickstart (e.g., "copy the .db file before major operations")

## Technical Constraints

**Language/Stack** (from requirements.md):
- Frontend: TypeScript (framework TBD: Next.js, SvelteKit, or similar)
- Backend: Rust (Axum framework + utoipa for OpenAPI)
- Database: SQLite (single-file, local persistence)
- LLM Integration: OpenAI-compatible `/v1/chat/completions` endpoint only

**Performance Targets**:
- LLM concurrency: Maximum 10 parallel requests (hard limit to respect API rate limits)
- Tooltip latency: <100ms from hover to display (UI responsiveness)
- Chunk processing: Support papers up to 50 pages (~400 chunks at 800-1200 word chunks)

**Out-of-Scope** (enforced to maintain simplicity):
- OCR, LaTeX math reconstruction, figure/table extraction (MVP uses original PDF side-by-side)
- English definitions (Japanese-only glossary)
- Spaced repetition systems (SRS) or Anki export (future scope)
- Vector embeddings, semantic search, or clustering (future scope)
- Multi-user support, authentication, or cloud sync

## Development Workflow

### Code Review and Quality Gates

- All changes MUST pass tests before merge (CI/local validation)
- Constitution compliance MUST be verified during spec and plan reviews:
  - Does this violate YAGNI? (Are we building for hypothetical needs?)
  - Are tests written first? (Is TDD being followed?)
  - Does this degrade UX? (Is the interface still clear and responsive?)
  - Does this break module boundaries? (Are dependencies still acyclic?)
  - Does this risk data integrity? (Are migrations reversible? Are failures recoverable?)
- Complexity MUST be justified explicitly in plan.md "Complexity Tracking" section when constitution violations are unavoidable

### Testing Discipline

- **Contract Tests**: All API endpoints (e.g., `POST /api/papers/import`, `GET /api/terms?q=`)
- **Integration Tests**: End-to-end workflows (PDF upload → translation display, term search → highlight sync)
- **UI Component Tests**: Term highlighting, tooltip rendering, glossary search/filter

### Observability

- Structured logging (INFO, ERROR levels minimum) to stdout and file
- LLM request/response logging to `artifacts/papers/{paper_id}/llm_logs/` (for debugging and reproducibility)
- Progress tracking for long-running operations (PDF processing pipeline status updates)

## Governance

**Amendment Process**:
1. Propose amendment with rationale (GitHub issue or discussion)
2. Document impact on existing code and templates
3. Update constitution version (semantic versioning: MAJOR for backward-incompatible changes, MINOR for new principles/sections, PATCH for clarifications)
4. Propagate changes to all dependent templates (plan, spec, tasks, commands)
5. Migration plan required for MAJOR version changes

**Versioning Policy**:
- **MAJOR**: Backward-incompatible governance changes (e.g., removing a principle, fundamentally changing TDD requirement)
- **MINOR**: New principle added, new section added, materially expanded guidance
- **PATCH**: Clarifications, wording improvements, typo fixes, non-semantic refinements

**Compliance Verification**:
- Every plan.md MUST include a "Constitution Check" section validating adherence to principles
- Every spec.md MUST organize user stories by independent testability (supports Test-First and Simplicity principles)
- Every tasks.md MUST include test tasks before implementation tasks (enforces TDD)

**Conflict Resolution**:
- Constitution supersedes all other practices
- When principles conflict (e.g., Simplicity vs. Data Integrity), Data Integrity wins (principle V is foundational for trust)
- When in doubt, consult this document and update it if the guidance is unclear

**Version**: 1.0.0 | **Ratified**: 2025-10-19 | **Last Amended**: 2025-10-19
