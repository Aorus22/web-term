import { Card } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { 
  DropdownMenu, 
  DropdownMenuContent, 
  DropdownMenuItem, 
  DropdownMenuTrigger 
} from '@/components/ui/dropdown-menu'
import { Key, MoreVertical, Edit2, Trash2 } from 'lucide-react'
import type { SSHKey } from '@/lib/api'

interface SSHKeyCardProps {
  sshKey: SSHKey
  onEdit: (sshKey: SSHKey) => void
  onDelete: (sshKey: SSHKey) => void
}

export const SSHKeyCard = ({ sshKey, onEdit, onDelete }: SSHKeyCardProps) => {
  return (
    <Card className="group relative overflow-hidden transition-all border-border/40 hover:border-primary/40 hover:shadow-md py-0 gap-0">
      <div className="flex items-center py-4 gap-4">
        {/* Left: Icon */}
        <div className="flex-shrink-0 w-8 h-8 rounded-md flex items-center justify-center bg-muted/50">
          <Key className="h-4 w-4 text-muted-foreground/70" />
        </div>

        {/* Middle: Info */}
        <div className="flex-1 min-w-0 py-0">
          <div className="flex items-center gap-2">
            <h3 className="text-base font-bold truncate leading-tight" title={sshKey.name}>
              {sshKey.name}
            </h3>
            {sshKey.has_passphrase && (
              <span className="inline-flex items-center rounded-full px-2 py-0.5 text-[10px] font-medium bg-primary/10 text-primary">
                Encrypted
              </span>
            )}
          </div>
          <p className="text-xs text-muted-foreground truncate leading-none mt-1" title={sshKey.fingerprint}>
            {sshKey.fingerprint}
          </p>
        </div>

        {/* Right: Kebab */}
        <div className="flex-shrink-0">
          <DropdownMenu>
            <DropdownMenuTrigger
              render={
                <Button variant="ghost" size="icon" className="h-7 w-7 rounded-md opacity-0 group-hover:opacity-100 transition-opacity">
                  <MoreVertical className="h-4 w-4 text-muted-foreground" />
                </Button>
              }
            />
            <DropdownMenuContent align="end" className="w-40">
              <DropdownMenuItem onClick={() => onEdit(sshKey)}>
                <Edit2 className="mr-2 h-3.5 w-3.5" /> Edit
              </DropdownMenuItem>
              <DropdownMenuItem onClick={() => onDelete(sshKey)} className="text-destructive focus:bg-destructive/10">
                <Trash2 className="mr-2 h-3.5 w-3.5" /> Delete
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </div>
    </Card>
  )
}
