import { describe, it, expect, vi } from 'vitest'
import React from 'react'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { http, HttpResponse } from 'msw'
import { server } from '../msw/server'

import PaperImport from '../../src/components/PaperImport'

function renderWithQuery(ui: React.ReactElement) {
  const qc = new QueryClient()
  return render(<QueryClientProvider client={qc}>{ui}</QueryClientProvider>)
}

describe('PaperImport with MSW', () => {
  it('POST /api/papers/import returns 201 (MSW hit)', async () => {
    let called = 0
    server.use(
      http.post('/api/papers/import', async () => {
        called += 1
        return HttpResponse.json({ paper_id: 'p1', status: 'processing', message: 'ok' }, { status: 201 })
      })
    )

    renderWithQuery(<PaperImport />)

    const fileInput = screen.getByLabelText(/PDF File/i)
    const file = new File([new Uint8Array([1, 2, 3])], 'sample.pdf', { type: 'application/pdf' })
    await userEvent.upload(fileInput, file)
    await userEvent.click(screen.getByRole('button', { name: /Import Paper/i }))

    const { waitFor } = await import('@testing-library/react')
    await waitFor(() => expect(called).toBeGreaterThanOrEqual(1))
  })

  it('URL import returns 422 and shows error banner', async () => {
    server.use(
      http.post('/api/papers/import', async () => {
        return HttpResponse.json({ error: 'UnprocessableEntity', message: 'invalid input' }, { status: 422 })
      })
    )

    renderWithQuery(<PaperImport />)

    await userEvent.click(screen.getByRole('button', { name: /Import from arXiv/i }))
    await userEvent.type(screen.getByLabelText(/arXiv URL/i), 'https://arxiv.org/abs/2212.14578')
    await userEvent.type(screen.getByLabelText(/^Title \*/i), 'Bad Paper')
    await userEvent.click(screen.getByRole('button', { name: /Import Paper/i }))

    // Error banner shows MSW-provided error code
    expect(await screen.findByText(/UnprocessableEntity/i)).toBeInTheDocument()
  })

  it('URL import returns 504 and shows timeout error', async () => {
    server.use(
      http.post('/api/papers/import', async () => {
        return HttpResponse.json({ error: 'GatewayTimeout', message: 'arXiv download timed out' }, { status: 504 })
      })
    )

    renderWithQuery(<PaperImport />)

    await userEvent.click(screen.getByRole('button', { name: /Import from arXiv/i }))
    await userEvent.type(screen.getByLabelText(/arXiv URL/i), 'https://arxiv.org/abs/2212.14578')
    await userEvent.type(screen.getByLabelText(/^Title \*/i), 'My Paper')
    await userEvent.click(screen.getByRole('button', { name: /Import Paper/i }))

    expect(await screen.findByText(/GatewayTimeout/i)).toBeInTheDocument()
  })
})
