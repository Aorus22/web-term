import { useState, useCallback } from 'react'
import { toast } from 'sonner'
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuShortcut,
  ContextMenuTrigger,
} from '@/components/ui/context-menu'
import type { TerminalHandle } from './types'

interface TerminalContextMenuProps {
  terminalRef: React.RefObject<TerminalHandle | null>
  sendData: (data: string) => void
  children: React.ReactNode
}

export function TerminalContextMenu({
  terminalRef,
  sendData,
  children,
}: TerminalContextMenuProps) {
  const [hasSelection, setHasSelection] = useState(false)

  const refreshSelectionState = useCallback(() => {
    setHasSelection(terminalRef.current?.getSelection() !== '')
  }, [terminalRef])

  const handleCopy = useCallback(async () => {
    try {
      const selection = terminalRef.current?.getSelection() ?? ''
      if (selection) {
        await navigator.clipboard.writeText(selection)
        toast.success('Copied to clipboard')
      }
    } catch {
      toast.error('Failed to copy')
    }
  }, [terminalRef])

  const handlePaste = useCallback(async () => {
    try {
      const text = await navigator.clipboard.readText()
      if (text) {
        terminalRef.current?.paste(text)
      }
    } catch {
      toast.error('Failed to read clipboard')
    }
  }, [terminalRef])

  const handleSelectAll = useCallback(() => {
    terminalRef.current?.selectAll()
    // Refresh selection state after select-all completes
    setTimeout(refreshSelectionState, 0)
  }, [terminalRef, refreshSelectionState])

  const handleClear = useCallback(() => {
    terminalRef.current?.clear()
  }, [terminalRef])

  const handleCtrlKey = useCallback(
    (key: string) => {
      // Send the raw control character (e.g. Ctrl+C = \x03, Ctrl+Z = \x1a)
      const code = key.toLowerCase().charCodeAt(0)
      if (code >= 97 && code <= 122) {
        sendData(String.fromCharCode(code - 96))
      }
    },
    [sendData]
  )

  return (
    <ContextMenu
      onOpenChange={(open) => {
        if (open) refreshSelectionState()
      }}
    >
      <ContextMenuTrigger className="h-full w-full">
        {children}
      </ContextMenuTrigger>
      <ContextMenuContent className="min-w-52">
        <ContextMenuItem onClick={handleCopy} disabled={!hasSelection}>
          Copy
          <ContextMenuShortcut>⌘C</ContextMenuShortcut>
        </ContextMenuItem>
        <ContextMenuItem onClick={handlePaste}>
          Paste
          <ContextMenuShortcut>⌘V</ContextMenuShortcut>
        </ContextMenuItem>
        <ContextMenuItem onClick={handleSelectAll}>
          Select All
          <ContextMenuShortcut>⌘A</ContextMenuShortcut>
        </ContextMenuItem>
        <ContextMenuSeparator />
        <ContextMenuItem onClick={handleClear}>
          Clear
          <ContextMenuShortcut>⌘K</ContextMenuShortcut>
        </ContextMenuItem>
        <ContextMenuSeparator />
        <ContextMenuItem onClick={() => handleCtrlKey('c')}>
          Send Ctrl+C
          <ContextMenuShortcut>Interrupt</ContextMenuShortcut>
        </ContextMenuItem>
        <ContextMenuItem onClick={() => handleCtrlKey('z')}>
          Send Ctrl+Z
          <ContextMenuShortcut>Suspend</ContextMenuShortcut>
        </ContextMenuItem>
        <ContextMenuItem onClick={() => handleCtrlKey('d')}>
          Send Ctrl+D
          <ContextMenuShortcut>EOF</ContextMenuShortcut>
        </ContextMenuItem>
        <ContextMenuItem onClick={() => handleCtrlKey('l')}>
          Send Ctrl+L
          <ContextMenuShortcut>Clear line</ContextMenuShortcut>
        </ContextMenuItem>
      </ContextMenuContent>
    </ContextMenu>
  )
}
