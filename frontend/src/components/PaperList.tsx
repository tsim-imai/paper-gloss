import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { Link } from 'react-router-dom'
import { apiClient } from '../services/api'
import { Paper, PaperStatus } from '../types'

export default function PaperList() {
  const [page, setPage] = useState(1)
  const [statusFilter, setStatusFilter] = useState<PaperStatus | 'all'>('all')
  const limit = 10

  const { data, isLoading, error } = useQuery({
    queryKey: ['papers', page, statusFilter],
    queryFn: async () => {
      const response = await apiClient.listPapers(
        page,
        limit,
        statusFilter === 'all' ? undefined : statusFilter
      )
      return response.data
    },
  })

  if (isLoading) {
    return <div style={{ textAlign: 'center', padding: '2rem' }}>Loading papers...</div>
  }

  if (error) {
    return (
      <div style={{ color: 'red', padding: '1rem' }}>
        Error loading papers: {(error as any).message}
      </div>
    )
  }

  const papers: Paper[] = data?.papers || []
  const totalPages = data?.total_pages || 1

  const getStatusColor = (status: PaperStatus) => {
    switch (status) {
      case 'completed':
        return '#4caf50'
      case 'processing':
        return '#ff9800'
      case 'failed':
        return '#f44336'
      default:
        return '#9e9e9e'
    }
  }

  const getStatusLabel = (status: PaperStatus) => {
    return status.charAt(0).toUpperCase() + status.slice(1)
  }

  return (
    <div>
      {/* Status filter */}
      <div style={{ marginBottom: '1rem', display: 'flex', gap: '0.5rem', flexWrap: 'wrap' }}>
        <span style={{ alignSelf: 'center', marginRight: '0.5rem', fontWeight: 'bold', color: '#e0e0e0' }}>
          Filter:
        </span>
        {(['all', 'pending', 'processing', 'completed', 'failed'] as const).map((status) => (
          <button
            key={status}
            onClick={() => {
              setStatusFilter(status)
              setPage(1)
            }}
            style={{
              padding: '0.5rem 1rem',
              backgroundColor: statusFilter === status ? '#646cff' : '#444',
              color: statusFilter === status ? 'white' : '#999',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
              fontSize: '0.875rem',
            }}
          >
            {status === 'all' ? 'All' : getStatusLabel(status as PaperStatus)}
          </button>
        ))}
      </div>

      {/* Papers list */}
      {papers.length === 0 ? (
        <div style={{
          padding: '2rem',
          textAlign: 'center',
          border: '1px dashed #444',
          borderRadius: '8px',
          color: '#999',
          backgroundColor: '#2a2a2a'
        }}>
          No papers found. Import a paper to get started!
        </div>
      ) : (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
          {papers.map((paper) => (
            <Link
              key={paper.id}
              to={`/papers/${paper.id}`}
              style={{
                textDecoration: 'none',
                color: 'inherit',
                border: '1px solid #444',
                borderRadius: '8px',
                padding: '1rem',
                backgroundColor: '#2a2a2a',
                transition: 'box-shadow 0.2s',
                display: 'block',
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.boxShadow = '0 4px 12px rgba(100, 108, 255, 0.3)'
                e.currentTarget.style.borderColor = '#646cff'
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.boxShadow = 'none'
                e.currentTarget.style.borderColor = '#444'
              }}
            >
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'start' }}>
                <div style={{ flex: 1 }}>
                  <h3 style={{ margin: '0 0 0.5rem 0', fontSize: '1.125rem', color: '#e0e0e0' }}>
                    {paper.title}
                  </h3>
                  {paper.arxiv_id && (
                    <div style={{ fontSize: '0.875rem', color: '#999', marginBottom: '0.5rem' }}>
                      arXiv: {paper.arxiv_id}
                    </div>
                  )}
                  <div style={{ fontSize: '0.875rem', color: '#999' }}>
                    Source: {paper.source === 'upload' ? 'Upload' : 'URL'} •
                    Created: {new Date(paper.created_at).toLocaleDateString()}
                  </div>
                  {paper.total_chunks !== undefined && (
                    <div style={{ fontSize: '0.875rem', color: '#999', marginTop: '0.5rem' }}>
                      Progress: {paper.translated_chunks || 0} / {paper.total_chunks} chunks translated
                      {paper.failed_chunks ? ` (${paper.failed_chunks} failed)` : ''}
                    </div>
                  )}
                </div>
                <div
                  style={{
                    padding: '0.25rem 0.75rem',
                    borderRadius: '12px',
                    backgroundColor: getStatusColor(paper.status),
                    color: 'white',
                    fontSize: '0.75rem',
                    fontWeight: 'bold',
                    textTransform: 'uppercase',
                    marginLeft: '1rem',
                  }}
                >
                  {getStatusLabel(paper.status)}
                </div>
              </div>
            </Link>
          ))}
        </div>
      )}

      {/* Pagination */}
      {totalPages > 1 && (
        <div style={{
          marginTop: '2rem',
          display: 'flex',
          justifyContent: 'center',
          alignItems: 'center',
          gap: '1rem'
        }}>
          <button
            onClick={() => setPage((p) => Math.max(1, p - 1))}
            disabled={page === 1}
            style={{
              padding: '0.5rem 1rem',
              backgroundColor: page === 1 ? '#444' : '#646cff',
              color: page === 1 ? '#666' : 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: page === 1 ? 'not-allowed' : 'pointer',
            }}
          >
            Previous
          </button>
          <span style={{ color: '#e0e0e0' }}>
            Page {page} of {totalPages}
          </span>
          <button
            onClick={() => setPage((p) => Math.min(totalPages, p + 1))}
            disabled={page === totalPages}
            style={{
              padding: '0.5rem 1rem',
              backgroundColor: page === totalPages ? '#444' : '#646cff',
              color: page === totalPages ? '#666' : 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: page === totalPages ? 'not-allowed' : 'pointer',
            }}
          >
            Next
          </button>
        </div>
      )}
    </div>
  )
}
