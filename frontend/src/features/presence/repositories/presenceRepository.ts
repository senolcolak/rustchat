// Presence Repository - Data access for user presence

import { withRetry } from '../../../core/services/retry'
import type { UserId, PresenceStatus } from '../../../core/entities/User'

// Note: Most presence operations are WebSocket-based
// This repository is minimal as presence is real-time

export const presenceRepository = {
  // Presence is primarily WebSocket-driven
  // API endpoints would be for bulk operations or initial load
  
  async getBulkPresence(userIds: UserId[]): Promise<Map<UserId, PresenceStatus>> {
    // If there's a bulk presence API, use it here
    // For now, return empty (WebSocket handles real-time updates)
    return new Map()
  }
}
