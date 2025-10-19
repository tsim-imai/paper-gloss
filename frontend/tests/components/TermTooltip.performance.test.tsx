import { describe, it, expect } from 'vitest'
import React from 'react'
import { render } from '@testing-library/react'

// T053: Tooltip初回表示が <100ms であること（SPEC: FR-027）
// 実装がまだ無いので現状は import で失敗し赤になります（TDD）
describe('T053 TermTooltip performance', () => {
  it('renders tooltip within 100ms on first hover', async () => {
    const start = performance.now()
    // @ts-expect-error pending implementation
    const { TermTooltip } = await import('../../src/components/translation/TermTooltip')
    const { getByTestId } = render(
      React.createElement(TermTooltip, { termId: 'dummy', open: true, 'data-testid': 'tooltip' })
    )
    const elapsed = performance.now() - start
    expect(getByTestId('tooltip')).toBeInTheDocument()
    expect(elapsed).toBeLessThan(100)
  })
})

