import { useEffect, type RefObject } from 'react'

interface SFTPShortcutHandlers {
  onEnter?: () => void
  onBackspace?: () => void
  onDelete?: () => void
  onRefresh?: () => void
  onCopy?: () => void
  onCut?: () => void
  onPaste?: () => void
}

export function useSFTPShortcuts(
  ref: RefObject<HTMLElement | null>,
  handlers: SFTPShortcutHandlers,
  enabled: boolean = true
) {
  useEffect(() => {
    if (!enabled) return

    const handleKeyDown = (e: KeyboardEvent) => {
      // Focus guard: ensure focus is within this pane
      // We check ref.current which should be the root element of DirectoryBrowser
      if (!ref.current || !ref.current.contains(document.activeElement)) {
        return
      }

      // If typing in an input or textarea, don't trigger shortcuts
      const activeTag = document.activeElement?.tagName
      const isTyping = activeTag === 'INPUT' || activeTag === 'TEXTAREA'
      if (isTyping) return

      const { key, ctrlKey, altKey, metaKey } = e

      // Enter: Open directory or download
      if (key === 'Enter' && !ctrlKey && !altKey && !metaKey) {
        e.preventDefault()
        handlers.onEnter?.()
      }
      
      // Backspace: Go up
      if (key === 'Backspace' && !ctrlKey && !altKey && !metaKey) {
        e.preventDefault()
        handlers.onBackspace?.()
      }

      // Delete: Trigger delete
      if (key === 'Delete' && !ctrlKey && !altKey && !metaKey) {
        e.preventDefault()
        handlers.onDelete?.()
      }

      // Refresh: F5 or Ctrl+R
      if ((key === 'F5' || (key === 'r' && ctrlKey)) && !altKey && !metaKey) {
        e.preventDefault()
        handlers.onRefresh?.()
      }

      // Clipboard actions
      if (ctrlKey && !altKey && !metaKey) {
        if (key === 'c') {
          e.preventDefault()
          handlers.onCopy?.()
        } else if (key === 'x') {
          e.preventDefault()
          handlers.onCut?.()
        } else if (key === 'v') {
          e.preventDefault()
          handlers.onPaste?.()
        }
      }
    }

    window.addEventListener('keydown', handleKeyDown, true) // Use capture to intercept before other listeners
    return () => window.removeEventListener('keydown', handleKeyDown, true)
  }, [ref, handlers, enabled])
}
