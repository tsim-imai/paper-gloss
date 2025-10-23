import { useMemo } from 'react'
import { Occurrence } from '../../types'

interface TermHighlightProps {
  text: string
  chunkId: string
  occurrences: Occurrence[]
  highlightedTermId?: string
  onTermHover?: (termId: string | null, event: React.MouseEvent) => void
  onTermClick?: (termId: string, event: React.MouseEvent) => void
}

interface TextSegment {
  text: string
  termId?: string
  occurrenceId?: string
  isHighlighted: boolean
}

/**
 * Component that highlights ML/AI terms in translated text
 * Supports hover tooltips and click-to-pin functionality
 */
export default function TermHighlight({
  text,
  chunkId,
  occurrences,
  highlightedTermId,
  onTermHover,
  onTermClick,
}: TermHighlightProps) {
  // Filter occurrences for this chunk and sort by position
  const chunkOccurrences = useMemo(() => {
    return occurrences
      .filter(occ => occ.chunk_id === chunkId)
      .sort((a, b) => a.start_pos - b.start_pos)
  }, [occurrences, chunkId])

  // Split text into segments (plain text and highlighted terms)
  const segments = useMemo(() => {
    if (chunkOccurrences.length === 0) {
      return [{ text, isHighlighted: false }] as TextSegment[]
    }

    const result: TextSegment[] = []
    let currentPos = 0

    for (const occ of chunkOccurrences) {
      // Add plain text before this occurrence
      if (currentPos < occ.start_pos) {
        result.push({
          text: text.substring(currentPos, occ.start_pos),
          isHighlighted: false,
        })
      }

      // Add highlighted term
      result.push({
        text: text.substring(occ.start_pos, occ.end_pos),
        termId: occ.term_id,
        occurrenceId: occ.id,
        isHighlighted: true,
      })

      currentPos = occ.end_pos
    }

    // Add remaining plain text
    if (currentPos < text.length) {
      result.push({
        text: text.substring(currentPos),
        isHighlighted: false,
      })
    }

    return result
  }, [text, chunkOccurrences])

  return (
    <div style={{ lineHeight: '1.8', whiteSpace: 'pre-wrap', color: '#ffffff' }}>
      {segments.map((segment, index) => {
        if (!segment.isHighlighted) {
          return <span key={index}>{segment.text}</span>
        }

        const isActive = highlightedTermId === segment.termId

        return (
          <span
            key={index}
            data-term-id={segment.termId}
            data-occurrence-id={segment.occurrenceId}
            onMouseEnter={(e) => onTermHover?.(segment.termId!, e)}
            onMouseLeave={(e) => onTermHover?.(null, e)}
            onClick={(e) => onTermClick?.(segment.termId!, e)}
            style={{
              backgroundColor: isActive ? '#fff9c4' : '#e3f2fd',
              borderBottom: '2px solid #1976d2',
              cursor: 'pointer',
              padding: '2px 1px',
              borderRadius: '2px',
              transition: 'background-color 0.15s ease',
            }}
          >
            {segment.text}
          </span>
        )
      })}
    </div>
  )
}
