import { describe, it, expect, vi } from 'vitest'
import React from 'react'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import GlossaryPanel from '../../src/components/glossary/GlossaryPanel'
import { TermListItem } from '../../src/types'

describe('GlossaryPanel', () => {
  it('renders empty state', () => {
    render(
      <GlossaryPanel terms={[]} isLoading={false} onTermClick={() => {}} onDeleteTerm={() => {}} />
    )
    expect(screen.getByText(/No terms found/i)).toBeInTheDocument()
  })

  it('renders loading state', () => {
    render(
      <GlossaryPanel terms={[]} isLoading={true} onTermClick={() => {}} onDeleteTerm={() => {}} />
    )
    expect(screen.getByText(/Loading glossary/i)).toBeInTheDocument()
  })

  it('click handlers fire for item and delete', async () => {
    const items: TermListItem[] = [
      {
        id: 't1', slug: 'neural-network', lemma_en: 'neural network', lemma_ja: 'ニューラルネットワーク',
        occurrence_count: 2, created_at: '', updated_at: '',
        reading_kana: 'にゅーらる', pos: 'noun', tags: 'ml'
      }
    ]
    const onTermClick = vi.fn()
    const onDeleteTerm = vi.fn()
    // confirm returns true
    vi.stubGlobal('confirm', () => true)

    render(
      <GlossaryPanel terms={items} isLoading={false} onTermClick={onTermClick} onDeleteTerm={onDeleteTerm} />
    )

    await userEvent.click(screen.getByText('ニューラルネットワーク'))
    expect(onTermClick).toHaveBeenCalled()

    await userEvent.click(screen.getByRole('button', { name: /Delete/i }))
    expect(onDeleteTerm).toHaveBeenCalledWith('t1')
  })
})

