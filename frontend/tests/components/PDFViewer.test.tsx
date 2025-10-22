import { describe, it, expect, vi, beforeEach } from 'vitest'
import React from 'react'
import { render, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'

// Create a mock pdfjs object that we can spy on using vi.hoisted
const { mockGlobalWorkerOptions, mockPdfjs } = vi.hoisted(() => {
  const mockGlobalWorkerOptions = { workerSrc: '' }
  const mockPdfjs = {
    version: '3.11.174',
    GlobalWorkerOptions: mockGlobalWorkerOptions
  }
  return { mockGlobalWorkerOptions, mockPdfjs }
})

// Mock react-pdf Document/Page to avoid loading real PDFs
vi.mock('react-pdf', async () => {
  const React = await import('react')
  const Document = ({ onLoadSuccess, children }: any) => {
    React.useEffect(() => {
      onLoadSuccess?.({ numPages: 5 })
    }, [onLoadSuccess])
    return <div data-testid="doc">{children}</div>
  }
  const Page = ({ pageNumber }: any) => <div>Page {pageNumber}</div>
  return { Document, Page, pdfjs: mockPdfjs }
})

import PDFViewer from '../../src/components/PDFViewer'

describe('PDFViewer (mocked)', () => {
  it('sets up PDF.js worker from local public directory on module import', () => {
    // PDFViewer sets workerSrc at module import time (top-level code)
    // This test verifies it's set to the local path, not CDN
    expect(mockGlobalWorkerOptions.workerSrc).toBe('/pdf.worker.min.js')
  })

  it('navigates pages and zoom', async () => {
    render(<PDFViewer fileUrl="/dummy.pdf" />)
    // Wait for load
    expect(await screen.findByText(/Page 1 of 5/)).toBeInTheDocument()

    // Next page button is enabled after load
    const nextBtn = screen.getByRole('button', { name: 'Next' })
    expect(nextBtn).toBeEnabled()

    // Zoom in/out
    await userEvent.click(screen.getByRole('button', { name: '+' }))
    expect(screen.getByText('120%')).toBeInTheDocument()
    await userEvent.click(screen.getByRole('button', { name: '−' }))
    expect(screen.getByText('100%')).toBeInTheDocument()

    // Reset
    await userEvent.click(screen.getByRole('button', { name: 'Reset' }))
    expect(screen.getByText('100%')).toBeInTheDocument()
  })
})
