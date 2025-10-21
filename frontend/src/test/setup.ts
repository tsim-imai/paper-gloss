import { afterEach, beforeAll, afterAll } from 'vitest'
import { cleanup } from '@testing-library/react'
import '@testing-library/jest-dom'

// Cleanup after each test
afterEach(() => {
  cleanup()
})

// Avoid modal alerts blocking tests
// @ts-ignore
globalThis.alert = () => {}

// MSW (only for tests that register handlers)
import { server } from '../../tests/msw/server'

beforeAll(() => server.listen({ onUnhandledRequest: 'warn' }))
afterEach(() => server.resetHandlers())
afterAll(() => server.close())

// Suppress noisy React warnings in tests (act and cross-render setState) while keeping others
const originalError = console.error
// @ts-ignore
console.error = (...args: any[]) => {
  const msg = String(args[0] ?? '')
  if (
    msg.includes('not wrapped in act(...)') ||
    msg.includes('Cannot update a component while rendering a different component')
  ) {
    return
  }
  // @ts-ignore
  originalError(...args)
}
