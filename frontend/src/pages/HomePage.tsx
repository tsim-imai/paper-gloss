import PaperImport from '../components/PaperImport'
import PaperList from '../components/PaperList'

export default function HomePage() {
  return (
    <div style={{ maxWidth: '1200px', margin: '0 auto', padding: '1rem' }}>
      <PaperImport />

      <div style={{ marginTop: '2rem' }}>
        <h2 style={{ color: '#e0e0e0', marginBottom: '1rem' }}>Your Papers</h2>
        <PaperList />
      </div>
    </div>
  )
}
