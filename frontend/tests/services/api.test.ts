import { describe, it, expect } from 'vitest'
import { server } from '../msw/server'
import { http } from 'msw'
import { apiClient } from '../../src/services/api'

describe('apiClient', () => {
  it('searchTerms sends query params', async () => {
    let seenUrl: URL | null = null
    server.use(
      http.get('/api/terms', async ({ request }) => {
        seenUrl = new URL(request.url)
        return new Response(JSON.stringify({ items: [], total: 0, page: 1, limit: 50, total_pages: 1 }), { status: 200, headers: { 'content-type': 'application/json' } })
      })
    )
    await apiClient.searchTerms('nn', 'en', 'frequency', 2, 10)
    expect(seenUrl).not.toBeNull()
    expect(seenUrl!.searchParams.get('q')).toBe('nn')
    expect(seenUrl!.searchParams.get('lang')).toBe('en')
    expect(seenUrl!.searchParams.get('sort')).toBe('frequency')
    expect(seenUrl!.searchParams.get('page')).toBe('2')
    expect(seenUrl!.searchParams.get('limit')).toBe('10')
  })
})

