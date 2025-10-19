# Implementation Plan: Paper Translation & Glossary System

**Branch**: `001-paper-translation-glossary` | **Date**: 2025-10-19 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-paper-translation-glossary/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Build a local web application that imports English ML papers (PDF), translates them to Japanese via LLM, extracts ML terms, and displays translations with highlighted terms + instant tooltip definitions. Users can manage a personal glossary with bilingual search, manual editing, and duplicate merging. Core value: enable Japanese-speaking researchers to read English papers with ML terminology comprehension support.

## Technical Context

**Language/Version**: Rust (latest stable, 1.75+), TypeScript (5.x+)
**Primary Dependencies**:
- Backend: Axum (web framework), utoipa-axum (OpenAPI generation), SQLx (SQLite ORM), reqwest (HTTP client for LLM API), pdf-extract (PDF text extraction)
- Frontend: Vite + React (SPA, no framework), React Router, react-pdf (PDF.js wrapper), @tanstack/react-query (API client/cache)
**Storage**: SQLite (local file-based database), filesystem (`artifacts/` for PDFs and LLM logs)
**Testing**:
- Backend: cargo test (unit + integration), utoipa-axum + Schemathesis (contract tests), Tower Service trait (fast integration tests)
- Frontend: Vitest + React Testing Library + @testing-library/user-event
**Target Platform**: Local desktop/laptop (macOS, Linux, Windows) - web UI served locally (http://localhost:PORT)
**Project Type**: Web application (Rust backend API + TypeScript frontend SPA, monolithic deployment)
**Performance Goals**:
- Tooltip latency <100ms (critical UX requirement; Vite + React estimated 30-50ms)
- LLM concurrency max 10 parallel requests
- Process 10-page paper in <5 minutes
- Handle 50-page papers (~400 chunks) without failure
**Constraints**:
- Offline-capable except during PDF download and LLM API calls
- Single-user, no network authentication
- Max LLM token context 30,000 tokens per request
**Scale/Scope**: Personal-use MVP, ~10-20 papers processed, ~500-1000 terms in glossary, single active user

**Research Resolution**: All NEEDS CLARIFICATION items resolved in [research.md](./research.md). Key decisions: Vite + React (simplicity, <100ms tooltips), pdf-extract (MIT license, pure Rust), SQLx (compile-time safety, async-native), Vitest (10-20x faster than Jest), utoipa-axum + Schemathesis (contract testing).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Based on [constitution v1.0.0](../../.specify/memory/constitution.md):

### I. Simplicity-First (YAGNI)

- ✅ **PASS**: Feature scope tightly constrained to MVP requirements (PDF import → translation → term highlighting → glossary)
- ✅ **PASS**: Out-of-scope items explicitly documented (OCR, SRS, vector search, multi-user, cloud sync)
- ✅ **PASS**: Single-user, local-first design avoids authentication/authorization complexity
- ✅ **PASS**: Monolithic deployment (single Rust binary + single frontend build) avoids microservice overhead
- ⚠️ **WATCH**: Ensure frontend framework choice (Next.js vs SvelteKit) prioritizes simplicity over features (research required)

**Verdict**: PASS - No unjustified complexity. All features serve immediate, documented user needs.

### II. Test-First Development (NON-NEGOTIABLE)

- ✅ **PASS**: Clear module boundaries enable isolated unit testing (PDF processing, LLM client, term extraction, normalization, DB persistence, API layer)
- ✅ **PASS**: Contract tests required for all API endpoints (enforced in tasks.md)
- ✅ **PASS**: Integration tests identified for critical paths (PDF upload → translation, term search → highlight sync)
- ✅ **PASS**: UI component tests required (term highlighting, tooltip rendering, glossary search)
- ⚠️ **ACTION REQUIRED**: Choose contract test framework (research required: e.g., `utoipa` + generated OpenAPI tests, or manual contract tests)

**Verdict**: PASS - TDD-compatible design with clear test boundaries.

### III. UI/UX-Centric Design

- ✅ **PASS**: Tooltip latency <100ms enforced as performance goal (critical UX requirement)
- ✅ **PASS**: Progress indicators specified for granular feedback (import → extraction → translation → term processing)
- ✅ **PASS**: Error recovery without data loss (failed chunks retriable without full reprocess)
- ✅ **PASS**: Bilingual search with normalization (katakana, hyphen, singular/plural fuzzy matching)
- ✅ **PASS**: Term highlighting "visually distinct but non-intrusive" (explicit UX guidance in spec)

**Verdict**: PASS - UX prioritized in requirements, performance targets, and error handling.

### IV. Modular Monolith Architecture

- ✅ **PASS**: Backend modules explicitly identified: PDF processing, LLM client, term extraction, normalization/matching, DB persistence, API layer
- ✅ **PASS**: Frontend modules explicitly identified: PDF viewer, translation display, term highlighting engine, tooltip renderer, glossary UI, API client
- ✅ **PASS**: Shared types/schemas via Rust utoipa OpenAPI generation (single source of truth for API contracts)
- ✅ **PASS**: SQLite as single source of truth for data (no inter-module state sharing except DB)
- ✅ **PASS**: Monolithic deployment (single Rust binary + single frontend build)
- ⚠️ **ACTION REQUIRED**: Verify module dependencies are acyclic during design (draw dependency graph in research.md or data-model.md)

**Verdict**: PASS - Clear module boundaries with monolithic deployment simplicity.

### V. Data Integrity and User Control

- ✅ **PASS**: LLM prompts/responses logged to disk (`artifacts/papers/{paper_id}/llm_logs/`)
- ✅ **PASS**: Database migrations versioned and reversible (requirement FR-036)
- ✅ **PASS**: Term merging requires explicit user confirmation (requirement FR-020)
- ✅ **PASS**: Glossary export to JSON/CSV (requirement FR-035)
- ✅ **PASS**: Failed operations preserve partial results (requirements FR-013, FR-032, FR-033)
- ✅ **PASS**: Database backup documented in quickstart (to be generated in Phase 1)

**Verdict**: PASS - Data integrity and user control enforced in requirements.

---

**OVERALL GATE STATUS**: ✅ **PASS**

- No constitution violations
- All principles satisfied by current plan
- Action items (NEEDS CLARIFICATION) will be resolved in Phase 0 research
- No Complexity Tracking section required (no violations to justify)

**Re-evaluation trigger**: After Phase 1 design (data-model.md, contracts/, quickstart.md) to verify module dependencies remain acyclic and no complexity creep.

## Project Structure

### Documentation (this feature)

```
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```
backend/
├── src/
│   ├── models/          # Database entities (Paper, Chunk, Term, etc.)
│   ├── services/        # Business logic modules
│   │   ├── pdf/         # PDF extraction & chunking
│   │   ├── llm/         # LLM client (translation, term extraction, definition generation)
│   │   ├── terms/       # Term normalization, matching, duplicate detection
│   │   └── persistence/ # Database operations, migrations
│   ├── api/             # REST endpoints (Axum routes + utoipa schemas)
│   └── main.rs          # Server entry point
├── tests/
│   ├── contract/        # API contract tests (OpenAPI compliance)
│   ├── integration/     # End-to-end workflow tests
│   └── unit/            # Module-level unit tests
└── migrations/          # SQLite schema migrations (up/down)

frontend/
├── src/
│   ├── components/      # UI components
│   │   ├── pdf/         # PDF viewer (PDF.js wrapper)
│   │   ├── translation/ # Translation display, term highlighting
│   │   ├── tooltip/     # Tooltip renderer
│   │   └── glossary/    # Glossary UI (search, CRUD, merge)
│   ├── pages/           # Route pages (paper list, translation view, glossary view)
│   ├── services/        # API client (generated from OpenAPI or manual)
│   └── lib/             # Utilities (text normalization, fuzzy matching)
└── tests/               # UI component tests (Vitest + Testing Library)

artifacts/               # Runtime-generated files (gitignored)
└── papers/{paper_id}/
    ├── source.pdf
    ├── text/            # Extracted text per page
    ├── translations/    # Translated chunks
    ├── terms/           # Extracted terms JSON
    └── llm_logs/        # LLM request/response logs

data/                    # Static seed data (optional, version-controlled)
└── seeds/
    └── initial_terms.json  # Pre-populated ML terms (if any)
```

**Structure Decision**: Web application layout (Option 2) selected. Backend (Rust/Axum) and frontend (TypeScript SPA) are separate top-level directories with independent build processes but deployed together (monolith). This aligns with constitution principle IV (Modular Monolith) by providing clear module boundaries while maintaining deployment simplicity.

## Complexity Tracking

*Fill ONLY if Constitution Check has violations that must be justified*

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., 4th project] | [current need] | [why 3 projects insufficient] |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient] |

