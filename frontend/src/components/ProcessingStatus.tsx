import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { apiClient } from '../services/api'
import { Paper } from '../types'

interface ProcessingStatusProps {
  paper: Paper
}

export default function ProcessingStatus({ paper }: ProcessingStatusProps) {
  const queryClient = useQueryClient()

  // Poll for processing progress if paper is processing
  const { data: progressResponse } = useQuery({
    queryKey: ['paper-progress', paper.id],
    queryFn: async () => {
      const response = await apiClient.getPaperStatus(paper.id)
      return response.data
    },
    enabled: paper.status === 'processing',
    refetchInterval: paper.status === 'processing' ? 3000 : false, // Poll every 3 seconds
  })

  const progress = progressResponse?.progress?.translation

  const processMutation = useMutation({
    mutationFn: () => apiClient.processPaper(paper.id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['papers'] })
      queryClient.invalidateQueries({ queryKey: ['paper', paper.id] })
      queryClient.invalidateQueries({ queryKey: ['paper-progress', paper.id] })
    },
  })

  const getStatusColor = () => {
    switch (paper.status) {
      case 'completed':
        return '#4caf50'
      case 'processing':
        return '#ff9800'
      case 'failed':
        return '#f44336'
      default:
        return '#9e9e9e'
    }
  }

  const getProgressPercentage = () => {
    const total = progress?.total_chunks || paper.total_chunks || 0
    const completed = (progress as any)?.completed_chunks ?? paper.translated_chunks ?? 0
    if (total === 0) return 0
    return Math.round((completed / total) * 100)
  }

  const canProcess = paper.status === 'pending' || paper.status === 'failed'
  const isProcessing = paper.status === 'processing'

  return (
    <div style={{
      border: '1px solid #ccc',
      borderRadius: '8px',
      padding: '1.5rem',
      backgroundColor: '#f9f9f9'
    }}>
      <h3 style={{ marginTop: 0, marginBottom: '1rem' }}>Processing Status</h3>

      {/* Status badge */}
      <div style={{ marginBottom: '1rem' }}>
        <span
          style={{
            padding: '0.5rem 1rem',
            borderRadius: '4px',
            backgroundColor: getStatusColor(),
            color: 'white',
            fontWeight: 'bold',
            textTransform: 'uppercase',
            fontSize: '0.875rem',
          }}
        >
          {paper.status}
        </span>
      </div>

      {/* Progress bar for processing */}
      {(isProcessing || paper.status === 'completed') && (
        <div style={{ marginBottom: '1rem' }}>
          <div style={{
            display: 'flex',
            justifyContent: 'space-between',
            marginBottom: '0.5rem',
            fontSize: '0.875rem'
          }}>
            <span>Translation Progress</span>
            <span>{getProgressPercentage()}%</span>
          </div>
          <div style={{
            width: '100%',
            height: '20px',
            backgroundColor: '#e0e0e0',
            borderRadius: '10px',
            overflow: 'hidden'
          }}>
            <div
              style={{
                width: `${getProgressPercentage()}%`,
                height: '100%',
                backgroundColor: getStatusColor(),
                transition: 'width 0.3s ease',
              }}
            />
          </div>
        </div>
      )}

      {/* Details */}
      <div style={{ marginBottom: '1rem', fontSize: '0.875rem', color: '#666' }}>
        {progress || (paper.total_chunks !== undefined) ? (
          <>
            <div>Total Chunks: {progress?.total_chunks ?? paper.total_chunks ?? 0}</div>
            <div>Translated: {(progress as any)?.completed_chunks ?? paper.translated_chunks ?? 0}</div>
            {(progress?.failed_chunks || paper.failed_chunks) ? (
              <div style={{ color: '#f44336' }}>
                Failed: {progress?.failed_chunks ?? paper.failed_chunks}
              </div>
            ) : null}
            {progress?.extracted_terms !== undefined && (
              <div>Terms Extracted: {progress.extracted_terms}</div>
            )}
          </>
        ) : (
          <div>No processing information available</div>
        )}
      </div>

      {/* Action button */}
      {canProcess && (
        <button
          onClick={() => processMutation.mutate()}
          disabled={processMutation.isPending}
          style={{
            padding: '0.75rem 1.5rem',
            backgroundColor: processMutation.isPending ? '#ccc' : '#646cff',
            color: 'white',
            border: 'none',
            borderRadius: '4px',
            cursor: processMutation.isPending ? 'not-allowed' : 'pointer',
            fontSize: '1rem',
            fontWeight: 'bold',
          }}
        >
          {processMutation.isPending ? 'Starting...' : 'Start Processing'}
        </button>
      )}

      {isProcessing && (
        <div style={{
          padding: '0.75rem',
          backgroundColor: '#fff3cd',
          color: '#856404',
          borderRadius: '4px',
          fontSize: '0.875rem'
        }}>
          Processing in progress... This page will update automatically.
        </div>
      )}

      {paper.status === 'failed' && (
        <div style={{
          padding: '0.75rem',
          backgroundColor: '#ffebee',
          color: '#c62828',
          borderRadius: '4px',
          fontSize: '0.875rem',
          marginBottom: '1rem'
        }}>
          Processing failed. You can retry by clicking the button above.
        </div>
      )}
    </div>
  )
}
