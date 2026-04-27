import { useConnections } from '../hooks/useConnections'
import { Badge } from '@/components/ui/badge'
import { useAppStore } from '@/stores/app-store'
import { X } from 'lucide-react'

export const TagFilter = () => {
  const { data: connections = [] } = useConnections()
  const { selectedTags, toggleTag, clearTags } = useAppStore()

  const allTags = Array.from(
    new Set(connections.flatMap((c) => c.tags || []))
  ).sort()

  if (allTags.length === 0) return null

  return (
    <div className="px-4 py-2 space-y-2">
      <div className="flex items-center justify-between text-xs text-muted-foreground">
        <span>Filter by tags</span>
        {selectedTags.length > 0 && (
          <button 
            onClick={clearTags}
            className="flex items-center hover:text-foreground transition-colors"
          >
            Clear <X className="ml-1 h-3 w-3" />
          </button>
        )}
      </div>
      <div className="flex flex-wrap gap-1.5">
        {allTags.map((tag) => (
          <Badge
            key={tag}
            variant={selectedTags.includes(tag) ? 'default' : 'outline'}
            className="cursor-pointer transition-all hover:bg-accent hover:text-accent-foreground"
            onClick={() => toggleTag(tag)}
          >
            {tag}
          </Badge>
        ))}
      </div>
    </div>
  )
}
