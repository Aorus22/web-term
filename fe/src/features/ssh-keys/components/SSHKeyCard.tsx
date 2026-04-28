import React, { useState } from 'react'
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { 
  DropdownMenu, 
  DropdownMenuContent, 
  DropdownMenuItem, 
  DropdownMenuTrigger 
} from '@/components/ui/dropdown-menu'
import { MoreVertical, Edit2, Trash2 } from 'lucide-react'
import type { SSHKey } from '@/lib/api'

interface SSHKeyCardProps {
  sshKey: SSHKey
  onRename: (id: string, name: string) => Promise<void>
  onDelete: (sshKey: SSHKey) => void
}

export const SSHKeyCard = ({ sshKey, onRename, onDelete }: SSHKeyCardProps) => {
  const [isEditing, setIsEditing] = useState(false)
  const [editName, setEditName] = useState(sshKey.name)

  const handleRename = async () => {
    if (editName.trim() && editName !== sshKey.name) {
      await onRename(sshKey.id, editName)
    }
    setIsEditing(false)
  }

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') handleRename()
    if (e.key === 'Escape') {
      setEditName(sshKey.name)
      setIsEditing(false)
    }
  }

  return (
    <Card className="group relative overflow-hidden transition-all hover:shadow-md">
      <CardHeader className="p-4 pb-2 flex flex-row items-center justify-between space-y-0">
        {isEditing ? (
          <Input
            value={editName}
            onChange={(e) => setEditName(e.target.value)}
            onBlur={handleRename}
            onKeyDown={handleKeyDown}
            autoFocus
            className="h-7 text-sm py-0"
          />
        ) : (
          <CardTitle className="text-sm font-bold truncate pr-6" title={sshKey.name}>
            {sshKey.name}
          </CardTitle>
        )}
        
        <div className="absolute right-2 top-2">
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="ghost" size="icon" className="h-8 w-8 opacity-0 group-hover:opacity-100 transition-opacity">
                <MoreVertical className="h-4 w-4" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuItem onClick={() => setIsEditing(true)}>
                <Edit2 className="mr-2 h-4 w-4" /> Rename
              </DropdownMenuItem>
              <DropdownMenuItem onClick={() => onDelete(sshKey)} className="text-destructive">
                <Trash2 className="mr-2 h-4 w-4" /> Delete
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </CardHeader>
      
      <CardContent className="p-4 pt-0">
        <div className="flex flex-col gap-2">
          <div className="flex items-center gap-2">
            <Badge variant="outline" className="text-[10px] px-1 py-0 h-4">
              {sshKey.key_type}
            </Badge>
            {sshKey.has_passphrase && (
              <Badge variant="secondary" className="text-[10px] px-1 py-0 h-4">
                Encrypted
              </Badge>
            )}
          </div>
          <p className="text-[10px] text-muted-foreground font-mono truncate" title={sshKey.fingerprint}>
            {sshKey.fingerprint}
          </p>
        </div>
      </CardContent>
    </Card>
  )
}
