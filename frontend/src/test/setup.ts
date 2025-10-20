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
