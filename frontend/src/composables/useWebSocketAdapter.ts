// WebSocket Adapter - Bridge between old and new systems
// Allows gradual migration: old code still works, new code uses features/

import { onMounted, onUnmounted } from 'vue'
import { wsManager, type ConnectionState } from '../core/websocket/WebSocketManager'
import { registerWebSocketHandlers } from '../core/websocket/registerHandlers'

// Singleton to track if handlers are registered
let handlersRegistered = false

/**
 * Adapter composable that replaces the old useWebSocket
 * Maintains the same API but delegates to the new WebSocketManager
 * 
 * @deprecated Use feature-specific handlers from @/features/[feature] instead
 */
export function useWebSocket() {
  // Auto-register handlers on first use
  if (!handlersRegistered) {
    registerWebSocketHandlers()
    handlersRegistered = true
  }

  /**
   * Register an event listener (legacy API)
   * @deprecated Use wsManager.on() directly or feature handlers
   */
  function onEvent(event: string, callback: (data: any) => void): () => void {
    return wsManager.on(event, (wsEvent) => {
      // Unwrap the WebSocket event to match old API
      try {
        const data = JSON.parse(wsEvent.data)
        callback({ ...data, ...wsEvent.broadcast })
      } catch {
        callback(wsEvent.data)
      }
    })
  }

  /**
   * Send a message (legacy API)
   * @deprecated Use wsManager.send() directly
   */
  function send(message: object): boolean {
    return wsManager.send(message)
  }

  /**
   * Connect to WebSocket (legacy API)
   * @deprecated Use wsManager.connect() directly
   */
  function connect(url: string, token: string): void {
    wsManager.connect(url, token)
  }

  /**
   * Disconnect from WebSocket (legacy API)
   * @deprecated Use wsManager.disconnect() directly
   */
  function disconnect(): void {
    wsManager.disconnect()
  }

  return {
    // Reactive state
    connectionState: wsManager.state,
    connectionId: wsManager.connectionId,
    isConnected: wsManager.isConnected,

    // Actions
    connect,
    disconnect,
    send,
    onEvent,

    // New API (for gradual migration)
    $ws: wsManager
  }
}

/**
 * Composable for components that need reactive connection state
 */
export function useWebSocketConnection() {
  return {
    state: wsManager.state,
    isConnected: wsManager.isConnected,
    connectionId: wsManager.connectionId
  }
}

/**
 * Helper to register a one-time event listener
 * Automatically unsubscribes when component unmounts
 */
export function useWebSocketEvent(
  event: string,
  callback: (data: any) => void
) {
  let unsubscribe: (() => void) | null = null

  onMounted(() => {
    unsubscribe = wsManager.on(event, callback)
  })

  onUnmounted(() => {
    unsubscribe?.()
  })
}

// Re-export types
export type { ConnectionState }
