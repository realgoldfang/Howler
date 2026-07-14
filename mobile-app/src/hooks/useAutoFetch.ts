import { useState, useCallback } from 'react'
import { useQueryClient, useQuery } from '@tanstack/react-query'
import { fetchSightings } from '../api'
import { useNetworkStatus } from '../services/NetworkStatus'

export function useAutoFetch(queryKey: any[] = ['sightings']) {
  const queryClient = useQueryClient()
  const { isConnected, isInternetReachable } = useNetworkStatus()
  const [lastFetchTime, setLastFetchTime] = useState<Date | null>(null)

  const isOnline = isConnected && (isInternetReachable ?? true)

  const { data, isLoading, refetch } = useQuery({
    queryKey,
    queryFn: async () => {
      const result = await fetchSightings()
      setLastFetchTime(new Date())
      return result
    },
    refetchInterval: isOnline ? 30000 : false,
    refetchIntervalInBackground: false,
  })

  return {
    isOnline,
    lastFetchTime,
    refetch: useCallback(() => {
      if (isOnline) {
        refetch()
      }
    }, [isOnline, refetch]),
    data,
    isLoading,
  }
}
