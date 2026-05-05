import { forwardRef, useImperativeHandle } from 'react'
import { WTermEngine } from './WTermEngine'
import { XTermEngine } from './XTermEngine'
import type { TerminalHandle } from '../types'
import { cn } from '@/lib/utils'

interface TerminalWrapperProps {
  engine: 'wterm' | 'xterm'
  sendData: (data: string) => void
  sendResize?: (cols: number, rows: number) => void
  onReady?: () => void
  theme?: string
  cursorBlink?: boolean
  fontFamily?: string
  fontSize?: string
  cursorStyle?: 'block' | 'underline' | 'bar'
  className?: string
  style?: React.CSSProperties
}

export const TerminalWrapper = forwardRef<TerminalHandle, TerminalWrapperProps>(
  (
    {
      engine,
      sendData,
      sendResize,
      onReady,
      theme = 'default',
      cursorBlink = true,
      fontFamily = 'Geist Mono',
      fontSize = '14',
      cursorStyle,
      className,
      style,
    },
    ref
  ) => {
    const terminalRef: React.MutableRefObject<TerminalHandle | null> = { current: null }

    useImperativeHandle(
      ref,
      () => ({
        write: (data: Uint8Array | string) => {
          terminalRef.current?.write(data)
        },
        focus: () => {
          terminalRef.current?.focus()
        },
      }),
      [],
    )

    const baseStyle: React.CSSProperties = {
      height: '100%',
      fontFamily: `'${fontFamily}', monospace`,
      fontSize: `${fontSize}px`,
    }

    const cursorStyleClass = cursorStyle === 'underline'
      ? 'cursor-underline'
      : cursorStyle === 'bar'
        ? 'cursor-bar'
        : undefined

    const terminalClassName = cn(
      engine === 'wterm' ? 'wterm' : 'xterm',
      engine === 'wterm' && `theme-${theme}`,
      engine === 'wterm' && cursorStyleClass,
      engine === 'wterm' && 'has-scrollback',
      className
    )

    if (engine === 'xterm') {
      return (
        <XTermEngine
          sendData={sendData}
          sendResize={sendResize}
          onReady={onReady}
          terminalRef={terminalRef}
          fontFamily={fontFamily}
          fontSize={fontSize}
          cursorBlink={cursorBlink !== false}
          className={terminalClassName}
          style={{ ...baseStyle, ...style }}
        />
      )
    }

    return (
      <WTermEngine
        sendData={sendData}
        sendResize={sendResize}
        onReady={onReady}
        terminalRef={terminalRef}
        theme={theme}
        className={terminalClassName}
        style={{ ...baseStyle, ...style }}
      />
    )
  }
)

TerminalWrapper.displayName = 'TerminalWrapper'