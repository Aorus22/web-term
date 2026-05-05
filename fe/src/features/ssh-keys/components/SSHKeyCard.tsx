import { Card, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
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
  onEdit: (sshKey: SSHKey) => void
  onDelete: (sshKey: SSHKey) => void
}

export const SSHKeyCard = ({ sshKey, onEdit, onDelete }: SSHKeyCardProps) => {
  return (
    <Card className="group relative overflow-hidden transition-all border-border/50 hover:border-primary/50 hover:shadow-sm">
      <div className="p-4 flex flex-row items-center gap-4">
        <div className="flex-1 min-w-0">
          <CardTitle className="text-sm font-bold truncate" title={sshKey.name}>
            {sshKey.name}
          </CardTitle>
          {sshKey.has_passphrase && (
            <Badge variant="outline" className="text-[9px] px-1.5 py-0 h-4 font-normal text-primary border-primary/20 mt-1.5">
              Encrypted
            </Badge>
          )}
          <p className="text-[10px] text-muted-foreground truncate mt-1" title={sshKey.fingerprint}>
            {sshKey.fingerprint}
          </p>
        </div>
        
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
