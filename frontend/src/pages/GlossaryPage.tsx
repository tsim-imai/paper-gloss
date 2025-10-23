import { useState, useEffect } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { apiClient } from '../services/api'
import { TermListResponse, TermListItem } from '../types'
import GlossarySearch from '../components/glossary/GlossarySearch'
import GlossaryPanel from '../components/glossary/GlossaryPanel'
import TermForm, { TermFormData } from '../components/glossary/TermForm'
import TermMerge from '../components/glossary/TermMerge'

export default function GlossaryPage() {
  const [searchQuery, setSearchQuery] = useState('')
  const [searchLang, setSearchLang] = useState('both')
  const [searchSort, setSearchSort] = useState('alphabetical')
  const [page, setPage] = useState(1)
  const [viewMode, setViewMode] = useState<'list' | 'block'>(() => {
    const saved = localStorage.getItem('glossaryViewMode')
    return (saved === 'list' || saved === 'block') ? saved : 'list'
  })
  const [showForm, setShowForm] = useState(false)
  const [showMerge, setShowMerge] = useState(false)
  const [editingTerm, setEditingTerm] = useState<TermListItem | null>(null)

  const queryClient = useQueryClient()

  // Save view mode to localStorage
  useEffect(() => {
    localStorage.setItem('glossaryViewMode', viewMode)
  }, [viewMode])

  // Fetch terms
  const { data, isLoading } = useQuery({
    queryKey: ['terms', searchQuery, searchLang, searchSort, page],
    queryFn: async () => {
      const response = await apiClient.searchTerms(
        searchQuery || undefined,
        searchLang,
        searchSort,
        page,
        50
      )
      return response.data as TermListResponse
    },
  })

  // Create term mutation
  const createMutation = useMutation({
    mutationFn: (data: TermFormData) => apiClient.createTerm(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['terms'] })
      setShowForm(false)
    },
  })

  // Update term mutation
  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: Partial<TermFormData> }) =>
      apiClient.updateTerm(id, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['terms'] })
      setShowForm(false)
      setEditingTerm(null)
    },
  })

  // Delete term mutation
  const deleteMutation = useMutation({
    mutationFn: (id: string) => apiClient.deleteTerm(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['terms'] })
    },
  })

  // Event handlers
  const handleSearch = (query: string, lang: string, sort: string) => {
    setSearchQuery(query)
    setSearchLang(lang)
    setSearchSort(sort)
    setPage(1)
  }

  const handleTermClick = (term: TermListItem) => {
    setEditingTerm(term)
    setShowForm(true)
  }

  const handleFormSubmit = (formData: TermFormData) => {
    if (editingTerm) {
      // Update existing term
      updateMutation.mutate({ id: editingTerm.id, data: formData })
    } else {
      // Create new term
      createMutation.mutate(formData)
    }
  }

  const handleFormCancel = () => {
    setShowForm(false)
    setEditingTerm(null)
  }

  const handleDeleteTerm = (termId: string) => {
    deleteMutation.mutate(termId)
  }

  const handleAddNewTerm = () => {
    setEditingTerm(null)
    setShowForm(true)
  }

  const terms = data?.items || []
  const totalPages = data?.total_pages || 1

  return (
    <div style={{ maxWidth: '1200px', margin: '0 auto' }}>
      {/* Action buttons */}
      <div style={{
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        marginBottom: '1rem',
      }}>
        <div style={{ display: 'flex', gap: '0.5rem', alignItems: 'center' }}>
          <span style={{ fontSize: '0.875rem', color: '#999', marginRight: '0.25rem' }}>View:</span>
          <button
            onClick={() => setViewMode('list')}
            style={{
              padding: '0.5rem 1rem',
              fontSize: '0.875rem',
              backgroundColor: viewMode === 'list' ? '#646cff' : '#444',
              color: viewMode === 'list' ? 'white' : '#999',
              border: 'none',
              borderRadius: '6px',
              cursor: 'pointer',
              transition: 'all 0.2s ease',
            }}
          >
            List
          </button>
          <button
            onClick={() => setViewMode('block')}
            style={{
              padding: '0.5rem 1rem',
              fontSize: '0.875rem',
              backgroundColor: viewMode === 'block' ? '#646cff' : '#444',
              color: viewMode === 'block' ? 'white' : '#999',
              border: 'none',
              borderRadius: '6px',
              cursor: 'pointer',
              transition: 'all 0.2s ease',
            }}
          >
            Block
          </button>
        </div>
        <div style={{ display: 'flex', gap: '0.5rem' }}>
          <button
            onClick={() => setShowMerge(true)}
            style={{
              padding: '0.625rem 1.5rem',
              fontSize: '0.875rem',
              backgroundColor: '#f59e0b',
              color: 'white',
              border: 'none',
              borderRadius: '6px',
              cursor: 'pointer',
              fontWeight: '500',
            }}
          >
            Merge Duplicates
          </button>
          <button
            onClick={handleAddNewTerm}
            style={{
              padding: '0.625rem 1.5rem',
              fontSize: '0.875rem',
              backgroundColor: '#646cff',
              color: 'white',
              border: 'none',
              borderRadius: '6px',
              cursor: 'pointer',
              fontWeight: '500',
            }}
          >
            + Add Term
          </button>
        </div>
      </div>

      {/* Search */}
      <GlossarySearch onSearch={handleSearch} />

      {/* Stats */}
      {data && (
        <div style={{
          fontSize: '0.875rem',
          color: '#999',
          marginBottom: '1rem',
        }}>
          {data.total} term{data.total !== 1 ? 's' : ''} found
        </div>
      )}

      {/* Glossary panel */}
      <GlossaryPanel
        terms={terms}
        isLoading={isLoading}
        viewMode={viewMode}
        onTermClick={handleTermClick}
        onDeleteTerm={handleDeleteTerm}
      />

      {/* Pagination */}
      {totalPages > 1 && (
        <div style={{
          display: 'flex',
          justifyContent: 'center',
          gap: '0.5rem',
          marginTop: '1.5rem',
        }}>
          <button
            onClick={() => setPage(p => Math.max(1, p - 1))}
            disabled={page === 1}
            style={{
              padding: '0.5rem 1rem',
              fontSize: '0.875rem',
              backgroundColor: page === 1 ? '#444' : '#646cff',
              color: page === 1 ? '#666' : 'white',
              border: 'none',
              borderRadius: '6px',
              cursor: page === 1 ? 'not-allowed' : 'pointer',
            }}
          >
            Previous
          </button>
          <span style={{
            padding: '0.5rem 1rem',
            fontSize: '0.875rem',
            color: '#999',
            display: 'flex',
            alignItems: 'center',
          }}>
            Page {page} of {totalPages}
          </span>
          <button
            onClick={() => setPage(p => Math.min(totalPages, p + 1))}
            disabled={page === totalPages}
            style={{
              padding: '0.5rem 1rem',
              fontSize: '0.875rem',
              backgroundColor: page === totalPages ? '#444' : '#646cff',
              color: page === totalPages ? '#666' : 'white',
              border: 'none',
              borderRadius: '6px',
              cursor: page === totalPages ? 'not-allowed' : 'pointer',
            }}
          >
            Next
          </button>
        </div>
      )}

      {/* Term edit/add modal */}
      {showForm && (
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
            padding: '1rem',
          }}
          onClick={handleFormCancel}
        >
          <div
            style={{
              maxWidth: '600px',
              width: '100%',
              maxHeight: '90vh',
              overflowY: 'auto',
            }}
            onClick={(e) => e.stopPropagation()}
          >
            <TermForm
              term={editingTerm}
              onSubmit={handleFormSubmit}
              onCancel={handleFormCancel}
            />
          </div>
        </div>
      )}

      {/* Merge duplicates modal */}
      {showMerge && (
        <TermMerge onClose={() => setShowMerge(false)} />
      )}
    </div>
  )
}
