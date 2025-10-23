-- Pipeline locks table to ensure one running pipeline per paper
-- Date: 2025-10-23

CREATE TABLE IF NOT EXISTS pipeline_locks (
    paper_id TEXT PRIMARY KEY,
    pipeline TEXT NOT NULL,
    locked_at DATETIME NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_pipeline_locks_locked_at ON pipeline_locks(locked_at);

