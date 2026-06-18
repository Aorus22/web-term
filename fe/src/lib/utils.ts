import { clsx, type ClassValue } from "clsx"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

export function generateId(): string {
  if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
    return crypto.randomUUID()
  }
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
    const r = (Math.random() * 16) | 0
    const v = c === 'x' ? r : (r & 0x3) | 0x8
    return v.toString(16)
  })
}

/**
 * Join a parent path with a child name, using the correct separator for the
 * path's platform. Windows paths (drive letter or UNC) use '\\'; everything
 * else uses '/'.
 */
export function joinPath(parent: string, name: string): string {
  if (!parent || parent === '/' || parent === '.') return `/${name}`
  const isWindows = /^[A-Za-z]:[\\/]/.test(parent) || parent.startsWith('\\\\')
  if (isWindows) {
    const clean = parent.replace(/[\\/]+$/, '')
    return `${clean}\\${name}`
  }
  const clean = parent.replace(/\/+$/, '')
  return `${clean}/${name}`
}

/**
 * Find an unused name in the form "name (N).ext" / "name (N)" for a duplicate.
 * Increments N until a free slot is found. Handles:
 *   - multiple dots (archive.tar.gz → archive.tar (1).gz)
 *   - no extension (Makefile → Makefile (1))
 *   - hidden files starting with '.' (.gitignore → .gitignore (1))
 */
export function findUniqueName(fileName: string, existingNames: string[]): string {
  const existing = new Set(existingNames)
  const isHidden = fileName.startsWith('.') && fileName.indexOf('.') === fileName.lastIndexOf('.')
  if (isHidden) {
    let i = 1
    while (existing.has(`${fileName} (${i})`)) i++
    return `${fileName} (${i})`
  }
  const lastDot = fileName.lastIndexOf('.')
  const hasExt = lastDot > 0
  const base = hasExt ? fileName.slice(0, lastDot) : fileName
  const ext = hasExt ? fileName.slice(lastDot) : ''
  let i = 1
  while (existing.has(`${base} (${i})${ext}`)) i++
  return `${base} (${i})${ext}`
}
