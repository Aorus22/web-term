import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { keysApi } from '@/lib/api'

export const useSSHKeys = () => {
  return useQuery({
    queryKey: ['ssh-keys'],
    queryFn: keysApi.list,
  })
}

export const useCreateSSHKey = () => {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: keysApi.create,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['ssh-keys'] })
    },
  })
}

export const useUpdateSSHKey = () => {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ id, data }: { id: string; data: { name: string } }) =>
      keysApi.update(id, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['ssh-keys'] })
    },
  })
}

export const useDeleteSSHKey = () => {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: keysApi.delete,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['ssh-keys'] })
    },
  })
}
