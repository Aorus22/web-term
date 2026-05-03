import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from "@/components/ui/resizable"
import { DirectoryBrowser } from "./DirectoryBrowser"

export function SFTPView() {
  return (
    <div className="flex-1 p-4 overflow-hidden">
      <ResizablePanelGroup orientation="horizontal" className="h-full gap-2">
        <ResizablePanel defaultSize={50} minSize={20}>
          <DirectoryBrowser />
        </ResizablePanel>
        
        <ResizableHandle withHandle className="bg-transparent hover:bg-accent/50 w-2 transition-colors rounded-full" />
        
        <ResizablePanel defaultSize={50} minSize={20}>
          <DirectoryBrowser />
        </ResizablePanel>
      </ResizablePanelGroup>
    </div>
  )
}
