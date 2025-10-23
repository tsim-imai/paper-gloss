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

  const progress = statusData?.progress

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

  // Run all pipelines sequentially
  const runAllMutation = useMutation({
    mutationFn: async () => {
      await apiClient.translatePaper(paper.id)
      // Wait a bit for translation to start
      await new Promise((resolve) => setTimeout(resolve, 1000))
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['papers'] })
      queryClient.invalidateQueries({ queryKey: ['paper', paper.id] })
      queryClient.invalidateQueries({ queryKey: ['paper-status', paper.id] })
    },
    onError: (error: any) => {
      alert(`Failed to start pipelines: ${error.response?.data?.message || error.message}`)
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
      {/* Quick Actions */}
      <div
        style={{
          marginBottom: '1rem',
          padding: '0.75rem',
          backgroundColor: '#2a2a2a',
          borderRadius: '8px',
          border: '1px solid #444',
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
          <span style={{ fontSize: '0.875rem', fontWeight: 'bold', color: '#e0e0e0' }}>Quick Actions:</span>
          <button
            onClick={() => runAllMutation.mutate()}
            disabled={runAllMutation.isPending || progress?.translation.status === 'processing'}
            style={getButtonStyle(runAllMutation.isPending || progress?.translation.status === 'processing')}
          >
            {runAllMutation.isPending ? 'Starting...' : '▶ Start Translation'}
          </button>
          <span style={{ fontSize: '0.75rem', color: '#999' }}>
            (Other pipelines can be run after translation completes)
          </span>
        </div>
      </div>

      {/* Pipeline A: Translation */}
      <PipelineCard
        title="Pipeline A: Translation"
        pipelineId="translate"
        status={progress?.translation.status || 'idle'}
        description="Translate English text to Japanese using LLM"
        lastRunAt={paper.created_at}
        actions={
          <button
            onClick={() => translateMutation.mutate()}
            disabled={translateMutation.isPending || progress?.translation.status === 'processing'}
            style={getButtonStyle(translateMutation.isPending || progress?.translation.status === 'processing')}
          >
            {translateMutation.isPending ? 'Starting...' : '▶ Translate'}
          </button>
        }
      >
        <div style={{ fontSize: '0.875rem' }}>
          {progress ? (
            <>
              <div>
                Total Chunks: <strong>{progress.translation.total_chunks}</strong>
              </div>
              <div>
                Completed: <strong style={{ color: '#4caf50' }}>{progress.translation.completed_chunks}</strong>
              </div>
              {progress.translation.failed_chunks > 0 && (
                <div>
                  Failed: <strong style={{ color: '#f44336' }}>{progress.translation.failed_chunks}</strong>
                </div>
              )}
              {progress.translation.status === 'processing' && (
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
                        width: `${Math.round((progress.translation.completed_chunks / progress.translation.total_chunks) * 100)}%`,
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

      {/* Pipeline B: Extract Terms (JP) */}
      <PipelineCard
        title="Pipeline B: Extract Terms (Japanese)"
        pipelineId="extract-terms-jp"
        status={progress?.terms_jp.status || 'idle'}
        description="Extract Japanese technical terms from translations and register to dictionary"
        lastRunAt={progress?.terms_jp.last_run_at}
        actions={
          <button
            onClick={() => extractTermsMutation.mutate()}
            disabled={
              extractTermsMutation.isPending ||
              progress?.terms_jp.status === 'processing' ||
              progress?.translation.status !== 'completed'
            }
            style={getButtonStyle(
              extractTermsMutation.isPending ||
                progress?.terms_jp.status === 'processing' ||
                progress?.translation.status !== 'completed'
            )}
          >
            {extractTermsMutation.isPending ? 'Extracting...' : '▶ Extract Terms'}
          </button>
        }
      >
        <div style={{ fontSize: '0.875rem' }}>
          {progress?.terms_jp ? (
            <>
              <div>
                Extracted Terms: <strong style={{ color: '#4caf50' }}>{progress.terms_jp.total_terms}</strong>
              </div>
              {progress.translation.status !== 'completed' && (
                <div style={{ marginTop: '0.5rem', color: '#ff9800', fontSize: '0.75rem' }}>
                  ⚠ Requires translation to be completed first
                </div>
              )}
            </>
          ) : (
            <div style={{ color: '#999' }}>Not started yet</div>
          )}
        </div>
      </PipelineCard>

      {/* Pipeline C: Scan JP Occurrences */}
      <PipelineCard
        title="Pipeline C: Scan JP Occurrences"
        pipelineId="scan-jp"
        status={progress?.scan_jp.status || 'idle'}
        description="Scan Japanese translations for term occurrences using dictionary"
        lastRunAt={progress?.scan_jp.last_run_at}
        actions={
          <button
            onClick={() => scanJpMutation.mutate()}
            disabled={
              scanJpMutation.isPending ||
              progress?.scan_jp.status === 'processing' ||
              progress?.translation.status !== 'completed'
            }
            style={getButtonStyle(
              scanJpMutation.isPending ||
                progress?.scan_jp.status === 'processing' ||
                progress?.translation.status !== 'completed'
            )}
          >
            {scanJpMutation.isPending ? 'Scanning...' : '▶ Scan Occurrences'}
          </button>
        }
      >
        <div style={{ fontSize: '0.875rem' }}>
          {progress?.scan_jp ? (
            <>
              <div>
                Found Occurrences: <strong style={{ color: '#4caf50' }}>{progress.scan_jp.total_occurrences}</strong>
              </div>
              {progress.translation.status !== 'completed' && (
                <div style={{ marginTop: '0.5rem', color: '#ff9800', fontSize: '0.75rem' }}>
                  ⚠ Requires translation to be completed first
                </div>
              )}
            </>
          ) : (
            <div style={{ color: '#999' }}>Not started yet</div>
          )}
        </div>
      </PipelineCard>

      {/* Pipeline D: Generate Definitions */}
      <PipelineCard
        title="Pipeline D: Generate Definitions"
        pipelineId="generate-definitions"
        status={progress?.definitions.status || 'idle'}
        resultState={progress?.definitions.result_state}
        description="Generate Japanese definitions for terms using LLM"
        lastRunAt={progress?.definitions.last_run_at}
        actions={
          <button
            onClick={() => generateDefsMutation.mutate()}
            disabled={generateDefsMutation.isPending || progress?.definitions.status === 'processing'}
            style={getButtonStyle(generateDefsMutation.isPending || progress?.definitions.status === 'processing')}
          >
            {generateDefsMutation.isPending ? 'Generating...' : '▶ Generate Definitions'}
          </button>
        }
      >
        <div style={{ fontSize: '0.875rem' }}>
          {progress?.definitions ? (
            <>
              <div>
                Generated: <strong style={{ color: '#4caf50' }}>{progress.definitions.generated}</strong>
              </div>
              {progress.definitions.failed > 0 && (
                <div>
                  Failed: <strong style={{ color: '#f44336' }}>{progress.definitions.failed}</strong>
                </div>
              )}
              {progress.definitions.result_state === 'completed_empty' && (
                <div style={{ marginTop: '0.5rem', color: '#ff9800', fontSize: '0.75rem' }}>
                  ⚠ No definitions generated (no terms found or all already have definitions)
                </div>
              )}
            </>
          ) : (
            <div style={{ color: '#999' }}>Not started yet</div>
          )}
        </div>
      </PipelineCard>
    </div>
  )
}
