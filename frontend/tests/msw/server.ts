import { setupServer } from 'msw/node'

// Start with no default handlers. Individual tests will register per-case handlers via server.use(...).
export const server = setupServer()

