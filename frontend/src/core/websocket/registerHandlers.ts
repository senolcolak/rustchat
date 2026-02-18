// WebSocket Handler Registration
// Central registration of all feature-specific WebSocket handlers

import { wsManager } from './WebSocketManager'
import { handleMessageWebSocketEvent } from '../../features/messages'
import { handleCallWebSocketEvent } from '../../features/calls'
import { handleChannelWebSocketEvent } from '../../features/channels'

/**
 * Register all WebSocket event handlers
 * Call this once during app initialization
 */
export function registerWebSocketHandlers(): void {
  // Message events
  wsManager.on('posted', handleMessageWebSocketEvent)
  wsManager.on('post_edited', handleMessageWebSocketEvent)
  wsManager.on('post_deleted', handleMessageWebSocketEvent)
  wsManager.on('reaction_added', handleMessageWebSocketEvent)
  wsManager.on('reaction_removed', handleMessageWebSocketEvent)

  // Call events
  wsManager.on('custom_com.mattermost.calls_call_start', handleCallWebSocketEvent)
  wsManager.on('custom_com.mattermost.calls_call_end', handleCallWebSocketEvent)
  wsManager.on('custom_com.mattermost.calls_user_joined', handleCallWebSocketEvent)
  wsManager.on('custom_com.mattermost.calls_user_left', handleCallWebSocketEvent)
  wsManager.on('custom_com.mattermost.calls_user_muted', handleCallWebSocketEvent)
  wsManager.on('custom_com.mattermost.calls_user_unmuted', handleCallWebSocketEvent)
  wsManager.on('custom_com.mattermost.calls_raise_hand', handleCallWebSocketEvent)
  wsManager.on('custom_com.mattermost.calls_lower_hand', handleCallWebSocketEvent)
  wsManager.on('custom_com.mattermost.calls_user_voice_on', handleCallWebSocketEvent)
  wsManager.on('custom_com.mattermost.calls_user_voice_off', handleCallWebSocketEvent)
  wsManager.on('custom_com.mattermost.calls_host_mute', handleCallWebSocketEvent)
  wsManager.on('custom_com.mattermost.calls_host_removed', handleCallWebSocketEvent)
  wsManager.on('custom_com.mattermost.calls_host_changed', handleCallWebSocketEvent)
  wsManager.on('custom_com.mattermost.calls_ringing', handleCallWebSocketEvent)
  wsManager.on('custom_com.mattermost.calls_screen_on', handleCallWebSocketEvent)
  wsManager.on('custom_com.mattermost.calls_screen_off', handleCallWebSocketEvent)
  wsManager.on('custom_com.mattermost.calls_signal', handleCallWebSocketEvent)

  // Channel events
  wsManager.on('channel_created', handleChannelWebSocketEvent)
  wsManager.on('channel_updated', handleChannelWebSocketEvent)
  wsManager.on('channel_deleted', handleChannelWebSocketEvent)
  wsManager.on('user_added', handleChannelWebSocketEvent)
  wsManager.on('user_removed', handleChannelWebSocketEvent)
  wsManager.on('channel_viewed', handleChannelWebSocketEvent)

  // Presence events (to be implemented)
  // wsManager.on('status_change', handlePresenceWebSocketEvent)
  // wsManager.on('typing', handlePresenceWebSocketEvent)

  console.log('[WebSocket] All handlers registered')
}

/**
 * Unregister all handlers (useful for testing or hot reload)
 */
export function unregisterWebSocketHandlers(): void {
  // Note: Current implementation doesn't support unregistering
  // Would need to store unsubscribe functions from wsManager.on()
  console.log('[WebSocket] Handler unregistration not implemented')
}
