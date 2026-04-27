import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { PanelLeft, Plus } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Separator } from '@/components/ui/separator'
import { ScrollArea } from '@/components/ui/scroll-area'
import { TooltipProvider } from '@/components/ui/tooltip'
import { useAppStore } from '@/stores/app-store'
import { cn } from '@/lib/utils'

import { QuickConnect } from '@/features/connections/components/QuickConnect'
import { TagFilter } from '@/features/connections/components/TagFilter'
import { ConnectionList } from '@/features/connections/components/ConnectionList'
import { ExportImport } from '@/features/connections/components/ExportImport'
import { ConnectionForm } from '@/features/connections/components/ConnectionForm'

const queryClient = new QueryClient()

function AppContent() {
  const { sidebarOpen, toggleSidebar, setCreatingConnection } = useAppStore()

  return (
    <div className="flex h-screen w-full bg-background text-foreground overflow-hidden">
      {/* Sidebar */}
      <aside
        className={cn(
          "flex flex-col border-r bg-muted/30 transition-all duration-300 ease-in-out",
          sidebarOpen ? "w-[280px]" : "w-0 overflow-hidden border-none"
        )}
      >
        <div className="p-4 flex items-center justify-between">
          <h2 className="font-semibold text-lg tracking-tight">WebTerm</h2>
          <Button variant="ghost" size="icon" onClick={() => setCreatingConnection(true)}>
            <Plus className="h-4 w-4" />
          </Button>
        </div>
        
        <div className="px-4 pb-4">
          <QuickConnect />
        </div>
        
        <Separator />
        
        <div className="py-2">
          <TagFilter />
        </div>
        
        <Separator />
        
        <ScrollArea className="flex-1">
          <ConnectionList />
        </ScrollArea>
        
        <Separator />
        
        <div className="p-4">
          <ExportImport />
        </div>
      </aside>

      {/* Main Area */}
      <main className="flex-1 flex flex-col min-w-0 bg-background">
        {/* Tab Bar / Header */}
        <header className="h-12 border-b flex items-center px-4 gap-4 bg-muted/10">
          {!sidebarOpen && (
            <Button variant="ghost" size="icon" onClick={toggleSidebar} className="h-8 w-8">
              <PanelLeft className="h-4 w-4" />
            </Button>
          )}
          {sidebarOpen && (
             <Button variant="ghost" size="icon" onClick={toggleSidebar} className="h-8 w-8">
                <PanelLeft className="h-4 w-4" />
             </Button>
          )}
          
          <div className="flex-1 flex items-center gap-2 overflow-x-auto no-scrollbar">
            {/* Tabs will go here in Phase 3 */}
            <div className="px-3 py-1 bg-background border rounded-t-md text-sm font-medium">
              Dashboard
            </div>
          </div>
        </header>

        {/* Content Area */}
        <div className="flex-1 flex items-center justify-center text-muted-foreground p-8">
          <div className="max-w-md text-center">
            <h3 className="text-lg font-medium text-foreground mb-2">No active sessions</h3>
            <p className="text-sm">
              Select a saved connection from the sidebar or use Quick Connect to start a new SSH session.
            </p>
          </div>
        </div>
      </main>

      {/* Overlays */}
      <ConnectionForm />
    </div>
  )
}

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <TooltipProvider>
        <AppContent />
      </TooltipProvider>
    </QueryClientProvider>
  )
}

export default App
