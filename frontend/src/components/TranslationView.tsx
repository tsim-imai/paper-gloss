import { useState, useEffect } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { apiClient } from '../services/api'
import { Chunk, OccurrencesListResponse } from '../types'
import TermHighlight from './translation/TermHighlight'
import TermTooltip from './translation/TermTooltip'

interface TranslationViewProps {
  paperId: string
}

type LanguageMode = 'en' | 'ja'

export default function TranslationView({ paperId }: TranslationViewProps) {
  const [languageMode, setLanguageMode] = useState<LanguageMode>('ja')
  const [hoveredTermId, setHoveredTermId] = useState<string | null>(null)
  const [pinnedTermId, setPinnedTermId] = useState<string | null>(null)
  const [tooltipPosition, setTooltipPosition] = useState({ x: 0, y: 0 })
  const queryClient = useQueryClient()

  const { data, isLoading, error } = useQuery({
    queryKey: ['translation', paperId],
    queryFn: async () => {
      const response = await apiClient.getTranslation(paperId)
      return response.data.chunks as Chunk[]
    },
  })

  const { data: occurrencesData } = useQuery({
    queryKey: ['occurrences', paperId],
    queryFn: async () => {
      const response = await apiClient.listOccurrences(paperId)
      return response.data as OccurrencesListResponse
    },
    enabled: !!paperId,
  })

  const retryMutation = useMutation({
    mutationFn: (chunkId: string) => apiClient.retryChunk(chunkId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['translation', paperId] })
      queryClient.invalidateQueries({ queryKey: ['paper', paperId] })
    },
  })

  // Handle ESC key to close pinned tooltip
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && pinnedTermId) {
        setPinnedTermId(null)
      }
    }
    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [pinnedTermId])

  // Event handlers
  const handleTermHover = (termId: string | null, event: React.MouseEvent) => {
    if (pinnedTermId) return // Don't show hover tooltip when pinned
    setHoveredTermId(termId)
    if (termId) {
      setTooltipPosition({ x: event.clientX, y: event.clientY })
    }
  }

  const handleTermClick = (termId: string, event: React.MouseEvent) => {
    event.stopPropagation()
    setPinnedTermId(termId)
    setHoveredTermId(null)
    setTooltipPosition({ x: event.clientX, y: event.clientY })
  }

  const handleCloseTooltip = () => {
    setPinnedTermId(null)
  }

  const occurrences = occurrencesData?.occurrences || []
  const activeTermId = hoveredTermId || pinnedTermId

  if (isLoading) {
    return <div style={{ textAlign: 'center', padding: '2rem', color: '#999' }}>Loading translation...</div>
  }

  if (error) {
    return (
      <div style={{ color: '#f44336', padding: '1rem' }}>
        Error loading translation: {(error as any).message}
      </div>
    )
  }

  const chunks = data || []

  if (chunks.length === 0) {
    return (
      <div
        style={{
          padding: '2rem',
          textAlign: 'center',
          border: '1px dashed #444',
          borderRadius: '8px',
          color: '#999',
          backgroundColor: '#2a2a2a',
        }}
      >
        No translations available yet. Start translation pipeline to generate translations.
      </div>
    )
  }

  return (
    <div>
      {/* Language toggle controls */}
      <div
        style={{
          marginBottom: '1rem',
          padding: '0.75rem',
          backgroundColor: '#2a2a2a',
          border: '1px solid #444',
          borderRadius: '8px',
          display: 'flex',
          gap: '0.5rem',
          alignItems: 'center',
        }}
      >
        <span style={{ fontWeight: 'bold', fontSize: '0.875rem', color: '#999', marginRight: '0.25rem' }}>
          Language:
        </span>
        <button
          onClick={() => setLanguageMode('en')}
          style={{
            padding: '0.375rem 0.75rem',
            fontSize: '0.75rem',
            backgroundColor: languageMode === 'en' ? '#646cff' : '#444',
            color: languageMode === 'en' ? 'white' : '#999',
            border: 'none',
            borderRadius: '4px',
            cursor: 'pointer',
          }}
        >
          English
        </button>
        <button
          onClick={() => setLanguageMode('ja')}
          style={{
            padding: '0.375rem 0.75rem',
            fontSize: '0.75rem',
            backgroundColor: languageMode === 'ja' ? '#646cff' : '#444',
            color: languageMode === 'ja' ? 'white' : '#999',
            border: 'none',
            borderRadius: '4px',
            cursor: 'pointer',
          }}
        >
          Japanese
        </button>
      </div>

      {/* Chunks list */}
      <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
        {chunks.map((chunk) => (
          <div
            key={chunk.id}
            style={{
              border: '1px solid #444',
              borderRadius: '8px',
              padding: '1rem',
              backgroundColor: chunk.status === 'failed' ? '#3a1a1a' : '#2a2a2a',
            }}
          >
            {/* Chunk header */}
            <div
              style={{
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'center',
                marginBottom: '0.75rem',
                paddingBottom: '0.5rem',
                borderBottom: '1px solid #444',
              }}
            >
              <div style={{ display: 'flex', gap: '0.5rem', alignItems: 'center' }}>
                <div style={{ fontSize: '0.75rem', fontWeight: 'bold', color: '#999' }}>
                  Chunk #{chunk.chunk_index + 1}
                </div>
                {chunk.status === 'pending' && (
                  <span style={{ fontSize: '0.75rem', color: '#ff9800', fontStyle: 'italic' }}>
                    Not translated yet
                  </span>
                )}
                {chunk.status === 'translated' && (
                  <span style={{ fontSize: '0.75rem', color: '#4caf50' }}>
                    ✓ Translated
                  </span>
                )}
                {chunk.status === 'failed' && (
                  <span style={{ fontSize: '0.75rem', color: '#f44336' }}>
                    ✗ Failed (retried {chunk.retry_count} times)
                  </span>
                )}
              </div>
              <div style={{ display: 'flex', gap: '0.5rem', alignItems: 'center' }}>
                {/* Show translate button for all chunks */}
                <button
                  onClick={() => retryMutation.mutate(chunk.id)}
                  disabled={retryMutation.isPending}
                  style={{
                    padding: '0.25rem 0.75rem',
                    fontSize: '0.75rem',
                    backgroundColor: chunk.status === 'pending' ? '#ff9800' : '#646cff',
                    color: 'white',
                    border: 'none',
                    borderRadius: '4px',
                    cursor: retryMutation.isPending ? 'not-allowed' : 'pointer',
                    opacity: retryMutation.isPending ? 0.6 : 1,
                  }}
                  title={
                    chunk.status === 'pending'
                      ? 'Translate this chunk'
                      : chunk.status === 'failed'
                      ? 'Retry translation'
                      : 'Re-translate this chunk'
                  }
                >
                  {chunk.status === 'pending' ? 'Translate' : chunk.status === 'failed' ? 'Retry' : 'Re-translate'}
                </button>
              </div>
            </div>

            {/* Original text */}
            {languageMode === 'en' && (
              <div>
                <div
                  style={{
                    fontSize: '0.75rem',
                    fontWeight: 'bold',
                    color: '#999',
                    marginBottom: '0.5rem',
                    textTransform: 'uppercase',
                  }}
                >
                  Original
                </div>
                <div
                  style={{
                    lineHeight: '1.6',
                    whiteSpace: 'pre-wrap',
                    color: '#ffffff',
                  }}
                >
                  {chunk.original_text}
                </div>
              </div>
            )}

            {/* Translation */}
            {languageMode === 'ja' && (
              <div>
                <div
                  style={{
                    fontSize: '0.75rem',
                    fontWeight: 'bold',
                    color: '#999',
                    marginBottom: '0.5rem',
                    textTransform: 'uppercase',
                  }}
                >
                  Translation
                </div>
                {chunk.translated_text ? (
                  <div style={{ color: '#ffffff', lineHeight: '1.6' }}>
                    <TermHighlight
                      text={chunk.translated_text}
                      chunkId={chunk.id}
                      occurrences={occurrences}
                      highlightedTermId={activeTermId || undefined}
                      onTermHover={handleTermHover}
                      onTermClick={handleTermClick}
                    />
                  </div>
                ) : (
                  <div style={{ fontStyle: 'italic', color: '#999' }}>
                    {chunk.status === 'pending' ? 'Translation pending...' : 'Translation failed'}
                  </div>
                )}
              </div>
            )}

            {/* Error message for failed chunks */}
            {chunk.status === 'failed' && chunk.error_message && (
              <div
                style={{
                  marginTop: '0.75rem',
                  padding: '0.75rem',
                  backgroundColor: '#3a1a1a',
                  border: '1px solid #f44336',
                  borderRadius: '4px',
                  fontSize: '0.75rem',
                  color: '#f44336',
                }}
              >
                <strong>Error:</strong> {chunk.error_message}
              </div>
            )}
          </div>
        ))}
      </div>

      {/* Tooltip overlay */}
      {activeTermId && (
        <TermTooltip
          termId={activeTermId}
          position={tooltipPosition}
          isPinned={!!pinnedTermId}
          onClose={handleCloseTooltip}
        />
      )}
    </div>
  )
}
