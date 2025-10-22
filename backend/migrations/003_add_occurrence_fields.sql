-- Extend occurrences with surface/method/variant_id for tagged-translation pipeline

ALTER TABLE occurrences ADD COLUMN surface TEXT;
ALTER TABLE occurrences ADD COLUMN method TEXT NOT NULL DEFAULT 'tagged-translation';
ALTER TABLE occurrences ADD COLUMN variant_id TEXT NULL;

-- Optional index for method-based queries
CREATE INDEX IF NOT EXISTS idx_occurrences_method ON occurrences(method);

