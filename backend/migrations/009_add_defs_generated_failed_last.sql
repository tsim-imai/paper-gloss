-- Track per-paper last D run generated/failed counts for Status
-- Date: 2025-10-24

ALTER TABLE papers ADD COLUMN definitions_generated_last INTEGER DEFAULT 0;
ALTER TABLE papers ADD COLUMN definitions_failed_last INTEGER DEFAULT 0;

