# Repository Guidelines

## Project Structure & Module Organization
The repository splits a Rust backend (`backend/`) and a Vite + React frontend (`frontend/`). Backend routes sit in `backend/src/api`, supporting services in `backend/src/services`, shared types in `backend/src/models`, migrations in `backend/migrations`, and test helpers in `backend/tests/common`. The frontend keeps UI and hooks in `frontend/src`, with feature-aligned tests in `frontend/tests/{components,pages,services,integration,performance}`; documentation and specs live in `docs/` and `specs/`, while `artifacts/` stores generated assets.

## Build, Test, and Development Commands
- `cp backend/.env.example backend/.env` / `cp frontend/.env.example frontend/.env.local`: create env files and point `AI_API_*` plus `VITE_API_BASE_URL` at your local services.
- `cd backend && cargo run`: launch the Axum API (uses `DATABASE_URL`).
- `cd backend && cargo test`: run unit, integration, and contract suites.
- `cd backend && cargo fmt && cargo clippy -- -D warnings`: format and lint the Rust codebase.
- `cd frontend && npm install`: install or refresh frontend dependencies.
- `cd frontend && npm run dev`: start the Vite dev server.
- `cd frontend && npm run build && npm run preview`: build and smoke-test the bundle.
- `cd frontend && npm run test` / `npm run lint` / `npm run format`: run Vitest, ESLint, and Prettier.

## Coding Style & Naming Conventions
Rust code uses edition 2021 with `rustfmt` defaults (4-space indent, snake_case modules, UpperCamelCase types); treat `clippy` warnings as errors and keep feature modules lean. Frontend code follows ESLint + Prettier, PascalCase React components in `src/components`, camelCase hooks and utilities, and colocated test fixtures or styles.

## Testing Guidelines
Favor Rust unit tests near their modules and cross-cutting scenarios in `backend/tests/integration` or `backend/tests/contract`. Use `serial_test` only when stateful, and reset temporary databases with helpers in `backend/tests/common`. Frontend tests rely on Vitest + Testing Library, mock HTTP calls through `frontend/tests/msw`, and mirror component names (e.g., `ComponentName.test.tsx`). Ship matching backend and frontend coverage whenever a feature spans both.

## Commit & Pull Request Guidelines
Follow the Conventional Commits style in the history (`feat(backend): ...`, `test(frontend): ...`). Keep subjects ≤72 characters, imperative, and add body context for migrations or API changes. Pull requests should describe user impact, reference issues, list validation steps (`cargo test`, `npm run test`, etc.), and attach screenshots or API samples when behavior shifts.
