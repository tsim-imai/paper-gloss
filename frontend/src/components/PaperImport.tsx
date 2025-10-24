import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { apiClient } from '../services/api'

export default function PaperImport() {
  const [url, setUrl] = useState('')
  const [title, setTitle] = useState('')
  const [error, setError] = useState<string | null>(null)

  const queryClient = useQueryClient()

  const importUrlMutation = useMutation({
    mutationFn: (data: { url: string; title: string }) =>
      apiClient.importPaperUrl(data.url, data.title),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['papers'] })
      setUrl('')
      setTitle('')
      setError(null)
      alert('Paper imported successfully!')
    },
    onError: (err: any) => {
      setError(err.response?.data?.error || 'Failed to import paper')
    },
  })

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)

    if (!url) {
      setError('Please enter an arXiv URL')
      return
    }
    if (!title) {
      setError('Please enter a paper title')
      return
    }
    // Validate arXiv URL format
    const arxivRegex = /^https:\/\/(www\.)?arxiv\.org\/abs\/\d{4}\.\d{4,5}(v\d+)?$/
    if (!arxivRegex.test(url)) {
      setError('Invalid arXiv URL format. Example: https://arxiv.org/abs/2212.14578')
      return
    }
    importUrlMutation.mutate({ url, title })
  }

  const isLoading = importUrlMutation.isPending

  return (
    <div style={{
      border: '1px solid #444',
      borderRadius: '8px',
      padding: '1rem',
      backgroundColor: '#2a2a2a'
    }}>
      <h2 style={{ marginTop: 0, marginBottom: '1.5rem', color: '#e0e0e0', fontSize: '1.25rem' }}>
        Import Paper from arXiv
      </h2>

      <form onSubmit={handleSubmit}>
        <div style={{ marginBottom: '1rem' }}>
          <label htmlFor="url-input" style={{ display: 'block', marginBottom: '0.5rem', color: '#e0e0e0' }}>
            arXiv URL *
          </label>
          <input
            id="url-input"
            type="text"
            value={url}
            onChange={(e) => setUrl(e.target.value)}
            placeholder="https://arxiv.org/abs/2212.14578"
            disabled={isLoading}
            style={{
              width: '100%',
              padding: '0.5rem',
              backgroundColor: '#1a1a1a',
              color: '#e0e0e0',
              border: '1px solid #444',
              borderRadius: '4px'
            }}
          />
          <small style={{ color: '#999', fontSize: '0.875rem' }}>
            LaTeX source will be downloaded from arXiv
          </small>
        </div>

        <div style={{ marginBottom: '1rem' }}>
          <label htmlFor="title-input" style={{ display: 'block', marginBottom: '0.5rem', color: '#e0e0e0' }}>
            Title *
          </label>
          <input
            id="title-input"
            type="text"
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            placeholder="Enter paper title"
            disabled={isLoading}
            style={{
              width: '100%',
              padding: '0.5rem',
              backgroundColor: '#1a1a1a',
              color: '#e0e0e0',
              border: '1px solid #444',
              borderRadius: '4px'
            }}
          />
        </div>

        {error && (
          <div style={{
            padding: '0.75rem',
            marginBottom: '1rem',
            backgroundColor: '#3d1a1a',
            color: '#ff6b6b',
            borderRadius: '4px',
            fontSize: '0.875rem',
            border: '1px solid #8b0000'
          }}>
            {error}
          </div>
        )}

        <button
          type="submit"
          disabled={isLoading}
          style={{
            padding: '0.625rem 1.5rem',
            backgroundColor: isLoading ? '#444' : '#646cff',
            color: isLoading ? '#666' : 'white',
            border: 'none',
            borderRadius: '6px',
            cursor: isLoading ? 'not-allowed' : 'pointer',
            fontSize: '0.875rem',
            fontWeight: '500',
            transition: 'all 0.2s ease',
          }}
          onMouseEnter={(e) => {
            if (!isLoading) {
              e.currentTarget.style.backgroundColor = '#7c84ff'
            }
          }}
          onMouseLeave={(e) => {
            if (!isLoading) {
              e.currentTarget.style.backgroundColor = '#646cff'
            }
          }}
        >
          {isLoading ? 'Importing...' : 'Import Paper'}
        </button>
      </form>
    </div>
  )
}
