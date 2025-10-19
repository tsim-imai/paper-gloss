# Frontend Framework Evaluation

**Date**: 2025-10-19
**Project**: paper-gloss MVP
**Decision Maker**: Technical Lead
**Status**: Recommendation Ready

## Executive Summary

**Recommended Choice**: **Vite + React (no framework)**

**Rationale**: This option best aligns with the Constitution's Simplicity-First (YAGNI) principle, provides the fastest client-side rendering performance for <100ms tooltip requirements, requires minimal boilerplate for a local single-user SPA, and avoids unnecessary server-side rendering complexity that Next.js/SvelteKit introduce.

**Key Trade-off Accepted**: No built-in routing conventions or server components. This is acceptable because the app is a single-page application with minimal navigation (translation view + glossary view), and the backend is already implemented in Rust (Axum), eliminating the need for API routes or SSR.

---

## Context

### Project Requirements Summary

- **Architecture**: Rust backend (Axum) serving REST API + TypeScript frontend SPA
- **Deployment**: Local-only, single-user, static build served by Axum
- **Critical Performance Requirement**: Tooltip display latency <100ms on hover/click
- **Core Features**:
  - PDF viewer integration (PDF.js)
  - Term highlighting in translated text with instant tooltips
  - Bilingual glossary search with real-time filtering
- **Constitution Principle**: Simplicity-First (YAGNI) - prefer straightforward solutions, avoid hypothetical complexity

### Why This Decision Matters

The frontend framework choice directly impacts:
1. **Development velocity**: Learning curve and boilerplate overhead
2. **Client-side performance**: Tooltip rendering latency (<100ms requirement)
3. **PDF.js integration complexity**: How easily can we embed and control PDF rendering?
4. **Testing ecosystem**: TDD is non-negotiable per Constitution Principle II
5. **Deployment simplicity**: Static build that Axum can serve as a single binary

---

## Options Evaluated

### 1. Next.js (App Router or Pages Router)

**Strengths**:
- ✅ **Production-grade ecosystem**: Mature, well-documented, large community
- ✅ **TypeScript support**: First-class, excellent DX
- ✅ **Testing ecosystem**: Vitest/Jest + React Testing Library fully supported
- ✅ **Static export**: `next build && next export` produces a static SPA

**Weaknesses**:
- ❌ **Unnecessary SSR complexity**: App Router/Pages Router are designed for server-side rendering and API routes. This project has a Rust backend, making Next.js API routes redundant.
- ❌ **Routing overhead**: File-based routing is overkill for a 2-3 view SPA (translation view, glossary view, settings).
- ❌ **Bundle size**: Next.js adds framework overhead (~70KB gzipped) for features this project doesn't need.
- ❌ **Learning curve**: App Router introduces Server Components, Suspense, and hydration concepts irrelevant for a client-only SPA.
- ❌ **Violates YAGNI**: Constitution Principle I explicitly rejects "flexible architectures" when simpler solutions exist. Next.js is designed for multi-page applications with server-side logic.

**Client-Side Rendering Performance**: Excellent once hydrated, but initial hydration adds 50-100ms overhead compared to plain React.

**PDF.js Integration**: Straightforward (React components can wrap PDF.js canvas), but no advantages over plain React.

**Verdict**: ❌ **Not Recommended** - Too complex for a local SPA. The framework's strengths (SSR, API routes, file-based routing) are unused.

---

### 2. SvelteKit

**Strengths**:
- ✅ **Compiled framework**: Svelte compiles to vanilla JS, resulting in smaller bundle sizes (~30-40% smaller than React).
- ✅ **Reactive by default**: Less boilerplate for state management compared to React's `useState`/`useEffect`.
- ✅ **TypeScript support**: First-class via `lang="ts"` in `.svelte` files.
- ✅ **Adapter-static**: SvelteKit can produce a static SPA via `adapter-static`.

**Weaknesses**:
- ❌ **Smaller ecosystem**: Fewer third-party libraries, less Stack Overflow coverage compared to React.
- ❌ **SvelteKit-specific abstractions**: Load functions, server/client boundary, filesystem-based routing add concepts unnecessary for a client-only SPA.
- ❌ **Testing maturity**: Svelte Testing Library is less mature than React Testing Library; fewer examples for complex UI interactions (tooltip hover, glossary search).
- ❌ **Learning curve**: Team/developer familiarity is lower than React (personal factor, but relevant for MVP velocity).
- ❌ **Violates YAGNI**: Like Next.js, SvelteKit is designed for full-stack apps. Adapters and server hooks are unused in this project.

**Client-Side Rendering Performance**: Excellent (compiled framework = less runtime overhead). Tooltip rendering likely 10-20ms faster than React.

**PDF.js Integration**: Straightforward but less community examples compared to React ecosystem.

**Verdict**: ⚠️ **Conditionally Acceptable** - If the team has Svelte experience and values smaller bundles, this works. However, the ecosystem and testing maturity concerns outweigh the bundle size benefits for a local-only app.

---

### 3. Vite + React (No Framework)

**Strengths**:
- ✅ **Simplicity**: Zero framework abstractions beyond React. No routing conventions, no SSR, no build-time magic. Just a bundler (Vite) and a library (React).
- ✅ **Full control**: Complete control over bundle, routing (use `react-router` only if needed), and build output.
- ✅ **Fast development**: Vite HMR is instant; no framework reload overhead.
- ✅ **Minimal boilerplate**: A few config files (`vite.config.ts`, `tsconfig.json`), no adapter/export setup.
- ✅ **Excellent client-side performance**: No hydration overhead, pure CSR. Tooltip rendering can hit <50ms easily.
- ✅ **Testing ecosystem**: Vitest (Vite-native) + React Testing Library is the gold standard. Full TDD support.
- ✅ **PDF.js integration**: React wrappers like `react-pdf` or direct `pdf.js` canvas integration well-documented.
- ✅ **TypeScript support**: First-class via Vite plugins.
- ✅ **Static build simplicity**: `vite build` produces a `dist/` folder with `index.html` + assets that Axum can serve directly.

**Weaknesses**:
- ⚠️ **No built-in routing**: Requires manual `react-router` setup if multiple views needed. Mitigation: This app needs 2-3 views max (translation, glossary, settings). A simple tab-based UI or conditional rendering is sufficient—no routing library required unless complexity grows.
- ⚠️ **No server-side conventions**: No API routes, no server actions. Mitigation: Irrelevant—backend is Rust (Axum).

**Client-Side Rendering Performance**: Best-in-class for this use case. No framework overhead, pure React rendering. Tooltip latency <50ms achievable.

**PDF.js Integration**: Most React ecosystem examples use Vite or CRA (Create React App). Direct translation to this setup.

**Verdict**: ✅ **Strongly Recommended** - Aligns perfectly with Constitution Principle I (Simplicity-First). No unused abstractions, minimal learning curve, optimal client-side performance.

---

### 4. Other Lightweight Options

#### Vite + Vue 3 (Composition API)

**Evaluation**:
- Similar to Vite + React in simplicity
- Smaller bundle than React (~20-30% smaller)
- **Weakness**: Less ecosystem maturity for PDF.js integration, fewer testing examples for complex UI interactions
- **Verdict**: ⚠️ **Acceptable alternative** if team prefers Vue's Composition API over React hooks, but React ecosystem is safer for PDF.js integration.

#### Astro (Static Site Generator)

**Evaluation**:
- Designed for content-heavy sites with partial hydration (islands architecture)
- **Weakness**: Overkill for a fully interactive SPA with real-time search and tooltips. Astro shines when most content is static (blogs, marketing pages).
- **Verdict**: ❌ **Not Recommended** - Wrong tool for this use case.

#### Solid.js

**Evaluation**:
- Ultra-fast reactive primitives, no virtual DOM
- **Weakness**: Tiny ecosystem, bleeding-edge, limited testing resources, high learning curve
- **Verdict**: ❌ **Not Recommended** - Violates Simplicity-First (unproven for production use).

---

## Decision Matrix

| Criterion | Weight | Next.js (App) | SvelteKit | Vite + React | Vite + Vue |
|-----------|--------|---------------|-----------|--------------|------------|
| **Simplicity (YAGNI)** | 🔴 Critical | ❌ Poor (SSR/routing overhead) | ⚠️ Fair (adapter abstractions) | ✅ Excellent (zero abstractions) | ✅ Excellent |
| **Client-Side Performance (<100ms tooltips)** | 🔴 Critical | ⚠️ Good (hydration delay) | ✅ Excellent (compiled) | ✅ Excellent (pure CSR) | ✅ Excellent |
| **PDF.js Integration Ease** | 🟡 High | ✅ Excellent (React ecosystem) | ⚠️ Fair (fewer examples) | ✅ Excellent (React ecosystem) | ⚠️ Fair |
| **TypeScript Support** | 🟡 High | ✅ Excellent | ✅ Excellent | ✅ Excellent | ✅ Excellent |
| **Testing Ecosystem (TDD)** | 🔴 Critical | ✅ Excellent | ⚠️ Fair (less mature) | ✅ Excellent | ⚠️ Good |
| **Deployment Simplicity (static build)** | 🟡 High | ✅ Good (`next export`) | ✅ Good (`adapter-static`) | ✅ Excellent (native) | ✅ Excellent |
| **Learning Curve** | 🟢 Medium | ❌ High (App Router concepts) | ⚠️ Medium (Svelte syntax) | ✅ Low (standard React) | ⚠️ Medium |
| **Bundle Size** | 🟢 Low | ⚠️ Fair (~70KB framework) | ✅ Excellent (~20KB) | ⚠️ Good (~40KB React) | ✅ Excellent (~25KB) |

**Legend**: 🔴 Critical = Non-negotiable per constitution, 🟡 High = Significant impact, 🟢 Medium/Low = Nice-to-have

---

## Recommendation: Vite + React

### Why This Choice Wins

1. **Simplicity-First Alignment**: No unused server-side abstractions. Just a bundler (Vite) and a UI library (React). Zero framework magic.
2. **Optimal Client-Side Performance**: Pure CSR with no hydration overhead. Tooltip rendering can easily hit <50ms (well under 100ms requirement).
3. **Mature Testing Ecosystem**: Vitest + React Testing Library is the gold standard for TDD. Non-negotiable per Constitution Principle II.
4. **PDF.js Integration**: Most examples and community support are in the React ecosystem (e.g., `react-pdf`, direct canvas integration).
5. **Static Build Simplicity**: `vite build` → `dist/` → serve via Axum. One command, no adapters.
6. **Developer Velocity**: Minimal learning curve (standard React + Vite). Fast HMR, simple mental model.

### Architecture Sketch

```
Frontend (Vite + React)
├── src/
│   ├── main.tsx              # Entry point
│   ├── App.tsx               # Root component (tab navigation: Translation | Glossary)
│   ├── components/
│   │   ├── PdfViewer.tsx     # PDF.js integration (react-pdf or direct canvas)
│   │   ├── TranslationView.tsx # Rendered translation with term highlights
│   │   ├── TermTooltip.tsx   # <100ms tooltip component (portal-based)
│   │   ├── GlossaryPanel.tsx # Bilingual search + term list
│   ├── hooks/
│   │   ├── useTermHighlight.ts # Manages tooltip state + hover detection
│   │   ├── useGlossarySearch.ts # Real-time search with normalization
│   ├── api/
│   │   ├── client.ts          # Axios/fetch wrapper for Rust API
│   ├── types/
│   │   ├── api.ts             # TypeScript types from Rust (utoipa-generated)
├── vite.config.ts
├── tsconfig.json
├── index.html

Build Output (served by Axum)
dist/
├── index.html
├── assets/
│   ├── index.[hash].js
│   ├── index.[hash].css
```

### Key Implementation Notes

- **Routing**: Use React state or `react-router` (v6) only if needed. For 2-3 views, tab-based navigation (`useState` to toggle `<TranslationView>` vs `<GlossaryPanel>`) is sufficient.
- **Tooltip Performance**: Use React Portal + CSS transitions. Pre-fetch term definitions on page load to avoid API latency during hover.
- **PDF.js**: Use `react-pdf` library for quick integration, or raw `pdf.js` if fine-grained canvas control is needed.
- **Testing**: Vitest for unit tests, React Testing Library for component tests, MSW (Mock Service Worker) for API mocking.

---

## Alternative If Team Prefers Svelte

If the team has strong Svelte experience and values minimal bundle size over ecosystem maturity:

**Acceptable Alternative**: SvelteKit with `adapter-static`

**Justification**: Smaller bundle (~30-40% reduction) and reactive syntax reduce boilerplate for glossary search filtering. However, this trades ecosystem safety (PDF.js examples, testing maturity) for bundle size in a local-only app where bundle size is not a bottleneck.

**Recommendation**: Only choose SvelteKit if the team commits to:
1. Thoroughly testing PDF.js integration early (validate canvas rendering, scroll sync).
2. Investing in Svelte Testing Library setup for tooltip and glossary tests (less Stack Overflow support).

Otherwise, stick with Vite + React.

---

## Rejected Options

### Next.js
**Reason**: Violates Constitution Principle I (Simplicity-First). SSR, API routes, and file-based routing are unused in a client-only SPA with a Rust backend.

### Astro
**Reason**: Wrong architecture for a fully interactive SPA. Designed for content sites with partial hydration.

### Solid.js
**Reason**: Ecosystem immaturity and bleeding-edge status violate risk tolerance for MVP.

---

## Action Items

1. ✅ Initialize Vite + React project:
   ```bash
   npm create vite@latest frontend -- --template react-ts
   cd frontend
   npm install
   ```

2. ✅ Add core dependencies:
   ```bash
   npm install react-router-dom react-pdf axios
   npm install -D vitest @testing-library/react @testing-library/jest-dom jsdom
   ```

3. ✅ Configure Vite to proxy API requests to Axum during development:
   ```ts
   // vite.config.ts
   export default defineConfig({
     server: {
       proxy: {
         '/api': 'http://localhost:8000'
       }
     }
   })
   ```

4. ✅ Set up Vitest for TDD:
   ```ts
   // vite.config.ts
   export default defineConfig({
     test: {
       globals: true,
       environment: 'jsdom',
       setupFiles: './src/test/setup.ts'
     }
   })
   ```

5. ✅ Configure Axum to serve static frontend build:
   ```rust
   // Serve dist/ folder at root
   Router::new()
       .nest_service("/", ServeDir::new("frontend/dist"))
       .nest("/api", api_routes())
   ```

---

## Conclusion

**Final Decision**: **Vite + React**

This choice maximizes simplicity (Constitution Principle I), ensures <100ms tooltip performance, leverages the mature React ecosystem for PDF.js and testing, and produces a trivial static build that Axum can serve directly. No framework abstractions are wasted.

**Confidence Level**: High (9/10)
**Risk**: Low - Standard React + Vite stack is well-proven for SPAs.
**Review Date**: N/A (decision locked unless new requirements emerge)

---

## Appendix: Performance Benchmarks

### Tooltip Rendering Latency (Estimated)

- **Next.js (App Router)**: 80-120ms (hydration + React render)
- **SvelteKit**: 40-60ms (compiled, minimal runtime)
- **Vite + React**: 30-50ms (pure CSR, optimized React)
- **Vite + Vue**: 30-50ms (similar to React)

**Conclusion**: All options meet the <100ms requirement, but Vite + React/Vue have the lowest floor.

### Bundle Size (Production Build, Gzipped)

- **Next.js**: ~85KB (framework + React)
- **SvelteKit**: ~25KB (compiled Svelte)
- **Vite + React**: ~45KB (React + ReactDOM)
- **Vite + Vue**: ~30KB (Vue runtime)

**Conclusion**: Bundle size differences are negligible for a local-only app. Network latency is zero (localhost). Optimize for developer velocity, not bytes.

---

## References

- [Vite Documentation](https://vitejs.dev)
- [React Testing Library Best Practices](https://testing-library.com/docs/react-testing-library/intro/)
- [PDF.js React Integration Guide](https://github.com/wojtekmaj/react-pdf)
- [Constitution Principle I: Simplicity-First (YAGNI)](/Users/imai/Projects/paper-gloss/.specify/memory/constitution.md#i-simplicity-first-yagni)
