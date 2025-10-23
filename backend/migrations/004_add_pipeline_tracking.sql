-- Add pipeline execution tracking fields to papers table
-- Date: 2025-10-23

ALTER TABLE papers ADD COLUMN translation_last_run_at TEXT;
ALTER TABLE papers ADD COLUMN terms_jp_last_run_at TEXT;
ALTER TABLE papers ADD COLUMN scan_jp_last_run_at TEXT;
ALTER TABLE papers ADD COLUMN definitions_last_run_at TEXT;
ALTER TABLE papers ADD COLUMN definitions_result_state TEXT;
