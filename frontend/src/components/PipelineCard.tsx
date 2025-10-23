import { ReactNode } from 'react'
import { PipelineStatus, PipelineResultState } from '../types'

interface PipelineCardProps {
  title: string
  pipelineId: string
  status: PipelineStatus
  resultState?: PipelineResultState
  lastRunAt?: string
  description: string
  children: ReactNode
  actions?: ReactNode
}

export default function PipelineCard({
  title,
  pipelineId,
  status,
  resultState,
  lastRunAt,
  description,
  children,
  actions,
}: PipelineCardProps) {
  const getStatusColor = () => {
    switch (status) {
      case 'completed':
        return resultState === 'completed_empty' ? '#ff9800' : '#4caf50'
      case 'processing':
        return '#2196f3'
      case 'failed':
        return '#f44336'
      case 'idle':
      default:
        return '#666'
    }
  }

  const getStatusIcon = () => {
    switch (status) {
      case 'completed':
        return resultState === 'completed_empty' ? '⚠' : '✓'
      case 'processing':
        return '⟳'
      case 'failed':
        return '✗'
      case 'idle':
      default:
        return '○'
    }
  }

  const getStatusText = () => {
    if (status === 'completed' && resultState) {
      switch (resultState) {
        case 'completed_nonempty':
          return 'Completed'
        case 'completed_empty':
          return 'Completed (No Results)'
        case 'failed':
          return 'Failed'
      }
    }
    return status.charAt(0).toUpperCase() + status.slice(1)
  }

  const formatDate = (dateStr?: string) => {
    if (!dateStr) return 'Never'
    try {
      const date = new Date(dateStr)
      return date.toLocaleString('ja-JP', {
        year: 'numeric',
        month: '2-digit',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit',
      })
    } catch {
      return dateStr
    }
  }

  return (
    <div
      style={{
        border: '1px solid #444',
        borderRadius: '8px',
        padding: '1rem',
        backgroundColor: '#2a2a2a',
        flex: 1,
        minWidth: 0,
      }}
    >
      {/* Header */}
      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'start',
          marginBottom: '0.75rem',
        }}
      >
        <div style={{ flex: 1 }}>
          <h3 style={{ margin: 0, fontSize: '1rem', color: '#e0e0e0' }}>{title}</h3>
          <p style={{ margin: '0.25rem 0 0 0', fontSize: '0.75rem', color: '#999' }}>{description}</p>
        </div>
        <span
          style={{
            fontSize: '1.25rem',
            fontWeight: 'bold',
            color: '#999',
            fontFamily: 'monospace',
            minWidth: '2rem',
            textAlign: 'center',
          }}
        >
          {pipelineId}
        </span>
      </div>

      {/* Content */}
      <div style={{ marginBottom: '0.75rem', color: '#ccc', minHeight: '3rem' }}>{children}</div>

      {/* Status and Last Run */}
      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          marginBottom: '0.75rem',
        }}
      >
        <div style={{ fontSize: '0.75rem', color: '#999' }}>
          Last run: <span style={{ color: '#ccc' }}>{formatDate(lastRunAt)}</span>
        </div>
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '0.5rem',
            padding: '0.25rem 0.75rem',
            borderRadius: '4px',
            backgroundColor: getStatusColor(),
            color: 'white',
            fontSize: '0.75rem',
            fontWeight: 'bold',
            whiteSpace: 'nowrap',
          }}
        >
          <span>{getStatusIcon()}</span>
          <span>{getStatusText()}</span>
        </div>
      </div>

      {/* Footer - Actions */}
      <div
        style={{
          display: 'flex',
          justifyContent: 'flex-end',
          alignItems: 'center',
          paddingTop: '0.75rem',
          borderTop: '1px solid #444',
        }}
      >
        <div style={{ display: 'flex', gap: '0.5rem' }}>{actions}</div>
      </div>
    </div>
  )
}
