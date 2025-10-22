import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { apiClient } from '../services/api'

type ImportMode = 'file' | 'url'

export default function PaperImport() {
  const [mode, setMode] = useState<ImportMode>('file')
  const [file, setFile] = useState<File | null>(null)
  const [url, setUrl] = useState('')
  const [title, setTitle] = useState('')
  const [error, setError] = useState<string | null>(null)

  const queryClient = useQueryClient()

  const importFileMutation = useMutation({
    mutationFn: (data: { file: File; title: string }) =>
      apiClient.importPaperFile(data.file, data.title || ''),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['papers'] })
      setFile(null)
      setTitle('')
      setError(null)
      alert('Paper imported successfully!')
    },
    onError: (err: any) => {
      setError(err.response?.data?.error || 'Failed to import paper')
    },
  })

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

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const selectedFile = e.target.files?.[0]
    if (selectedFile) {
      setFile(selectedFile)
      setError(null)
      // Set default title from filename if not already set
      if (!title) {
        setTitle(selectedFile.name.replace('.pdf', ''))
      }
    }
  }

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)

    if (mode === 'file') {
      if (!file) {
        setError('Please select a PDF file')
        return
      }
      importFileMutation.mutate({ file, title: title })
    } else {
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
  }

  const isLoading = importFileMutation.isPending || importUrlMutation.isPending

  return (
    <div style={{
      border: '1px solid #ccc',
      borderRadius: '8px',
      padding: '1.5rem',
      backgroundColor: '#f9f9f9'
    }}>
      {/* Mode toggle */}
      <div style={{ marginBottom: '1rem', display: 'flex', gap: '1rem' }}>
        <button
          onClick={() => setMode('file')}
          style={{
            padding: '0.5rem 1rem',
            backgroundColor: mode === 'file' ? '#646cff' : '#e0e0e0',
            color: mode === 'file' ? 'white' : 'black',
            border: 'none',
            borderRadius: '4px',
            cursor: 'pointer',
          }}
        >
          Upload PDF
        </button>
        <button
          onClick={() => setMode('url')}
          style={{
            padding: '0.5rem 1rem',
            backgroundColor: mode === 'url' ? '#646cff' : '#e0e0e0',
            color: mode === 'url' ? 'white' : 'black',
            border: 'none',
            borderRadius: '4px',
            cursor: 'pointer',
          }}
        >
          Import from arXiv
        </button>
      </div>

      <form onSubmit={handleSubmit}>
        {mode === 'file' ? (
          <div key="file-mode">
            <div style={{ marginBottom: '1rem' }}>
              <label htmlFor="file-input" style={{ display: 'block', marginBottom: '0.5rem' }}>
                PDF File *
              </label>
              <input
                id="file-input"
                type="file"
                accept=".pdf"
                onChange={handleFileChange}
                disabled={isLoading}
                style={{ width: '100%' }}
              />
            </div>

            <div style={{ marginBottom: '1rem' }}>
              <label htmlFor="title-input" style={{ display: 'block', marginBottom: '0.5rem' }}>
                Title (optional)
              </label>
              <input
                id="title-input"
                type="text"
                value={title}
                onChange={(e) => setTitle(e.target.value)}
                placeholder="Defaults to filename"
                disabled={isLoading}
                style={{ width: '100%' }}
              />
            </div>
          </div>
        ) : (
          <div key="url-mode">
            <div style={{ marginBottom: '1rem' }}>
              <label htmlFor="url-input" style={{ display: 'block', marginBottom: '0.5rem' }}>
                arXiv URL *
              </label>
              <input
                id="url-input"
                type="text"
                value={url}
                onChange={(e) => setUrl(e.target.value)}
                placeholder="https://arxiv.org/abs/2212.14578"
                disabled={isLoading}
                style={{ width: '100%' }}
              />
              <small style={{ color: '#666', fontSize: '0.875rem' }}>
                Only arXiv URLs are supported in this version
              </small>
            </div>

            <div style={{ marginBottom: '1rem' }}>
              <label htmlFor="title-input" style={{ display: 'block', marginBottom: '0.5rem' }}>
                Title *
              </label>
              <input
                id="title-input"
                type="text"
                value={title}
                onChange={(e) => setTitle(e.target.value)}
                placeholder="Enter paper title"
                disabled={isLoading}
                style={{ width: '100%' }}
              />
            </div>
          </div>
        )}

        {error && (
          <div style={{
            padding: '0.75rem',
            marginBottom: '1rem',
            backgroundColor: '#ffebee',
            color: '#c62828',
            borderRadius: '4px',
            fontSize: '0.875rem'
          }}>
            {error}
          </div>
        )}

        <button
          type="submit"
          disabled={isLoading}
          style={{
            padding: '0.75rem 2rem',
            backgroundColor: isLoading ? '#ccc' : '#646cff',
            color: 'white',
            border: 'none',
            borderRadius: '4px',
            cursor: isLoading ? 'not-allowed' : 'pointer',
            fontSize: '1rem',
            fontWeight: 'bold',
          }}
        >
          {isLoading ? 'Importing...' : 'Import Paper'}
        </button>
      </form>
    </div>
  )
}
