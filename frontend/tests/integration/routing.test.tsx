import { describe, it, expect } from 'vitest'
import React from 'react'
import { render, screen } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { http, HttpResponse } from 'msw'
import { server } from '../msw/server'
import App from '../../src/App'

function renderApp(path: string) {
  const qc = new QueryClient()
  return render(
    <MemoryRouter initialEntries={[path]}>
      <QueryClientProvider client={qc}>
        <App />
      </QueryClientProvider>
    </MemoryRouter>
  )
}

describe('App routing', () => {
  it('navigates to /glossary', async () => {
    server.use(
      http.get('/api/terms', async () => {
        return HttpResponse.json({ items: [], total: 0, page: 1, limit: 50, total_pages: 1 })
      })
    )
    renderApp('/glossary')
    expect(await screen.findByRole('heading', { name: 'Glossary' })).toBeInTheDocument()
  })
})
