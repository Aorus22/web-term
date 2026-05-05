import { forwardRef, useEffect, useRef, useImperativeHandle, useCallback } from 'react'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import { terminalThemes } from '@/features/settings/data/terminal-themes'
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
      theme = 'default',
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

    // Helper to get ITheme object from preset name
    const getXTermTheme = useCallback((themeName: string) => {
      const preset = terminalThemes.find((t) => t.name === themeName) || terminalThemes[0]
      return {
        background: preset.colors.background,
        foreground: preset.colors.foreground,
        cursor: preset.colors.cursor,
        black: preset.colors.black,
        red: preset.colors.red,
        green: preset.colors.green,
        yellow: preset.colors.yellow,
        blue: preset.colors.blue,
        magenta: preset.colors.magenta,
        cyan: preset.colors.cyan,
        white: preset.colors.white,
        brightBlack: preset.colors.brightBlack,
        brightRed: preset.colors.brightRed,
        brightGreen: preset.colors.brightGreen,
        brightYellow: preset.colors.brightYellow,
        brightBlue: preset.colors.brightBlue,
        brightMagenta: preset.colors.brightMagenta,
        brightCyan: preset.colors.brightCyan,
        brightWhite: preset.colors.brightWhite,
      }
    }, [])

    // Update theme when it changes without recreating the terminal
    useEffect(() => {
      if (terminalRefInternal.current) {
        terminalRefInternal.current.options.theme = getXTermTheme(theme)
      }
    }, [theme, getXTermTheme])

    useEffect(() => {
      if (!containerRef.current) return

      let terminal: Terminal | null = null

      try {
        terminal = new Terminal({
          fontFamily: `'${fontFamily}', monospace`,
          fontSize: parseInt(fontSize, 10),
          cursorBlink,
          allowProposedApi: true,
          allowTransparency: true,
          theme: getXTermTheme(theme),
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
      // We intentionally exclude 'theme' from this dependency array to avoid
      // full terminal re-initialization when the theme changes.
      // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [fontFamily, fontSize, cursorBlink, sendData, sendResize, onReady, terminalRef, getXTermTheme])

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