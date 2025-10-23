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
  text: string
  provider: string
}

export interface TermVariant {
  lang: string
  surface: string
}

export interface TermDetail {
  id: string
  slug: string
  lemma_en: string
  lemma_ja: string
  reading_kana?: string
  pos?: string
  tags?: string
  note?: string
  definition?: Definition
  variants: TermVariant[]
  created_at: string
  updated_at: string
}

export interface Occurrence {
  id: string
  term_id: string
  paper_id: string
  chunk_id: string
  start_pos: number
  end_pos: number
  created_at: string
}

export interface OccurrencesListResponse {
  occurrences: Occurrence[]
  total: number
}

export interface TermListItem {
  id: string
  slug: string
  lemma_en: string
  lemma_ja: string
  reading_kana?: string
  pos?: string
  tags?: string
  occurrence_count: number
  created_at: string
  updated_at: string
}

export interface TermListResponse {
  items: TermListItem[]
  total: number
  page: number
  limit: number
  total_pages: number
}

export interface PaginatedResponse<T> {
  items: T[]
  total: number
  page: number
  limit: number
  total_pages: number
}

// Pipeline status types (JP-first architecture)
export type PipelineStatus = 'idle' | 'processing' | 'completed' | 'failed'
export type PipelineResultState = 'completed_nonempty' | 'completed_empty' | 'failed'

export interface TranslationProgress {
  total_chunks: number
  completed_chunks: number
  failed_chunks: number
  status: PipelineStatus
}

export interface TermsJpProgress {
  total_terms: number
  last_run_at?: string
  status: PipelineStatus
}

export interface ScanJpProgress {
  total_occurrences: number
  last_run_at?: string
  status: PipelineStatus
}

export interface DefinitionsProgress {
  generated: number
  failed: number
  last_run_at?: string
  status: PipelineStatus
  result_state?: PipelineResultState
}

export interface PipelineProgress {
  translation: TranslationProgress
  terms_jp: TermsJpProgress
  scan_jp: ScanJpProgress
  definitions: DefinitionsProgress
}

export interface ProcessingStatusResponse {
  paper_id: string
  status: PaperStatus
  progress: PipelineProgress
}
