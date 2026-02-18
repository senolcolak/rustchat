// WebSocket Module - Public API
// Central export for all WebSocket-related functionality

// Core manager
export { 
  wsManager, 
  useWebSocket as useWebSocketCore, 
  type ConnectionState 
} from './WebSocketManager'

// Handler registration
export { 
  registerWebSocketHandlers, 
  unregisterWebSocketHandlers 
} from './registerHandlers'

// Adapter for legacy code
export { 
  useWebSocket, 
  useWebSocketConnection, 
  useWebSocketEvent 
} from '../../composables/useWebSocketAdapter'
