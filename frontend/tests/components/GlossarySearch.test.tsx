import { describe, it, expect, vi } from 'vitest'
import React from 'react'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import GlossarySearch from '../../src/components/glossary/GlossarySearch'

describe('T079 GlossarySearch component', () => {
  it('calls onSearch when typing and changing filters', async () => {
    const onSearch = vi.fn()
    render(<GlossarySearch onSearch={onSearch} />)

    // Type query
    const input = screen.getByPlaceholderText(/Search terms/i)
    await userEvent.type(input, 'neural')

    // onSearch called with (query, lang, sort)
    expect(onSearch).toHaveBeenCalled()
    const last = onSearch.mock.calls.at(-1)
    expect(last[0]).toBe('neural')

    // Change language filter
    const selects = screen.getAllByRole('combobox')
    const lang = selects[0]
    await userEvent.selectOptions(lang, 'ja')
    const last2 = onSearch.mock.calls.at(-1)
    expect(last2).toEqual(['neural', 'ja', expect.any(String)])

    // Change sort
    const sort = selects[1]
    await userEvent.selectOptions(sort, 'frequency')
    const last3 = onSearch.mock.calls.at(-1)
    expect(last3).toEqual(['neural', 'ja', 'frequency'])
  })
})
