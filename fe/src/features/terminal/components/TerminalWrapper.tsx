import { forwardRef, useImperativeHandle, useRef, useState, useEffect } from 'react'
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
    const [currentEngine, setCurrentEngine] = useState(engine)
    // Use useRef for terminalRef so it persists across renders
    const terminalRef = useRef<TerminalHandle | null>(null)

    // Debounce engine changes
    useEffect(() => {
      if (engine === currentEngine) return
      
      const timer = setTimeout(() => {
        setCurrentEngine(engine)
      }, 100)
      
      return () => clearTimeout(timer)
    }, [engine, currentEngine])

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
      [currentEngine],
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
      currentEngine === 'wterm' ? 'wterm' : 'xterm',
      currentEngine === 'wterm' && `theme-${theme}`,
      currentEngine === 'wterm' && cursorStyleClass,
      currentEngine === 'wterm' && 'has-scrollback',
      className
    )

    if (currentEngine === 'xterm') {
      return (
        <XTermEngine
          sendData={sendData}
          sendResize={sendResize}
          onReady={onReady}
          terminalRef={terminalRef as any}
          fontFamily={fontFamily}
          fontSize={fontSize}
          theme={theme}
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
        terminalRef={terminalRef as any}
        theme={theme}
        className={terminalClassName}
        style={{ ...baseStyle, ...style }}
      />
    )
  }
)

TerminalWrapper.displayName = 'TerminalWrapper'