import PaperImport from '../components/PaperImport'
import PaperList from '../components/PaperList'

export default function HomePage() {
  return (
    <div style={{ maxWidth: '1200px', margin: '0 auto' }}>
      <PaperImport />

      <div style={{ marginTop: '2rem' }}>
        <PaperList />
      </div>
    </div>
  )
}
