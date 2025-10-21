import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import '@testing-library/jest-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import TermDefinitionRegenerate from '../../src/components/glossary/TermDefinitionRegenerate'
import { apiClient } from '../../src/services/api'

// Mock the API client
vi.mock('../../src/services/api', () => ({
  apiClient: {
    generateDefinition: vi.fn(),
  },
}))

describe('TermDefinitionRegenerate', () => {
  let queryClient: QueryClient

  beforeEach(() => {
    queryClient = new QueryClient({
      defaultOptions: {
        queries: { retry: false },
        mutations: { retry: false },
      },
    })
    vi.clearAllMocks()
  })

  const mockTerm = {
    id: 'term-123',
    lemma_en: 'neural network',
    lemma_ja: 'ニューラルネットワーク',
    definition: {
      text: '複数の層で構成されたニューロンの集合体。',
      provider: 'gpt-4',
    },
  }

  it('should render regenerate button with current definition', () => {
    render(
      <QueryClientProvider client={queryClient}>
        <TermDefinitionRegenerate term={mockTerm} onDefinitionUpdated={vi.fn()} />
      </QueryClientProvider>
    )

    expect(screen.getByText('複数の層で構成されたニューロンの集合体。')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /regenerate definition/i })).toBeInTheDocument()
  })

  it('should show confirmation dialog when regenerate button is clicked', () => {
    render(
      <QueryClientProvider client={queryClient}>
        <TermDefinitionRegenerate term={mockTerm} onDefinitionUpdated={vi.fn()} />
      </QueryClientProvider>
    )

    const regenerateButton = screen.getByRole('button', { name: /regenerate definition/i })
    fireEvent.click(regenerateButton)

    expect(screen.getByText(/are you sure you want to regenerate/i)).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /confirm/i })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /cancel/i })).toBeInTheDocument()
  })

  it('should close dialog when cancel is clicked', () => {
    render(
      <QueryClientProvider client={queryClient}>
        <TermDefinitionRegenerate term={mockTerm} onDefinitionUpdated={vi.fn()} />
      </QueryClientProvider>
    )

    fireEvent.click(screen.getByRole('button', { name: /regenerate definition/i }))
    expect(screen.getByText(/are you sure/i)).toBeInTheDocument()

    fireEvent.click(screen.getByRole('button', { name: /cancel/i }))
    expect(screen.queryByText(/are you sure/i)).not.toBeInTheDocument()
  })

  it('should call API and update definition when confirmed', async () => {
    const mockUpdatedDefinition = {
      text: '新しく生成された定義。機械学習の基本構成要素。',
      provider: 'gpt-4-turbo',
    }

    const onDefinitionUpdated = vi.fn()
    vi.mocked(apiClient.generateDefinition).mockResolvedValue({
      data: mockUpdatedDefinition,
    })

    render(
      <QueryClientProvider client={queryClient}>
        <TermDefinitionRegenerate term={mockTerm} onDefinitionUpdated={onDefinitionUpdated} />
      </QueryClientProvider>
    )

    // Click regenerate button
    fireEvent.click(screen.getByRole('button', { name: /regenerate definition/i }))

    // Confirm regeneration
    fireEvent.click(screen.getByRole('button', { name: /confirm/i }))

    // Check loading state
    expect(screen.getByText(/regenerating/i)).toBeInTheDocument()

    // Wait for API call to complete
    await waitFor(() => {
      expect(apiClient.generateDefinition).toHaveBeenCalledWith('term-123')
      expect(onDefinitionUpdated).toHaveBeenCalledWith(mockUpdatedDefinition)
    })

    // Check that new definition is displayed
    expect(screen.getByText('新しく生成された定義。機械学習の基本構成要素。')).toBeInTheDocument()
  })

  it('should show error message when API call fails', async () => {
    const onDefinitionUpdated = vi.fn()
    vi.mocked(apiClient.generateDefinition).mockRejectedValue(
      new Error('Failed to generate definition')
    )

    render(
      <QueryClientProvider client={queryClient}>
        <TermDefinitionRegenerate term={mockTerm} onDefinitionUpdated={onDefinitionUpdated} />
      </QueryClientProvider>
    )

    fireEvent.click(screen.getByRole('button', { name: /regenerate definition/i }))
    fireEvent.click(screen.getByRole('button', { name: /confirm/i }))

    await waitFor(() => {
      expect(screen.getByText(/failed to regenerate/i)).toBeInTheDocument()
    })

    expect(onDefinitionUpdated).not.toHaveBeenCalled()
  })

  it('should handle terms without existing definitions', () => {
    const termWithoutDefinition = {
      id: 'term-456',
      lemma_en: 'deep learning',
      lemma_ja: 'ディープラーニング',
      definition: undefined,
    }

    render(
      <QueryClientProvider client={queryClient}>
        <TermDefinitionRegenerate term={termWithoutDefinition} onDefinitionUpdated={vi.fn()} />
      </QueryClientProvider>
    )

    expect(screen.getByText(/no definition available/i)).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /generate definition/i })).toBeInTheDocument()
  })

  it('should be disabled while regenerating', async () => {
    vi.mocked(apiClient.generateDefinition).mockImplementation(
      () => new Promise((resolve) => setTimeout(resolve, 1000))
    )

    render(
      <QueryClientProvider client={queryClient}>
        <TermDefinitionRegenerate term={mockTerm} onDefinitionUpdated={vi.fn()} />
      </QueryClientProvider>
    )

    const regenerateButton = screen.getByRole('button', { name: /regenerate definition/i })
    fireEvent.click(regenerateButton)
    fireEvent.click(screen.getByRole('button', { name: /confirm/i }))

    await waitFor(() => {
      expect(regenerateButton).toBeDisabled()
    })
  })
})