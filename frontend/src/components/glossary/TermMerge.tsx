import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { apiClient } from '../../services/api'

interface DuplicatePair {
  term1: {
    id: string
    lemma_en: string
    lemma_ja: string
    reading_kana?: string
  }
  term2: {
    id: string
    lemma_en: string
    lemma_ja: string
    reading_kana?: string
  }
  similarity_score: number
  reason: string
}

interface TermMergeProps {
  onClose: () => void
}

/**
 * Term merge component for handling duplicate terms
 * Allows users to select duplicate pairs and merge them with confirmation
 */
export default function TermMerge({ onClose }: TermMergeProps) {
  const [selectedPair, setSelectedPair] = useState<DuplicatePair | null>(null)
  const [keepTerm, setKeepTerm] = useState<'term1' | 'term2'>('term1')
  const queryClient = useQueryClient()

  // Fetch duplicate pairs
  const { data: duplicatesData, isLoading } = useQuery({
    queryKey: ['duplicates'],
    queryFn: async () => {
      const response = await apiClient.findDuplicates()
      return response.data as { duplicates: DuplicatePair[]; total: number }
    },
  })

  const duplicates = duplicatesData?.duplicates || []

  // Merge mutation
  const mergeMutation = useMutation({
    mutationFn: ({ sourceId, targetId }: { sourceId: string; targetId: string }) =>
      apiClient.mergeTerms(sourceId, targetId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['terms'] })
      queryClient.invalidateQueries({ queryKey: ['duplicates'] })
      setSelectedPair(null)
    },
  })

  const handleMerge = () => {
    if (!selectedPair) return

    const sourceId = keepTerm === 'term1' ? selectedPair.term2.id : selectedPair.term1.id
    const targetId = keepTerm === 'term1' ? selectedPair.term1.id : selectedPair.term2.id

    const sourceTerm = keepTerm === 'term1' ? selectedPair.term2 : selectedPair.term1
    const targetTerm = keepTerm === 'term1' ? selectedPair.term1 : selectedPair.term2

    if (confirm(
      `Merge "${sourceTerm.lemma_en}" (${sourceTerm.lemma_ja}) into "${targetTerm.lemma_en}" (${targetTerm.lemma_ja})?\n\n` +
      `The merged term will be deleted, and all its occurrences will be transferred to the kept term.`
    )) {
      mergeMutation.mutate({ sourceId, targetId })
    }
  }

  if (isLoading) {
    return (
      <div style={{
        position: 'fixed',
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        backgroundColor: 'rgba(0, 0, 0, 0.8)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        zIndex: 1000,
      }}>
        <div style={{
          backgroundColor: '#2a2a2a',
          borderRadius: '8px',
          padding: '2rem',
          maxWidth: '800px',
          width: '90%',
          border: '1px solid #444',
        }}>
          <p style={{ color: '#e0e0e0', margin: 0 }}>Detecting duplicates...</p>
        </div>
      </div>
    )
  }

  return (
    <div
      style={{
        position: 'fixed',
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        backgroundColor: 'rgba(0, 0, 0, 0.8)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        zIndex: 1000,
      }}
      onClick={onClose}
    >
      <div
        style={{
          backgroundColor: '#2a2a2a',
          borderRadius: '8px',
          padding: '2rem',
          maxWidth: '800px',
          width: '90%',
          maxHeight: '80vh',
          overflow: 'auto',
          border: '1px solid #444',
        }}
        onClick={(e) => e.stopPropagation()}
      >
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1.5rem' }}>
          <h2 style={{ margin: 0, color: '#e0e0e0' }}>Merge Duplicate Terms</h2>
          <button
            onClick={onClose}
            style={{
              background: 'none',
              border: 'none',
              fontSize: '1.5rem',
              cursor: 'pointer',
              color: '#999',
            }}
          >
            ×
          </button>
        </div>

        {!duplicates || duplicates.length === 0 ? (
          <div style={{
            textAlign: 'center',
            padding: '2rem',
            color: '#999',
            border: '1px dashed #444',
            borderRadius: '8px',
            backgroundColor: '#1a1a1a',
          }}>
            No duplicate terms detected. All terms are unique!
          </div>
        ) : (
          <>
            <div style={{ marginBottom: '1.5rem' }}>
              <p style={{ color: '#999', fontSize: '0.875rem', margin: 0 }}>
                Found {duplicates.length} potential duplicate pair{duplicates.length !== 1 ? 's' : ''}.
                Select a pair to merge.
              </p>
            </div>

            {/* Duplicate pairs list */}
            <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem', marginBottom: '1.5rem' }}>
              {duplicates.map((pair, index) => (
                <div
                  key={index}
                  style={{
                    border: selectedPair === pair ? '2px solid #646cff' : '1px solid #444',
                    borderRadius: '8px',
                    padding: '1rem',
                    cursor: 'pointer',
                    backgroundColor: selectedPair === pair ? '#1e2a4a' : '#1a1a1a',
                    transition: 'all 0.2s ease',
                  }}
                  onClick={() => setSelectedPair(pair)}
                >
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'start', marginBottom: '0.5rem' }}>
                    <div style={{ flex: 1 }}>
                      <div style={{ fontWeight: '600', marginBottom: '0.25rem', color: '#e0e0e0' }}>
                        {pair.term1.lemma_ja} vs {pair.term2.lemma_ja}
                      </div>
                      <div style={{ fontSize: '0.875rem', color: '#999' }}>
                        {pair.term1.lemma_en} vs {pair.term2.lemma_en}
                      </div>
                    </div>
                    <div style={{
                      padding: '0.25rem 0.5rem',
                      backgroundColor: '#1e3a5f',
                      borderRadius: '4px',
                      fontSize: '0.75rem',
                      color: '#64b5f6',
                    }}>
                      {Math.round(pair.similarity_score * 100)}% similar
                    </div>
                  </div>
                  <div style={{ fontSize: '0.75rem', color: '#666' }}>
                    {pair.reason}
                  </div>
                </div>
              ))}
            </div>

            {/* Merge controls */}
            {selectedPair && (
              <div style={{
                border: '1px solid #444',
                borderRadius: '8px',
                padding: '1.5rem',
                backgroundColor: '#1a1a1a',
              }}>
                <h3 style={{ marginTop: 0, marginBottom: '1rem', color: '#e0e0e0' }}>Select term to keep:</h3>

                <div style={{ display: 'flex', gap: '1rem', marginBottom: '1.5rem' }}>
                  {/* Option 1 */}
                  <label style={{
                    flex: 1,
                    border: keepTerm === 'term1' ? '2px solid #646cff' : '1px solid #444',
                    borderRadius: '8px',
                    padding: '1rem',
                    cursor: 'pointer',
                    backgroundColor: keepTerm === 'term1' ? '#1e2a4a' : '#2a2a2a',
                  }}>
                    <input
                      type="radio"
                      name="keepTerm"
                      value="term1"
                      checked={keepTerm === 'term1'}
                      onChange={() => setKeepTerm('term1')}
                      style={{ marginRight: '0.5rem' }}
                    />
                    <div>
                      <div style={{ fontWeight: '600', marginBottom: '0.25rem', color: '#e0e0e0' }}>
                        {selectedPair.term1.lemma_ja}
                        {selectedPair.term1.reading_kana && ` (${selectedPair.term1.reading_kana})`}
                      </div>
                      <div style={{ fontSize: '0.875rem', color: '#999' }}>
                        {selectedPair.term1.lemma_en}
                      </div>
                    </div>
                  </label>

                  {/* Option 2 */}
                  <label style={{
                    flex: 1,
                    border: keepTerm === 'term2' ? '2px solid #646cff' : '1px solid #444',
                    borderRadius: '8px',
                    padding: '1rem',
                    cursor: 'pointer',
                    backgroundColor: keepTerm === 'term2' ? '#1e2a4a' : '#2a2a2a',
                  }}>
                    <input
                      type="radio"
                      name="keepTerm"
                      value="term2"
                      checked={keepTerm === 'term2'}
                      onChange={() => setKeepTerm('term2')}
                      style={{ marginRight: '0.5rem' }}
                    />
                    <div>
                      <div style={{ fontWeight: '600', marginBottom: '0.25rem', color: '#e0e0e0' }}>
                        {selectedPair.term2.lemma_ja}
                        {selectedPair.term2.reading_kana && ` (${selectedPair.term2.reading_kana})`}
                      </div>
                      <div style={{ fontSize: '0.875rem', color: '#999' }}>
                        {selectedPair.term2.lemma_en}
                      </div>
                    </div>
                  </label>
                </div>

                <div style={{
                  padding: '0.75rem',
                  backgroundColor: '#3d2a1a',
                  border: '1px solid #f59e0b',
                  borderRadius: '6px',
                  marginBottom: '1rem',
                  fontSize: '0.875rem',
                  color: '#ffd666',
                }}>
                  ⚠️ The other term will be deleted, and all its variants and occurrences will be transferred to the kept term.
                </div>

                <button
                  onClick={handleMerge}
                  disabled={mergeMutation.isPending}
                  style={{
                    width: '100%',
                    padding: '0.75rem',
                    fontSize: '0.875rem',
                    backgroundColor: '#646cff',
                    color: 'white',
                    border: 'none',
                    borderRadius: '6px',
                    cursor: mergeMutation.isPending ? 'not-allowed' : 'pointer',
                    fontWeight: '500',
                  }}
                >
                  {mergeMutation.isPending ? 'Merging...' : 'Merge Terms'}
                </button>
              </div>
            )}
          </>
        )}
      </div>
    </div>
  )
}
