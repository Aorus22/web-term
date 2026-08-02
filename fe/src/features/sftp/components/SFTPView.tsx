import { useQuery } from "@tanstack/react-query"
import { Gauge } from "lucide-react"
import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from "@/components/ui/resizable"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { sftpApi } from "@/lib/api"
import { useAppStore } from "@/stores/app-store"
import { useSettings } from "@/features/settings/hooks/useSettings"
import { DirectoryBrowser } from "./DirectoryBrowser"
import { TransferManager } from "./TransferManager"

export function SFTPView() {
  const { sftpLeftPanel, sftpRightPanel, fileEngine, setFileEngine } = useAppStore()
  const { data: settings } = useSettings()

  // Rsync availability is only known once the engines query for EACH panel
  // connection has returned. While probing (undefined data) we keep rsync
  // disabled; it becomes usable only when both sides report it and the
  // rsync_enabled setting is 'true'.
  const { data: enginesForLeft } = useQuery({
    queryKey: ['engines', sftpLeftPanel.connectionId],
    queryFn: () => sftpApi.engines(sftpLeftPanel.connectionId),
  })
  const { data: enginesForRight } = useQuery({
    queryKey: ['engines', sftpRightPanel.connectionId],
    queryFn: () => sftpApi.engines(sftpRightPanel.connectionId),
  })

  const rsyncViewAvailable =
    settings?.rsync_enabled === 'true' &&
    enginesForLeft?.available.includes('rsync') === true &&
    enginesForRight?.available.includes('rsync') === true

  const handleEngineChange = (v: string | null) => {
    if (v === 'rsync' && !rsyncViewAvailable) return // keep current value when disabled
    if (v === 'sftp' || v === 'rsync') setFileEngine(v)
  }

  return (
    <div className="flex flex-col flex-1 overflow-hidden relative">
      {/* Transfer engine selector */}
      <div className="h-10 border-b px-3 flex items-center justify-end gap-2 bg-muted/30 shrink-0">
        <Select value={fileEngine} onValueChange={handleEngineChange}>
          <SelectTrigger className="h-8 gap-1.5 text-xs" aria-label="Transfer engine">
            <Gauge className="size-3.5 text-muted-foreground" />
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="sftp">SFTP</SelectItem>
            <SelectItem value="rsync" disabled={!rsyncViewAvailable}>Rsync</SelectItem>
          </SelectContent>
        </Select>
      </div>

      <div className="flex-1 relative min-h-0">
        <ResizablePanelGroup orientation="horizontal" className="h-full">
          <ResizablePanel defaultSize={50} minSize={20}>
            <DirectoryBrowser panelId="left" engine={fileEngine} />
          </ResizablePanel>

          <ResizableHandle withHandle className="bg-border hover:bg-border/70 w-[1px] transition-colors" />

          <ResizablePanel defaultSize={50} minSize={20}>
            <DirectoryBrowser panelId="right" engine={fileEngine} />
          </ResizablePanel>
        </ResizablePanelGroup>
      </div>

      <TransferManager />
    </div>
  )
}
