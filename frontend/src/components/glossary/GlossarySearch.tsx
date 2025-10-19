import { useState } from 'react'

interface GlossarySearchProps {
  onSearch: (query: string, lang: string, sort: string) => void
}

/**
 * Bilingual glossary search component with real-time filtering
 */
export default function GlossarySearch({ onSearch }: GlossarySearchProps) {
  const [query, setQuery] = useState('')
  const [lang, setLang] = useState('both')
  const [sort, setSort] = useState('alphabetical')

  const handleSearchChange = (newQuery: string) => {
    setQuery(newQuery)
    onSearch(newQuery, lang, sort)
  }

  const handleLangChange = (newLang: string) => {
    setLang(newLang)
    onSearch(query, newLang, sort)
  }

  const handleSortChange = (newSort: string) => {
    setSort(newSort)
    onSearch(query, lang, newSort)
  }

  return (
    <div style={{
      padding: '1rem',
      backgroundColor: '#f9f9f9',
      borderRadius: '8px',
      marginBottom: '1rem',
    }}>
      {/* Search input */}
      <div style={{ marginBottom: '1rem' }}>
        <input
          type="text"
          value={query}
          onChange={(e) => handleSearchChange(e.target.value)}
          placeholder="Search terms in English or Japanese..."
          style={{
            width: '100%',
            padding: '0.75rem',
            fontSize: '1rem',
            border: '1px solid #ccc',
            borderRadius: '4px',
          }}
        />
      </div>

      {/* Filters */}
      <div style={{
        display: 'flex',
        gap: '1rem',
        alignItems: 'center',
        flexWrap: 'wrap',
      }}>
        {/* Language filter */}
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
          <label style={{ fontSize: '0.875rem', fontWeight: 'bold', color: '#666' }}>
            Language:
          </label>
          <select
            value={lang}
            onChange={(e) => handleLangChange(e.target.value)}
            style={{
              padding: '0.5rem',
              fontSize: '0.875rem',
              border: '1px solid #ccc',
              borderRadius: '4px',
              cursor: 'pointer',
            }}
          >
            <option value="both">Both</option>
            <option value="en">English</option>
            <option value="ja">Japanese</option>
          </select>
        </div>

        {/* Sort order */}
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
          <label style={{ fontSize: '0.875rem', fontWeight: 'bold', color: '#666' }}>
            Sort by:
          </label>
          <select
            value={sort}
            onChange={(e) => handleSortChange(e.target.value)}
            style={{
              padding: '0.5rem',
              fontSize: '0.875rem',
              border: '1px solid #ccc',
              borderRadius: '4px',
              cursor: 'pointer',
            }}
          >
            <option value="alphabetical">Alphabetical</option>
            <option value="frequency">Frequency</option>
          </select>
        </div>
      </div>
    </div>
  )
}
