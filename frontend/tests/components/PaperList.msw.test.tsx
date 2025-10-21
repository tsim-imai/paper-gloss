import { describe, it, expect } from 'vitest'
import React from 'react'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { http, HttpResponse } from 'msw'
import { server } from '../msw/server'
import PaperList from '../../src/components/PaperList'

function renderWithProviders(ui: React.ReactElement) {
  const qc = new QueryClient()
  return render(
    <MemoryRouter>
      <QueryClientProvider client={qc}>{ui}</QueryClientProvider>
    </MemoryRouter>
  )
}

describe('PaperList with MSW', () => {
  it('renders papers and supports filtering', async () => {
    // First call: no status filter, two items
    server.use(
      http.get('/api/papers', async ({ request }) => {
        const url = new URL(request.url)
        const status = url.searchParams.get('status')
        if (!status) {
          return HttpResponse.json({
            items: [
              { id: 'p1', title: 'Paper 1', source: 'upload', status: 'completed', file_path: '', created_at: new Date().toISOString(), updated_at: new Date().toISOString(), total_chunks: 10, translated_chunks: 10 },
              { id: 'p2', title: 'Paper 2', source: 'url', status: 'processing', file_path: '', created_at: new Date().toISOString(), updated_at: new Date().toISOString(), total_chunks: 5, translated_chunks: 2 },
            ], total: 2, page: 1, limit: 10, total_pages: 1
          })
        }
        // When filtered status=completed
        if (status === 'completed') {
          return HttpResponse.json({
            items: [
              { id: 'p1', title: 'Paper 1', source: 'upload', status: 'completed', file_path: '', created_at: new Date().toISOString(), updated_at: new Date().toISOString(), total_chunks: 10, translated_chunks: 10 },
            ], total: 1, page: 1, limit: 10, total_pages: 1
          })
        }
        return HttpResponse.json({ items: [], total: 0, page: 1, limit: 10, total_pages: 1 })
      })
    )

    renderWithProviders(<PaperList />)

    // shows items
    expect(await screen.findByText('Paper 1')).toBeInTheDocument()
    expect(screen.getByText('Paper 2')).toBeInTheDocument()

    // filter to completed
    await userEvent.click(screen.getByRole('button', { name: 'Completed' }))
    expect(await screen.findByText('Paper 1')).toBeInTheDocument()
    // Paper 2 filtered out
    expect(screen.queryByText('Paper 2')).toBeNull()
  })
})

