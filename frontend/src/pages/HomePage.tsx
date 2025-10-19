import PaperImport from '../components/PaperImport'
import PaperList from '../components/PaperList'

export default function HomePage() {
  return (
    <div style={{ maxWidth: '1200px', margin: '0 auto' }}>
      <h2>Import Paper</h2>
      <PaperImport />

      <div style={{ marginTop: '3rem' }}>
        <h2>Your Papers</h2>
        <PaperList />
      </div>
    </div>
  )
}
