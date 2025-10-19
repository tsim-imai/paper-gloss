# Quickstart: Paper Translation & Glossary System

**Date**: 2025-10-19
**Branch**: `001-paper-translation-glossary`
**Purpose**: Setup guide for developers to get the system running locally

---

## Prerequisites

### Required Software

1. **Rust** (latest stable, 1.75+)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup update stable
   ```

2. **Node.js** (v18+ with npm)
   ```bash
   # macOS (Homebrew)
   brew install node

   # Or use nvm for version management
   curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
   nvm install 18
   nvm use 18
   ```

3. **SQLx CLI** (for database migrations)
   ```bash
   cargo install sqlx-cli --no-default-features --features sqlite
   ```

4. **Optional: Schemathesis** (for contract testing)
   ```bash
   pip install schemathesis
   ```

### LLM API Access

Configure access to an OpenAI-compatible LLM endpoint:

- **Local**: `http://localhost:8000/v1` (e.g., LM Studio, Ollama with OpenAI shim)
- **Remote**: Any OpenAI-compatible API (OpenAI, Anthropic Claude via proxy, etc.)

**Requirements**:
- Max token context: 30,000 tokens
- Concurrent requests: Up to 10

---

## Project Setup

### 1. Clone Repository (if applicable)

```bash
git clone <repository-url>
cd paper-gloss
git checkout 001-paper-translation-glossary
```

### 2. Backend Setup (Rust)

#### Install Dependencies

```bash
cd backend
cargo build
```

**Key Dependencies**:
- Axum (web framework)
- utoipa-axum (OpenAPI generation)
- SQLx (SQLite ORM with compile-time checking)
- reqwest (HTTP client for LLM API)
- pdf-extract (PDF text extraction)

#### Configure Environment

Create `.env` file in `backend/`:

```env
# Database
DATABASE_URL=sqlite://paper-gloss.db

# LLM API
AI_API_BASE=http://localhost:8000/v1
AI_API_KEY=your-api-key-here

# Server
HOST=127.0.0.1
PORT=3000

# Logging
RUST_LOG=info
```

#### Run Database Migrations

```bash
# Create database and run initial schema migration
sqlx database create
sqlx migrate run

# Verify migrations applied
sqlx migrate info
```

**Migration Files**:
- Located in `backend/migrations/`
- Format: `<timestamp>_<description>.sql`
- Each migration has `up` (apply) and `down` (revert) sections

#### Compile-Time Query Checking (Optional)

SQLx verifies SQL queries at compile time against the actual database schema.

```bash
# Prepare query metadata for offline compilation
sqlx prepare

# Now cargo build will verify queries without database connection
cargo build
```

#### Run Backend Server

```bash
# Development mode (auto-reload with cargo-watch)
cargo install cargo-watch
cargo watch -x run

# Or standard run
cargo run
```

Server starts at: `http://localhost:3000`

**Verify**:
```bash
curl http://localhost:3000/api/papers
# Expected: {"papers":[],"total":0,"page":1,"limit":20}
```

---

### 3. Frontend Setup (TypeScript + React)

#### Install Dependencies

```bash
cd frontend
npm install
```

**Key Dependencies**:
- Vite (build tool)
- React + React Router
- react-pdf (PDF.js wrapper)
- @tanstack/react-query (API client/cache)
- Vitest + React Testing Library (testing)

#### Configure Environment

Create `.env` file in `frontend/`:

```env
# Backend API URL (proxied in dev, direct in production)
VITE_API_BASE_URL=http://localhost:3000/api
```

#### Configure Vite Proxy (Development)

Edit `vite.config.ts`:

```typescript
export default defineConfig({
  server: {
    proxy: {
      '/api': {
        target: 'http://localhost:3000',
        changeOrigin: true,
      }
    }
  }
})
```

#### Run Frontend Dev Server

```bash
npm run dev
```

Frontend starts at: `http://localhost:5173`

**Verify**:
- Open browser to `http://localhost:5173`
- UI should load (empty paper list initially)

---

## Running the Full Stack

### Development Mode

**Terminal 1** (Backend):
```bash
cd backend
cargo watch -x run
```

**Terminal 2** (Frontend):
```bash
cd frontend
npm run dev
```

Access application: `http://localhost:5173`

---

## Database Management

### Backup Database (CRITICAL)

**Constitution Requirement**: Data Integrity and User Control (Principle V)

Before any risky operations (schema changes, major updates), **back up the database**:

```bash
# Manual backup
cp backend/paper-gloss.db backend/paper-gloss.db.backup-$(date +%Y%m%d-%H%M%S)

# Or use SQLite backup command
sqlite3 backend/paper-gloss.db ".backup 'paper-gloss.db.backup-$(date +%Y%m%d-%H%M%S)'"
```

**Restore from Backup**:
```bash
cp backend/paper-gloss.db.backup-YYYYMMDD-HHMMSS backend/paper-gloss.db
```

**Automated Backup** (Optional):
Add to cron or systemd timer for daily backups:
```bash
# Daily backup at 2 AM
0 2 * * * cp /path/to/backend/paper-gloss.db /path/to/backups/paper-gloss-$(date +\%Y\%m\%d).db
```

### Inspect Database

```bash
# SQLite CLI
sqlite3 backend/paper-gloss.db

# List tables
.tables

# Describe schema
.schema papers

# Query
SELECT * FROM papers LIMIT 5;

# Exit
.quit
```

### Reset Database (Development Only)

**⚠️ WARNING: Deletes all data**

```bash
cd backend
sqlx database drop
sqlx database create
sqlx migrate run
```

### Migration Management

**Create New Migration**:
```bash
cd backend
sqlx migrate add <description>

# Example
sqlx migrate add add_term_frequency_column
```

This creates `backend/migrations/<timestamp>_<description>.sql` with `up` and `down` sections.

**Apply Migrations**:
```bash
sqlx migrate run
```

**Revert Last Migration**:
```bash
sqlx migrate revert
```

**Check Migration Status**:
```bash
sqlx migrate info
```

---

## Testing

### Backend Tests (Rust)

**Unit Tests**:
```bash
cd backend
cargo test
```

**Integration Tests** (requires database):
```bash
# Run all tests including integration tests
cargo test --all

# Run specific integration test
cargo test --test integration_paper_import
```

**Contract Tests** (Schemathesis):
```bash
# Start backend server first
cargo run &

# Run contract tests against OpenAPI schema
schemathesis run ../specs/001-paper-translation-glossary/contracts/openapi.yaml \
  --base-url http://localhost:3000/api \
  --checks all
```

### Frontend Tests (TypeScript)

**Unit + Component Tests**:
```bash
cd frontend
npm test
```

**Watch Mode** (TDD):
```bash
npm test -- --watch
```

**Coverage**:
```bash
npm test -- --coverage
```

---

## First-Time Usage

### 1. Import Your First Paper

**Option A: Upload PDF from Filesystem**

```bash
curl -X POST http://localhost:3000/api/papers/import \
  -F "file=@/path/to/paper.pdf" \
  -F "title=Optional Paper Title"
```

**Option B: Import from URL**

```bash
curl -X POST http://localhost:3000/api/papers/import \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://arxiv.org/pdf/1706.03762.pdf",
    "title": "Attention Is All You Need"
  }'
```

Response:
```json
{
  "paper_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "processing",
  "message": "Paper imported successfully. Processing started."
}
```

### 2. Check Processing Status

```bash
curl http://localhost:3000/api/papers/550e8400-e29b-41d4-a716-446655440000/process
```

Response shows progress:
```json
{
  "paper_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "processing",
  "progress": {
    "extraction": "completed",
    "translation": {
      "total_chunks": 45,
      "completed_chunks": 32,
      "failed_chunks": 0
    },
    "term_extraction": "in_progress",
    "definitions": {
      "total_terms": 28,
      "completed_definitions": 15
    }
  }
}
```

### 3. View Translation

Once `status` is `completed`:

```bash
curl http://localhost:3000/api/papers/550e8400-e29b-41d4-a716-446655440000/translation
```

Returns translated chunks with term highlighting.

### 4. Browse Glossary

```bash
# List all terms
curl http://localhost:3000/api/terms

# Search for specific term
curl http://localhost:3000/api/terms?q=neural%20network&lang=both
```

---

## Directory Structure After Setup

```
paper-gloss/
├── backend/
│   ├── src/
│   ├── tests/
│   ├── migrations/
│   ├── .env
│   ├── paper-gloss.db         # SQLite database (created after migration)
│   └── Cargo.toml
├── frontend/
│   ├── src/
│   ├── tests/
│   ├── .env
│   ├── vite.config.ts
│   └── package.json
├── artifacts/                   # Created at runtime (git ignored)
│   └── papers/{paper_id}/
│       ├── source.pdf
│       ├── text/
│       ├── translations/
│       ├── terms/
│       └── llm_logs/
├── data/
│   └── seeds/                   # Optional initial data
└── specs/
    └── 001-paper-translation-glossary/
        ├── spec.md
        ├── plan.md
        ├── research.md
        ├── data-model.md
        ├── contracts/
        └── quickstart.md        # This file
```

---

## Troubleshooting

### Backend Issues

**SQLx compile-time verification fails**:
```bash
# Regenerate query metadata
sqlx prepare

# Or disable offline mode temporarily
cargo build --no-default-features
```

**Port 3000 already in use**:
```bash
# Change PORT in backend/.env
PORT=3001
```

**LLM API connection fails**:
```bash
# Verify LLM endpoint
curl http://localhost:8000/v1/models

# Check backend/.env has correct AI_API_BASE and AI_API_KEY
```

**PDF extraction fails**:
- Ensure PDF is text-based (not scanned image)
- Check `backend/src/services/pdf/` logs for specific error
- Try with different PDF (some PDFs have complex structures)

### Frontend Issues

**Vite dev server fails to start**:
```bash
# Clear npm cache and reinstall
rm -rf node_modules package-lock.json
npm install
```

**API requests failing (CORS)**:
- Verify Vite proxy configuration in `vite.config.ts`
- Ensure backend is running on `http://localhost:3000`

**PDF viewer not loading**:
- Check browser console for errors
- Verify `react-pdf` worker is configured correctly

---

## Production Build (Local Deployment)

### 1. Build Frontend

```bash
cd frontend
npm run build
```

Output: `frontend/dist/` directory with static files.

### 2. Serve Frontend from Backend

Configure Axum to serve static files from `frontend/dist/`:

```rust
// backend/src/main.rs
use tower_http::services::ServeDir;

let app = Router::new()
    .nest("/api", api_routes())
    .nest_service("/", ServeDir::new("../frontend/dist"));
```

### 3. Run Production Server

```bash
cd backend
cargo build --release
./target/release/paper-gloss
```

Access application: `http://localhost:3000` (serves both API and frontend)

---

## Next Steps

1. **Read Constitution**: Review [.specify/memory/constitution.md](../../.specify/memory/constitution.md) for development principles
2. **Implement Tasks**: Run `/speckit.tasks` to generate task breakdown
3. **TDD Workflow**: Write tests first (Principle II: Test-First Development)
4. **Backup Database**: Set up automated backups before processing real papers

---

**Questions?** Refer to:
- [spec.md](./spec.md) - Functional requirements
- [data-model.md](./data-model.md) - Database schema
- [contracts/openapi.yaml](./contracts/openapi.yaml) - API documentation
- [research.md](./research.md) - Technology choices and rationale
