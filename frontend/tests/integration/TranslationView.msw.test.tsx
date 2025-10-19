import { describe, it, expect } from 'vitest'
import React from 'react'
import { render, screen } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { http, HttpResponse } from 'msw'
import { server } from '../msw/server'

import TranslationView from '../../src/components/TranslationView'

function renderWithQuery(ui: React.ReactElement) {
  const qc = new QueryClient()
  return render(<QueryClientProvider client={qc}>{ui}</QueryClientProvider>)
}

describe('TranslationView with MSW', () => {
  it('GET /api/papers/:id/translation returns 200 with empty chunks and shows placeholder', async () => {
    const paperId = 'p123'
    server.use(
      http.get(`/api/papers/${paperId}/translation`, async () => {
        return HttpResponse.json({ paper_id: paperId, chunks: [] }, { status: 200 })
      })
    )
    // Silence occurrences fetch
    server.use(
      http.get('/api/occurrences', async () => {
        return HttpResponse.json([])
      })
    )

    renderWithQuery(<TranslationView paperId={paperId} />)

    expect(
      await screen.findByText(/No translations available yet/i)
    ).toBeInTheDocument()
  })
})
