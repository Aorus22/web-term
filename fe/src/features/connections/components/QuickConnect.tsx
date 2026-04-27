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
import { Terminal, Plus } from 'lucide-react'
import { useAppStore } from '@/stores/app-store'

export const QuickConnect = () => {
  const [value, setValue] = React.useState('')
  const { data: connections = [] } = useConnections()
  const setCreatingConnection = useAppStore((state) => state.setCreatingConnection)

  const handleSelect = (selectedValue: string) => {
    console.log('Connecting to:', selectedValue)
    // Phase 3 will handle actual SSH connection
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
                  <span>{conn.label} ({conn.host})</span>
                </CommandItem>
              ))}
            </CommandGroup>
          )}
        </CommandList>
      </Command>
    </div>
  )
}
