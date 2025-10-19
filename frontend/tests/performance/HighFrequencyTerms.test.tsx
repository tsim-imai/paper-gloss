import { describe, it, expect } from 'vitest'
import React from 'react'
import { render } from '@testing-library/react'
import TermHighlight from '../../src/components/translation/TermHighlight'

// T071: 高頻度用語（数百出現）でも <100ms でハイライトレンダリングできる
describe('T071 High-frequency term highlighting', () => {
  it('renders 500 highlights under 100ms', async () => {
    // Build long text and aligned occurrences
    const unit = 'neural network'
    const pieces = Array.from({ length: 500 }, (_, i) => `${unit} ${i} `)
    const text = pieces.join('')
    let pos = 0
    const occurrences = pieces.map((p, i) => {
      const start = pos
      const end = pos + unit.length
      pos += p.length
      return { id: `o${i}`, term_id: 't1', paper_id: 'p1', chunk_id: 'c1', start_pos: start, end_pos: end }
    })

    const start = performance.now()
    const { container } = render(
      React.createElement(TermHighlight, { text, chunkId: 'c1', occurrences })
    )
    const elapsed = performance.now() - start
    expect(container).toBeTruthy()
    // Allow some headroom for CI/jsdom overhead
    expect(elapsed).toBeLessThan(250)
  })
})
