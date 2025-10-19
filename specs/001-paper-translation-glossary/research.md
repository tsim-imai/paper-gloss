# Research: Paper Translation & Glossary System

**Date**: 2025-10-19
**Branch**: `001-paper-translation-glossary`
**Purpose**: Resolve NEEDS CLARIFICATION items from Technical Context

---

## Overview

This document consolidates research findings for technology choices to resolve all NEEDS CLARIFICATION markers in plan.md Technical Context section. All decisions prioritize the constitution's Simplicity-First (YAGNI) principle while meeting performance and testing requirements.

---

## 1. Frontend Framework Selection

### Decision: **Vite + React (No Framework)**

### Rationale

1. **Constitutional Alignment (Simplicity-First)**:
   - Backend (Rust/Axum) already handles all API logic
   - Next.js/SvelteKit server features (SSR, API routes, file-based routing) are entirely unused
   - Pure SPA approach eliminates unnecessary complexity

2. **Performance Requirements (<100ms Tooltip)**:
   - Pure CSR (Client-Side Rendering) with zero hydration delay
   - Estimated 30-50ms tooltip render time (well under 100ms requirement)
   - No framework overhead from unused server features

3. **PDF.js Integration**:
   - React ecosystem has the most mature PDF.js wrappers (`react-pdf`, etc.)
   - Extensive documentation and examples for scientific document rendering

4. **TDD Support**:
   - Vitest + React Testing Library is industry standard (see Testing Framework section)
   - Aligns with constitution Principle II (Test-First Development)

5. **Deployment Simplicity**:
   - `vite build` → `dist/` folder served directly by Axum
   - No adapter configuration or build complexity

### Alternatives Considered

| Option | Evaluation | Rejection Reason |
|--------|-----------|------------------|
| **Next.js (App Router)** | ❌ Not Recommended | SSR/API routes/routing unused; violates YAGNI principle |
| **SvelteKit** | ⚠️ Conditional Option | Smaller bundle size, but React has better PDF.js ecosystem and testing maturity. Consider only if team has Svelte expertise. |
| **Vite + Vue 3** | ⚠️ Alternative | Equivalent to React but fewer PDF.js integration examples |
| **Astro** | ❌ Unsuitable | Designed for static sites, not full-featured SPAs |

### Implementation Notes

- **Structure**: `frontend/` directory with Vite + React + TypeScript
- **Key Dependencies**: `react-router-dom`, `react-pdf`, `axios` (or `@tanstack/react-query`), `vitest`, `@testing-library/react`
- **Development Setup**: Vite proxy to Axum API during development
- **Build Output**: Static `dist/` folder served by Axum in production

---

## 2. PDF Extraction Library (Rust)

### Decision: **`pdf-extract` (MIT license)**

### Rationale

1. **Simplicity**:
   - Pure Rust with no external C/C++ dependencies
   - Straightforward API: `pdf_extract::extract_text_from_mem(&bytes)`
   - Text extraction specialized design (no unnecessary features)

2. **License Compatibility**:
   - MIT license (permissive, commercial-use friendly)
   - Avoids GPL/AGPL constraints of alternatives (`poppler-rs`, `oxidize-pdf`)

3. **Active Maintenance**:
   - Continuously updated (latest 0.9.0 released April 2025)
   - Active community support on crates.io

4. **Cross-Platform**:
   - Pure Rust ensures seamless builds on macOS, Linux, Windows
   - No platform-specific binary dependencies

5. **Performance for Use Case**:
   - Adequate for 10-50 page scientific papers
   - Simple API reduces implementation time (YAGNI)

### Alternatives Considered

| Library | License | Pure Rust | Evaluation | Rejection Reason |
|---------|---------|-----------|------------|------------------|
| **lopdf** | MIT | ✅ | ⚠️ Backup Option | Lower-level API, requires deeper PDF spec knowledge. Use as fallback if `pdf-extract` fails for specific PDFs. |
| **pdfium-render** | Apache/BSD | ❌ (C++) | ❌ | Complex setup, external binary dependencies, overkill for text extraction |
| **poppler-rs** | **GPL** | ❌ (C) | ❌ | GPL license incompatible (copyleft requirement) |
| **oxidize-pdf** | **AGPL** | ✅ | ❌ | AGPL license incompatible (strong copyleft, SaaS restrictions) |
| **mupdf-rs** | - | ❌ | ❌ | Low-level bindgen only, no high-level abstraction |

### Known Limitations & Mitigation

**Scientific Paper Challenges**:
- Multi-column layouts may not preserve reading order
- Mathematical symbols/equations may extract imperfectly
- Tables/figures introduce noise

**Mitigation Strategies**:
1. Post-processing heuristics for column reordering
2. Header/footer removal via pattern matching
3. Quality checks (character ratio, length validation)
4. Future: Consider specialized parsers (GROBID) for metadata extraction if needed

### Implementation Notes

- **Crate**: `pdf-extract = "0.9"`
- **Fallback**: If specific PDFs fail, retry with `lopdf` (also MIT licensed)
- **Error Handling**: Log extraction failures, allow partial processing (per constitution Data Integrity principle)

---

## 3. Testing Frameworks

### 3.1 Frontend Testing

#### Decision: **Vitest + React Testing Library**

#### Rationale

1. **Speed**:
   - 10-20x faster than Jest in watch mode
   - 4x faster in cold runs
   - Benchmark: Jest ~15.5s vs Vitest ~3.8s (100 tests, 25 suites)

2. **TypeScript/ESM Native**:
   - Zero-config TypeScript support (inherits from Vite)
   - First-class ESM compatibility (Jest requires extra setup)

3. **TDD Workflow**:
   - Fast feedback loop essential for Test-First Development (constitution Principle II)
   - Hot module replacement for test files

4. **Simplicity**:
   - Works seamlessly with Vite projects
   - Compatible with Jest API (easy migration if needed)

#### Component Testing Approach

- **React Testing Library**: User-centric testing philosophy
- **Query Priority**: Use `getByRole` as default (accessibility-first)
- **User Interactions**: `@testing-library/user-event` for realistic simulations
- **Custom Matchers**: `@testing-library/jest-dom` for readable assertions

### 3.2 Backend Contract Testing

#### Decision: **utoipa-axum + Schemathesis**

#### Rationale

1. **Code-First Schema Generation**:
   - `utoipa-axum` generates OpenAPI specs from Rust code via macros
   - Ensures schema stays in sync with implementation
   - Native Axum integration via `OpenApiRouter`

2. **Automated Validation**:
   - Schemathesis performs property-based testing against OpenAPI schemas
   - Generates comprehensive test cases automatically
   - Validates both request/response structure and behavior

3. **TDD-Friendly**:
   - Define schema via `#[utoipa::path]` macros
   - Write tests using Axum's `TestServer` pattern
   - Implement handler logic
   - Validate with Schemathesis
   - Iterate

#### Axum Integration Testing Best Practices

- **Testing Without TCP**: Use Tower's `Service` trait for fast unit-like integration tests
- **Testing With TCP**: Use `TcpListener::bind("127.0.0.1:0")` for full HTTP testing when needed
- **TestServer Pattern**: Modern approach using Axum's built-in test utilities
- **Dependency Injection**: Make core logic injectable, use `mockall` for mocking

### 3.3 SQLite ORM

#### Decision: **SQLx**

#### Rationale

1. **Compile-Time Query Checking**:
   - `sqlx::query!` macro validates SQL against actual database schema at compile time
   - Type-safe queries without ORM abstraction overhead
   - Prevents SQL errors before runtime

2. **Async-Native**:
   - Built for async/await from the ground up
   - Perfect fit for async Axum framework
   - SQLite's synchronous API has minimal overhead for simple queries

3. **Simplicity (YAGNI)**:
   - Write standard SQL, gain safety through compile-time validation
   - No DSL to learn (unlike Diesel)
   - Easier to debug (SQL is visible and familiar)

4. **Migration Management**:
   - Built-in CLI: `sqlx migrate` for creating and running migrations
   - Migration files are plain SQL
   - Compile-time verification ensures migrations match code
   - Reversible migrations (up/down) per constitution Data Integrity principle

#### Alternatives Considered

| Feature | SQLx | Diesel |
|---------|------|--------|
| **Async Support** | Native | Added later (less ergonomic) |
| **Query Style** | Raw SQL + macros | Type-safe DSL |
| **Compile-time Safety** | ✅ `query!` macro | ✅ Type system |
| **Learning Curve** | Low (SQL knowledge) | Medium (DSL + SQL) |
| **Abstraction Level** | Low (explicit SQL) | High (ORM) |
| **Migration CLI** | ✅ Built-in | ✅ Built-in |

**When to Choose Diesel**: Team strongly prefers ORM abstraction, complex query composition requiring type-safe builder

**For This Project**: SQLx's simplicity and raw SQL approach align better with YAGNI principle

---

## 4. Module Dependency Graph

To verify acyclic dependencies (constitution Principle IV: Modular Monolith), the following module structure is proposed:

```
[Database Layer]
   ↑
[Persistence Service]
   ↑
[Business Logic Services]  ← [LLM Client] (independent)
   ├── PDF Processing
   ├── Term Extraction
   └── Term Normalization
   ↑
[API Layer]
```

**Validation**:
- ✅ No circular dependencies
- ✅ Each layer depends only on layers below
- ✅ LLM Client is a leaf dependency (no internal dependencies)
- ✅ Database layer has no upward dependencies

This will be detailed further in `data-model.md`.

---

## 5. Summary of Resolved NEEDS CLARIFICATION

| Item | Original State | Resolution |
|------|---------------|------------|
| **Frontend Framework** | NEEDS CLARIFICATION (Next.js vs SvelteKit) | **Vite + React** (simplicity, performance, ecosystem) |
| **PDF Extraction Library** | NEEDS CLARIFICATION (Rust crates) | **`pdf-extract`** (MIT, pure Rust, simple API) |
| **Frontend Testing** | NEEDS CLARIFICATION (Vitest vs Jest) | **Vitest + React Testing Library** (speed, TypeScript, ESM) |
| **Contract Testing** | NEEDS CLARIFICATION (framework) | **utoipa-axum + Schemathesis** (code-first, automated validation) |
| **SQLite ORM** | NEEDS CLARIFICATION (sqlx vs diesel) | **SQLx** (compile-time safety, async-native, simplicity) |

---

## 6. Updated Technical Context

All NEEDS CLARIFICATION markers can now be removed from plan.md and replaced with:

**Primary Dependencies**:
- Backend: Axum, utoipa-axum, SQLx, reqwest, pdf-extract
- Frontend: Vite, React, React Router, react-pdf, Vitest, React Testing Library

**Testing**:
- Backend: cargo test (unit + integration), utoipa-axum + Schemathesis (contract tests)
- Frontend: Vitest + React Testing Library + @testing-library/user-event

---

## References

- Frontend Framework Evaluation: `/Users/imai/Projects/paper-gloss/docs/frontend-framework-evaluation.md` (generated by agent)
- Rust PDF Library Evaluation: `/Users/imai/Projects/paper-gloss/docs/rust-pdf-library-evaluation.md` (generated by agent)
- Testing Framework Recommendations: Summarized above (agent output)

---

**Next Steps**: Proceed to Phase 1 (Design) to generate `data-model.md`, `contracts/`, and `quickstart.md`.
