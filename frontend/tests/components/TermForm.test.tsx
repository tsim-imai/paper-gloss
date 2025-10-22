import { describe, it, expect, vi, beforeEach } from 'vitest'
import React from 'react'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import TermForm from '../../src/components/glossary/TermForm'
import type { TermListItem } from '../../src/types'

// Mock the API client using vi.hoisted
const mockGetTerm = vi.hoisted(() => vi.fn())

// Mock the TermDefinitionRegenerate component to avoid complex nested dependencies
vi.mock('../../src/components/glossary/TermDefinitionRegenerate', () => ({
  default: ({ term, onDefinitionUpdated }: any) => (
    <div data-testid="term-definition-regenerate">
      TermDefinitionRegenerate for {term.lemma_en}
    </div>
  ),
}))

// Mock the API client
vi.mock('../../src/services/api', () => ({
  apiClient: {
    getTerm: mockGetTerm,
  },
}))

function renderWithQuery(ui: React.ReactElement) {
  const qc = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  })
  return render(<QueryClientProvider client={qc}>{ui}</QueryClientProvider>)
}

describe('TermForm', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('Add mode (new term)', () => {
    it('renders empty form with "Add New Term" title', () => {
      const onSubmit = vi.fn()
      const onCancel = vi.fn()

      renderWithQuery(<TermForm onSubmit={onSubmit} onCancel={onCancel} />)

      expect(screen.getByText('Add New Term')).toBeInTheDocument()
      expect(screen.getByRole('button', { name: /Add Term/i })).toBeInTheDocument()
    })

    it('does not show TermDefinitionRegenerate in add mode', () => {
      const onSubmit = vi.fn()
      const onCancel = vi.fn()

      renderWithQuery(<TermForm onSubmit={onSubmit} onCancel={onCancel} />)

      expect(screen.queryByTestId('term-definition-regenerate')).not.toBeInTheDocument()
    })

    it('accepts user input in all fields', async () => {
      const onSubmit = vi.fn()
      const onCancel = vi.fn()

      renderWithQuery(<TermForm onSubmit={onSubmit} onCancel={onCancel} />)

      const englishInput = screen.getByLabelText(/English Term/i)
      const japaneseInput = screen.getByLabelText(/Japanese Term/i)
      const readingInput = screen.getByLabelText(/Reading \(Kana\)/i)
      const posInput = screen.getByLabelText(/Part of Speech/i)
      const tagsInput = screen.getByLabelText(/Tags/i)
      const noteInput = screen.getByLabelText(/Note/i)

      await userEvent.type(englishInput, 'neural network')
      await userEvent.type(japaneseInput, 'ニューラルネットワーク')
      await userEvent.type(readingInput, 'にゅーらるねっとわーく')
      await userEvent.type(posInput, 'noun')
      await userEvent.type(tagsInput, 'ML, AI')
      await userEvent.type(noteInput, 'Test note')

      expect(englishInput).toHaveValue('neural network')
      expect(japaneseInput).toHaveValue('ニューラルネットワーク')
      expect(readingInput).toHaveValue('にゅーらるねっとわーく')
      expect(posInput).toHaveValue('noun')
      expect(tagsInput).toHaveValue('ML, AI')
      expect(noteInput).toHaveValue('Test note')
    })

    it('calls onSubmit with form data when submitted', async () => {
      const onSubmit = vi.fn()
      const onCancel = vi.fn()

      renderWithQuery(<TermForm onSubmit={onSubmit} onCancel={onCancel} />)

      await userEvent.type(screen.getByLabelText(/English Term/i), 'neural network')
      await userEvent.type(screen.getByLabelText(/Japanese Term/i), 'ニューラルネットワーク')
      await userEvent.type(screen.getByLabelText(/Reading \(Kana\)/i), 'にゅーらるねっとわーく')
      await userEvent.type(screen.getByLabelText(/Part of Speech/i), 'noun')
      await userEvent.type(screen.getByLabelText(/Tags/i), 'ML')
      await userEvent.type(screen.getByLabelText(/Note/i), 'Test note')

      await userEvent.click(screen.getByRole('button', { name: /Add Term/i }))

      expect(onSubmit).toHaveBeenCalledWith({
        lemma_en: 'neural network',
        lemma_ja: 'ニューラルネットワーク',
        reading_kana: 'にゅーらるねっとわーく',
        pos: 'noun',
        tags: 'ML',
        note: 'Test note',
      })
    })

    it('calls onCancel when Cancel button is clicked', async () => {
      const onSubmit = vi.fn()
      const onCancel = vi.fn()

      renderWithQuery(<TermForm onSubmit={onSubmit} onCancel={onCancel} />)

      await userEvent.click(screen.getByRole('button', { name: /Cancel/i }))

      expect(onCancel).toHaveBeenCalled()
      expect(onSubmit).not.toHaveBeenCalled()
    })

    it('requires English and Japanese terms', async () => {
      const onSubmit = vi.fn()
      const onCancel = vi.fn()

      renderWithQuery(<TermForm onSubmit={onSubmit} onCancel={onCancel} />)

      const submitButton = screen.getByRole('button', { name: /Add Term/i })
      await userEvent.click(submitButton)

      // HTML5 validation prevents submission, so onSubmit should not be called
      expect(onSubmit).not.toHaveBeenCalled()
    })
  })

  describe('Edit mode (existing term)', () => {
    const mockTerm: TermListItem = {
      id: 'term-123',
      slug: 'neural-network',
      lemma_en: 'neural network',
      lemma_ja: 'ニューラルネットワーク',
      reading_kana: 'にゅーらるねっとわーく',
      pos: 'noun',
      tags: 'ML, AI',
      occurrence_count: 5,
      created_at: '2024-01-01T00:00:00Z',
      updated_at: '2024-01-01T00:00:00Z',
    }

    const mockTermDetail = {
      ...mockTerm,
      definition: {
        text: '複数の層で構成されたニューロンの集合体。',
        provider: 'gpt-4',
      },
    }

    beforeEach(() => {
      mockGetTerm.mockResolvedValue({ data: mockTermDetail })
    })

    it('renders form with "Edit Term" title', () => {
      const onSubmit = vi.fn()
      const onCancel = vi.fn()

      renderWithQuery(<TermForm term={mockTerm} onSubmit={onSubmit} onCancel={onCancel} />)

      expect(screen.getByText('Edit Term')).toBeInTheDocument()
      expect(screen.getByRole('button', { name: /Update Term/i })).toBeInTheDocument()
    })

    it('populates form fields with existing term data', () => {
      const onSubmit = vi.fn()
      const onCancel = vi.fn()

      renderWithQuery(<TermForm term={mockTerm} onSubmit={onSubmit} onCancel={onCancel} />)

      expect(screen.getByLabelText(/English Term/i)).toHaveValue('neural network')
      expect(screen.getByLabelText(/Japanese Term/i)).toHaveValue('ニューラルネットワーク')
      expect(screen.getByLabelText(/Reading \(Kana\)/i)).toHaveValue('にゅーらるねっとわーく')
      expect(screen.getByLabelText(/Part of Speech/i)).toHaveValue('noun')
      expect(screen.getByLabelText(/Tags/i)).toHaveValue('ML, AI')
      // Note field is always empty (not stored in term)
      expect(screen.getByLabelText(/Note/i)).toHaveValue('')
    })

    it('fetches term details and shows TermDefinitionRegenerate', async () => {
      const onSubmit = vi.fn()
      const onCancel = vi.fn()

      renderWithQuery(<TermForm term={mockTerm} onSubmit={onSubmit} onCancel={onCancel} />)

      // Wait for useQuery to fetch term details
      await waitFor(() => {
        expect(mockGetTerm).toHaveBeenCalledWith('term-123')
      })

      // TermDefinitionRegenerate should be rendered
      expect(
        await screen.findByTestId('term-definition-regenerate')
      ).toBeInTheDocument()
      expect(screen.getByText(/TermDefinitionRegenerate for neural network/i)).toBeInTheDocument()
    })

    it('calls onSubmit with updated data when form is submitted', async () => {
      const onSubmit = vi.fn()
      const onCancel = vi.fn()

      renderWithQuery(<TermForm term={mockTerm} onSubmit={onSubmit} onCancel={onCancel} />)

      // Modify some fields
      const tagsInput = screen.getByLabelText(/Tags/i)
      await userEvent.clear(tagsInput)
      await userEvent.type(tagsInput, 'ML, AI, deep-learning')

      await userEvent.type(screen.getByLabelText(/Note/i), 'Updated note')

      await userEvent.click(screen.getByRole('button', { name: /Update Term/i }))

      expect(onSubmit).toHaveBeenCalledWith({
        lemma_en: 'neural network',
        lemma_ja: 'ニューラルネットワーク',
        reading_kana: 'にゅーらるねっとわーく',
        pos: 'noun',
        tags: 'ML, AI, deep-learning',
        note: 'Updated note',
      })
    })

    it('does not show TermDefinitionRegenerate while term details are loading', () => {
      const onSubmit = vi.fn()
      const onCancel = vi.fn()

      // Make getTerm return a promise that never resolves
      mockGetTerm.mockImplementation(() => new Promise(() => {}))

      renderWithQuery(<TermForm term={mockTerm} onSubmit={onSubmit} onCancel={onCancel} />)

      // Should not show TermDefinitionRegenerate immediately
      expect(screen.queryByTestId('term-definition-regenerate')).not.toBeInTheDocument()
    })

    it('handles getTerm API error gracefully', async () => {
      const onSubmit = vi.fn()
      const onCancel = vi.fn()

      // Make getTerm fail
      mockGetTerm.mockRejectedValue(new Error('Failed to fetch term'))

      renderWithQuery(<TermForm term={mockTerm} onSubmit={onSubmit} onCancel={onCancel} />)

      // Should still show the form with basic data
      expect(screen.getByLabelText(/English Term/i)).toHaveValue('neural network')
      expect(screen.getByLabelText(/Japanese Term/i)).toHaveValue('ニューラルネットワーク')

      // Should not show TermDefinitionRegenerate when fetch fails
      await waitFor(() => {
        expect(mockGetTerm).toHaveBeenCalledWith('term-123')
      })
      expect(screen.queryByTestId('term-definition-regenerate')).not.toBeInTheDocument()
    })
  })

  describe('Edge cases', () => {
    it('handles term with minimal data (only required fields)', () => {
      const minimalTerm: TermListItem = {
        id: 'term-456',
        slug: 'test',
        lemma_en: 'test',
        lemma_ja: 'テスト',
        reading_kana: null,
        pos: null,
        tags: null,
        occurrence_count: 0,
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-01T00:00:00Z',
      }

      const onSubmit = vi.fn()
      const onCancel = vi.fn()

      mockGetTerm.mockResolvedValue({ data: minimalTerm })

      renderWithQuery(<TermForm term={minimalTerm} onSubmit={onSubmit} onCancel={onCancel} />)

      expect(screen.getByLabelText(/English Term/i)).toHaveValue('test')
      expect(screen.getByLabelText(/Japanese Term/i)).toHaveValue('テスト')
      expect(screen.getByLabelText(/Reading \(Kana\)/i)).toHaveValue('')
      expect(screen.getByLabelText(/Part of Speech/i)).toHaveValue('')
      expect(screen.getByLabelText(/Tags/i)).toHaveValue('')
    })

    it('handles form submission with only required fields filled', async () => {
      const onSubmit = vi.fn()
      const onCancel = vi.fn()

      renderWithQuery(<TermForm onSubmit={onSubmit} onCancel={onCancel} />)

      await userEvent.type(screen.getByLabelText(/English Term/i), 'test')
      await userEvent.type(screen.getByLabelText(/Japanese Term/i), 'テスト')

      await userEvent.click(screen.getByRole('button', { name: /Add Term/i }))

      expect(onSubmit).toHaveBeenCalledWith({
        lemma_en: 'test',
        lemma_ja: 'テスト',
        reading_kana: '',
        pos: '',
        tags: '',
        note: '',
      })
    })

    it('updates form fields when term prop changes', () => {
      const onSubmit = vi.fn()
      const onCancel = vi.fn()

      const term1: TermListItem = {
        id: 'term-1',
        slug: 'term1',
        lemma_en: 'first term',
        lemma_ja: '最初の用語',
        reading_kana: null,
        pos: null,
        tags: null,
        occurrence_count: 0,
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-01T00:00:00Z',
      }

      const term2: TermListItem = {
        id: 'term-2',
        slug: 'term2',
        lemma_en: 'second term',
        lemma_ja: '二番目の用語',
        reading_kana: null,
        pos: null,
        tags: null,
        occurrence_count: 0,
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-01T00:00:00Z',
      }

      mockGetTerm.mockResolvedValue({ data: term1 })

      const { rerender } = renderWithQuery(
        <TermForm term={term1} onSubmit={onSubmit} onCancel={onCancel} />
      )

      expect(screen.getByLabelText(/English Term/i)).toHaveValue('first term')

      // Update term prop
      mockGetTerm.mockResolvedValue({ data: term2 })
      rerender(
        <QueryClientProvider
          client={
            new QueryClient({
              defaultOptions: {
                queries: { retry: false },
                mutations: { retry: false },
              },
            })
          }
        >
          <TermForm term={term2} onSubmit={onSubmit} onCancel={onCancel} />
        </QueryClientProvider>
      )

      expect(screen.getByLabelText(/English Term/i)).toHaveValue('second term')
      expect(screen.getByLabelText(/Japanese Term/i)).toHaveValue('二番目の用語')
    })
  })
})
