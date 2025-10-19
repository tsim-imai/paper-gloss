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
  const { data: duplicates, isLoading } = useQuery({
    queryKey: ['duplicates'],
    queryFn: async () => {
      // Note: This endpoint doesn't exist in the backend yet
      // For now, we'll use a placeholder
      // TODO: Add GET /terms/duplicates endpoint
      return [] as DuplicatePair[]
    },
  })

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
        backgroundColor: 'rgba(0, 0, 0, 0.5)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        zIndex: 1000,
      }}>
        <div style={{
          backgroundColor: 'white',
          borderRadius: '8px',
          padding: '2rem',
          maxWidth: '800px',
          width: '90%',
        }}>
          <p>Detecting duplicates...</p>
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
        backgroundColor: 'rgba(0, 0, 0, 0.5)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        zIndex: 1000,
      }}
      onClick={onClose}
    >
      <div
        style={{
          backgroundColor: 'white',
          borderRadius: '8px',
          padding: '2rem',
          maxWidth: '800px',
          width: '90%',
          maxHeight: '80vh',
          overflow: 'auto',
        }}
        onClick={(e) => e.stopPropagation()}
      >
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1.5rem' }}>
          <h2 style={{ margin: 0 }}>Merge Duplicate Terms</h2>
          <button
            onClick={onClose}
            style={{
              background: 'none',
              border: 'none',
              fontSize: '1.5rem',
              cursor: 'pointer',
              color: '#666',
            }}
          >
            ×
          </button>
        </div>

        {!duplicates || duplicates.length === 0 ? (
          <div style={{
            textAlign: 'center',
            padding: '2rem',
            color: '#666',
            border: '1px dashed #ccc',
            borderRadius: '8px',
          }}>
            No duplicate terms detected. All terms are unique!
          </div>
        ) : (
          <>
            <div style={{ marginBottom: '1.5rem' }}>
              <p style={{ color: '#666', fontSize: '0.875rem' }}>
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
                    border: selectedPair === pair ? '2px solid #646cff' : '1px solid #ccc',
                    borderRadius: '8px',
                    padding: '1rem',
                    cursor: 'pointer',
                    backgroundColor: selectedPair === pair ? '#f0f0ff' : 'white',
                  }}
                  onClick={() => setSelectedPair(pair)}
                >
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'start', marginBottom: '0.5rem' }}>
                    <div style={{ flex: 1 }}>
                      <div style={{ fontWeight: 'bold', marginBottom: '0.25rem' }}>
                        {pair.term1.lemma_ja} vs {pair.term2.lemma_ja}
                      </div>
                      <div style={{ fontSize: '0.875rem', color: '#666' }}>
                        {pair.term1.lemma_en} vs {pair.term2.lemma_en}
                      </div>
                    </div>
                    <div style={{
                      padding: '0.25rem 0.5rem',
                      backgroundColor: '#e3f2fd',
                      borderRadius: '4px',
                      fontSize: '0.75rem',
                      color: '#1976d2',
                    }}>
                      {Math.round(pair.similarity_score * 100)}% similar
                    </div>
                  </div>
                  <div style={{ fontSize: '0.75rem', color: '#999' }}>
                    {pair.reason}
                  </div>
                </div>
              ))}
            </div>

            {/* Merge controls */}
            {selectedPair && (
              <div style={{
                border: '1px solid #ccc',
                borderRadius: '8px',
                padding: '1.5rem',
                backgroundColor: '#f9f9f9',
              }}>
                <h3 style={{ marginTop: 0, marginBottom: '1rem' }}>Select term to keep:</h3>

                <div style={{ display: 'flex', gap: '1rem', marginBottom: '1.5rem' }}>
                  {/* Option 1 */}
                  <label style={{
                    flex: 1,
                    border: keepTerm === 'term1' ? '2px solid #646cff' : '1px solid #ccc',
                    borderRadius: '8px',
                    padding: '1rem',
                    cursor: 'pointer',
                    backgroundColor: keepTerm === 'term1' ? '#f0f0ff' : 'white',
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
                      <div style={{ fontWeight: 'bold', marginBottom: '0.25rem' }}>
                        {selectedPair.term1.lemma_ja}
                        {selectedPair.term1.reading_kana && ` (${selectedPair.term1.reading_kana})`}
                      </div>
                      <div style={{ fontSize: '0.875rem', color: '#666' }}>
                        {selectedPair.term1.lemma_en}
                      </div>
                    </div>
                  </label>

                  {/* Option 2 */}
                  <label style={{
                    flex: 1,
                    border: keepTerm === 'term2' ? '2px solid #646cff' : '1px solid #ccc',
                    borderRadius: '8px',
                    padding: '1rem',
                    cursor: 'pointer',
                    backgroundColor: keepTerm === 'term2' ? '#f0f0ff' : 'white',
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
                      <div style={{ fontWeight: 'bold', marginBottom: '0.25rem' }}>
                        {selectedPair.term2.lemma_ja}
                        {selectedPair.term2.reading_kana && ` (${selectedPair.term2.reading_kana})`}
                      </div>
                      <div style={{ fontSize: '0.875rem', color: '#666' }}>
                        {selectedPair.term2.lemma_en}
                      </div>
                    </div>
                  </label>
                </div>

                <div style={{
                  padding: '0.75rem',
                  backgroundColor: '#fff3cd',
                  border: '1px solid #ffc107',
                  borderRadius: '4px',
                  marginBottom: '1rem',
                  fontSize: '0.875rem',
                }}>
                  ⚠️ The other term will be deleted, and all its variants and occurrences will be transferred to the kept term.
                </div>

                <button
                  onClick={handleMerge}
                  disabled={mergeMutation.isPending}
                  style={{
                    width: '100%',
                    padding: '0.75rem',
                    fontSize: '1rem',
                    backgroundColor: '#646cff',
                    color: 'white',
                    border: 'none',
                    borderRadius: '4px',
                    cursor: mergeMutation.isPending ? 'not-allowed' : 'pointer',
                    fontWeight: 'bold',
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
