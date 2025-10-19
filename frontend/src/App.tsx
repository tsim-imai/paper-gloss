import { Routes, Route, Link } from 'react-router-dom'
import PaperPage from './pages/PaperPage'
import HomePage from './pages/HomePage'
import GlossaryPage from './pages/GlossaryPage'

function App() {
  return (
    <div style={{ minHeight: '100vh', display: 'flex', flexDirection: 'column' }}>
      {/* Header */}
      <header style={{
        padding: '1rem 2rem',
        borderBottom: '1px solid #ccc',
        display: 'flex',
        alignItems: 'center',
        gap: '2rem'
      }}>
        <h1 style={{ margin: 0, fontSize: '1.5rem' }}>
          <Link to="/" style={{ textDecoration: 'none', color: 'inherit' }}>
            Paper Gloss
          </Link>
        </h1>
        <nav>
          <Link to="/" style={{ marginRight: '1rem' }}>Home</Link>
          <Link to="/glossary" style={{ marginRight: '1rem' }}>Glossary</Link>
        </nav>
      </header>

      {/* Main content */}
      <main style={{ flex: 1, padding: '2rem' }}>
        <Routes>
          <Route path="/" element={<HomePage />} />
          <Route path="/papers/:id" element={<PaperPage />} />
          <Route path="/glossary" element={<GlossaryPage />} />
        </Routes>
      </main>

      {/* Footer */}
      <footer style={{
        padding: '1rem 2rem',
        borderTop: '1px solid #ccc',
        textAlign: 'center',
        fontSize: '0.875rem'
      }}>
        <p>Paper Gloss - Academic Paper Translation Assistant</p>
      </footer>
    </div>
  )
}

export default App
