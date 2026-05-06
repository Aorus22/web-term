import { useState } from 'react'
import { useAppStore } from '@/stores/app-store'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Trash2, Copy, Search, Clipboard } from 'lucide-react'

export function ClipboardView() {
  const { clipboardHistory, clearClipboardHistory } = useAppStore()
  const [searchQuery, setSearchQuery] = useState('')

  const filteredHistory = searchQuery
    ? clipboardHistory.filter((entry) =>
        entry.content.toLowerCase().includes(searchQuery.toLowerCase())
      )
    : clipboardHistory

  const copyToLocal = async (content: string) => {
    try {
      await navigator.clipboard.writeText(content)
    } catch (err) {
      console.error('Failed to copy to local clipboard:', err)
    }
  }

  const formatTime = (timestamp: number) => {
    const date = new Date(timestamp)
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
  }

  const truncateContent = (content: string, maxLength: number = 150) => {
    if (content.length <= maxLength) return content
    return content.substring(0, maxLength) + '...'
  }

  return (
    <div className="flex flex-col h-full p-4 gap-4">
      <div className="flex items-center gap-2">
        <Clipboard className="h-5 w-5" />
        <h2 className="text-lg font-semibold">Clipboard History</h2>
      </div>

      <div className="flex gap-2">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder="Search clipboard history..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="pl-9"
          />
        </div>
        <Button
          variant="outline"
          size="icon"
          onClick={clearClipboardHistory}
          title="Clear history"
        >
          <Trash2 className="h-4 w-4" />
        </Button>
      </div>

      <ScrollArea className="flex-1">
        <div className="space-y-2">
          {filteredHistory.length === 0 ? (
            <div className="text-center text-muted-foreground py-8">
              <Clipboard className="h-8 w-8 mx-auto mb-2 opacity-50" />
              <p>No clipboard history yet</p>
              <p className="text-sm">Clipboard entries from SSH sessions will appear here</p>
            </div>
          ) : (
            filteredHistory.map((entry) => (
              <Card key={entry.id} className="cursor-pointer hover:bg-accent/50">
                <CardContent className="p-3">
                  <div className="flex items-start justify-between gap-2">
                    <div className="flex-1 min-w-0">
                      <pre className="text-sm whitespace-pre-wrap break-all font-mono">
                        {truncateContent(entry.content)}
                      </pre>
                      <p className="text-xs text-muted-foreground mt-1">
                        {formatTime(entry.timestamp)}
                      </p>
                    </div>
                    <Button
                      variant="ghost"
                      size="icon"
                      onClick={() => copyToLocal(entry.content)}
                      title="Copy to local clipboard"
                    >
                      <Copy className="h-4 w-4" />
                    </Button>
                  </div>
                </CardContent>
              </Card>
            ))
          )}
        </div>
      </ScrollArea>
    </div>
  )
}