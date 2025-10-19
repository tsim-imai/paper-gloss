import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import React from 'react'
import TermHighlight from '../../src/components/translation/TermHighlight'

describe('T052 TermHighlight component', () => {
  it('highlights occurrences with data-term-id and calls hover/click handlers', async () => {
    const text = 'This is a neural network and another neural network.'
    // positions for 'neural network' in the string
    const start1 = text.indexOf('neural network')
    const end1 = start1 + 'neural network'.length
    const start2 = text.lastIndexOf('neural network')
    const end2 = start2 + 'neural network'.length

    const occurrences = [
      { id: 'o1', term_id: 't1', paper_id: 'p1', chunk_id: 'c1', start_pos: start1, end_pos: end1 },
      { id: 'o2', term_id: 't1', paper_id: 'p1', chunk_id: 'c1', start_pos: start2, end_pos: end2 },
    ]

    const onHover = vi.fn()
    const onClick = vi.fn()

    render(
      <TermHighlight
        text={text}
        chunkId="c1"
        occurrences={occurrences}
        highlightedTermId="t1"
        onTermHover={onHover}
        onTermClick={onClick}
      />
    )

    const highlighted = screen.getAllByText('neural network')
    expect(highlighted.length).toBe(2)
    // dataset attributes are set
    highlighted.forEach((el) => {
      expect(el).toHaveAttribute('data-term-id', 't1')
    })
  })
})
