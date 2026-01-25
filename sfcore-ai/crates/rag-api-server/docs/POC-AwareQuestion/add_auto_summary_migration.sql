-- Migration: Add auto_summary column for AI-generated document summaries
-- This column will store LLM-generated summaries created during document upload
-- Used to answer meta-questions like "what is this document about?"

-- Add auto_summary column to TblDocuments
ALTER TABLE "TblDocuments" 
ADD COLUMN IF NOT EXISTS auto_summary TEXT;

-- Add comment to explain purpose
COMMENT ON COLUMN "TblDocuments".auto_summary IS 
'AI-generated summary of document content, created during upload processing. Used for quick overview responses.';

-- Create index for filtering documents with summaries (optional, for analytics)
CREATE INDEX IF NOT EXISTS idx_documents_has_summary 
ON "TblDocuments"(auto_summary) 
WHERE auto_summary IS NOT NULL;

-- Migration complete
-- After this migration:
-- 1. Existing documents will have auto_summary = NULL
-- 2. New uploads will generate auto_summary via DocumentService
-- 3. Optionally: Run batch job to generate summaries for existing documents
