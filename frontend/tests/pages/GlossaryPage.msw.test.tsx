import { describe, it, expect } from 'vitest'
import React from 'react'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { http, HttpResponse } from 'msw'
import { server } from '../msw/server'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import GlossaryPage from '../../src/pages/GlossaryPage'

function renderWithQuery(ui: React.ReactElement) {
  const qc = new QueryClient()
  return render(<QueryClientProvider client={qc}>{ui}</QueryClientProvider>)
}

describe('GlossaryPage with MSW', () => {
  it('lists terms and opens TermForm to add', async () => {
    server.use(
      http.get('/api/terms', async () => {
        return HttpResponse.json({
          items: [
            { id: 't1', slug: 'nn', lemma_en: 'neural network', lemma_ja: 'ニューラルネットワーク', occurrence_count: 1, created_at: '', updated_at: '' }
          ], total: 1, page: 1, limit: 50, total_pages: 1
        })
      }),
      http.post('/api/terms', async () => {
        return HttpResponse.json({ id: 't2' }, { status: 201 })
      })
    )

    renderWithQuery(<GlossaryPage />)

    expect(await screen.findByText('ニューラルネットワーク')).toBeInTheDocument()

    await userEvent.click(screen.getByRole('button', { name: /Add Term/i }))
    // Form opens with title "Add New Term"
    expect(screen.getByText('Add New Term')).toBeInTheDocument()
  })
})
