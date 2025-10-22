import { useState } from 'react'
import { useParams, Link, useNavigate } from 'react-router-dom'
import { useQuery, useMutation } from '@tanstack/react-query'
import { apiClient, queryClient } from '../services/api'
import { Paper } from '../types'
import PDFViewer from '../components/PDFViewer'
import TranslationView from '../components/TranslationView'
import ProcessingStatus from '../components/ProcessingStatus'

type ViewMode = 'both' | 'pdf' | 'translation'

export default function PaperPage() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const [viewMode, setViewMode] = useState<ViewMode>('both')
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

  // Construct file URL for PDF viewer
  const pdfUrl = `/api/papers/${paper.id}/file`

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: 'calc(100vh - 200px)' }}>
      {/* Header */}
      <div style={{ marginBottom: '1rem' }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'start' }}>
          <div style={{ flex: 1 }}>
            <Link to="/" style={{ fontSize: '0.875rem', marginBottom: '0.5rem', display: 'inline-block' }}>
              ← Back to Papers
            </Link>
            <h2 style={{ margin: '0 0 0.5rem 0' }}>{paper.title}</h2>
            {paper.arxiv_id && (
              <div style={{ fontSize: '0.875rem', color: '#666', marginBottom: '0.5rem' }}>
                arXiv: <a href={`https://arxiv.org/abs/${paper.arxiv_id}`} target="_blank" rel="noopener noreferrer">
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
              fontSize: '0.875rem',
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.backgroundColor = '#d32f2f'
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.backgroundColor = '#f44336'
            }}
          >
            Delete Paper
          </button>
        </div>
      </div>

      {/* Delete confirmation dialog (FR-038) */}
      {showDeleteConfirm && (
        <div style={{
          position: 'fixed',
          top: 0,
          left: 0,
          right: 0,
          bottom: 0,
          backgroundColor: 'rgba(0, 0, 0, 0.5)',
          display: 'flex',
          justifyContent: 'center',
          alignItems: 'center',
          zIndex: 1000,
        }}>
          <div style={{
            backgroundColor: 'white',
            padding: '2rem',
            borderRadius: '8px',
            maxWidth: '500px',
            width: '90%',
          }}>
            <h3 style={{ marginTop: 0 }}>Confirm Deletion</h3>
            <p>
              Are you sure you want to delete this paper? This will permanently remove:
            </p>
            <ul>
              <li>The paper and all its translations</li>
              <li>All extracted terms and occurrences</li>
              <li>The stored PDF file</li>
            </ul>
            <p style={{ color: '#f44336', fontWeight: 'bold' }}>
              This action cannot be undone.
            </p>
            <div style={{ display: 'flex', gap: '1rem', justifyContent: 'flex-end', marginTop: '1.5rem' }}>
              <button
                onClick={() => setShowDeleteConfirm(false)}
                style={{
                  padding: '0.5rem 1rem',
                  backgroundColor: '#e0e0e0',
                  color: 'black',
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

      {/* Processing Status */}
      <div style={{ marginBottom: '1rem' }}>
        <ProcessingStatus paper={paper} />
      </div>

      {/* View mode selector */}
      <div style={{
        marginBottom: '1rem',
        padding: '0.75rem',
        backgroundColor: '#f9f9f9',
        borderRadius: '8px',
        display: 'flex',
        gap: '0.5rem'
      }}>
        <span style={{ alignSelf: 'center', marginRight: '0.5rem', fontWeight: 'bold' }}>
          View:
        </span>
        <button
          onClick={() => setViewMode('both')}
          style={{
            padding: '0.5rem 1rem',
            backgroundColor: viewMode === 'both' ? '#646cff' : '#e0e0e0',
            color: viewMode === 'both' ? 'white' : 'black',
            border: 'none',
            borderRadius: '4px',
            cursor: 'pointer',
            fontSize: '0.875rem',
          }}
        >
          Side by Side
        </button>
        <button
          onClick={() => setViewMode('pdf')}
          style={{
            padding: '0.5rem 1rem',
            backgroundColor: viewMode === 'pdf' ? '#646cff' : '#e0e0e0',
            color: viewMode === 'pdf' ? 'white' : 'black',
            border: 'none',
            borderRadius: '4px',
            cursor: 'pointer',
            fontSize: '0.875rem',
          }}
        >
          PDF Only
        </button>
        <button
          onClick={() => setViewMode('translation')}
          style={{
            padding: '0.5rem 1rem',
            backgroundColor: viewMode === 'translation' ? '#646cff' : '#e0e0e0',
            color: viewMode === 'translation' ? 'white' : 'black',
            border: 'none',
            borderRadius: '4px',
            cursor: 'pointer',
            fontSize: '0.875rem',
          }}
        >
          Translation Only
        </button>
      </div>

      {/* Content area */}
      <div style={{
        flex: 1,
        display: 'flex',
        gap: '1rem',
        overflow: 'hidden',
        border: '1px solid #ccc',
        borderRadius: '8px'
      }}>
        {/* PDF Viewer */}
        {(viewMode === 'both' || viewMode === 'pdf') && (
          <div style={{
            flex: viewMode === 'both' ? 1 : '0 0 100%',
            overflow: 'hidden',
            display: 'flex',
            flexDirection: 'column'
          }}>
            <PDFViewer fileUrl={pdfUrl} />
          </div>
        )}

        {/* Translation View */}
        {(viewMode === 'both' || viewMode === 'translation') && (
          <div style={{
            flex: viewMode === 'both' ? 1 : '0 0 100%',
            overflow: 'auto',
            padding: '1rem',
            backgroundColor: 'white'
          }}>
            <h3 style={{ marginTop: 0 }}>Translation</h3>
            <TranslationView paperId={id} />
          </div>
        )}
      </div>
    </div>
  )
}
