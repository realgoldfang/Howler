import { useState, useEffect, useCallback } from 'react'
import { useQuery } from '@tanstack/react-query'
import { sightingsApi } from '../api'

export function useAutoFetch(queryKey: string[] = ['sightings', 'dashboard']) {
  const [isOnline, setIsOnline] = useState(navigator.onLine)
  const [lastFetchTime, setLastFetchTime] = useState<Date | null>(null)

  useEffect(() => {
    const handleOnline = () => setIsOnline(true)
    const handleOffline = () => setIsOnline(false)

    window.addEventListener('online', handleOnline)
    window.addEventListener('offline', handleOffline)

    return () => {
      window.removeEventListener('online', handleOnline)
      window.removeEventListener('offline', handleOffline)
    }
  }, [])

  const { data, isLoading, refetch } = useQuery({
    queryKey,
    queryFn: async () => {
      const result = await sightingsApi.list({ per_page: 50, sort_by: 'timestamp', sort_dir: 'desc' })
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
