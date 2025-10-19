import { describe, it, expect, vi } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import React from 'react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'

vi.mock('../../src/services/api', () => {
  return {
    apiClient: {
      getTerm: vi.fn().mockResolvedValue({
        data: {
          id: 't1', slug: 'neural-network', lemma_en: 'neural network', lemma_ja: 'ニューラルネットワーク',
          reading_kana: 'にゅーらるねっとわーく', pos: 'noun', tags: 'ml,dl', note: null,
          variants: [{ id: 'v1', term_id: 't1', lang: 'en', surface: 'neural-network' }],
          definition: { id: 'd1', term_id: 't1', lang: 'ja', text: '多層のニューラルモデル。', provider: 'gpt-4', created_at: '', updated_at: '' },
          created_at: '', updated_at: ''
        }
      })
    }
  }
})

import TermTooltip from '../../src/components/translation/TermTooltip'

function renderWithQuery(ui: React.ReactElement) {
  const qc = new QueryClient()
  return render(<QueryClientProvider client={qc}>{ui}</QueryClientProvider>)
}

describe('T053 TermTooltip component', () => {
  it('loads and displays term details quickly', async () => {
    const start = performance.now()
    renderWithQuery(
      <TermTooltip termId="t1" position={{ x: 100, y: 100 }} />
    )
    await waitFor(() => expect(screen.getByText('ニューラルネットワーク')).toBeInTheDocument())
    const elapsed = performance.now() - start
    expect(elapsed).toBeLessThan(100)
  })
})
