import { useMemo } from 'react'
import { Occurrence } from '../../types'
import { parseLatexSegments, renderLatexToHtml } from '../../utils/latexRenderer'

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

interface RenderSegment {
  type: 'text' | 'math'
  content: string
  displayMode?: boolean
  textSegments?: TextSegment[]
}

/**
 * Component that highlights ML/AI terms in translated text
 * Supports hover tooltips and click-to-pin functionality
 * Now with LaTeX math rendering support
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

  // Parse text into LaTeX segments (math vs text)
  const latexSegments = useMemo(() => {
    return parseLatexSegments(text)
  }, [text])

  // Process each segment: apply term highlighting to text segments only
  const renderSegments = useMemo(() => {
    const result: RenderSegment[] = []
    let textOffset = 0 // Track position in original text

    for (const segment of latexSegments) {
      if (segment.type === 'math') {
        // Math segment: render as-is with KaTeX
        result.push({
          type: 'math',
          content: segment.content,
          displayMode: segment.displayMode,
        })
        // Math segments appear in original text with delimiters
        // We need to account for the length in the original text
        // For now, we'll skip occurrence matching in math segments
      } else {
        // Text segment: apply term highlighting
        const segmentStartPos = textOffset
        const segmentEndPos = textOffset + segment.content.length

        // Find occurrences that fall within this text segment
        const segmentOccurrences = chunkOccurrences.filter(
          occ => occ.start_pos >= segmentStartPos && occ.end_pos <= segmentEndPos
        )

        if (segmentOccurrences.length === 0) {
          // No highlights in this segment
          result.push({
            type: 'text',
            content: segment.content,
            textSegments: [{ text: segment.content, isHighlighted: false }],
          })
        } else {
          // Split segment by occurrences
          const textSegments: TextSegment[] = []
          let currentPos = 0

          for (const occ of segmentOccurrences) {
            const localStart = occ.start_pos - segmentStartPos
            const localEnd = occ.end_pos - segmentStartPos

            // Add plain text before occurrence
            if (currentPos < localStart) {
              textSegments.push({
                text: segment.content.substring(currentPos, localStart),
                isHighlighted: false,
              })
            }

            // Add highlighted term
            textSegments.push({
              text: segment.content.substring(localStart, localEnd),
              termId: occ.term_id,
              occurrenceId: occ.id,
              isHighlighted: true,
            })

            currentPos = localEnd
          }

          // Add remaining text
          if (currentPos < segment.content.length) {
            textSegments.push({
              text: segment.content.substring(currentPos),
              isHighlighted: false,
            })
          }

          result.push({
            type: 'text',
            content: segment.content,
            textSegments,
          })
        }

        textOffset += segment.content.length
      }
    }

    return result
  }, [latexSegments, chunkOccurrences])

  return (
    <div style={{ lineHeight: '1.8', whiteSpace: 'pre-wrap', color: '#ffffff' }}>
      {renderSegments.map((segment, segmentIndex) => {
        if (segment.type === 'math') {
          // Render math with KaTeX
          const html = renderLatexToHtml(segment.content, segment.displayMode || false)
          return (
            <span
              key={segmentIndex}
              dangerouslySetInnerHTML={{ __html: html }}
              style={{
                display: segment.displayMode ? 'block' : 'inline',
                margin: segment.displayMode ? '1rem 0' : '0',
                textAlign: segment.displayMode ? 'center' : 'left',
              }}
            />
          )
        } else {
          // Render text with term highlighting
          return (
            <span key={segmentIndex}>
              {segment.textSegments?.map((textSeg, textSegIndex) => {
                if (!textSeg.isHighlighted) {
                  return <span key={textSegIndex}>{textSeg.text}</span>
                }

                const isActive = highlightedTermId === textSeg.termId

                return (
                  <span
                    key={textSegIndex}
                    data-term-id={textSeg.termId}
                    data-occurrence-id={textSeg.occurrenceId}
                    onMouseEnter={(e) => onTermHover?.(textSeg.termId!, e)}
                    onMouseLeave={(e) => onTermHover?.(null, e)}
                    onClick={(e) => onTermClick?.(textSeg.termId!, e)}
                    style={{
                      backgroundColor: isActive ? '#8b6914' : '#2c5282',
                      color: isActive ? '#fff9c4' : '#a8d4ff',
                      borderBottom: '2px solid #64b5f6',
                      cursor: 'pointer',
                      padding: '2px 1px',
                      borderRadius: '2px',
                      transition: 'background-color 0.15s ease, color 0.15s ease',
                    }}
                  >
                    {textSeg.text}
                  </span>
                )
              })}
            </span>
          )
        }
      })}
    </div>
  )
}
