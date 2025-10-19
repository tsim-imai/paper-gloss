import { TermListItem } from '../../types'

interface GlossaryPanelProps {
  terms: TermListItem[]
  isLoading: boolean
  onTermClick: (term: TermListItem) => void
  onDeleteTerm: (termId: string) => void
}

/**
 * Glossary panel component displaying term list with frequency sorting
 */
export default function GlossaryPanel({
  terms,
  isLoading,
  onTermClick,
  onDeleteTerm,
}: GlossaryPanelProps) {
  if (isLoading) {
    return (
      <div style={{ textAlign: 'center', padding: '2rem', color: '#666' }}>
        Loading glossary...
      </div>
    )
  }

  if (terms.length === 0) {
    return (
      <div style={{
        textAlign: 'center',
        padding: '2rem',
        color: '#666',
        border: '1px dashed #ccc',
        borderRadius: '8px',
      }}>
        No terms found. Try a different search query or add new terms.
      </div>
    )
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
      {terms.map((term) => (
        <div
          key={term.id}
          style={{
            border: '1px solid #ccc',
            borderRadius: '4px',
            padding: '0.75rem 1rem',
            backgroundColor: 'white',
            cursor: 'pointer',
            transition: 'background-color 0.15s ease',
          }}
          onMouseEnter={(e) => {
            e.currentTarget.style.backgroundColor = '#f5f5f5'
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.backgroundColor = 'white'
          }}
          onClick={() => onTermClick(term)}
        >
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'start' }}>
            <div style={{ flex: 1 }}>
              {/* Term headings */}
              <div style={{ marginBottom: '0.25rem' }}>
                <span style={{ fontSize: '1rem', fontWeight: 'bold', color: '#333' }}>
                  {term.lemma_ja}
                </span>
                {term.reading_kana && (
                  <span style={{ marginLeft: '0.5rem', fontSize: '0.875rem', color: '#666' }}>
                    （{term.reading_kana}）
                  </span>
                )}
              </div>
              <div style={{ fontSize: '0.875rem', color: '#666', marginBottom: '0.5rem' }}>
                {term.lemma_en}
              </div>

              {/* Metadata */}
              <div style={{ display: 'flex', gap: '0.75rem', alignItems: 'center', flexWrap: 'wrap' }}>
                {term.pos && (
                  <span style={{
                    padding: '2px 6px',
                    backgroundColor: '#e3f2fd',
                    borderRadius: '4px',
                    fontSize: '0.75rem',
                    color: '#1976d2',
                  }}>
                    {term.pos}
                  </span>
                )}
                {term.tags && (
                  <span style={{
                    padding: '2px 6px',
                    backgroundColor: '#f3e5f5',
                    borderRadius: '4px',
                    fontSize: '0.75rem',
                    color: '#7b1fa2',
                  }}>
                    {term.tags}
                  </span>
                )}
                <span style={{ fontSize: '0.75rem', color: '#999' }}>
                  {term.occurrence_count} occurrence{term.occurrence_count !== 1 ? 's' : ''}
                </span>
              </div>
            </div>

            {/* Delete button */}
            <button
              onClick={(e) => {
                e.stopPropagation()
                if (confirm(`Delete term "${term.lemma_en}" (${term.lemma_ja})?`)) {
                  onDeleteTerm(term.id)
                }
              }}
              style={{
                padding: '0.25rem 0.5rem',
                fontSize: '0.75rem',
                backgroundColor: '#ffebee',
                color: '#c62828',
                border: '1px solid #ef9a9a',
                borderRadius: '4px',
                cursor: 'pointer',
              }}
            >
              Delete
            </button>
          </div>
        </div>
      ))}
    </div>
  )
}
