/**
 * Activity Feed Types
 */

export type ActivityType = 'mention' | 'reply' | 'reaction' | 'dm' | 'thread_reply'

export const ActivityType = {
  MENTION: 'mention' as const,
  REPLY: 'reply' as const,
  REACTION: 'reaction' as const,
  DM: 'dm' as const,
  THREAD_REPLY: 'thread_reply' as const,
}

export interface Activity {
  id: string
  type: ActivityType
  actorId: string
  actorUsername: string
  actorAvatarUrl?: string
  channelId: string
  channelName: string
  teamId: string
  teamName: string
  postId: string
  rootId?: string
  message?: string
  reaction?: string
  read: boolean
  createdAt: Date
}

export interface ActivityFeedResponse {
  order: string[]
  activities: Record<string, Activity>
  unreadCount: number
  nextCursor?: string
}

export interface ActivityQueryParams {
  cursor?: string
  limit?: number
  type?: ActivityType | string
  unreadOnly?: boolean
}
