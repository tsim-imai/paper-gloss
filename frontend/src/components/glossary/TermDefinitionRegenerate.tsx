import { useState } from 'react'
import { useMutation } from '@tanstack/react-query'
import { apiClient } from '../../services/api'
import { Definition } from '../../types'

interface TermDefinitionRegenerateProps {
  term: {
    id: string
    lemma_en: string
    lemma_ja: string
    definition?: Definition
  }
  onDefinitionUpdated: (definition: Definition) => void
}

/**
 * Component for regenerating term definitions using LLM
 * Includes confirmation dialog to prevent accidental overwrites
 */
export default function TermDefinitionRegenerate({
  term,
  onDefinitionUpdated,
}: TermDefinitionRegenerateProps) {
  const [showConfirmDialog, setShowConfirmDialog] = useState(false)
  const [currentDefinition, setCurrentDefinition] = useState(term.definition)

  const regenerateMutation = useMutation({
    mutationFn: () => apiClient.generateDefinition(term.id),
    onSuccess: (response) => {
      const newDefinition = response.data as Definition
      setCurrentDefinition(newDefinition)
      onDefinitionUpdated(newDefinition)
      setShowConfirmDialog(false)
    },
    onError: (error) => {
      console.error('Failed to regenerate definition:', error)
    },
  })

  const handleRegenerateClick = () => {
    setShowConfirmDialog(true)
  }

  const handleConfirm = () => {
    regenerateMutation.mutate()
  }

  const handleCancel = () => {
    setShowConfirmDialog(false)
  }

  return (
    <div style={{ marginTop: '1rem' }}>
      {/* Definition Display */}
      <div style={{
        padding: '1rem',
        backgroundColor: '#f8f9fa',
        borderRadius: '8px',
        marginBottom: '1rem',
      }}>
        <h4 style={{ margin: '0 0 0.5rem 0', fontSize: '0.875rem', color: '#666' }}>
          Definition
        </h4>
        {currentDefinition ? (
          <div>
            <p style={{ margin: '0 0 0.5rem 0', lineHeight: 1.5 }}>
              {currentDefinition.text}
            </p>
            <p style={{ margin: 0, fontSize: '0.75rem', color: '#999' }}>
              Provider: {currentDefinition.provider}
            </p>
          </div>
        ) : (
          <p style={{ margin: 0, color: '#999', fontStyle: 'italic' }}>
            No definition available
          </p>
        )}
      </div>

      {/* Regenerate Button */}
      <button
        onClick={handleRegenerateClick}
        disabled={regenerateMutation.isPending}
        style={{
          padding: '0.5rem 1rem',
          fontSize: '0.875rem',
          backgroundColor: regenerateMutation.isPending ? '#ccc' : '#17a2b8',
          color: 'white',
          border: 'none',
          borderRadius: '4px',
          cursor: regenerateMutation.isPending ? 'not-allowed' : 'pointer',
          display: 'flex',
          alignItems: 'center',
          gap: '0.5rem',
        }}
      >
        {regenerateMutation.isPending ? (
          <>
            <span style={{
              display: 'inline-block',
              width: '12px',
              height: '12px',
              border: '2px solid #fff',
              borderTopColor: 'transparent',
              borderRadius: '50%',
              animation: 'spin 0.6s linear infinite',
            }} />
            Regenerating...
          </>
        ) : (
          <>
            🔄 {currentDefinition ? 'Regenerate Definition' : 'Generate Definition'}
          </>
        )}
      </button>

      {/* Error Message */}
      {regenerateMutation.isError && (
        <div style={{
          marginTop: '0.5rem',
          padding: '0.5rem',
          backgroundColor: '#ffebee',
          color: '#c62828',
          borderRadius: '4px',
          fontSize: '0.875rem',
        }}>
          Failed to regenerate definition. Please try again.
        </div>
      )}

      {/* Confirmation Dialog */}
      {showConfirmDialog && (
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
            maxWidth: '400px',
            boxShadow: '0 4px 6px rgba(0, 0, 0, 0.1)',
          }}>
            <h3 style={{ marginTop: 0, marginBottom: '1rem' }}>
              Confirm Regeneration
            </h3>
            <p style={{ marginBottom: '1.5rem', lineHeight: 1.5 }}>
              Are you sure you want to regenerate the definition for "{term.lemma_en}"?
              {currentDefinition && ' This will overwrite the current definition.'}
            </p>
            <div style={{ display: 'flex', gap: '0.5rem', justifyContent: 'flex-end' }}>
              <button
                onClick={handleCancel}
                style={{
                  padding: '0.5rem 1rem',
                  fontSize: '1rem',
                  backgroundColor: '#f5f5f5',
                  border: '1px solid #ccc',
                  borderRadius: '4px',
                  cursor: 'pointer',
                }}
              >
                Cancel
              </button>
              <button
                onClick={handleConfirm}
                style={{
                  padding: '0.5rem 1rem',
                  fontSize: '1rem',
                  backgroundColor: '#dc3545',
                  color: 'white',
                  border: 'none',
                  borderRadius: '4px',
                  cursor: 'pointer',
                }}
              >
                Confirm
              </button>
            </div>
          </div>
        </div>
      )}

      {/* CSS for spinner animation */}
      <style>{`
        @keyframes spin {
          to { transform: rotate(360deg); }
        }
      `}</style>
    </div>
  )
}