import { describe, it } from 'vitest'

// T026 [US1] TranslationView must exist (component-level)
describe('T026 TranslationView component', () => {
  it('is importable', async () => {
    await import('../../src/components/TranslationView')
  })
})
