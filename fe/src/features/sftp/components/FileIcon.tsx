import { 
  Folder, 
  File, 
  FileImage, 
  FileArchive, 
  FileCode, 
  FileText,
  FileAudio,
  FileVideo,
  FileJson,
  type LucideIcon
} from 'lucide-react'

interface FileIconProps {
  name: string
  isDir: boolean
  className?: string
}

export function FileIcon({ name, isDir, className }: FileIconProps) {
  if (isDir) {
    return <Folder className={`h-4 w-4 text-blue-500 fill-blue-500/20 ${className || ''}`} />
  }

  const ext = name.split('.').pop()?.toLowerCase() || ''

  let Icon: LucideIcon = File
  let iconColor = 'text-muted-foreground'

  switch (ext) {
    case 'jpg':
    case 'jpeg':
    case 'png':
    case 'gif':
    case 'svg':
    case 'webp':
    case 'ico':
      Icon = FileImage
      iconColor = 'text-purple-500'
      break
    case 'zip':
    case 'tar':
    case 'gz':
    case 'rar':
    case '7z':
      Icon = FileArchive
      iconColor = 'text-amber-600'
      break
    case 'js':
    case 'ts':
    case 'tsx':
    case 'jsx':
    case 'go':
    case 'py':
    case 'rs':
    case 'c':
    case 'cpp':
    case 'java':
    case 'php':
    case 'rb':
    case 'sh':
    case 'bat':
      Icon = FileCode
      iconColor = 'text-blue-600'
      break
    case 'json':
    case 'yaml':
    case 'yml':
    case 'toml':
    case 'xml':
      Icon = FileJson
      iconColor = 'text-orange-500'
      break
    case 'txt':
    case 'md':
    case 'log':
    case 'conf':
    case 'ini':
    case 'env':
      Icon = FileText
      iconColor = 'text-slate-500'
      break
    case 'mp3':
    case 'wav':
    case 'ogg':
    case 'flac':
      Icon = FileAudio
      iconColor = 'text-pink-500'
      break
    case 'mp4':
    case 'mov':
    case 'avi':
    case 'mkv':
      Icon = FileVideo
      iconColor = 'text-red-500'
      break
  }

  return <Icon className={`h-4 w-4 ${iconColor} ${className || ''}`} />
}
