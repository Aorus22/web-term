import { useEffect, useRef, useCallback } from 'react'
import type { WTerm } from '@wterm/dom'

interface MouseMode {
  basic: boolean
  button: boolean
  anyMotion: boolean
  sgr: boolean
}

const MOTION = 32
const SCROLL_UP = 64
const SCROLL_DOWN = 65

function parseMouseModes(str: string, mode: MouseMode) {
  const re = /\x1b\[\?([\d;]+)([hl])/g
  let m
  while ((m = re.exec(str)) !== null) {
    const enable = m[2] === 'h'
    for (const p of m[1].split(';').map(Number)) {
      if (p === 1000) mode.basic = enable
      else if (p === 1002) mode.button = enable
      else if (p === 1003) mode.anyMotion = enable
      else if (p === 1006) mode.sgr = enable
    }
  }
}

function isTracking(m: MouseMode) {
  return m.basic || m.button || m.anyMotion
}

export function useTerminalMouse(sendData: (data: string) => void) {
  const sendDataRef = useRef(sendData)
  sendDataRef.current = sendData
  const modeRef = useRef<MouseMode>({ basic: false, button: false, anyMotion: false, sgr: false })
  const cleanupRef = useRef<(() => void) | null>(null)

  const onReady = useCallback((wt: WTerm) => {
    const el = wt.element
    const originalWrite = wt.write.bind(wt)
    const mode = modeRef.current

    wt.write = (data: string | Uint8Array) => {
      const str = typeof data === 'string' ? data : new TextDecoder().decode(data)
      parseMouseModes(str, mode)
      originalWrite(data)
    }

    const cellFromEvent = (e: MouseEvent) => {
      const bridge = wt.bridge
      if (!bridge) return null
      const rect = el.getBoundingClientRect()
      const cs = getComputedStyle(el)
      const padLeft = parseFloat(cs.paddingLeft) || 0
      const padTop = parseFloat(cs.paddingTop) || 0
      const padRight = parseFloat(cs.paddingRight) || 0
      const cols = bridge.getCols()
      const rows = bridge.getRows()
      const cw = (rect.width - padLeft - padRight) / cols
      const rh = (wt as any)._rowHeight || 17
      const col = Math.floor((e.clientX - rect.left - padLeft) / cw) + 1
      const row = Math.floor((e.clientY - rect.top - padTop) / rh) + 1
      if (col < 1 || col > cols || row < 1 || row > rows) return null
      return { col, row }
    }

    const x10 = (b: number, col: number, row: number) =>
      `\x1b[M${String.fromCharCode(32 + b)}${String.fromCharCode(32 + col)}${String.fromCharCode(32 + row)}`

    const sgr = (b: number, col: number, row: number, rel: boolean) =>
      `\x1b[<${b};${col};${row}${rel ? 'm' : 'M'}`

    const onDown = (e: MouseEvent) => {
      if (!isTracking(mode)) return
      e.preventDefault()
      e.stopPropagation()
      wt.focus()
      const pos = cellFromEvent(e)
      if (!pos || e.button > 2) return
      sendDataRef.current(
        mode.sgr ? sgr(e.button, pos.col, pos.row, false) : x10(e.button, pos.col, pos.row),
      )
    }

    const onUp = (e: MouseEvent) => {
      if (!isTracking(mode)) return
      e.preventDefault()
      const pos = cellFromEvent(e)
      if (!pos || e.button > 2) return
      if (mode.sgr) sendDataRef.current(sgr(e.button, pos.col, pos.row, true))
    }

    let lastCol = -1
    let lastRow = -1

    const onMove = (e: MouseEvent) => {
      if (!mode.anyMotion && !mode.button) return
      if (!mode.anyMotion && !(e.buttons & 7)) return
      const pos = cellFromEvent(e)
      if (!pos || (pos.col === lastCol && pos.row === lastRow)) return
      lastCol = pos.col
      lastRow = pos.row
      let btn: number
      if (e.buttons & 1) btn = 0
      else if (e.buttons & 4) btn = 1
      else if (e.buttons & 2) btn = 2
      else btn = 3
      const cb = btn + MOTION
      sendDataRef.current(mode.sgr ? sgr(cb, pos.col, pos.row, false) : x10(cb, pos.col, pos.row))
    }

    const onWheel = (e: WheelEvent) => {
      if (!isTracking(mode)) return
      e.preventDefault()
      e.stopPropagation()
      const pos = cellFromEvent(e)
      if (!pos) return
      const b = e.deltaY < 0 ? SCROLL_UP : SCROLL_DOWN
      if (mode.sgr) {
        sendDataRef.current(sgr(b, pos.col, pos.row, false))
        sendDataRef.current(sgr(b, pos.col, pos.row, true))
      } else {
        sendDataRef.current(x10(b, pos.col, pos.row))
      }
    }

    el.addEventListener('mousedown', onDown, true)
    el.addEventListener('mouseup', onUp, true)
    el.addEventListener('mousemove', onMove, true)
    el.addEventListener('wheel', onWheel, { capture: true, passive: false })

    cleanupRef.current = () => {
      el.removeEventListener('mousedown', onDown, true)
      el.removeEventListener('mouseup', onUp, true)
      el.removeEventListener('mousemove', onMove, true)
      el.removeEventListener('wheel', onWheel, true)
      wt.write = originalWrite
      mode.basic = mode.button = mode.anyMotion = mode.sgr = false
    }
  }, [])

  useEffect(
    () => () => {
      cleanupRef.current?.()
      cleanupRef.current = null
    },
    [],
  )

  return { onReady }
}
