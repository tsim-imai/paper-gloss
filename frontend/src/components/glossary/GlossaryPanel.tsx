import { TermListItem } from '../../types'

interface GlossaryPanelProps {
  terms: TermListItem[]
  isLoading: boolean
  viewMode: 'list' | 'block'
  onTermClick: (term: TermListItem) => void
  onDeleteTerm: (termId: string) => void
}

/**
 * Glossary panel component displaying term list with frequency sorting
 */
export default function GlossaryPanel({
  terms,
  isLoading,
  viewMode,
  onTermClick,
  onDeleteTerm,
}: GlossaryPanelProps) {
  if (isLoading) {
    return (
      <div style={{ textAlign: 'center', padding: '2rem', color: '#999' }}>
        Loading glossary...
      </div>
    )
  }

  if (terms.length === 0) {
    return (
      <div style={{
        textAlign: 'center',
        padding: '2rem',
        color: '#999',
        border: '1px dashed #444',
        borderRadius: '8px',
        backgroundColor: '#2a2a2a',
      }}>
        No terms found. Try a different search query or add new terms.
      </div>
    )
  }

  // List view
  if (viewMode === 'list') {
    return (
      <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
        {terms.map((term) => (
          <div
            key={term.id}
            style={{
              border: '1px solid #444',
              borderRadius: '6px',
              padding: '0.75rem 1rem',
              backgroundColor: '#2a2a2a',
              cursor: 'pointer',
              transition: 'all 0.2s ease',
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.backgroundColor = '#333'
              e.currentTarget.style.borderColor = '#646cff'
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.backgroundColor = '#2a2a2a'
              e.currentTarget.style.borderColor = '#444'
            }}
            onClick={() => onTermClick(term)}
          >
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'start' }}>
              <div style={{ flex: 1 }}>
                {/* Term headings */}
                <div style={{ marginBottom: '0.25rem' }}>
                  <span style={{ fontSize: '1rem', fontWeight: '600', color: '#e0e0e0' }}>
                    {term.lemma_ja}
                  </span>
                  {term.reading_kana && (
                    <span style={{ marginLeft: '0.5rem', fontSize: '0.875rem', color: '#999' }}>
                      （{term.reading_kana}）
                    </span>
                  )}
                </div>
                <div style={{ fontSize: '0.875rem', color: '#999', marginBottom: '0.5rem' }}>
                  {term.lemma_en}
                </div>

                {/* Metadata */}
                <div style={{ display: 'flex', gap: '0.75rem', alignItems: 'center', flexWrap: 'wrap' }}>
                  {term.pos && (
                    <span style={{
                      padding: '2px 6px',
                      backgroundColor: '#1e3a5f',
                      borderRadius: '4px',
                      fontSize: '0.75rem',
                      color: '#64b5f6',
                    }}>
                      {term.pos}
                    </span>
                  )}
                  {term.tags && (
                    <span style={{
                      padding: '2px 6px',
                      backgroundColor: '#3d2145',
                      borderRadius: '4px',
                      fontSize: '0.75rem',
                      color: '#ba68c8',
                    }}>
                      {term.tags}
                    </span>
                  )}
                  <span style={{ fontSize: '0.75rem', color: '#666' }}>
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
                  backgroundColor: '#3d1a1a',
                  color: '#ff6b6b',
                  border: '1px solid #8b0000',
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

  // Block view
  return (
    <div style={{
      display: 'grid',
      gridTemplateColumns: 'repeat(auto-fill, minmax(200px, 1fr))',
      gap: '0.75rem',
    }}>
      {terms.map((term) => (
        <div
          key={term.id}
          style={{
            border: '1px solid #444',
            borderRadius: '6px',
            padding: '1rem',
            backgroundColor: '#2a2a2a',
            cursor: 'pointer',
            transition: 'all 0.2s ease',
            display: 'flex',
            flexDirection: 'column',
            justifyContent: 'space-between',
            minHeight: '120px',
          }}
          onMouseEnter={(e) => {
            e.currentTarget.style.backgroundColor = '#333'
            e.currentTarget.style.borderColor = '#646cff'
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.backgroundColor = '#2a2a2a'
            e.currentTarget.style.borderColor = '#444'
          }}
          onClick={() => onTermClick(term)}
        >
          <div>
            <div style={{ fontSize: '1rem', fontWeight: '600', color: '#e0e0e0', marginBottom: '0.25rem' }}>
              {term.lemma_ja}
            </div>
            {term.reading_kana && (
              <div style={{ fontSize: '0.75rem', color: '#999', marginBottom: '0.5rem' }}>
                （{term.reading_kana}）
              </div>
            )}
            <div style={{ fontSize: '0.875rem', color: '#999' }}>
              {term.lemma_en}
            </div>
          </div>
          <div style={{ marginTop: '0.75rem', fontSize: '0.75rem', color: '#666' }}>
            {term.occurrence_count} occurrence{term.occurrence_count !== 1 ? 's' : ''}
          </div>
        </div>
      ))}
    </div>
  )
}
