import { useState, useEffect } from 'react'
import { useParams, Link, useNavigate } from 'react-router-dom'
import { useQuery, useMutation } from '@tanstack/react-query'
import { apiClient, queryClient } from '../services/api'
import { Paper } from '../types'
import PDFViewer from '../components/PDFViewer'
import TranslationView from '../components/TranslationView'
import PipelineStatus from '../components/PipelineStatus'

type ViewMode = 'pdf' | 'translation'

export default function PaperPage() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const [viewMode, setViewMode] = useState<ViewMode>('translation')
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

  // Check if translation exists (use different query key to avoid cache collision)
  const { data: translationCheck } = useQuery({
    queryKey: ['translation-check', id],
    queryFn: async () => {
      if (!id) return null
      try {
        const response = await apiClient.getTranslation(id)
        return response.data
      } catch {
        return null
      }
    },
    enabled: !!id,
  })

  // Auto-switch to PDF if no translation available
  useEffect(() => {
    if (translationCheck && Array.isArray(translationCheck.chunks) && translationCheck.chunks.length === 0) {
      setViewMode('pdf')
    }
  }, [translationCheck])

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

  // Construct file URL for PDF viewer
  const pdfUrl = `/api/papers/${paper.id}/file`

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

      {/* Pipeline Status */}
      <div style={{ flex: '0 0 auto', overflowY: 'auto', maxHeight: '50vh' }}>
        <PipelineStatus paper={paper} />
      </div>

      {/* View mode selector */}
      <div
        style={{
          padding: '0.5rem',
          backgroundColor: '#2a2a2a',
          borderRadius: '8px',
          border: '1px solid #444',
          display: 'flex',
          gap: '0.5rem',
          alignItems: 'center',
        }}
      >
        <span style={{ fontSize: '0.875rem', fontWeight: 'bold', color: '#999', marginRight: '0.25rem' }}>
          View:
        </span>
        <button
          onClick={() => setViewMode('translation')}
          style={{
            padding: '0.375rem 0.75rem',
            backgroundColor: viewMode === 'translation' ? '#646cff' : '#444',
            color: viewMode === 'translation' ? 'white' : '#999',
            border: 'none',
            borderRadius: '4px',
            cursor: 'pointer',
            fontSize: '0.75rem',
          }}
        >
          Translation
        </button>
        <button
          onClick={() => setViewMode('pdf')}
          style={{
            padding: '0.375rem 0.75rem',
            backgroundColor: viewMode === 'pdf' ? '#646cff' : '#444',
            color: viewMode === 'pdf' ? 'white' : '#999',
            border: 'none',
            borderRadius: '4px',
            cursor: 'pointer',
            fontSize: '0.75rem',
          }}
        >
          PDF
        </button>
      </div>

      {/* Content area */}
      <div
        style={{
          flex: 1,
          overflow: 'hidden',
          border: '1px solid #444',
          borderRadius: '8px',
          backgroundColor: '#1a1a1a',
        }}
      >
        {/* PDF Viewer */}
        {viewMode === 'pdf' && (
          <div style={{ width: '100%', height: '100%', overflow: 'hidden', display: 'flex', flexDirection: 'column' }}>
            <PDFViewer fileUrl={pdfUrl} />
          </div>
        )}

        {/* Translation View */}
        {viewMode === 'translation' && (
          <div style={{ width: '100%', height: '100%', overflow: 'auto', padding: '1rem', backgroundColor: '#1a1a1a' }}>
            <TranslationView paperId={id} />
          </div>
        )}
      </div>
    </div>
  )
}
