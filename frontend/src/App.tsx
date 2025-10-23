import { Routes, Route, Link } from 'react-router-dom'
import PaperPage from './pages/PaperPage'
import HomePage from './pages/HomePage'
import GlossaryPage from './pages/GlossaryPage'

function App() {
  return (
    <div style={{ minHeight: '100vh', display: 'flex', flexDirection: 'column' }}>
      {/* Header */}
      <header style={{
        padding: '1rem 1.5rem',
        borderBottom: '1px solid #444',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        backgroundColor: '#1a1a1a'
      }}>
        <h1 style={{ margin: 0, fontSize: '1.25rem', fontWeight: '600', letterSpacing: '-0.02em' }}>
          <Link to="/" style={{ textDecoration: 'none', color: '#e0e0e0' }}>
            Paper Gloss
          </Link>
        </h1>
        <nav style={{ display: 'flex', gap: '0.5rem' }}>
          <Link
            to="/"
            style={{
              padding: '0.5rem 1rem',
              textDecoration: 'none',
              color: '#999',
              fontSize: '0.875rem',
              borderRadius: '6px',
              transition: 'all 0.2s ease'
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.backgroundColor = '#2a2a2a'
              e.currentTarget.style.color = '#e0e0e0'
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.backgroundColor = 'transparent'
              e.currentTarget.style.color = '#999'
            }}
          >
            Home
          </Link>
          <Link
            to="/glossary"
            style={{
              padding: '0.5rem 1rem',
              textDecoration: 'none',
              color: '#999',
              fontSize: '0.875rem',
              borderRadius: '6px',
              transition: 'all 0.2s ease'
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.backgroundColor = '#2a2a2a'
              e.currentTarget.style.color = '#e0e0e0'
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.backgroundColor = 'transparent'
              e.currentTarget.style.color = '#999'
            }}
          >
            Glossary
          </Link>
        </nav>
      </header>

      {/* Main content */}
      <main style={{ flex: 1, padding: '1rem' }}>
        <Routes>
          <Route path="/" element={<HomePage />} />
          <Route path="/papers/:id" element={<PaperPage />} />
          <Route path="/glossary" element={<GlossaryPage />} />
        </Routes>
      </main>

      {/* Footer */}
      <footer style={{
        padding: '1rem 1.5rem',
        borderTop: '1px solid #444',
        textAlign: 'center',
        fontSize: '0.75rem',
        color: '#666',
        backgroundColor: '#1a1a1a'
      }}>
        <p style={{ margin: 0 }}>Paper Gloss - Academic Paper Translation Assistant</p>
      </footer>
    </div>
  )
}

export default App
