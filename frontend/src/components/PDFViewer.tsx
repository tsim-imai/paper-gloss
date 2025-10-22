import { useState } from 'react'
import { Document, Page, pdfjs } from 'react-pdf'
import 'react-pdf/dist/esm/Page/AnnotationLayer.css'
import 'react-pdf/dist/esm/Page/TextLayer.css'

// Set up PDF.js worker - use local worker file from public directory
pdfjs.GlobalWorkerOptions.workerSrc = '/pdf.worker.min.js'

interface PDFViewerProps {
  fileUrl: string
}

export default function PDFViewer({ fileUrl }: PDFViewerProps) {
  const [numPages, setNumPages] = useState<number | null>(null)
  const [pageNumber, setPageNumber] = useState(1)
  const [scale, setScale] = useState(1.0)

  function onDocumentLoadSuccess({ numPages }: { numPages: number }) {
    setNumPages(numPages)
    setPageNumber(1)
  }

  function onDocumentLoadError(error: Error) {
    console.error('Error loading PDF:', error)
  }

  const changePage = (offset: number) => {
    setPageNumber((prevPageNumber) => {
      const newPage = prevPageNumber + offset
      return Math.min(Math.max(1, newPage), numPages || 1)
    })
  }

  const previousPage = () => changePage(-1)
  const nextPage = () => changePage(1)

  const zoomIn = () => setScale((prevScale) => Math.min(prevScale + 0.2, 2.0))
  const zoomOut = () => setScale((prevScale) => Math.max(prevScale - 0.2, 0.5))
  const resetZoom = () => setScale(1.0)

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      {/* Controls */}
      <div style={{
        padding: '1rem',
        backgroundColor: '#f9f9f9',
        borderBottom: '1px solid #ccc',
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        flexWrap: 'wrap',
        gap: '1rem'
      }}>
        {/* Page navigation */}
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
          <button
            onClick={previousPage}
            disabled={pageNumber <= 1}
            style={{
              padding: '0.5rem 1rem',
              backgroundColor: pageNumber <= 1 ? '#ccc' : '#646cff',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: pageNumber <= 1 ? 'not-allowed' : 'pointer',
            }}
          >
            Previous
          </button>
          <span style={{ fontSize: '0.875rem' }}>
            Page {pageNumber} of {numPages || '?'}
          </span>
          <button
            onClick={nextPage}
            disabled={!numPages || pageNumber >= numPages}
            style={{
              padding: '0.5rem 1rem',
              backgroundColor: !numPages || pageNumber >= numPages ? '#ccc' : '#646cff',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: !numPages || pageNumber >= numPages ? 'not-allowed' : 'pointer',
            }}
          >
            Next
          </button>
        </div>

        {/* Zoom controls */}
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
          <button
            onClick={zoomOut}
            disabled={scale <= 0.5}
            style={{
              padding: '0.5rem 1rem',
              backgroundColor: scale <= 0.5 ? '#ccc' : '#646cff',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: scale <= 0.5 ? 'not-allowed' : 'pointer',
            }}
          >
            −
          </button>
          <span style={{ fontSize: '0.875rem', minWidth: '60px', textAlign: 'center' }}>
            {Math.round(scale * 100)}%
          </span>
          <button
            onClick={zoomIn}
            disabled={scale >= 2.0}
            style={{
              padding: '0.5rem 1rem',
              backgroundColor: scale >= 2.0 ? '#ccc' : '#646cff',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: scale >= 2.0 ? 'not-allowed' : 'pointer',
            }}
          >
            +
          </button>
          <button
            onClick={resetZoom}
            style={{
              padding: '0.5rem 1rem',
              backgroundColor: '#646cff',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
            }}
          >
            Reset
          </button>
        </div>
      </div>

      {/* PDF viewer */}
      <div style={{
        flex: 1,
        overflow: 'auto',
        backgroundColor: '#525659',
        display: 'flex',
        justifyContent: 'center',
        padding: '2rem'
      }}>
        <Document
          file={fileUrl}
          onLoadSuccess={onDocumentLoadSuccess}
          onLoadError={onDocumentLoadError}
          loading={
            <div style={{ color: 'white', textAlign: 'center' }}>
              Loading PDF...
            </div>
          }
          error={
            <div style={{
              color: 'white',
              textAlign: 'center',
              padding: '2rem',
              backgroundColor: 'rgba(244, 67, 54, 0.2)',
              borderRadius: '8px'
            }}>
              Failed to load PDF. Please check if the file exists.
            </div>
          }
        >
          <Page
            pageNumber={pageNumber}
            scale={scale}
            renderTextLayer={true}
            renderAnnotationLayer={true}
            loading={
              <div style={{ color: 'white' }}>
                Loading page...
              </div>
            }
          />
        </Document>
      </div>
    </div>
  )
}
