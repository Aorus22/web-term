import { forwardRef, useEffect, useRef, useImperativeHandle } from 'react'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
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
    const fitAddonRef = useRef<FitAddon | null>(null)

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
          allowProposedApi: true,
          theme: {
            background: 'transparent',
            foreground: '#d4d4d4',
            cursor: '#cccccc',
            cursorAccent: '#000000',
            selectionBackground: '#264f78',
          },
        })

        const fitAddon = new FitAddon()
        terminal.loadAddon(fitAddon)
        fitAddonRef.current = fitAddon

        terminal.open(containerRef.current)
        
        // Initial fit
        setTimeout(() => {
          fitAddon.fit()
        }, 50)

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

        // Setup ResizeObserver
        const resizeObserver = new ResizeObserver(() => {
          if (terminal && fitAddon) {
            // Small timeout to let layout settle
            setTimeout(() => {
              try {
                fitAddon.fit()
              } catch (e) {
                // ignore fit errors during transitions
              }
            }, 10)
          }
        })
        resizeObserver.observe(containerRef.current)

        onReady?.()

        return () => {
          resizeObserver.disconnect()
          terminal?.dispose()
          terminalRefInternal.current = null
          terminalRef.current = null
        }
      } catch (err) {
        console.error('[XTermEngine] Failed to initialize:', err)
      }
    }, [fontFamily, fontSize, cursorBlink, sendData, sendResize, onReady, terminalRef])

    return (
      <div
        ref={containerRef}
        className={`xterm-container ${className || ''}`}
        style={{
          height: '100%',
          width: '100%',
          display: 'flex',
          flexDirection: 'column',
          backgroundColor: 'transparent',
          ...style,
        }}
      />
    )
  }
)

XTermEngine.displayName = 'XTermEngine'