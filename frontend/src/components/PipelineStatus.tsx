import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { apiClient } from '../services/api'
import { Paper, ProcessingStatusResponse } from '../types'
import PipelineCard from './PipelineCard'

interface PipelineStatusProps {
  paper: Paper
}

export default function PipelineStatus({ paper }: PipelineStatusProps) {
  const queryClient = useQueryClient()

  // Poll for processing progress
  const { data: statusData } = useQuery({
    queryKey: ['paper-status', paper.id],
    queryFn: async () => {
      const response = await apiClient.getPaperStatus(paper.id)
      return response.data as ProcessingStatusResponse
    },
    refetchInterval: 3000, // Poll every 3 seconds
  })

  // Pipeline mutations
  const translateMutation = useMutation({
    mutationFn: () => apiClient.translatePaper(paper.id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['papers'] })
      queryClient.invalidateQueries({ queryKey: ['paper', paper.id] })
      queryClient.invalidateQueries({ queryKey: ['paper-status', paper.id] })
    },
    onError: (error: any) => {
      alert(`Translation failed: ${error.response?.data?.message || error.message}`)
    },
  })

  const extractTermsMutation = useMutation({
    mutationFn: () => apiClient.extractTermsJp(paper.id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['paper-status', paper.id] })
    },
    onError: (error: any) => {
      alert(`Term extraction failed: ${error.response?.data?.message || error.message}`)
    },
  })

  const scanJpMutation = useMutation({
    mutationFn: () => apiClient.scanJp(paper.id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['paper-status', paper.id] })
    },
    onError: (error: any) => {
      alert(`JP scan failed: ${error.response?.data?.message || error.message}`)
    },
  })

  const generateDefsMutation = useMutation({
    mutationFn: () => apiClient.generateDefinitions(paper.id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['paper-status', paper.id] })
    },
    onError: (error: any) => {
      alert(`Definition generation failed: ${error.response?.data?.message || error.message}`)
    },
  })

  const getButtonStyle = (isDisabled: boolean) => ({
    padding: '0.375rem 0.75rem',
    fontSize: '0.75rem',
    backgroundColor: isDisabled ? '#444' : '#646cff',
    color: isDisabled ? '#999' : 'white',
    border: 'none',
    borderRadius: '4px',
    cursor: isDisabled ? 'not-allowed' : 'pointer',
    opacity: isDisabled ? 0.5 : 1,
  })

  return (
    <div>
      {/* Pipelines in horizontal layout */}
      <div style={{ display: 'flex', gap: '1rem', marginBottom: '1rem' }}>
        {/* A: Translation */}
        <PipelineCard
          title="Translation"
          pipelineId="A"
          status={statusData?.translation.status || 'idle'}
          description="Translate English text to Japanese using LLM"
          lastRunAt={paper.created_at}
          actions={
            <button
              onClick={() => translateMutation.mutate()}
              disabled={translateMutation.isPending || statusData?.translation.status === 'processing'}
              style={getButtonStyle(translateMutation.isPending || statusData?.translation.status === 'processing')}
            >
              {translateMutation.isPending ? 'Starting...' : '▶ Translate'}
            </button>
          }
        >
          <div style={{ fontSize: '0.875rem' }}>
            {statusData ? (
              <>
                <div>
                  Total Chunks: <strong>{statusData.translation.total_chunks}</strong>
                </div>
                <div>
                  Completed: <strong style={{ color: '#4caf50' }}>{statusData.translation.completed_chunks}</strong>
                </div>
                {statusData.translation.failed_chunks > 0 && (
                  <div>
                    Failed: <strong style={{ color: '#f44336' }}>{statusData.translation.failed_chunks}</strong>
                  </div>
                )}
                {statusData.translation.status === 'processing' && (
                  <div style={{ marginTop: '0.5rem' }}>
                    <div
                      style={{
                        width: '100%',
                        height: '8px',
                        backgroundColor: '#444',
                        borderRadius: '4px',
                        overflow: 'hidden',
                      }}
                    >
                      <div
                        style={{
                          width: `${Math.round((statusData.translation.completed_chunks / statusData.translation.total_chunks) * 100)}%`,
                          height: '100%',
                          backgroundColor: '#2196f3',
                          transition: 'width 0.3s ease',
                        }}
                      />
                    </div>
                  </div>
                )}
              </>
            ) : (
              <div style={{ color: '#999' }}>No translation data available</div>
            )}
          </div>
        </PipelineCard>

        {/* B: Extract Terms (JP) */}
        <PipelineCard
          title="Extract Terms"
          pipelineId="B"
          status={statusData?.terms_jp.status || 'idle'}
          description="Extract Japanese technical terms from translations"
          lastRunAt={statusData?.terms_jp.last_run_at}
          actions={
            <button
              onClick={() => extractTermsMutation.mutate()}
              disabled={
                extractTermsMutation.isPending ||
                statusData?.terms_jp.status === 'processing' ||
                statusData?.translation.status !== 'completed'
              }
              style={getButtonStyle(
                extractTermsMutation.isPending ||
                  statusData?.terms_jp.status === 'processing' ||
                  statusData?.translation.status !== 'completed'
              )}
            >
              {extractTermsMutation.isPending ? 'Extracting...' : '▶ Extract'}
            </button>
          }
        >
          <div style={{ fontSize: '0.875rem' }}>
            {statusData?.terms_jp ? (
              <>
                <div>
                  Terms: <strong style={{ color: '#4caf50' }}>{statusData.terms_jp.extracted_terms_count}</strong>
                </div>
                {statusData.translation.status !== 'completed' && (
                  <div style={{ marginTop: '0.5rem', color: '#ff9800', fontSize: '0.75rem' }}>
                    ⚠ Requires A first
                  </div>
                )}
              </>
            ) : (
              <div style={{ color: '#999' }}>Not started</div>
            )}
          </div>
        </PipelineCard>

        {/* C: Scan JP Occurrences */}
        <PipelineCard
          title="Scan Occurrences"
          pipelineId="C"
          status={statusData?.scan_jp.status || 'idle'}
          description="Scan translations for term occurrences"
          lastRunAt={statusData?.scan_jp.last_run_at}
          actions={
            <button
              onClick={() => scanJpMutation.mutate()}
              disabled={
                scanJpMutation.isPending ||
                statusData?.scan_jp.status === 'processing' ||
                statusData?.translation.status !== 'completed'
              }
              style={getButtonStyle(
                scanJpMutation.isPending ||
                  statusData?.scan_jp.status === 'processing' ||
                  statusData?.translation.status !== 'completed'
              )}
            >
              {scanJpMutation.isPending ? 'Scanning...' : '▶ Scan'}
            </button>
          }
        >
          <div style={{ fontSize: '0.875rem' }}>
            {statusData?.scan_jp ? (
              <>
                <div>
                  Occurrences: <strong style={{ color: '#4caf50' }}>{statusData.scan_jp.total_occurrences}</strong>
                </div>
                {statusData.translation.status !== 'completed' && (
                  <div style={{ marginTop: '0.5rem', color: '#ff9800', fontSize: '0.75rem' }}>
                    ⚠ Requires A first
                  </div>
                )}
              </>
            ) : (
              <div style={{ color: '#999' }}>Not started</div>
            )}
          </div>
        </PipelineCard>

        {/* D: Generate Definitions */}
        <PipelineCard
          title="Generate Definitions"
          pipelineId="D"
          status={statusData?.definitions.status || 'idle'}
          resultState={statusData?.definitions.result_state}
          description="Generate definitions for terms using LLM"
          lastRunAt={statusData?.definitions.last_run_at}
          actions={
            <button
              onClick={() => generateDefsMutation.mutate()}
              disabled={generateDefsMutation.isPending || statusData?.definitions.status === 'processing'}
              style={getButtonStyle(generateDefsMutation.isPending || statusData?.definitions.status === 'processing')}
            >
              {generateDefsMutation.isPending ? 'Generating...' : '▶ Generate'}
            </button>
          }
        >
          <div style={{ fontSize: '0.875rem' }}>
            {statusData?.definitions ? (
              <>
                <div>
                  Generated: <strong style={{ color: '#4caf50' }}>{statusData.definitions.generated}</strong>
                </div>
                {statusData.definitions.failed > 0 && (
                  <div>
                    Failed: <strong style={{ color: '#f44336' }}>{statusData.definitions.failed}</strong>
                  </div>
                )}
                {statusData.definitions.result_state === 'completed_empty' && (
                  <div style={{ marginTop: '0.5rem', color: '#ff9800', fontSize: '0.75rem' }}>
                    ⚠ No definitions generated
                  </div>
                )}
              </>
            ) : (
              <div style={{ color: '#999' }}>Not started</div>
            )}
          </div>
        </PipelineCard>
      </div>
    </div>
  )
}
