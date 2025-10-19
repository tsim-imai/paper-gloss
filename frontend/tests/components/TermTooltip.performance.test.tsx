import { describe, it, expect, vi } from 'vitest'
import React from 'react'
import { render, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'

vi.mock('../../src/services/api', () => {
  return {
    apiClient: {
      getTerm: vi.fn().mockResolvedValue({
        data: {
          id: 't1', slug: 'neural-network', lemma_en: 'neural network', lemma_ja: 'ニューラルネットワーク',
          reading_kana: 'にゅーらるねっとわーく', pos: 'noun', tags: 'ml,dl', note: null,
          variants: [], definition: { id: 'd1', term_id: 't1', lang: 'ja', text: '定義', provider: 'gpt-4', created_at: '', updated_at: '' },
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

// T053: Tooltip初回表示が <100ms であること（SPEC: FR-027）
describe('T053 TermTooltip performance', () => {
  it('renders tooltip within 100ms on first hover', async () => {
    const start = performance.now()
    const { getByText } = renderWithQuery(
      React.createElement(TermTooltip, { termId: 't1', position: { x: 0, y: 0 } })
    )
    await waitFor(() => expect(getByText('ニューラルネットワーク')).toBeInTheDocument())
    const elapsed = performance.now() - start
    expect(elapsed).toBeLessThan(100)
  })
})
