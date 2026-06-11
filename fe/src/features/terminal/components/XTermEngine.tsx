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
  cursorStyle?: 'block' | 'underline' | 'bar'
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
      cursorStyle = 'block',
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

    // Update theme when theme name changes
    useEffect(() => {
      if (terminalRefInternal.current) {
        terminalRefInternal.current.options.theme = getXTermTheme(theme)
      }
    }, [theme, getXTermTheme])

    // Update cursor settings when they change
    useEffect(() => {
      if (terminalRefInternal.current) {
        terminalRefInternal.current.options.cursorBlink = cursorBlink
        terminalRefInternal.current.options.cursorStyle = cursorStyle
      }
    }, [cursorBlink, cursorStyle])

    useEffect(() => {
      if (!containerRef.current) return

      let terminal: Terminal | null = null

      try {
        terminal = new Terminal({
          fontFamily: `'${fontFamily}', monospace`,
          fontSize: parseInt(fontSize, 10),
          cursorBlink,
          cursorStyle,
          allowProposedApi: true,
          allowTransparency: true,
          theme: getXTermTheme(theme),
        })

        const fitAddon = new FitAddon()
        terminal.loadAddon(fitAddon)
        fitAddonRef.current = fitAddon

        terminal.open(containerRef.current)

        // Handle clipboard copy: Ctrl+Shift+C always copies, Ctrl+C copies if selection exists
        terminal.attachCustomKeyEventHandler((event) => {
          if (event.type !== 'keydown') return true
          const isCtrlC = event.ctrlKey && event.key === 'c' && !event.shiftKey
          const isCtrlShiftC = event.ctrlKey && event.key === 'C' && event.shiftKey

          if (isCtrlShiftC || isCtrlC) {
            const selection = terminal?.getSelection()
            if (selection) {
              navigator.clipboard.writeText(selection)
              return false // consume the event
            }
            // No selection + Ctrl+C → let it through as interrupt
            if (isCtrlC) return true
            return false
          }
          return true
        })

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

        // Setup ResizeObserver with debounce to prevent flooding PTY during rapid resize
        let resizeTimer: ReturnType<typeof setTimeout> | null = null
        const resizeObserver = new ResizeObserver(() => {
          if (resizeTimer) clearTimeout(resizeTimer)
          resizeTimer = setTimeout(() => {
            if (terminal && fitAddon) {
              try {
                fitAddon.fit()
              } catch (e) {
                // ignore fit errors during transitions
              }
            }
          }, 80)
        })
        resizeObserver.observe(containerRef.current!)

        onReady?.()

        return () => {
          if (resizeTimer) clearTimeout(resizeTimer)
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
    }, [fontFamily, fontSize, cursorBlink, cursorStyle, sendData, sendResize, onReady, terminalRef])

    return (
      <div style={{ padding: '12px', height: '100%', width: '100%', boxSizing: 'border-box' }}>
        <div
          ref={containerRef}
          className={`xterm-container ${className || ''}`}
          style={{
            height: '100%',
            width: '100%',
            display: 'flex',
            flexDirection: 'column',
            backgroundColor: 'transparent',
            overflow: 'hidden',
            ...style,
          }}
        />
      </div>
    )
  }
)

XTermEngine.displayName = 'XTermEngine'