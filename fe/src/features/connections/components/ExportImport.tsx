import * as React from 'react'
import { Button } from '@/components/ui/button'
import { Separator } from '@/components/ui/separator'
import { Download, Upload } from 'lucide-react'
import { useExportConnections, useImportConnections } from '../hooks/useConnections'

export const ExportImport = () => {
  const exportConnections = useExportConnections()
  const importMutation = useImportConnections()
  const fileInputRef = React.useRef<HTMLInputElement>(null)

  const handleImport = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]
    if (!file) return

    const reader = new FileReader()
    reader.onload = async (event) => {
      try {
        const content = event.target?.result as string
        const data = JSON.parse(content)
        const result = await importMutation.mutateAsync(data)
        alert(`Import complete: ${result.imported} imported, ${result.skipped} skipped.`)
      } catch (err) {
        console.error('Import failed:', err)
        alert('Failed to import connections. Ensure the file is valid JSON.')
      }
    }
    reader.readAsText(file)
    if (fileInputRef.current) fileInputRef.current.value = ''
  }

  return (
    <div className="mt-auto pt-2 pb-4 space-y-2 px-2">
      <Separator className="mb-2" />
      <div className="grid grid-cols-2 gap-2">
        <Button 
          variant="outline" 
          size="sm" 
          className="h-8 text-xs"
          onClick={exportConnections}
        >
          <Download className="mr-2 h-3 w-3" /> Export
        </Button>
        <Button 
          variant="outline" 
          size="sm" 
          className="h-8 text-xs"
          onClick={() => fileInputRef.current?.click()}
          disabled={importMutation.isPending}
        >
          <Upload className="mr-2 h-3 w-3" /> Import
        </Button>
      </div>
      <input
        type="file"
        ref={fileInputRef}
        onChange={handleImport}
        accept=".json"
        className="hidden"
      />
    </div>
  )
}
