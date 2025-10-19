import { useQuery } from '@tanstack/react-query'
import { apiClient } from '../../services/api'
import { TermDetail } from '../../types'

interface TermTooltipProps {
  termId: string
  position: { x: number; y: number }
  isPinned?: boolean
  onClose?: () => void
}

/**
 * Tooltip component that displays term definitions
 * Optimized for <100ms latency with react-query caching
 */
export default function TermTooltip({
  termId,
  position,
  isPinned = false,
  onClose,
}: TermTooltipProps) {
  const { data, isLoading, error } = useQuery({
    queryKey: ['term', termId],
    queryFn: async () => {
      const response = await apiClient.getTerm(termId)
      return response.data as TermDetail
    },
    staleTime: 1000 * 60 * 10, // 10 minutes - terms don't change often
  })

  // Position tooltip to avoid overflow
  const tooltipStyle: React.CSSProperties = {
    position: 'fixed',
    left: `${position.x}px`,
    top: `${position.y + 20}px`, // 20px below cursor
    zIndex: 1000,
    backgroundColor: 'white',
    border: '1px solid #ccc',
    borderRadius: '8px',
    boxShadow: '0 4px 12px rgba(0, 0, 0, 0.15)',
    padding: '1rem',
    maxWidth: '400px',
    minWidth: '300px',
  }

  return (
    <div style={tooltipStyle}>
      {/* Close button for pinned tooltips */}
      {isPinned && (
        <button
          onClick={onClose}
          style={{
            position: 'absolute',
            top: '0.5rem',
            right: '0.5rem',
            background: 'none',
            border: 'none',
            fontSize: '1.25rem',
            cursor: 'pointer',
            color: '#666',
            lineHeight: 1,
            padding: '0.25rem',
          }}
          aria-label="Close"
        >
          ×
        </button>
      )}

      {isLoading && (
        <div style={{ color: '#666', fontSize: '0.875rem' }}>
          Loading...
        </div>
      )}

      {error && (
        <div style={{ color: '#f44336', fontSize: '0.875rem' }}>
          Failed to load term details
        </div>
      )}

      {data && (
        <>
          {/* Term header */}
          <div style={{ marginBottom: '0.75rem' }}>
            <div style={{ fontSize: '1.125rem', fontWeight: 'bold', marginBottom: '0.25rem' }}>
              {data.lemma_ja}
              {data.reading_kana && (
                <span style={{ fontSize: '0.875rem', color: '#666', marginLeft: '0.5rem' }}>
                  （{data.reading_kana}）
                </span>
              )}
            </div>
            <div style={{ fontSize: '0.875rem', color: '#666' }}>
              {data.lemma_en}
              {data.pos && (
                <span style={{
                  marginLeft: '0.5rem',
                  padding: '2px 6px',
                  backgroundColor: '#e3f2fd',
                  borderRadius: '4px',
                  fontSize: '0.75rem',
                }}>
                  {data.pos}
                </span>
              )}
            </div>
          </div>

          {/* Definition */}
          {data.definition && (
            <div style={{ marginBottom: '0.75rem' }}>
              <div style={{
                fontSize: '0.75rem',
                fontWeight: 'bold',
                color: '#666',
                textTransform: 'uppercase',
                marginBottom: '0.25rem',
              }}>
                Definition
              </div>
              <div style={{ fontSize: '0.875rem', lineHeight: '1.6' }}>
                {data.definition.text}
              </div>
            </div>
          )}

          {/* Variants */}
          {data.variants.length > 0 && (
            <div style={{ marginBottom: '0.5rem' }}>
              <div style={{
                fontSize: '0.75rem',
                fontWeight: 'bold',
                color: '#666',
                textTransform: 'uppercase',
                marginBottom: '0.25rem',
              }}>
                Variants
              </div>
              <div style={{ display: 'flex', flexWrap: 'wrap', gap: '0.25rem' }}>
                {data.variants.slice(0, 5).map((variant, idx) => (
                  <span
                    key={idx}
                    style={{
                      padding: '2px 6px',
                      backgroundColor: '#f5f5f5',
                      borderRadius: '4px',
                      fontSize: '0.75rem',
                      color: '#666',
                    }}
                  >
                    {variant.surface}
                  </span>
                ))}
                {data.variants.length > 5 && (
                  <span style={{ fontSize: '0.75rem', color: '#666' }}>
                    +{data.variants.length - 5} more
                  </span>
                )}
              </div>
            </div>
          )}

          {/* Tags */}
          {data.tags && (
            <div style={{ marginTop: '0.5rem' }}>
              <div style={{
                fontSize: '0.75rem',
                color: '#1976d2',
                fontWeight: 'bold',
              }}>
                🏷️ {data.tags}
              </div>
            </div>
          )}

          {/* Pinned indicator */}
          {isPinned && (
            <div style={{
              marginTop: '0.75rem',
              paddingTop: '0.5rem',
              borderTop: '1px solid #e0e0e0',
              fontSize: '0.75rem',
              color: '#666',
              fontStyle: 'italic',
            }}>
              📌 Click elsewhere or press ESC to close
            </div>
          )}
        </>
      )}
    </div>
  )
}
