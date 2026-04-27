import { useState } from 'react'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { KeyRound } from 'lucide-react'

interface PasswordPromptProps {
  host: string
  username: string
  onConnect: (password: string) => void
  onCancel: () => void
}

/**
 * Inline password prompt for SSH connections without a stored password.
 * Per D-02/D-03: displayed inline in terminal pane area, NOT a modal.
 */
export function PasswordPrompt({ host, username, onConnect, onCancel }: PasswordPromptProps) {
  const [password, setPassword] = useState('')

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (password.trim()) {
      onConnect(password)
    }
  }

  return (
    <div className="flex items-center justify-center h-full text-muted-foreground p-8">
      <form onSubmit={handleSubmit} className="max-w-sm w-full space-y-4">
        <div className="text-center space-y-2">
          <KeyRound className="h-8 w-8 mx-auto text-muted-foreground/50" />
          <h3 className="text-lg font-medium text-foreground">
            Connect to {username}@{host}
          </h3>
          <p className="text-sm text-muted-foreground">
            Enter the SSH password for this connection.
          </p>
        </div>
        <Input
          type="password"
          placeholder="Password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          autoFocus
          className="text-center"
        />
        <div className="flex gap-2 justify-center">
          <Button type="button" variant="ghost" onClick={onCancel}>
            Cancel
          </Button>
          <Button type="submit" disabled={!password.trim()}>
            Connect
          </Button>
        </div>
      </form>
    </div>
  )
}
