import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from "@/components/ui/resizable"
import { DirectoryBrowser } from "./DirectoryBrowser"
import { TransferManager } from "./TransferManager"

export function SFTPView() {
  return (
    <div className="flex-1 overflow-hidden relative">
      <ResizablePanelGroup orientation="horizontal" className="h-full">
        <ResizablePanel defaultSize={50} minSize={20}>
          <DirectoryBrowser panelId="left" />
        </ResizablePanel>
        
        <ResizableHandle withHandle className="bg-border hover:bg-border/70 w-[1px] transition-colors" />
        
        <ResizablePanel defaultSize={50} minSize={20}>
          <DirectoryBrowser panelId="right" />
        </ResizablePanel>
      </ResizablePanelGroup>
      
      <TransferManager />
    </div>
  )
}
