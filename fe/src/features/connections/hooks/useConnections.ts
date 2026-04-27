import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { connectionsApi } from '@/lib/api'
import type { Connection } from '@/lib/api'

export const useConnections = () => {
  return useQuery({
    queryKey: ['connections'],
    queryFn: connectionsApi.list,
  })
}

export const useCreateConnection = () => {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: connectionsApi.create,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['connections'] })
    },
  })
}

export const useUpdateConnection = () => {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ id, data }: { id: string; data: Partial<Connection> }) =>
      connectionsApi.update(id, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['connections'] })
    },
  })
}

export const useDeleteConnection = () => {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: connectionsApi.delete,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['connections'] })
    },
  })
}

export const useExportConnections = () => {
  return async () => {
    const blob = await connectionsApi.export()
    const url = window.URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `webterm-connections-${new Date().toISOString().split('T')[0]}.json`
    document.body.appendChild(a)
    a.click()
    window.URL.revokeObjectURL(url)
    document.body.removeChild(a)
  }
}

export const useImportConnections = () => {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: connectionsApi.import,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['connections'] })
    },
  })
}
