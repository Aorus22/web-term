import { Button } from '@/components/ui/button'
import { RefreshCw, Upload, FolderPlus, Trash2, Edit2 } from 'lucide-react'
import { Tooltip, TooltipContent, TooltipTrigger, TooltipProvider } from '@/components/ui/tooltip'

interface DirectoryToolbarProps {
  onRefresh: () => void
  onUpload: () => void
  onNewFolder: () => void
  onDelete?: () => void
  onRename?: () => void
  canModify: boolean
  hasSelection: boolean
}

export function DirectoryToolbar({
  onRefresh,
  onUpload,
  onNewFolder,
  onDelete,
  onRename,
  canModify,
  hasSelection
}: DirectoryToolbarProps) {
  return (
    <TooltipProvider>
      <div className="flex items-center gap-1 p-1 border-b bg-muted/30">
        <Tooltip>
          <TooltipTrigger>
            <Button variant="ghost" size="icon" className="h-8 w-8" onClick={onRefresh}>
              <RefreshCw className="h-4 w-4" />
            </Button>
          </TooltipTrigger>
          <TooltipContent>Refresh</TooltipContent>
        </Tooltip>

        <div className="w-px h-4 bg-border mx-1" />

        <Tooltip>
          <TooltipTrigger>
            <Button variant="ghost" size="icon" className="h-8 w-8" onClick={onUpload} disabled={!canModify}>
              <Upload className="h-4 w-4" />
            </Button>
          </TooltipTrigger>
          <TooltipContent>Upload File</TooltipContent>
        </Tooltip>

        <Tooltip>
          <TooltipTrigger>
            <Button variant="ghost" size="icon" className="h-8 w-8" onClick={onNewFolder} disabled={!canModify}>
              <FolderPlus className="h-4 w-4" />
            </Button>
          </TooltipTrigger>
          <TooltipContent>New Folder</TooltipContent>
        </Tooltip>

        <div className="w-px h-4 bg-border mx-1" />

        <Tooltip>
          <TooltipTrigger>
            <Button variant="ghost" size="icon" className="h-8 w-8" onClick={onRename} disabled={!canModify || !hasSelection}>
              <Edit2 className="h-4 w-4" />
            </Button>
          </TooltipTrigger>
          <TooltipContent>Rename</TooltipContent>
        </Tooltip>

        <Tooltip>
          <TooltipTrigger>
            <Button variant="ghost" size="icon" className="h-8 w-8 text-destructive hover:text-destructive" onClick={onDelete} disabled={!canModify || !hasSelection}>
              <Trash2 className="h-4 w-4" />
            </Button>
          </TooltipTrigger>
          <TooltipContent>Delete</TooltipContent>
        </Tooltip>
      </div>
    </TooltipProvider>
  )
}
