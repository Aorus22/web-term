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

      try {
        const terminal = new Terminal({
          fontFamily: `'${fontFamily}', monospace`,
          fontSize: parseInt(fontSize, 10),
          cursorBlink,
          theme: {
            background: '#000000',
            foreground: '#cccccc',
            cursor: '#ffffff',
            cursorAccent: '#000000',
            selectionBackground: '#264f78',
            black: '#000000',
            red: '#cd3131',
            green: '#0dbc79',
            yellow: '#e5e510',
            blue: '#2472c8',
            magenta: '#bc3fbc',
            cyan: '#11a8cd',
            white: '#e5e5e5',
            brightBlack: '#666666',
            brightRed: '#f14c4c',
            brightGreen: '#23d18b',
            brightYellow: '#f5f543',
            brightBlue: '#3b8eea',
            brightMagenta: '#d670d6',
            brightCyan: '#29b8db',
            brightWhite: '#ffffff',
          },
        })

        const fitAddon = new FitAddon()
        const webLinksAddon = new WebLinksAddon()

        terminal.loadAddon(fitAddon)
        terminal.loadAddon(webLinksAddon)

        terminal.open(containerRef.current)
        fitAddon.fit()

        terminalRefInternal.current = terminal

        terminal.onData((data) => {
          sendData(data)
        })

        terminal.onResize(({ cols, rows }) => {
          sendResize?.(cols, rows)
        })

        terminalRef.current = {
          write: (data: Uint8Array | string) => {
            terminal.write(data)
          },
          focus: () => {
            terminal.focus()
          },
        }

        fitAddonRef.current = fitAddon

        onReady?.()
      } catch (err) {
        console.error('XTermEngine: Failed to initialize:', err)
      }

      return () => {
        terminalRefInternal.current?.dispose()
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
        className={className}
        style={{
          height: '100%',
          width: '100%',
          ...style,
        }}
      />
    )
  }
)

XTermEngine.displayName = 'XTermEngine'