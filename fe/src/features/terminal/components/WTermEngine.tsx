import { forwardRef, useImperativeHandle, useRef } from 'react'
import { Terminal } from '@wterm/react'
import '@wterm/react/css'
import type { WTerm } from '@wterm/dom'
import type { TerminalHandle } from '../types'

interface WTermEngineProps {
  sendData: (data: string) => void
  sendResize?: (cols: number, rows: number) => void
  onReady?: () => void
  terminalRef: React.MutableRefObject<TerminalHandle | null>
  theme?: string
  cursorBlink?: boolean
  className?: string
  style?: React.CSSProperties
}

/** What @wterm/react exposes via its forwardRef TerminalHandle */
interface WTermReactHandle {
  write: (data: Uint8Array | string) => void
  resize: (cols: number, rows: number) => void
  focus: () => void
  readonly instance: WTerm | null
}

export const WTermEngine = forwardRef<TerminalHandle, WTermEngineProps>(
  ({ sendData, sendResize, onReady, terminalRef, theme, cursorBlink = true, className, style }, ref) => {
    const internalRef = useRef<WTermReactHandle | null>(null)

    const getInstance = (): WTerm | null => {
      return internalRef.current?.instance ?? null
    }

    const getSelection = (): string => {
      return window.getSelection()?.toString() ?? ''
    }

    const paste = (text: string): void => {
      if (!text) return
      const wt = getInstance()
      const bridge = wt?.bridge
      if (bridge && bridge.bracketedPaste()) {
        // Strip ESC bytes so clipboard payloads cannot break out of bracketed
        // paste mode and smuggle commands to the PTY (same logic as input.js).
        const safe = text.replace(/\x1b/g, '')
        sendData(`\x1b[200~${safe}\x1b[201~`)
      } else {
        sendData(text)
      }
    }

    const selectAll = (): void => {
      const wt = getInstance()
      const el = wt?.element
      if (!el) return
      const sel = window.getSelection()
      if (!sel) return
      const range = document.createRange()
      range.selectNodeContents(el)
      sel.removeAllRanges()
      sel.addRange(range)
    }

    const clear = (): void => {
      // Clear local terminal screen + scrollback. These escape sequences are
      // interpreted by the local terminal renderer: \x1b[3J clears scrollback,
      // \x1b[H homes cursor, \x1b[2J clears the visible screen.
      internalRef.current?.write('\x1b[3J\x1b[H\x1b[2J')
    }

    useImperativeHandle(
      ref,
      () => ({
        write: (data: Uint8Array | string) => {
          internalRef.current?.write(data)
        },
        focus: () => {
          internalRef.current?.focus()
        },
        getSelection,
        paste,
        selectAll,
        clear,
      }),
      [],
    )

    // Minimal shape accepted from both the ref callback (TerminalHandle) and
    // the onReady prop (raw WTerm instance) — both expose write & focus.
    const buildHandle = (): TerminalHandle => ({
      write: (data: Uint8Array | string) => {
        internalRef.current?.write(data)
      },
      focus: () => {
        internalRef.current?.focus()
      },
      getSelection,
      paste,
      selectAll,
      clear,
    })

    const handleReady = () => {
      terminalRef.current = buildHandle()
      onReady?.()
    }

    return (
      <Terminal
        ref={(el) => {
          internalRef.current = el
          if (el) {
            handleReady()
          }
        }}
        autoResize
        cursorBlink={cursorBlink}
        theme={theme}
        className={className}
        onData={sendData}
        onResize={sendResize}
        onReady={handleReady}
        style={{ width: '100%', height: '100%', ...style }}
      />
    )
  }
)

WTermEngine.displayName = 'WTermEngine'
