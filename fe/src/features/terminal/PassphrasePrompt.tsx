import { useState } from 'react'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { LockKeyhole } from 'lucide-react'

interface PassphrasePromptProps {
  keyName: string        // SSH key name to display
  keyType: string        // RSA/Ed25519/ECDSA for badge
  host: string           // target host for context
  onConnect: (passphrase: string) => void
  onCancel: () => void
}

/**
 * Inline passphrase prompt for SSH key-based authentication.
 * Per D-05, D-06: displayed inline in terminal pane area, NOT a modal.
 */
export function PassphrasePrompt({ 
  keyName, 
  keyType, 
  host, 
  onConnect, 
  onCancel 
}: PassphrasePromptProps) {
  const [passphrase, setPassphrase] = useState('')

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    // Passphrase could theoretically be empty if the user just hits enter, 
    // but usually keys with passphrases require one.
    // D-06 says Connect button disabled when empty.
    if (passphrase) {
      onConnect(passphrase)
    }
  }

  return (
    <div className="flex items-center justify-center h-full text-muted-foreground p-8">
      <form onSubmit={handleSubmit} className="max-w-sm w-full space-y-4">
        <div className="text-center space-y-2">
          <LockKeyhole className="h-8 w-8 mx-auto text-muted-foreground/50" />
          <h3 className="text-lg font-medium text-foreground">
            SSH Key Authentication
          </h3>
          <p className="text-sm text-muted-foreground">
            Connecting to {host}
          </p>
          <div className="flex items-center justify-center gap-2 pt-2">
            <span className="text-sm font-medium text-foreground">{keyName}</span>
            <Badge variant="secondary" className="text-[10px] px-1.5 py-0">
              {keyType}
            </Badge>
          </div>
        </div>
        <Input
          type="password"
          placeholder="Enter passphrase"
          value={passphrase}
          onChange={(e) => setPassphrase(e.target.value)}
          autoFocus
          className="text-center"
        />
        <div className="flex gap-2 justify-center">
          <Button type="button" variant="ghost" onClick={onCancel}>
            Cancel
          </Button>
          <Button type="submit" disabled={!passphrase}>
            Connect
          </Button>
        </div>
      </form>
    </div>
  )
}
