import { describe, it, expect, vi } from 'vitest'
import React from 'react'
import { render, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'

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
  const pdfjs = { version: '4.0.0', GlobalWorkerOptions: { workerSrc: '' } }
  return { Document, Page, pdfjs }
})

import PDFViewer from '../../src/components/PDFViewer'

describe('PDFViewer (mocked)', () => {
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
