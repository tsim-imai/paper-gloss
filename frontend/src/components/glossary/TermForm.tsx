import { useState, useEffect } from 'react'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import { TermListItem, TermDetail } from '../../types'
import { apiClient } from '../../services/api'
import TermDefinitionRegenerate from './TermDefinitionRegenerate'

interface TermFormProps {
  term?: TermListItem | null
  onSubmit: (data: TermFormData) => void
  onCancel: () => void
}

export interface TermFormData {
  lemma_en: string
  lemma_ja: string
  reading_kana?: string
  pos?: string
  tags?: string
  note?: string
}

/**
 * Term form component for adding/editing terms
 */
export default function TermForm({ term, onSubmit, onCancel }: TermFormProps) {
  const [formData, setFormData] = useState<TermFormData>({
    lemma_en: '',
    lemma_ja: '',
    reading_kana: '',
    pos: '',
    tags: '',
    note: '',
  })

  const queryClient = useQueryClient()

  // Fetch term details including definition when editing
  const { data: termDetail } = useQuery({
    queryKey: ['term', term?.id],
    queryFn: async () => {
      if (!term?.id) return null
      const response = await apiClient.getTerm(term.id)
      return response.data as TermDetail
    },
    enabled: !!term?.id,
  })

  useEffect(() => {
    if (term) {
      setFormData({
        lemma_en: term.lemma_en,
        lemma_ja: term.lemma_ja,
        reading_kana: term.reading_kana || '',
        pos: term.pos || '',
        tags: term.tags || '',
        note: '',
      })
    }
  }, [term])

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    onSubmit(formData)
  }

  const handleDefinitionUpdated = () => {
    // Invalidate the query to refresh the term details
    queryClient.invalidateQueries({ queryKey: ['term', term?.id] })
    queryClient.invalidateQueries({ queryKey: ['terms'] })
  }

  const isEditing = !!term

  return (
    <div style={{
      border: '1px solid #ccc',
      borderRadius: '8px',
      padding: '1.5rem',
      backgroundColor: 'white',
    }}>
      <h3 style={{ marginTop: 0, marginBottom: '1rem' }}>
        {isEditing ? 'Edit Term' : 'Add New Term'}
      </h3>

      <form onSubmit={handleSubmit}>
        {/* English lemma */}
        <div style={{ marginBottom: '1rem' }}>
          <label htmlFor="term-lemma-en" style={{
            display: 'block',
            fontSize: '0.875rem',
            fontWeight: 'bold',
            color: '#666',
            marginBottom: '0.25rem',
          }}>
            English Term <span style={{ color: '#c62828' }}>*</span>
          </label>
          <input
            id="term-lemma-en"
            type="text"
            required
            value={formData.lemma_en}
            onChange={(e) => setFormData({ ...formData, lemma_en: e.target.value })}
            placeholder="e.g., neural network"
            style={{
              width: '100%',
              padding: '0.5rem',
              fontSize: '1rem',
              border: '1px solid #ccc',
              borderRadius: '4px',
            }}
          />
        </div>

        {/* Japanese lemma */}
        <div style={{ marginBottom: '1rem' }}>
          <label htmlFor="term-lemma-ja" style={{
            display: 'block',
            fontSize: '0.875rem',
            fontWeight: 'bold',
            color: '#666',
            marginBottom: '0.25rem',
          }}>
            Japanese Term <span style={{ color: '#c62828' }}>*</span>
          </label>
          <input
            id="term-lemma-ja"
            type="text"
            required
            value={formData.lemma_ja}
            onChange={(e) => setFormData({ ...formData, lemma_ja: e.target.value })}
            placeholder="e.g., ニューラルネットワーク"
            style={{
              width: '100%',
              padding: '0.5rem',
              fontSize: '1rem',
              border: '1px solid #ccc',
              borderRadius: '4px',
            }}
          />
        </div>

        {/* Reading (kana) */}
        <div style={{ marginBottom: '1rem' }}>
          <label htmlFor="term-reading" style={{
            display: 'block',
            fontSize: '0.875rem',
            fontWeight: 'bold',
            color: '#666',
            marginBottom: '0.25rem',
          }}>
            Reading (Kana)
          </label>
          <input
            id="term-reading"
            type="text"
            value={formData.reading_kana}
            onChange={(e) => setFormData({ ...formData, reading_kana: e.target.value })}
            placeholder="e.g., にゅーらるねっとわーく"
            style={{
              width: '100%',
              padding: '0.5rem',
              fontSize: '1rem',
              border: '1px solid #ccc',
              borderRadius: '4px',
            }}
          />
        </div>

        {/* Part of speech */}
        <div style={{ marginBottom: '1rem' }}>
          <label htmlFor="term-pos" style={{
            display: 'block',
            fontSize: '0.875rem',
            fontWeight: 'bold',
            color: '#666',
            marginBottom: '0.25rem',
          }}>
            Part of Speech
          </label>
          <input
            id="term-pos"
            type="text"
            value={formData.pos}
            onChange={(e) => setFormData({ ...formData, pos: e.target.value })}
            placeholder="e.g., noun, verb"
            style={{
              width: '100%',
              padding: '0.5rem',
              fontSize: '1rem',
              border: '1px solid #ccc',
              borderRadius: '4px',
            }}
          />
        </div>

        {/* Tags */}
        <div style={{ marginBottom: '1rem' }}>
          <label htmlFor="term-tags" style={{
            display: 'block',
            fontSize: '0.875rem',
            fontWeight: 'bold',
            color: '#666',
            marginBottom: '0.25rem',
          }}>
            Tags
          </label>
          <input
            id="term-tags"
            type="text"
            value={formData.tags}
            onChange={(e) => setFormData({ ...formData, tags: e.target.value })}
            placeholder="e.g., ML, deep-learning"
            style={{
              width: '100%',
              padding: '0.5rem',
              fontSize: '1rem',
              border: '1px solid #ccc',
              borderRadius: '4px',
            }}
          />
        </div>

        {/* Note */}
        <div style={{ marginBottom: '1.5rem' }}>
          <label htmlFor="term-note" style={{
            display: 'block',
            fontSize: '0.875rem',
            fontWeight: 'bold',
            color: '#666',
            marginBottom: '0.25rem',
          }}>
            Note
          </label>
          <textarea
            id="term-note"
            value={formData.note}
            onChange={(e) => setFormData({ ...formData, note: e.target.value })}
            placeholder="Additional notes..."
            rows={3}
            style={{
              width: '100%',
              padding: '0.5rem',
              fontSize: '1rem',
              border: '1px solid #ccc',
              borderRadius: '4px',
              fontFamily: 'inherit',
            }}
          />
        </div>

        {/* Definition regeneration (only when editing) */}
        {isEditing && termDetail && (
          <div style={{ marginBottom: '1.5rem' }}>
            <TermDefinitionRegenerate
              term={{
                id: termDetail.id,
                lemma_en: termDetail.lemma_en,
                lemma_ja: termDetail.lemma_ja,
                definition: termDetail.definition,
              }}
              onDefinitionUpdated={handleDefinitionUpdated}
            />
          </div>
        )}

        {/* Buttons */}
        <div style={{ display: 'flex', gap: '0.5rem', justifyContent: 'flex-end' }}>
          <button
            type="button"
            onClick={onCancel}
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
            type="submit"
            style={{
              padding: '0.5rem 1rem',
              fontSize: '1rem',
              backgroundColor: '#646cff',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
            }}
          >
            {isEditing ? 'Update' : 'Add'} Term
          </button>
        </div>
      </form>
    </div>
  )
}
