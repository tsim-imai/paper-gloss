import { describe, it, expect, vi } from 'vitest'
import React from 'react'
import { render, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { http, HttpResponse } from 'msw'
import { server } from '../msw/server'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import TermMerge from '../../src/components/glossary/TermMerge'

function renderWithQuery(ui: React.ReactElement) {
  const qc = new QueryClient()
  return render(<QueryClientProvider client={qc}>{ui}</QueryClientProvider>)
}

describe('T080 TermMerge component', () => {
  it('lists duplicates and merges with confirmation', async () => {
    // Stub confirm
    vi.stubGlobal('confirm', () => true)

    // Mock duplicates
    const pair = {
      term1: { id: 't1', lemma_en: 'neural network', lemma_ja: 'ニューラルネットワーク' },
      term2: { id: 't2', lemma_en: 'neural-network', lemma_ja: 'ニューラル ネットワーク' },
      similarity_score: 1.0,
      reason: 'Exact normalized match',
    }
    let mergeCalled: any = null
    server.use(
      http.get('/api/terms/duplicates', async () => {
        return HttpResponse.json({ duplicates: [pair], total: 1 })
      }),
      http.post('/api/terms/merge', async ({ request }) => {
        mergeCalled = await request.json()
        return HttpResponse.json({ ok: true }, { status: 200 })
      })
    )

    renderWithQuery(<TermMerge onClose={() => {}} />)

    // Duplicate pair appears
    const row = await screen.findByText(/ニューラルネットワーク/)
    expect(row).toBeInTheDocument()

    // Select pair
    await userEvent.click(row)

    // Keep default (keep term1), click merge
    const mergeBtn = screen.getByRole('button', { name: /Merge Terms/i })
    await userEvent.click(mergeBtn)

    // API called with source_id = term2, target_id = term1
    expect(mergeCalled).toMatchObject({
      source_id: 't2',
      target_id: 't1',
      confirmed: true,
    })
  })
})
