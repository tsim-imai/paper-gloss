-- Add status tracking columns to chunks table

ALTER TABLE chunks ADD COLUMN status TEXT NOT NULL DEFAULT 'pending' CHECK(status IN ('pending', 'translated', 'failed'));
ALTER TABLE chunks ADD COLUMN retry_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE chunks ADD COLUMN error_message TEXT;

-- Create index for status queries
CREATE INDEX idx_chunks_status ON chunks(status);
CREATE INDEX idx_chunks_paper_status ON chunks(paper_id, status);
