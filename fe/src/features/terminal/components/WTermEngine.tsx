import { forwardRef, useImperativeHandle, useRef } from 'react'
import { Terminal } from '@wterm/react'
import '@wterm/react/css'
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

interface WTermInstance {
  write: (data: Uint8Array | string) => void
  focus: () => void
}

export const WTermEngine = forwardRef<TerminalHandle, WTermEngineProps>(
  ({ sendData, sendResize, onReady, terminalRef, theme, cursorBlink = true, className, style }, ref) => {
    const internalRef = useRef<WTermInstance | null>(null)

    useImperativeHandle(
      ref,
      () => ({
        write: (data: Uint8Array | string) => {
          if (internalRef.current?.write) {
            internalRef.current.write(data)
          }
        },
        focus: () => {
          if (internalRef.current?.focus) {
            internalRef.current.focus()
          }
        },
      }),
      [],
    )

    const handleReady = (wt: WTermInstance) => {
      if (!wt) return
      terminalRef.current = {
        write: (data: Uint8Array | string) => {
          if (wt.write) {
            wt.write(data)
          }
        },
        focus: () => {
          if (wt.focus) {
            wt.focus()
          }
        },
      }
      onReady?.()
    }

    return (
      <Terminal
        ref={(el) => {
          internalRef.current = el
          if (el) {
            handleReady(el)
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