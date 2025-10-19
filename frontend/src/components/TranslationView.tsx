import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { apiClient } from '../services/api'
import { Chunk } from '../types'

interface TranslationViewProps {
  paperId: string
}

export default function TranslationView({ paperId }: TranslationViewProps) {
  const [showOriginal, setShowOriginal] = useState(true)
  const [showTranslation, setShowTranslation] = useState(true)
  const queryClient = useQueryClient()

  const { data, isLoading, error } = useQuery({
    queryKey: ['translation', paperId],
    queryFn: async () => {
      const response = await apiClient.getTranslation(paperId)
      return response.data.chunks as Chunk[]
    },
  })

  const retryMutation = useMutation({
    mutationFn: (chunkId: string) => apiClient.retryChunk(chunkId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['translation', paperId] })
      queryClient.invalidateQueries({ queryKey: ['paper', paperId] })
    },
  })

  if (isLoading) {
    return <div style={{ textAlign: 'center', padding: '2rem' }}>Loading translation...</div>
  }

  if (error) {
    return (
      <div style={{ color: 'red', padding: '1rem' }}>
        Error loading translation: {(error as any).message}
      </div>
    )
  }

  const chunks = data || []

  if (chunks.length === 0) {
    return (
      <div style={{
        padding: '2rem',
        textAlign: 'center',
        border: '1px dashed #ccc',
        borderRadius: '8px',
        color: '#666'
      }}>
        No translations available yet. Start processing to generate translations.
      </div>
    )
  }

  return (
    <div>
      {/* View controls */}
      <div style={{
        marginBottom: '1rem',
        padding: '1rem',
        backgroundColor: '#f9f9f9',
        borderRadius: '8px',
        display: 'flex',
        gap: '1rem',
        alignItems: 'center'
      }}>
        <span style={{ fontWeight: 'bold' }}>Display:</span>
        <label style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', cursor: 'pointer' }}>
          <input
            type="checkbox"
            checked={showOriginal}
            onChange={(e) => setShowOriginal(e.target.checked)}
          />
          Original English
        </label>
        <label style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', cursor: 'pointer' }}>
          <input
            type="checkbox"
            checked={showTranslation}
            onChange={(e) => setShowTranslation(e.target.checked)}
          />
          Japanese Translation
        </label>
      </div>

      {/* Chunks list */}
      <div style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
        {chunks.map((chunk) => (
          <div
            key={chunk.id}
            style={{
              border: '1px solid #ccc',
              borderRadius: '8px',
              padding: '1rem',
              backgroundColor: chunk.status === 'failed' ? '#ffebee' : 'white',
            }}
          >
            {/* Chunk header */}
            <div style={{
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'center',
              marginBottom: '1rem',
              paddingBottom: '0.5rem',
              borderBottom: '1px solid #e0e0e0'
            }}>
              <div style={{ fontSize: '0.875rem', fontWeight: 'bold', color: '#666' }}>
                Chunk #{chunk.chunk_index + 1}
              </div>
              <div style={{ display: 'flex', gap: '0.5rem', alignItems: 'center' }}>
                {chunk.status === 'failed' && (
                  <>
                    <span style={{ fontSize: '0.75rem', color: '#f44336' }}>
                      Failed (retried {chunk.retry_count} times)
                    </span>
                    <button
                      onClick={() => retryMutation.mutate(chunk.id)}
                      disabled={retryMutation.isPending}
                      style={{
                        padding: '0.25rem 0.75rem',
                        fontSize: '0.75rem',
                        backgroundColor: '#646cff',
                        color: 'white',
                        border: 'none',
                        borderRadius: '4px',
                        cursor: 'pointer',
                      }}
                    >
                      Retry
                    </button>
                  </>
                )}
              </div>
            </div>

            {/* Original text */}
            {showOriginal && (
              <div style={{ marginBottom: showTranslation ? '1rem' : 0 }}>
                <div style={{
                  fontSize: '0.75rem',
                  fontWeight: 'bold',
                  color: '#666',
                  marginBottom: '0.5rem',
                  textTransform: 'uppercase'
                }}>
                  Original
                </div>
                <div style={{
                  lineHeight: '1.6',
                  whiteSpace: 'pre-wrap',
                  color: '#333'
                }}>
                  {chunk.original_text}
                </div>
              </div>
            )}

            {/* Translation */}
            {showTranslation && (
              <div>
                <div style={{
                  fontSize: '0.75rem',
                  fontWeight: 'bold',
                  color: '#666',
                  marginBottom: '0.5rem',
                  textTransform: 'uppercase'
                }}>
                  Translation
                </div>
                {chunk.translated_text ? (
                  <div style={{
                    lineHeight: '1.8',
                    whiteSpace: 'pre-wrap',
                    color: '#333'
                  }}>
                    {chunk.translated_text}
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
              <div style={{
                marginTop: '1rem',
                padding: '0.75rem',
                backgroundColor: '#fff',
                border: '1px solid #f44336',
                borderRadius: '4px',
                fontSize: '0.75rem',
                color: '#f44336'
              }}>
                <strong>Error:</strong> {chunk.error_message}
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  )
}
