import { useState } from 'react'
import { useParams, Link, useNavigate } from 'react-router-dom'
import { useQuery, useMutation } from '@tanstack/react-query'
import { apiClient, queryClient } from '../services/api'
import { Paper } from '../types'
import TranslationView from '../components/TranslationView'
import PipelineStatus from '../components/PipelineStatus'

export default function PaperPage() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false)

  const { data: paper, isLoading, error } = useQuery({
    queryKey: ['paper', id],
    queryFn: async () => {
      if (!id) throw new Error('Paper ID is required')
      const response = await apiClient.getPaper(id)
      return response.data as Paper
    },
    enabled: !!id,
    refetchInterval: (query) => {
      const paper = query.state.data
      // Refresh every 5 seconds if processing
      return paper?.status === 'processing' ? 5000 : false
    },
  })

  // Delete paper mutation (FR-037, FR-038)
  const deletePaperMutation = useMutation({
    mutationFn: async (paperId: string) => {
      return await apiClient.deletePaper(paperId)
    },
    onSuccess: () => {
      // Invalidate paper list cache
      queryClient.invalidateQueries({ queryKey: ['papers'] })
      // Navigate back to paper list
      navigate('/')
    },
    onError: (error: any) => {
      alert(`Failed to delete paper: ${error.response?.data?.message || error.message}`)
    },
  })

  if (!id) {
    return <div style={{ color: 'red' }}>Error: Paper ID is required</div>
  }

  if (isLoading) {
    return <div style={{ textAlign: 'center', padding: '2rem' }}>Loading paper...</div>
  }

  if (error) {
    return (
      <div style={{ color: 'red', padding: '1rem' }}>
        Error loading paper: {(error as any).message}
      </div>
    )
  }

  if (!paper) {
    return <div style={{ color: 'red', padding: '1rem' }}>Paper not found</div>
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: 'calc(100vh - 100px)', gap: '0.75rem' }}>
      {/* Header */}
      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          padding: '0.75rem',
          backgroundColor: '#2a2a2a',
          borderRadius: '8px',
          border: '1px solid #444',
        }}
      >
        <div style={{ flex: 1 }}>
          <Link
            to="/"
            style={{
              fontSize: '0.75rem',
              color: '#999',
              textDecoration: 'none',
              marginBottom: '0.25rem',
              display: 'inline-block',
            }}
          >
            ← Back to Papers
          </Link>
          <h2 style={{ margin: '0', fontSize: '1.25rem', color: '#e0e0e0' }}>{paper.title}</h2>
          {paper.arxiv_id && (
            <div style={{ fontSize: '0.75rem', color: '#999', marginTop: '0.25rem' }}>
              arXiv:{' '}
              <a
                href={`https://arxiv.org/abs/${paper.arxiv_id}`}
                target="_blank"
                rel="noopener noreferrer"
                style={{ color: '#646cff' }}
              >
                {paper.arxiv_id}
              </a>
            </div>
          )}
        </div>
        {/* Delete button (FR-037) */}
        <button
          onClick={() => setShowDeleteConfirm(true)}
          style={{
            padding: '0.5rem 1rem',
            backgroundColor: '#f44336',
            color: 'white',
            border: 'none',
            borderRadius: '4px',
            cursor: 'pointer',
            fontSize: '0.75rem',
          }}
          onMouseEnter={(e) => {
            e.currentTarget.style.backgroundColor = '#d32f2f'
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.backgroundColor = '#f44336'
          }}
        >
          Delete
        </button>
      </div>

      {/* Delete confirmation dialog (FR-038) */}
      {showDeleteConfirm && (
        <div
          style={{
            position: 'fixed',
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            backgroundColor: 'rgba(0, 0, 0, 0.8)',
            display: 'flex',
            justifyContent: 'center',
            alignItems: 'center',
            zIndex: 1000,
          }}
        >
          <div
            style={{
              backgroundColor: '#2a2a2a',
              padding: '2rem',
              borderRadius: '8px',
              maxWidth: '500px',
              width: '90%',
              border: '1px solid #444',
            }}
          >
            <h3 style={{ marginTop: 0, color: '#e0e0e0' }}>Confirm Deletion</h3>
            <p style={{ color: '#ccc' }}>
              Are you sure you want to delete this paper? This will permanently remove:
            </p>
            <ul style={{ color: '#ccc' }}>
              <li>The paper and all its translations</li>
              <li>All extracted terms and occurrences</li>
              <li>The stored PDF file</li>
            </ul>
            <p style={{ color: '#f44336', fontWeight: 'bold' }}>This action cannot be undone.</p>
            <div style={{ display: 'flex', gap: '1rem', justifyContent: 'flex-end', marginTop: '1.5rem' }}>
              <button
                onClick={() => setShowDeleteConfirm(false)}
                style={{
                  padding: '0.5rem 1rem',
                  backgroundColor: '#444',
                  color: '#e0e0e0',
                  border: 'none',
                  borderRadius: '4px',
                  cursor: 'pointer',
                }}
              >
                Cancel
              </button>
              <button
                onClick={() => {
                  setShowDeleteConfirm(false)
                  deletePaperMutation.mutate(paper.id)
                }}
                disabled={deletePaperMutation.isPending}
                style={{
                  padding: '0.5rem 1rem',
                  backgroundColor: '#f44336',
                  color: 'white',
                  border: 'none',
                  borderRadius: '4px',
                  cursor: deletePaperMutation.isPending ? 'not-allowed' : 'pointer',
                  opacity: deletePaperMutation.isPending ? 0.5 : 1,
                }}
              >
                {deletePaperMutation.isPending ? 'Deleting...' : 'Delete'}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Main content: Pipeline Status (left) and Translation View (right) */}
      <div style={{ flex: 1, display: 'flex', gap: '0.75rem', overflow: 'hidden' }}>
        {/* Pipeline Status - Left sidebar */}
        <div style={{
          width: '350px',
          flexShrink: 0,
          overflowY: 'auto',
          border: '1px solid #444',
          borderRadius: '8px',
          backgroundColor: '#2a2a2a',
          padding: '0.75rem'
        }}>
          <PipelineStatus paper={paper} />
        </div>

        {/* Translation View - Right main area */}
        <div
          style={{
            flex: 1,
            overflow: 'hidden',
            border: '1px solid #444',
            borderRadius: '8px',
            backgroundColor: '#1a1a1a',
          }}
        >
          <div style={{ width: '100%', height: '100%', overflow: 'auto', padding: '1rem', backgroundColor: '#1a1a1a' }}>
            <TranslationView paperId={id} />
          </div>
        </div>
      </div>
    </div>
  )
}
