import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { forwardsApi } from '@/lib/api'

export const useForwards = () => {
  return useQuery({
    queryKey: ['forwards'],
    queryFn: forwardsApi.list,
  })
}

export const useCreateForward = () => {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: forwardsApi.create,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['forwards'] })
    },
  })
}

export const useDeleteForward = () => {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: forwardsApi.delete,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['forwards'] })
    },
  })
}

export const useStartForward = () => {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: forwardsApi.start,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['forwards'] })
    },
  })
}

export const useStopForward = () => {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: forwardsApi.stop,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['forwards'] })
    },
  })
}
