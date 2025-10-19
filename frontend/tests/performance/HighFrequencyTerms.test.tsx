import { describe, it, expect } from 'vitest'
import React from 'react'
import { render } from '@testing-library/react'

// T071: 高頻度用語（数百出現）でも <100ms でハイライトレンダリングできる
// 実装がまだ無いので import 段階で赤になります（TDD）
describe('T071 High-frequency term highlighting', () => {
  it('renders 500 highlights under 100ms', async () => {
    const start = performance.now()
    // @ts-expect-error pending implementation
    const { TermHighlight } = await import('../../src/components/translation/TermHighlight')
    const text = Array.from({ length: 500 }, (_, i) => `neural network ${i}`).join(' ')
    const { container } = render(React.createElement(TermHighlight, { text, terms: ['neural network'] }))
    const elapsed = performance.now() - start
    expect(container).toBeTruthy()
    expect(elapsed).toBeLessThan(100)
  })
})

