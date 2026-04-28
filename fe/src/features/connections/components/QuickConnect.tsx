import * as React from 'react'
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from '@/components/ui/command'
import { useConnections } from '../hooks/useConnections'
import { Terminal, Plus, Key } from 'lucide-react'
import { useAppStore } from '@/stores/app-store'
import { generateId } from '@/lib/utils'
import type { SSHSession } from '@/features/terminal/types'

export const QuickConnect = () => {
  const [value, setValue] = React.useState('')
  const { data: connections = [] } = useConnections()
  const { setCreatingConnection, addSession } = useAppStore()

  const handleSelect = (selectedValue: string) => {
    // Check if this is a saved connection (selected from dropdown)
    const savedConn = connections.find(
      (c) => `${c.username}@${c.host}:${c.port}` === selectedValue
    )
    if (savedConn) {
      // Create new session (allow multiple connections to same host)
      const sessionId = generateId()
      const session: SSHSession = {
        id: sessionId,
        connectionId: savedConn.id,
        host: savedConn.host,
        port: savedConn.port,
        username: savedConn.username,
        label: savedConn.label,
        status: 'connecting',
        isQuickConnect: false,
      }
      addSession(session)
      setValue('')
      return
    }

    // Quick-connect: parse user@host[:port] (D-03)
    const atIndex = selectedValue.indexOf('@')
    if (atIndex < 0) return
    const username = selectedValue.slice(0, atIndex)
    const hostPort = selectedValue.slice(atIndex + 1)
    const colonIndex = hostPort.indexOf(':')
    const host = colonIndex >= 0 ? hostPort.slice(0, colonIndex) : hostPort
    const port = colonIndex >= 0 ? parseInt(hostPort.slice(colonIndex + 1), 10) : 22

    // Validate parsed input (T-03-09 mitigation)
    if (!host.trim() || !username.trim() || host.includes(' ') || username.includes(' ')) {
      return
    }
    if (isNaN(port) || port < 1 || port > 65535) {
      return
    }

    const sessionId = generateId()
    const session: SSHSession = {
      id: sessionId,
      host: host.trim(),
      port,
      username: username.trim(),
      label: `${username.trim()}@${host.trim()}`,
      status: 'connecting',
      isQuickConnect: true,
    }
    addSession(session)
    setValue('')
  }

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && value.includes('@')) {
      handleSelect(value)
    }
  }

  return (
    <div className="pb-2">
      <Command className="rounded-lg border bg-muted/50 shadow-md">
        <CommandInput 
          placeholder="Search or user@host..." 
          value={value}
          onValueChange={setValue}
          onKeyDown={handleKeyDown}
        />
        <CommandList className="max-h-[200px]">
          <CommandEmpty>No results found.</CommandEmpty>
          <CommandGroup heading="New Connection">
            <CommandItem 
              onSelect={() => setCreatingConnection(true)}
              className="cursor-pointer"
            >
              <Plus className="mr-2 h-4 w-4" />
              <span>Create New...</span>
            </CommandItem>
          </CommandGroup>
          {connections.length > 0 && (
            <CommandGroup heading="Saved Connections">
              {connections.map((conn) => (
                <CommandItem
                  key={conn.id}
                  onSelect={() => handleSelect(`${conn.username}@${conn.host}:${conn.port}`)}
                  className="cursor-pointer"
                >
                  <Terminal className="mr-2 h-4 w-4" />
                  <span className="flex items-center gap-2">
                    {conn.ssh_key_id && <Key className="h-3 w-3 text-muted-foreground" />}
                    {conn.label} ({conn.host})
                  </span>
                </CommandItem>
              ))}
            </CommandGroup>
          )}
        </CommandList>
      </Command>
    </div>
  )
}
