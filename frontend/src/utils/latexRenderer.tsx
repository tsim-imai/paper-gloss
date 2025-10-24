import React from 'react'
import katex from 'katex'
import 'katex/dist/katex.min.css'

/**
 * Parse text containing LaTeX math expressions and render them with KaTeX
 * Supports: $...$ (inline), $$...$$ (display), \[...\], \(...\), equation/align environments
 * Also cleans LaTeX environment commands (\begin{}, \end{}) for display
 */

interface MathSegment {
  type: 'text' | 'math'
  content: string
  displayMode: boolean
}

/**
 * Clean LaTeX text by removing environment commands and formatting directives
 * Keeps: actual content, math expressions
 * Removes: \begin{env}, \end{env}, \item, spacing commands
 */
function cleanLatexText(text: string): string {
  let result = text

  // Remove \begin{env}[options] and \begin{env} (keep content)
  // Common environments: theorem, lemma, proof, property, definition, etc.
  result = result.replace(/\\begin\{[^}]+\}(?:\[[^\]]*\])?/g, '')

  // Remove \end{env}
  result = result.replace(/\\end\{[^}]+\}/g, '')

  // Replace \item with bullet point
  result = result.replace(/\\item(?:\[[^\]]*\])?/g, '• ')

  // Remove spacing commands: \vspace, \hspace, \noindent, etc.
  result = result.replace(/\\(?:vspace|hspace|noindent|indent|medskip|bigskip|smallskip)\{[^}]*\}/g, '')
  result = result.replace(/\\(?:noindent|indent|medskip|bigskip|smallskip)/g, '')

  // Remove section commands but keep the title
  result = result.replace(/\\(?:sub)*section\*?\{([^}]*)\}/g, '$1')

  // Clean up multiple consecutive newlines
  result = result.replace(/\n\n\n+/g, '\n\n')

  return result.trim()
}

/**
 * Parse text into segments of text and math expressions
 * Also applies LaTeX cleaning to text segments
 */
function parseLatexMath(text: string): MathSegment[] {
  const segments: MathSegment[] = []
  let currentIndex = 0

  // Define math patterns in order of precedence (longest first)
  const patterns = [
    // Display math: $$...$$
    { regex: /\$\$(.*?)\$\$/gs, displayMode: true },
    // Inline math: $...$
    { regex: /\$(.*?)\$/g, displayMode: false },
    // Display math: \[...\]
    { regex: /\\\[(.*?)\\\]/gs, displayMode: true },
    // Inline math: \(...\)
    { regex: /\\\((.*?)\\\)/gs, displayMode: false },
    // Equation environment: \begin{equation}...\end{equation}
    { regex: /\\begin\{equation\*?\}(.*?)\\end\{equation\*?\}/gs, displayMode: true },
    // Align environment: \begin{align}...\end{align}
    { regex: /\\begin\{align\*?\}(.*?)\\end\{align\*?\}/gs, displayMode: true },
  ]

  // Find all math expressions with their positions
  const matches: Array<{ start: number; end: number; content: string; displayMode: boolean }> = []

  for (const pattern of patterns) {
    let match
    while ((match = pattern.regex.exec(text)) !== null) {
      const start = match.index
      const end = start + match[0].length
      const content = match[1]

      // Check if this position is already covered by a previous match
      const isOverlapping = matches.some(
        (m) => (start >= m.start && start < m.end) || (end > m.start && end <= m.end)
      )

      if (!isOverlapping) {
        matches.push({ start, end, content, displayMode: pattern.displayMode })
      }
    }
  }

  // Sort matches by start position
  matches.sort((a, b) => a.start - b.start)

  // Build segments
  for (const match of matches) {
    // Add text before math (with cleaning)
    if (currentIndex < match.start) {
      const textContent = text.substring(currentIndex, match.start)
      if (textContent.length > 0) {
        const cleanedText = cleanLatexText(textContent)
        if (cleanedText.length > 0) {
          segments.push({ type: 'text', content: cleanedText, displayMode: false })
        }
      }
    }

    // Add math segment (no cleaning)
    segments.push({ type: 'math', content: match.content, displayMode: match.displayMode })
    currentIndex = match.end
  }

  // Add remaining text (with cleaning)
  if (currentIndex < text.length) {
    const textContent = text.substring(currentIndex)
    if (textContent.length > 0) {
      const cleanedText = cleanLatexText(textContent)
      if (cleanedText.length > 0) {
        segments.push({ type: 'text', content: cleanedText, displayMode: false })
      }
    }
  }

  // If no math found, return entire text as one segment (with cleaning)
  if (segments.length === 0) {
    const cleanedText = cleanLatexText(text)
    if (cleanedText.length > 0) {
      segments.push({ type: 'text', content: cleanedText, displayMode: false })
    }
  }

  return segments
}

/**
 * Render a single math segment with KaTeX
 */
function renderMath(content: string, displayMode: boolean): string {
  try {
    return katex.renderToString(content, {
      displayMode,
      throwOnError: false,
      output: 'html',
      strict: false,
    })
  } catch (error) {
    console.error('KaTeX rendering error:', error)
    // Return original content wrapped in code tag on error
    return `<code style="color: #f44336;">${content}</code>`
  }
}

interface LatexRendererProps {
  text: string
  style?: React.CSSProperties
}

/**
 * Component that renders text with LaTeX math expressions
 */
export function LatexRenderer({ text, style }: LatexRendererProps) {
  const segments = parseLatexMath(text)

  return (
    <div style={style}>
      {segments.map((segment, index) => {
        if (segment.type === 'text') {
          // Render text with preserved whitespace
          return (
            <span key={index} style={{ whiteSpace: 'pre-wrap' }}>
              {segment.content}
            </span>
          )
        } else {
          // Render math with KaTeX
          const html = renderMath(segment.content, segment.displayMode)
          return (
            <span
              key={index}
              dangerouslySetInnerHTML={{ __html: html }}
              style={{
                display: segment.displayMode ? 'block' : 'inline',
                margin: segment.displayMode ? '1rem 0' : '0',
                textAlign: segment.displayMode ? 'center' : 'left',
              }}
            />
          )
        }
      })}
    </div>
  )
}

/**
 * Parse LaTeX math expressions and return segments for custom rendering
 * Useful when you need more control over rendering (e.g., with term highlighting)
 */
export function parseLatexSegments(text: string): MathSegment[] {
  return parseLatexMath(text)
}

/**
 * Render a LaTeX math expression to HTML string
 */
export function renderLatexToHtml(content: string, displayMode: boolean = false): string {
  return renderMath(content, displayMode)
}
