import { forwardRef, useEffect, useRef, useImperativeHandle } from 'react'
import { Terminal } from '@xterm/xterm'
import type { TerminalHandle } from '../types'

interface XTermEngineProps {
  sendData: (data: string) => void
  sendResize?: (cols: number, rows: number) => void
  onReady?: () => void
  terminalRef: React.MutableRefObject<TerminalHandle | null>
  fontFamily?: string
  fontSize?: string
  theme?: string
  cursorBlink?: boolean
  className?: string
  style?: React.CSSProperties
}

export const XTermEngine = forwardRef<TerminalHandle, XTermEngineProps>(
  (
    {
      sendData,
      sendResize,
      onReady,
      terminalRef,
      fontFamily = 'Geist Mono',
      fontSize = '14',
      cursorBlink = true,
      className,
      style,
    },
    ref
  ) => {
    const containerRef = useRef<HTMLDivElement>(null)
    const terminalRefInternal = useRef<Terminal | null>(null)

    useImperativeHandle(
      ref,
      () => ({
        write: (data: Uint8Array | string) => {
          if (terminalRefInternal.current) {
            terminalRefInternal.current.write(data)
          }
        },
        focus: () => {
          if (terminalRefInternal.current) {
            terminalRefInternal.current.focus()
          }
        },
      }),
      [],
    )

    useEffect(() => {
      if (!containerRef.current) return

      let terminal: Terminal | null = null

      try {
        terminal = new Terminal({
          fontFamily: `'${fontFamily}', monospace`,
          fontSize: parseInt(fontSize, 10),
          cursorBlink,
          theme: {
            background: '#1e1e1e',
            foreground: '#d4d4d4',
            cursor: '#cccccc',
            cursorAccent: '#000000',
            selectionBackground: '#264f78',
          },
        })

        terminal.open(containerRef.current)

        terminalRefInternal.current = terminal

        terminal.onData((data) => {
          sendData(data)
        })

        terminal.onResize(({ cols, rows }) => {
          sendResize?.(cols, rows)
        })

        terminalRef.current = {
          write: (data: Uint8Array | string) => {
            terminal?.write(data)
          },
          focus: () => {
            terminal?.focus()
          },
        }

        onReady?.()
      } catch (err) {
        console.error('[XTermEngine] Failed to initialize:', err)
      }

      return () => {
        terminal?.dispose()
        terminalRefInternal.current = null
        terminalRef.current = null
      }
    }, [fontFamily, fontSize, cursorBlink, sendData, sendResize, onReady, terminalRef])

    return (
      <div
        ref={containerRef}
        className={`xterm-container ${className || ''}`}
        style={{
          height: '100%',
          width: '100%',
          minHeight: '200px',
          minWidth: '100px',
          display: 'flex',
          flexDirection: 'column',
          backgroundColor: '#1e1e1e',
          ...style,
        }}
      />
    )
  }
)

XTermEngine.displayName = 'XTermEngine'