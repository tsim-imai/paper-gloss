-- Add fields for JP-first pipeline
-- Date: 2025-10-23

ALTER TABLE occurrences ADD COLUMN surface TEXT;
ALTER TABLE occurrences ADD COLUMN method TEXT NOT NULL DEFAULT 'jp-scan';
ALTER TABLE occurrences ADD COLUMN variant_id TEXT;

CREATE INDEX idx_occurrences_method ON occurrences(method);
