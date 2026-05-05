import { forwardRef, useEffect, useRef, useImperativeHandle } from 'react'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import { WebLinksAddon } from '@xterm/addon-web-links'
import type { TerminalHandle } from '../types'

// xterm.js CSS - loaded via global stylesheet

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
    const fitAddonRef = useRef<FitAddon | null>(null)

    useImperativeHandle(
      ref,
      () => ({
        write: (data: Uint8Array | string) => {
          terminalRefInternal.current?.write(data)
        },
        focus: () => {
          terminalRefInternal.current?.focus()
        },
      }),
      [],
    )

    useEffect(() => {
      if (!containerRef.current) return

      let terminal: Terminal | null = null
      let fitAddon: FitAddon | null = null

      try {
        terminal = new Terminal({
          fontFamily: `'${fontFamily}', monospace`,
          fontSize: parseInt(fontSize, 10),
          cursorBlink,
          allowProposedApi: true,
          theme: {
            background: '#1e1e1e',
            foreground: '#d4d4d4',
            cursor: '#ffffff',
            cursorAccent: '#000000',
            selectionBackground: '#264f78',
          },
        })

        fitAddon = new FitAddon()
        const webLinksAddon = new WebLinksAddon()

        terminal.loadAddon(fitAddon)
        terminal.loadAddon(webLinksAddon)

        terminal.open(containerRef.current)

        // Fit after a short delay to ensure container has dimensions
        setTimeout(() => {
          fitAddon?.fit()
        }, 10)

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

        fitAddonRef.current = fitAddon

        onReady?.()
      } catch (err) {
        console.error('XTermEngine: Failed to initialize:', err)
      }

      return () => {
        try {
          terminal?.dispose()
        } catch (e) {
          // Ignore disposal errors
        }
        terminalRefInternal.current = null
        terminalRef.current = null
      }
    }, [fontFamily, fontSize, cursorBlink, sendData, sendResize, onReady])

    useEffect(() => {
      const handleResize = () => {
        fitAddonRef.current?.fit()
      }
      window.addEventListener('resize', handleResize)
      return () => window.removeEventListener('resize', handleResize)
    }, [])

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