// Type definitions for Paper Gloss frontend

export type PaperStatus = 'pending' | 'processing' | 'completed' | 'failed'

export interface Paper {
  id: string
  title: string
  arxiv_id?: string
  source: 'upload' | 'url'
  status: PaperStatus
  file_path: string
  text_content?: string
  total_chunks?: number
  translated_chunks?: number
  failed_chunks?: number
  created_at: string
  updated_at: string
}

export interface Chunk {
  id: string
  paper_id: string
  chunk_index: number
  original_text: string
  translated_text?: string
  content_hash: string
  status: 'pending' | 'translated' | 'failed'
  retry_count: number
  error_message?: string
  created_at: string
  updated_at: string
}

export interface Term {
  id: string
  lemma_en: string
  lemma_ja: string
  reading_kana?: string
  pos?: string
  tags?: string
  note?: string
  created_at: string
  updated_at: string
}

export interface Definition {
  id: string
  term_id: string
  source: string
  definition_text_en: string
  definition_text_ja: string
  context?: string
  created_at: string
}

export interface Occurrence {
  id: string
  paper_id: string
  chunk_id: string
  term_id: string
  text_en: string
  text_ja?: string
  context_before: string
  context_after: string
  position: number
  created_at: string
}

export interface PaginatedResponse<T> {
  items: T[]
  total: number
  page: number
  limit: number
  total_pages: number
}

export interface ProcessingProgress {
  status: PaperStatus
  total_chunks: number
  translated_chunks: number
  failed_chunks: number
  extracted_terms: number
}
