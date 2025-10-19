import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import React from 'react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'

vi.mock('../../src/services/api', () => {
  return {
    apiClient: {
      importPaperFile: vi.fn().mockResolvedValue({ data: { paper_id: 'p1', status: 'processing' } }),
      importPaperUrl: vi.fn().mockResolvedValue({ data: { paper_id: 'p2', status: 'processing' } }),
    },
  }
})

import PaperImport from '../../src/components/PaperImport'
import { apiClient } from '../../src/services/api'

function renderWithQuery(ui: React.ReactElement) {
  const qc = new QueryClient()
  return render(<QueryClientProvider client={qc}>{ui}</QueryClientProvider>)
}

describe('T025 PaperImport component', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('imports by file and calls API with selected file', async () => {
    renderWithQuery(<PaperImport />)
    const fileInput = screen.getByLabelText(/PDF File/i)
    const file = new File([new Uint8Array([1, 2, 3])], 'sample.pdf', { type: 'application/pdf' })
    await userEvent.upload(fileInput, file)

    const submit = screen.getByRole('button', { name: /Import Paper/i })
    await userEvent.click(submit)

    expect(apiClient.importPaperFile).toHaveBeenCalled()
  })

  it('imports by arXiv URL and calls API with url + title', async () => {
    renderWithQuery(<PaperImport />)

    // Switch to URL mode
    await userEvent.click(screen.getByRole('button', { name: /Import from arXiv/i }))

    await userEvent.type(screen.getByLabelText(/arXiv URL/i), 'https://arxiv.org/abs/2212.14578')
    await userEvent.type(screen.getByLabelText(/^Title \*/i), 'My Paper')

    const submit = screen.getByRole('button', { name: /Import Paper/i })
    await userEvent.click(submit)

    expect(apiClient.importPaperUrl).toHaveBeenCalledWith('https://arxiv.org/abs/2212.14578', 'My Paper')
  })
})
