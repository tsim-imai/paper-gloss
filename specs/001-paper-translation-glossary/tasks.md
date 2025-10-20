# Tasks: Paper Translation & Glossary System

**Input**: Design documents from `/specs/001-paper-translation-glossary/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/openapi.yaml

**Tests**: Following constitution Principle II (Test-First Development), all tests MUST be written before implementation and MUST fail initially.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`
- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions
- **Web app**: `backend/src/`, `frontend/src/`
- Backend modules: `backend/src/models/`, `backend/src/services/`, `backend/src/api/`
- Frontend modules: `frontend/src/components/`, `frontend/src/pages/`, `frontend/src/services/`
- Tests: `backend/tests/`, `frontend/tests/`
- Migrations: `backend/migrations/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure per plan.md and quickstart.md

- [X] T001 Create backend directory structure (src/models, src/services, src/api, tests/, migrations/) per plan.md:120-131
- [X] T002 Initialize Rust backend with Cargo.toml dependencies (axum, utoipa-axum, sqlx, reqwest, pdf-extract) per research.md:268-270
- [X] T003 [P] Create frontend directory structure (src/components, src/pages, src/services, tests/) per plan.md:139-148
- [X] T004 [P] Initialize TypeScript frontend with package.json dependencies (vite, react, react-router, react-pdf, vitest) per research.md:268-270
- [X] T005 [P] Configure Vite proxy for backend API in frontend/vite.config.ts per quickstart.md:173-188
- [X] T006 [P] Setup linting and formatting (Cargo clippy, prettier/eslint) per constitution Principle I
- [X] T007 [P] Create .env template files for backend and frontend per quickstart.md:81-97, 164-171
- [X] T008 [P] Create artifacts/ directory structure (papers/{paper_id}/source.pdf, text/, translations/, terms/, llm_logs/) per plan.md:151-156

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T009 Create initial SQLite schema migration (001_initial_schema.sql) with all 6 entities per data-model.md:330-340
- [X] T010 Setup SQLx database connection pool in backend/src/main.rs per research.md:183-206
- [X] T011 [P] Implement LLM client module in backend/src/services/llm/client.rs (OpenAI-compatible API) per spec.md:FR-011, FR-014-016
- [X] T012 [P] Implement LLM request logging to artifacts/ per spec.md:FR-016 and constitution Principle V
- [X] T013 [P] Configure Axum web server with utoipa OpenAPI generation in backend/src/main.rs per research.md:153-173
- [X] T014 [P] Setup error handling middleware in backend/src/api/error.rs per plan.md:282
- [X] T015 [P] Setup logging infrastructure (RUST_LOG) in backend/src/main.rs per quickstart.md:96
- [X] T016 [P] Create API client service in frontend/src/services/api.ts using @tanstack/react-query per plan.md:161
- [ ] T017 [P] Run quickstart.md validation: verify backend builds, migrations run, frontend dev server starts per quickstart.md:204-220

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Read Translated Papers (Priority: P1) 🎯 MVP

**Goal**: Users can upload PDFs, view processing progress, and read translated Japanese text side-by-side with the original PDF

**Independent Test**: Upload a 10-page PDF (or provide URL), wait for translation completion, verify Japanese translated text displays in readable format with option to view original PDF side-by-side

### Tests for User Story 1

**NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [ ] T018 [P] [US1] Contract test for POST /papers/import in backend/tests/contract/test_paper_import.rs per contracts/openapi.yaml:42-89
- [ ] T019 [P] [US1] Contract test for GET /papers in backend/tests/contract/test_paper_list.rs per contracts/openapi.yaml:136-180
- [ ] T020 [P] [US1] Contract test for GET /papers/{id} in backend/tests/contract/test_paper_detail.rs per contracts/openapi.yaml:115-134
- [ ] T021 [P] [US1] Contract test for GET /papers/{id}/translation in backend/tests/contract/test_translation.rs per contracts/openapi.yaml:182-214
- [ ] T022 [P] [US1] Contract test for POST /papers/{id}/process in backend/tests/contract/test_paper_process.rs per contracts/openapi.yaml:91-113
- [ ] T023 [P] [US1] Contract test for POST /chunks/{id}/retry in backend/tests/contract/test_chunk_retry.rs per contracts/openapi.yaml:216-244
- [ ] T024 [P] [US1] Integration test for PDF upload → translation workflow in backend/tests/integration/test_paper_workflow.rs per spec.md:18-24
- [ ] T025 [P] [US1] Frontend component test for PDF upload UI in frontend/tests/components/PaperImport.test.tsx per spec.md:20
- [ ] T026 [P] [US1] Frontend component test for translation view in frontend/tests/components/TranslationView.test.tsx per spec.md:23-24

### Implementation for User Story 1

**Models & Database** (run in parallel):

- [X] T027 [P] [US1] Create Paper model in backend/src/models/paper.rs with SQLx queries per data-model.md:44-78
- [X] T028 [P] [US1] Create Chunk model in backend/src/models/chunk.rs with SQLx queries per data-model.md:82-115

**Services** (depends on T027, T028):

- [X] T029 [US1] Implement PDF extraction service in backend/src/services/pdf/extraction.rs using pdf-extract per spec.md:FR-006-007, research.md:62-115
- [X] T030 [US1] Implement text chunking service in backend/src/services/pdf/chunking.rs per spec.md:FR-008-010
- [X] T031 [US1] Implement translation service in backend/src/services/translation.rs using LLM client per spec.md:FR-011-016
- [X] T032 [US1] Implement paper processing orchestrator in backend/src/services/paper_processor.rs (coordinates extraction → chunking → translation) per spec.md:FR-031-033
- [X] T033 [US1] Implement chunk retry logic in backend/src/services/paper_processor.rs per spec.md:FR-032

**API Endpoints** (depends on T029-T033):

- [X] T034 [P] [US1] Implement POST /papers/import endpoint in backend/src/api/papers/import.rs per contracts/openapi.yaml:42-89
- [X] T035 [P] [US1] Implement GET /papers endpoint in backend/src/api/papers/list.rs per contracts/openapi.yaml:136-180
- [X] T036 [P] [US1] Implement GET /papers/{id} endpoint in backend/src/api/papers/detail.rs per contracts/openapi.yaml:115-134
- [X] T037 [P] [US1] Implement GET /papers/{id}/translation endpoint in backend/src/api/papers/translation.rs per contracts/openapi.yaml:182-214
- [X] T038 [P] [US1] Implement POST /papers/{id}/process endpoint in backend/src/api/papers/process.rs per contracts/openapi.yaml:91-113
- [X] T039 [P] [US1] Implement POST /chunks/{id}/retry endpoint in backend/src/api/chunks/retry.rs per contracts/openapi.yaml:216-244

**Frontend Components** (can run in parallel with backend endpoints):

- [X] T040 [P] [US1] Create PaperImport component in frontend/src/components/papers/PaperImport.tsx (file upload + URL input) per spec.md:20-22
- [X] T041 [P] [US1] Create PaperList component in frontend/src/components/papers/PaperList.tsx per spec.md:19
- [X] T042 [P] [US1] Create ProcessingStatus component in frontend/src/components/papers/ProcessingStatus.tsx (progress indicators) per spec.md:22, FR-031
- [X] T043 [P] [US1] Create TranslationView component in frontend/src/components/translation/TranslationView.tsx per spec.md:23
- [X] T044 [P] [US1] Create PDFViewer component in frontend/src/components/pdf/PDFViewer.tsx using react-pdf per spec.md:24, FR-029
- [X] T045 [US1] Create PaperPage route in frontend/src/pages/PaperPage.tsx (side-by-side layout) per spec.md:24

**Validation & Error Handling**:

- [ ] T046 [US1] Add PDF extraction error handling (detect unparseable pages, partial processing) in extraction service per spec.md:Edge Cases, FR-005, FR-007
- [ ] T047 [US1] Add LLM failure handling (exponential backoff, partial result preservation) in translation service per spec.md:Edge Cases, FR-013, FR-015
- [ ] T048 [US1] Add validation and error messages in PaperImport component per spec.md:20

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently. Users can upload PDFs, view processing progress, and read translations.

---

## Phase 4: User Story 2 - Understand ML Terms via Tooltips (Priority: P2)

**Goal**: Users see highlighted ML terms in translations and get instant tooltip definitions on hover

**Independent Test**: View a translated paper with extracted terms, hover over highlighted terms, verify tooltips appear within 100ms with Japanese definitions, English original forms, and category tags

### Tests for User Story 2

- [ ] T049 [P] [US2] Contract test for GET /terms/{id} in backend/tests/contract/test_term_detail.rs per contracts/openapi.yaml:337-354
- [ ] T050 [P] [US2] Contract test for GET /occurrences in backend/tests/contract/test_occurrences.rs per contracts/openapi.yaml:482-532
- [ ] T051 [P] [US2] Integration test for term extraction → highlighting workflow in backend/tests/integration/test_term_extraction.rs per spec.md:37-43
- [ ] T052 [P] [US2] Frontend component test for term highlighting in frontend/tests/components/TermHighlight.test.tsx per spec.md:38
- [ ] T053 [P] [US2] Frontend component test for tooltip performance (<100ms) in frontend/tests/components/TermTooltip.test.tsx per spec.md:39, FR-027

### Implementation for User Story 2

**Models & Database** (run in parallel):

- [X] T054 [P] [US2] Create Term model in backend/src/models/term.rs with SQLx queries per data-model.md:118-154
- [X] T055 [P] [US2] Create TermVariant model in backend/src/models/term_variant.rs with SQLx queries per data-model.md:158-186
- [X] T056 [P] [US2] Create Definition model in backend/src/models/definition.rs with SQLx queries per data-model.md:189-217
- [X] T057 [P] [US2] Create Occurrence model in backend/src/models/occurrence.rs with SQLx queries per data-model.md:220-254

**Services** (depends on T054-T057):

- [X] T058 [US2] Implement term extraction service in backend/src/services/terms/extraction.rs using LLM client per spec.md:FR-017
- [X] T059 [US2] Implement term normalization service in backend/src/services/terms/normalization.rs (English/Japanese variants) per data-model.md:294-322
- [X] T060 [US2] Implement definition generation service in backend/src/services/terms/definition.rs using LLM client per spec.md:FR-021
- [X] T061 [US2] Implement occurrence tracking service in backend/src/services/terms/occurrence_tracker.rs per data-model.md:220-254
- [X] T062 [US2] Integrate term extraction into paper processing orchestrator (backend/src/services/paper_processor.rs) per spec.md:FR-017

**API Endpoints** (depends on T058-T061):

- [X] T063 [P] [US2] Implement GET /terms/{id} endpoint in backend/src/api/terms/detail.rs per contracts/openapi.yaml:337-354
- [X] T064 [P] [US2] Implement GET /occurrences endpoint in backend/src/api/occurrences/list.rs per contracts/openapi.yaml:482-532

**Frontend Components** (can run in parallel):

- [X] T065 [P] [US2] Create TermHighlight component in frontend/src/components/translation/TermHighlight.tsx (highlighting logic) per spec.md:38, FR-025
- [X] T066 [P] [US2] Create TermTooltip component in frontend/src/components/translation/TermTooltip.tsx (<100ms latency) per spec.md:39, FR-027-028
- [X] T067 [US2] Integrate TermHighlight and TermTooltip into TranslationView component per spec.md:37-43
- [X] T068 [US2] Implement tooltip sticky mode (click to keep open) in TermTooltip component per spec.md:40
- [X] T069 [US2] Add occurrence synchronization (highlight all instances) in TermHighlight component per spec.md:41, FR-030

**Performance Optimization**:

- [ ] T070 [US2] Optimize tooltip rendering to ensure <100ms latency (preload definitions, memoization) per spec.md:FR-027, Edge Cases
- [ ] T071 [US2] Add performance test for high-frequency terms (hundreds of occurrences) per spec.md:Edge Cases

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently. Users can read translations with instant term comprehension via tooltips.

---

## Phase 5: User Story 3 - Manage and Review Glossary (Priority: P3)

**Goal**: Users can search glossary in both languages, add/edit/delete terms, and merge duplicates

**Independent Test**: Open glossary view, search for terms in English and Japanese, add a custom term, edit an existing definition, and merge two duplicate term entries

### Tests for User Story 3

- [ ] T072 [P] [US3] Contract test for GET /terms in backend/tests/contract/test_term_list.rs per contracts/openapi.yaml:246-305
- [ ] T073 [P] [US3] Contract test for POST /terms in backend/tests/contract/test_term_create.rs per contracts/openapi.yaml:307-335
- [ ] T074 [P] [US3] Contract test for PATCH /terms/{id} in backend/tests/contract/test_term_update.rs per contracts/openapi.yaml:356-381
- [ ] T075 [P] [US3] Contract test for DELETE /terms/{id} in backend/tests/contract/test_term_delete.rs per contracts/openapi.yaml:383-398
- [ ] T076 [P] [US3] Contract test for POST /terms/merge in backend/tests/contract/test_term_merge.rs per contracts/openapi.yaml:400-448
- [ ] T077 [P] [US3] Contract test for POST /terms/{id}/define in backend/tests/contract/test_term_define.rs per contracts/openapi.yaml:450-480
- [ ] T078 [P] [US3] Integration test for glossary CRUD workflow in backend/tests/integration/test_glossary.rs per spec.md:55-62
- [ ] T079 [P] [US3] Frontend component test for glossary search in frontend/tests/components/GlossarySearch.test.tsx per spec.md:57
- [ ] T080 [P] [US3] Frontend component test for term merge UI in frontend/tests/components/TermMerge.test.tsx per spec.md:60

### Implementation for User Story 3

**Services** (depends on Phase 4 models):

- [X] T081 [US3] Implement term search service in backend/src/services/terms/search.rs (bilingual, normalization) per spec.md:FR-023, data-model.md:294-322
- [X] T082 [US3] Implement duplicate detection service in backend/src/services/terms/duplicate_detection.rs per spec.md:FR-020
- [X] T083 [US3] Implement term merge service in backend/src/services/terms/merge.rs (preserve variants, require confirmation) per spec.md:FR-020, Edge Cases

**API Endpoints** (depends on T081-T083):

- [X] T084 [P] [US3] Implement GET /terms endpoint in backend/src/api/terms/list.rs (search, pagination) per contracts/openapi.yaml:246-305
- [X] T085 [P] [US3] Implement POST /terms endpoint in backend/src/api/terms/create.rs per contracts/openapi.yaml:307-335
- [X] T086 [P] [US3] Implement PATCH /terms/{id} endpoint in backend/src/api/terms/update.rs per contracts/openapi.yaml:356-381
- [X] T087 [P] [US3] Implement DELETE /terms/{id} endpoint in backend/src/api/terms/delete.rs per contracts/openapi.yaml:383-398
- [X] T088 [P] [US3] Implement POST /terms/merge endpoint in backend/src/api/terms/merge.rs per contracts/openapi.yaml:400-448
- [X] T089 [P] [US3] Implement POST /terms/{id}/define endpoint in backend/src/api/terms/define.rs per contracts/openapi.yaml:450-480

**Frontend Components** (can run in parallel):

- [X] T090 [P] [US3] Create GlossaryPanel component in frontend/src/components/glossary/GlossaryPanel.tsx (term list, frequency sorting) per spec.md:56
- [X] T091 [P] [US3] Create GlossarySearch component in frontend/src/components/glossary/GlossarySearch.tsx (bilingual, real-time filtering) per spec.md:57, FR-023
- [X] T092 [P] [US3] Create TermForm component in frontend/src/components/glossary/TermForm.tsx (add/edit term) per spec.md:58-59
- [X] T093 [P] [US3] Create TermMerge component in frontend/src/components/glossary/TermMerge.tsx (duplicate selection, confirmation) per spec.md:60
- [X] T094 [US3] Create GlossaryPage route in frontend/src/pages/GlossaryPage.tsx per spec.md:55

**Search & Normalization**:

- [ ] T095 [US3] Implement English normalization (lowercase, hyphen/space, stemming) in search service per data-model.md:296-307
- [ ] T096 [US3] Implement Japanese normalization (katakana, middle dot, long vowel) in search service per data-model.md:309-322
- [ ] T097 [US3] Add ambiguous search handling (show all matches for "NN") in GlossarySearch component per spec.md:Edge Cases

**Data Management**:

- [ ] T098 [US3] Implement glossary export (JSON/CSV) in backend/src/services/export.rs per spec.md:FR-035
- [ ] T099 [US3] Add export UI in GlossaryPanel component per spec.md:FR-035
- [ ] T100 [US3] Implement definition regeneration with user confirmation per spec.md:61

**Checkpoint**: All user stories should now be independently functional. Complete glossary management is available.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [ ] T101 [P] Add database backup documentation in quickstart.md per constitution Principle V, quickstart.md:225-250
- [ ] T102 [P] Implement Schemathesis contract validation per research.md:153-173
- [ ] T103 [P] Add comprehensive unit tests for services in backend/tests/unit/ per constitution Principle II
- [ ] T104 [P] Add accessibility improvements (ARIA labels, keyboard navigation) per constitution Principle III
- [ ] T105 [P] Performance profiling and optimization (identify bottlenecks) per spec.md:SC-004, SC-006
- [ ] T106 [P] Security review (input validation, SQL injection prevention) per constitution
- [ ] T107 [P] Code cleanup and refactoring per constitution Principle I (YAGNI)
- [ ] T108 Run full quickstart.md validation (fresh install, all features) per quickstart.md
- [ ] T109 Create production build and deployment guide per quickstart.md:532-565

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup (Phase 1) completion - BLOCKS all user stories
- **User Stories (Phase 3-5)**: All depend on Foundational (Phase 2) completion
  - User Story 1 (Phase 3): Can start after Phase 2 - No dependencies on other stories
  - User Story 2 (Phase 4): Can start after Phase 2 - Extends US1 translation view but independently testable
  - User Story 3 (Phase 5): Can start after Phase 2 - Manages terms from US2 but independently testable
- **Polish (Phase 6)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: No dependencies on other stories - Can proceed immediately after Foundational
- **User Story 2 (P2)**: Integrates with US1 (adds highlighting to TranslationView) but can be tested independently
- **User Story 3 (P3)**: Uses Term entities from US2 but provides independent glossary management UI

### Within Each User Story

- Tests MUST be written and FAIL before implementation (constitution Principle II)
- Models before services
- Services before endpoints
- Core implementation before integration
- Story complete before moving to next priority

### Parallel Opportunities

- **Phase 1 (Setup)**: T003-T008 can run in parallel (different directories/files)
- **Phase 2 (Foundational)**: T011-T017 can run in parallel (independent modules)
- **Phase 3 (US1)**:
  - Tests T018-T026 can run in parallel (all fail initially)
  - Models T027-T028 can run in parallel
  - API endpoints T034-T039 can run in parallel (after services complete)
  - Frontend components T040-T045 can run in parallel (after API contracts defined)
- **Phase 4 (US2)**:
  - Tests T049-T053 can run in parallel
  - Models T054-T057 can run in parallel
  - API endpoints T063-T064 can run in parallel
  - Frontend components T065-T066 can run in parallel
- **Phase 5 (US3)**:
  - Tests T072-T080 can run in parallel
  - API endpoints T084-T089 can run in parallel
  - Frontend components T090-T094 can run in parallel
- **Phase 6 (Polish)**: T101-T107 can run in parallel (cross-cutting improvements)

---

## Parallel Example: User Story 1

```bash
# Launch all contract tests for User Story 1 together (TDD - all fail initially):
Task: "Contract test for POST /papers/import in backend/tests/contract/test_paper_import.rs"
Task: "Contract test for GET /papers in backend/tests/contract/test_paper_list.rs"
Task: "Contract test for GET /papers/{id} in backend/tests/contract/test_paper_detail.rs"
Task: "Contract test for GET /papers/{id}/translation in backend/tests/contract/test_translation.rs"
Task: "Contract test for POST /papers/{id}/process in backend/tests/contract/test_paper_process.rs"
Task: "Contract test for POST /chunks/{id}/retry in backend/tests/contract/test_chunk_retry.rs"

# Launch all models for User Story 1 together:
Task: "Create Paper model in backend/src/models/paper.rs with SQLx queries"
Task: "Create Chunk model in backend/src/models/chunk.rs with SQLx queries"

# Launch all API endpoints for User Story 1 together (after services complete):
Task: "Implement POST /papers/import endpoint in backend/src/api/papers/import.rs"
Task: "Implement GET /papers endpoint in backend/src/api/papers/list.rs"
Task: "Implement GET /papers/{id} endpoint in backend/src/api/papers/detail.rs"
Task: "Implement GET /papers/{id}/translation endpoint in backend/src/api/papers/translation.rs"
Task: "Implement POST /papers/{id}/process endpoint in backend/src/api/papers/process.rs"
Task: "Implement POST /chunks/{id}/retry endpoint in backend/src/api/chunks/retry.rs"

# Launch all frontend components for User Story 1 together:
Task: "Create PaperImport component in frontend/src/components/papers/PaperImport.tsx"
Task: "Create PaperList component in frontend/src/components/papers/PaperList.tsx"
Task: "Create ProcessingStatus component in frontend/src/components/papers/ProcessingStatus.tsx"
Task: "Create TranslationView component in frontend/src/components/translation/TranslationView.tsx"
Task: "Create PDFViewer component in frontend/src/components/pdf/PDFViewer.tsx"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T008)
2. Complete Phase 2: Foundational (T009-T017) - CRITICAL, blocks all stories
3. Complete Phase 3: User Story 1 (T018-T048)
4. **STOP and VALIDATE**: Test User Story 1 independently per spec.md:16
5. Deploy/demo if ready

**Deliverable**: Users can upload PDFs, view processing progress, and read Japanese translations side-by-side with original PDF. This is the minimum viable product.

### Incremental Delivery

1. Complete Setup + Foundational (Phase 1-2) → Foundation ready
2. Add User Story 1 (Phase 3) → Test independently per spec.md:16 → Deploy/Demo (MVP!)
3. Add User Story 2 (Phase 4) → Test independently per spec.md:34 → Deploy/Demo
4. Add User Story 3 (Phase 5) → Test independently per spec.md:52 → Deploy/Demo
5. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together (Phase 1-2)
2. Once Foundational is done:
   - Developer A: User Story 1 (Phase 3)
   - Developer B: User Story 2 (Phase 4) - after US1 completes integration point
   - Developer C: User Story 3 (Phase 5) - can start after US2 models exist
3. Stories complete and integrate independently

**Note**: US2 and US3 have some integration dependencies (US2 creates Term entities, US3 manages them), so sequential delivery (P1 → P2 → P3) is recommended for single developer.

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable per spec.md acceptance scenarios
- Tests written FIRST, fail initially (constitution Principle II: Test-First Development)
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence
- All file paths are explicit and match plan.md project structure
- Performance targets enforced: <100ms tooltips (spec.md:FR-027), 10 concurrent LLM requests (spec.md:FR-014)
- Data integrity enforced: LLM logging (spec.md:FR-016), reversible migrations (spec.md:FR-036), user confirmation for merges (spec.md:FR-020)
