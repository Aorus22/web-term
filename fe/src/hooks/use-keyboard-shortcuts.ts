import { useEffect } from 'react'

interface KeyboardShortcutHandlers {
  onNewTab: () => void
  onCloseTab: () => void
  onNextTab: () => void
  onPrevTab: () => void
}

export function useKeyboardShortcuts({
  onNewTab,
  onCloseTab,
  onNextTab,
  onPrevTab,
}: KeyboardShortcutHandlers) {
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Focus guard: if focus is inside a terminal (.wterm), let keys pass through
      if (document.activeElement?.closest('.wterm')) {
        return
      }

      // We only care about Ctrl shortcuts (not Meta/Cmd per decision log)
      if (!e.ctrlKey || e.altKey || e.metaKey) {
        return
      }

      if (e.key === 't' && !e.shiftKey) {
        e.preventDefault()
        onNewTab()
      } else if (e.key === 'w' && !e.shiftKey) {
        e.preventDefault()
        onCloseTab()
      } else if (e.key === 'Tab') {
        e.preventDefault()
        if (e.shiftKey) {
          onPrevTab()
        } else {
          onNextTab()
        }
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [onNewTab, onCloseTab, onNextTab, onPrevTab])
}
