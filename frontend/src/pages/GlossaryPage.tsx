import { useState } from 'react'
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
  const [showForm, setShowForm] = useState(false)
  const [showMerge, setShowMerge] = useState(false)
  const [editingTerm, setEditingTerm] = useState<TermListItem | null>(null)

  const queryClient = useQueryClient()

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
    <div style={{ maxWidth: '1200px', margin: '0 auto', padding: '2rem' }}>
      {/* Header */}
      <div style={{
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        marginBottom: '1.5rem',
      }}>
        <h1 style={{ margin: 0 }}>Glossary</h1>
        <div style={{ display: 'flex', gap: '0.5rem' }}>
          <button
            onClick={() => setShowMerge(true)}
            style={{
              padding: '0.75rem 1.5rem',
              fontSize: '1rem',
              backgroundColor: '#f59e0b',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
              fontWeight: 'bold',
            }}
          >
            Merge Duplicates
          </button>
          <button
            onClick={handleAddNewTerm}
            style={{
              padding: '0.75rem 1.5rem',
              fontSize: '1rem',
              backgroundColor: '#646cff',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
              fontWeight: 'bold',
            }}
          >
            + Add Term
          </button>
        </div>
      </div>

      {/* Search */}
      <GlossarySearch onSearch={handleSearch} />

      {/* Form (when adding/editing) */}
      {showForm && (
        <div style={{ marginBottom: '1.5rem' }}>
          <TermForm
            term={editingTerm}
            onSubmit={handleFormSubmit}
            onCancel={handleFormCancel}
          />
        </div>
      )}

      {/* Stats */}
      {data && (
        <div style={{
          fontSize: '0.875rem',
          color: '#666',
          marginBottom: '1rem',
        }}>
          {data.total} term{data.total !== 1 ? 's' : ''} found
        </div>
      )}

      {/* Glossary panel */}
      <GlossaryPanel
        terms={terms}
        isLoading={isLoading}
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
              backgroundColor: page === 1 ? '#f5f5f5' : '#646cff',
              color: page === 1 ? '#999' : 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: page === 1 ? 'not-allowed' : 'pointer',
            }}
          >
            Previous
          </button>
          <span style={{
            padding: '0.5rem 1rem',
            fontSize: '0.875rem',
            color: '#666',
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
              backgroundColor: page === totalPages ? '#f5f5f5' : '#646cff',
              color: page === totalPages ? '#999' : 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: page === totalPages ? 'not-allowed' : 'pointer',
            }}
          >
            Next
          </button>
        </div>
      )}

      {/* Merge duplicates modal */}
      {showMerge && (
        <TermMerge onClose={() => setShowMerge(false)} />
      )}
    </div>
  )
}
