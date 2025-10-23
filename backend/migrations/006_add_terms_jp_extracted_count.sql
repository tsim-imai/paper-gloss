-- Add terms_jp_extracted_count column to track extracted terms from Pipeline B
-- Date: 2025-10-24

ALTER TABLE papers ADD COLUMN terms_jp_extracted_count INTEGER DEFAULT 0;

-- Update existing papers with current total term count
-- (Since there's only one paper with terms, set its count to the total number of terms)
UPDATE papers
SET terms_jp_extracted_count = (SELECT COUNT(*) FROM terms)
WHERE terms_jp_last_run_at IS NOT NULL;
