import { describe, it, expect } from 'vitest'
import React from 'react'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { http, HttpResponse } from 'msw'
import { server } from '../msw/server'

import TranslationView from '../../src/components/TranslationView'

function renderWithQuery(ui: React.ReactElement) {
  const qc = new QueryClient()
  return render(<QueryClientProvider client={qc}>{ui}</QueryClientProvider>)
}

describe('TranslationView hover -> TermTooltip (MSW)', () => {
  it('shows tooltip with term details on hover', async () => {
    const paperId = 'p-hover'
    const chunkId = 'c1'
    const termId = 't1'
    const text = 'This is neural network test.'
    const start = 8
    const end = start + 'neural network'.length

    // Handlers: translation, occurrences, term detail
    server.use(
      http.get(`/api/papers/${paperId}/translation`, async () => {
        return HttpResponse.json({
          paper_id: paperId,
          // Frontend expects Chunk-like shape in data.chunks
          chunks: [
            {
              id: chunkId,
              paper_id: paperId,
              chunk_index: 0,
              original_text: '',
              translated_text: text,
              content_hash: 'h',
              status: 'translated',
              retry_count: 0,
              created_at: '',
              updated_at: '',
            },
          ],
        })
      }),
      http.get('/api/occurrences', async ({ request }) => {
        const url = new URL(request.url)
        if (url.searchParams.get('paper_id') !== paperId) {
          return HttpResponse.json({ occurrences: [], total: 0 })
        }
        return HttpResponse.json({
          occurrences: [
            {
              id: 'o1',
              term_id: termId,
              paper_id: paperId,
              chunk_id: chunkId,
              start_pos: start,
              end_pos: end,
              created_at: '',
            },
          ],
          total: 1,
        })
      }),
      http.get(`/api/terms/${termId}`, async () => {
        return HttpResponse.json({
          id: termId,
          slug: 'neural-network',
          lemma_en: 'neural network',
          lemma_ja: 'ニューラルネットワーク',
          reading_kana: 'にゅーらるねっとわーく',
          pos: 'noun',
          tags: 'ml,dl',
          note: null,
          variants: [],
          definition: { id: 'd1', term_id: termId, lang: 'ja', text: '多層のニューラルモデル。', provider: 'gpt-4', created_at: '', updated_at: '' },
          created_at: '',
          updated_at: '',
        })
      })
    )

    renderWithQuery(<TranslationView paperId={paperId} />)

    // Wait for highlighted text to render
    const el = await screen.findByText('neural network')
    // Hover to trigger tooltip
    fireEvent.mouseEnter(el, { clientX: 100, clientY: 120 })

    // Tooltip shows Japanese lemma from GET /api/terms/{id}
    await waitFor(() => expect(screen.getByText('ニューラルネットワーク')).toBeInTheDocument())
  })
})

