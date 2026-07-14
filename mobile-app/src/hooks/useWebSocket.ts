import { useState, useEffect, useRef, useCallback } from 'react'
import { useQueryClient } from '@tanstack/react-query'
import { API_BASE } from '../api'

interface WebSocketEvent {
  type: string
  data: any
  timestamp: Date
}

export function useWebSocket(queryKey: any[] = ['sightings']) {
  const queryClient = useQueryClient()
  const [isConnected, setIsConnected] = useState(false)
  const [lastEvent, setLastEvent] = useState<WebSocketEvent | null>(null)
  const wsRef = useRef<any>(null)
  const reconnectTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const reconnectAttemptsRef = useRef(0)
  const maxReconnectAttempts = 10
  const baseReconnectDelay = 1000

  const connect = useCallback(() => {
    if (wsRef.current?.readyState === 1) {
      return
    }

    const wsUrl = API_BASE.replace('http', 'ws') + '/ws/stream'
    const ws = new WebSocket(wsUrl)

    ws.onopen = () => {
      setIsConnected(true)
      reconnectAttemptsRef.current = 0
    }

    ws.onmessage = (event: any) => {
      try {
        const data = JSON.parse(event.data)
        if (data.type === 'SightingCreated') {
          queryClient.invalidateQueries({ queryKey })
        }
        setLastEvent({
          type: data.type,
          data,
          timestamp: new Date(),
        })
      } catch (e) {
        console.error('Failed to parse WebSocket message:', e)
      }
    }

    ws.onerror = () => {
      setIsConnected(false)
    }

    ws.onclose = () => {
      setIsConnected(false)
      wsRef.current = null

      if (reconnectAttemptsRef.current < maxReconnectAttempts) {
        const delay = baseReconnectDelay * Math.pow(2, reconnectAttemptsRef.current)
        reconnectTimeoutRef.current = setTimeout(() => {
          reconnectAttemptsRef.current++
          connect()
        }, Math.min(delay, 30000))
      }
    }

    wsRef.current = ws
  }, [queryKey, queryClient])

  const disconnect = useCallback(() => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current)
    }
    if (wsRef.current) {
      wsRef.current.close()
      wsRef.current = null
    }
    setIsConnected(false)
  }, [])

  useEffect(() => {
    connect()
    return () => disconnect()
  }, [connect, disconnect])

  return {
    isConnected,
    lastEvent,
    reconnect: connect,
    disconnect,
  }
}
