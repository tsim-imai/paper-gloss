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

  const papers: Paper[] = data?.items || []
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
        <span style={{ alignSelf: 'center', marginRight: '0.5rem', fontWeight: 'bold' }}>
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
              backgroundColor: statusFilter === status ? '#646cff' : '#e0e0e0',
              color: statusFilter === status ? 'white' : 'black',
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
          border: '1px dashed #ccc',
          borderRadius: '8px',
          color: '#666'
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
                border: '1px solid #ccc',
                borderRadius: '8px',
                padding: '1rem',
                backgroundColor: '#f9f9f9',
                transition: 'box-shadow 0.2s',
                display: 'block',
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.boxShadow = '0 2px 8px rgba(0,0,0,0.1)'
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.boxShadow = 'none'
              }}
            >
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'start' }}>
                <div style={{ flex: 1 }}>
                  <h3 style={{ margin: '0 0 0.5rem 0', fontSize: '1.125rem' }}>
                    {paper.title}
                  </h3>
                  {paper.arxiv_id && (
                    <div style={{ fontSize: '0.875rem', color: '#666', marginBottom: '0.5rem' }}>
                      arXiv: {paper.arxiv_id}
                    </div>
                  )}
                  <div style={{ fontSize: '0.875rem', color: '#666' }}>
                    Source: {paper.source === 'upload' ? 'Upload' : 'URL'} •
                    Created: {new Date(paper.created_at).toLocaleDateString()}
                  </div>
                  {paper.total_chunks !== undefined && (
                    <div style={{ fontSize: '0.875rem', color: '#666', marginTop: '0.5rem' }}>
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
              backgroundColor: page === 1 ? '#ccc' : '#646cff',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: page === 1 ? 'not-allowed' : 'pointer',
            }}
          >
            Previous
          </button>
          <span>
            Page {page} of {totalPages}
          </span>
          <button
            onClick={() => setPage((p) => Math.min(totalPages, p + 1))}
            disabled={page === totalPages}
            style={{
              padding: '0.5rem 1rem',
              backgroundColor: page === totalPages ? '#ccc' : '#646cff',
              color: 'white',
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
