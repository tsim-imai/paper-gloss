import { describe, it, expect } from 'vitest'
import React from 'react'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { http, HttpResponse } from 'msw'
import { server } from '../msw/server'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import ProcessingStatus from '../../src/components/ProcessingStatus'

function renderWithQuery(ui: React.ReactElement) {
  const qc = new QueryClient()
  return render(<QueryClientProvider client={qc}>{ui}</QueryClientProvider>)
}

const paperBase = {
  id: 'p1',
  title: 'Paper',
  source: 'upload' as const,
  file_path: '',
  created_at: new Date().toISOString(),
  updated_at: new Date().toISOString(),
}

describe('ProcessingStatus with MSW', () => {
  it('shows Start button for pending and sends process POST', async () => {
    let called = 0
    server.use(
      http.post('/api/papers/p1/process', async () => {
        called += 1
        return HttpResponse.json({ paper_id: 'p1', message: 'ok' }, { status: 202 })
      })
    )

    renderWithQuery(
      <ProcessingStatus
        paper={{ ...paperBase, status: 'pending', total_chunks: 10, translated_chunks: 0 }}
      />
    )

    const btn = screen.getByRole('button', { name: /Start Processing/i })
    await userEvent.click(btn)
    expect(called).toBe(1)
  })

  it('polls progress when processing', async () => {
    server.use(
      http.get('/api/papers/p1/status', async () => {
        return HttpResponse.json({
          paper_id: 'p1',
          status: 'processing',
          progress: { extraction: 'completed', translation: { total_chunks: 10, completed_chunks: 3, failed_chunks: 0 }, term_extraction: 'pending', definitions: { total_terms: 0, completed_definitions: 0 } }
        })
      })
    )

    renderWithQuery(
      <ProcessingStatus
        paper={{ ...paperBase, status: 'processing', total_chunks: 10, translated_chunks: 2 }}
      />
    )

    expect(await screen.findByText(/30%/)).toBeInTheDocument()
  })
})

