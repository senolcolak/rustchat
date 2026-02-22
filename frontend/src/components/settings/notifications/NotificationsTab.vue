<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import SettingItemMin from '../SettingItemMin.vue'
import SettingItemMax from '../SettingItemMax.vue'
import { usePreferencesStore } from '../../../stores/preferences'
import { useToast } from '../../../composables/useToast'
import { 
  Bell, 
  Monitor, 
  Smartphone, 
  Volume2, 
  Mail, 
  MessageSquare, 
  AlertTriangle
} from 'lucide-vue-next'

const preferencesStore = usePreferencesStore()
const toast = useToast()
const expandedRow = ref<string | null>(null)
const saving = ref(false)

// Permission status
const notificationPermission = ref<NotificationPermission>('default')

// Local preference states for editing
const localNotifyDesktop = ref<'all' | 'mentions' | 'none'>('all')
const localNotifyPush = ref<'all' | 'mentions' | 'none'>('all')
const localNotifySounds = ref(true)
const localNotifyEmail = ref(true)
const localMentionKeywords = ref<string[]>([])

// Display labels
const desktopNotificationsLabel = computed(() => {
  const labels: Record<string, string> = {
    all: 'For all activity',
    mentions: 'For mentions and direct messages',
    none: 'Never'
  }
  return labels[preferencesStore.preferences?.notify_desktop || 'all'] || 'For all activity'
})

const pushNotificationsLabel = computed(() => {
  const labels: Record<string, string> = {
    all: 'For all activity',
    mentions: 'For mentions and direct messages', 
    none: 'Never'
  }
  return labels[preferencesStore.preferences?.notify_push || 'all'] || 'For all activity'
})

const desktopSoundsLabel = computed(() => {
  return preferencesStore.preferences?.notify_sounds !== false ? 'On' : 'Off'
})

const emailNotificationsLabel = computed(() => {
  const email = preferencesStore.preferences?.notify_email
  return email === 'true' || email === 'all' || email === 'mentions' ? 'On' : 'Off'
})

const mentionKeywordsLabel = computed(() => {
  const keywords = preferencesStore.preferences?.mention_keywords
  return keywords && keywords.length > 0 ? keywords.length + ' keywords' : 'None set'
})

// Permission status tag
const permissionStatusDisplay = computed(() => {
  switch (notificationPermission.value) {
    case 'granted':
      return { text: 'Allowed', class: 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400' }
    case 'denied':
      return { text: 'Blocked', class: 'bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400' }
    default:
      return { text: 'Not set', class: 'bg-amber-100 text-amber-700 dark:bg-amber-900/30 dark:text-amber-400' }
  }
})

// Initialize local states from preferences on mount
onMounted(() => {
  notificationPermission.value = Notification.permission
  preferencesStore.fetchPreferences().then(() => {
    syncLocalState()
  })
})

function syncLocalState() {
  const prefs = preferencesStore.preferences
  if (!prefs) return

  localNotifyDesktop.value = (prefs.notify_desktop as 'all' | 'mentions' | 'none') || 'all'
  localNotifyPush.value = (prefs.notify_push as 'all' | 'mentions' | 'none') || 'all'
  localNotifySounds.value = prefs.notify_sounds !== false
  localNotifyEmail.value = prefs.notify_email === 'true' || prefs.notify_email === 'all' || prefs.notify_email === 'mentions'
  localMentionKeywords.value = prefs.mention_keywords || []
}

function expandRow(rowId: string) {
  if (expandedRow.value === rowId) {
    return
  }
  syncLocalState()
  expandedRow.value = rowId
}

async function savePreference(updates: Record<string, unknown>) {
  saving.value = true
  try {
    await preferencesStore.updatePreferences(updates)
    expandedRow.value = null
  } finally {
    saving.value = false
  }
}

function cancelEdit() {
  syncLocalState()
  expandedRow.value = null
}

async function requestNotificationPermission() {
  try {
    const permission = await Notification.requestPermission()
    notificationPermission.value = permission
    if (permission === 'granted') {
      toast.success('Notifications enabled', 'You will now receive desktop notifications')
    } else if (permission === 'denied') {
      toast.error('Permission denied', 'Please enable notifications in your browser settings')
    }
  } catch (e) {
    console.error('Failed to request notification permission:', e)
  }
}

function testNotificationSound() {
  // Create a simple beep using Web Audio API
  try {
    const audioContext = new (window.AudioContext || (window as any).webkitAudioContext)()
    const oscillator = audioContext.createOscillator()
    const gainNode = audioContext.createGain()
    
    oscillator.connect(gainNode)
    gainNode.connect(audioContext.destination)
    
    oscillator.frequency.value = 800
    oscillator.type = 'sine'
    gainNode.gain.value = 0.3
    
    oscillator.start()
    oscillator.stop(audioContext.currentTime + 0.1)
    
    toast.success('Sound test', 'Notification sound played')
  } catch (e) {
    toast.error('Sound test failed', 'Could not play notification sound')
  }
}

// Keywords input handling
const keywordsInput = ref('')

function syncKeywordsInput() {
  keywordsInput.value = localMentionKeywords.value.join(', ')
}

function saveKeywords() {
  // Parse comma-separated keywords
  const keywords = keywordsInput.value
    .split(',')
    .map(k => k.trim())
    .filter(k => k.length > 0)
  savePreference({ mention_keywords: keywords })
}

function expandKeywordsRow() {
  syncLocalState()
  syncKeywordsInput()
  expandedRow.value = 'mention_keywords'
}
</script>

<template>
  <div class="space-y-1">
    <!-- Desktop Notifications Row -->
    <div v-if="expandedRow !== 'desktop_notifications'">
      <SettingItemMin
        label="Desktop Notifications"
        :value="desktopNotificationsLabel"
        description="Receive notifications on your desktop"
        @click="expandRow('desktop_notifications')"
      >
        <template #icon>
          <Monitor class="w-5 h-5 text-gray-400" />
        </template>
        <template #extra>
          <span 
            class="ml-2 px-2 py-0.5 text-xs font-medium rounded-full"
            :class="permissionStatusDisplay.class"
          >
            {{ permissionStatusDisplay.text }}
          </span>
        </template>
      </SettingItemMin>
    </div>
    
    <SettingItemMax
      v-else
      label="Desktop Notifications"
      description="Choose when to receive desktop notifications"
      :loading="saving"
      @save="savePreference({ notify_desktop: localNotifyDesktop })"
      @cancel="cancelEdit"
    >
      <div class="space-y-4">
        <!-- Permission Button -->
        <div v-if="notificationPermission !== 'granted'" class="p-3 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
          <div class="flex items-start gap-3">
            <Bell class="w-5 h-5 text-blue-500 mt-0.5" />
            <div class="flex-1">
              <p class="text-sm font-medium text-blue-900 dark:text-blue-300">Enable Desktop Notifications</p>
              <p class="text-xs text-blue-700 dark:text-blue-400 mt-1">
                You need to allow browser notifications to receive desktop alerts.
              </p>
              <button 
                @click="requestNotificationPermission"
                class="mt-2 px-3 py-1.5 bg-blue-600 text-white text-sm rounded-lg hover:bg-blue-700 transition-colors"
              >
                Allow Notifications
              </button>
            </div>
          </div>
        </div>

        <div class="space-y-2">
          <label class="flex items-center gap-3 p-3 border border-gray-200 dark:border-gray-700 rounded-lg cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-800">
            <input
              type="radio"
              v-model="localNotifyDesktop"
              value="all"
              class="w-4 h-4 text-primary"
            />
            <div class="flex-1">
              <div class="text-sm font-medium text-gray-900 dark:text-white">For all activity</div>
              <div class="text-xs text-gray-500">Notify me about all messages and activity</div>
            </div>
          </label>
          <label class="flex items-center gap-3 p-3 border border-gray-200 dark:border-gray-700 rounded-lg cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-800">
            <input
              type="radio"
              v-model="localNotifyDesktop"
              value="mentions"
              class="w-4 h-4 text-primary"
            />
            <div class="flex-1">
              <div class="text-sm font-medium text-gray-900 dark:text-white">For mentions and direct messages</div>
              <div class="text-xs text-gray-500">Only notify me when I'm mentioned or receive a DM</div>
            </div>
          </label>
          <label class="flex items-center gap-3 p-3 border border-gray-200 dark:border-gray-700 rounded-lg cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-800">
            <input
              type="radio"
              v-model="localNotifyDesktop"
              value="none"
              class="w-4 h-4 text-primary"
            />
            <div class="flex-1">
              <div class="text-sm font-medium text-gray-900 dark:text-white">Never</div>
              <div class="text-xs text-gray-500">Don't send any desktop notifications</div>
            </div>
          </label>
        </div>
      </div>
    </SettingItemMax>

    <!-- Mobile Push Notifications Row -->
    <div v-if="expandedRow !== 'push_notifications'">
      <SettingItemMin
        label="Mobile Push Notifications"
        :value="pushNotificationsLabel"
        description="Receive notifications on your mobile device"
        @click="expandRow('push_notifications')"
      >
        <template #icon>
          <Smartphone class="w-5 h-5 text-gray-400" />
        </template>
      </SettingItemMin>
    </div>
    
    <SettingItemMax
      v-else
      label="Mobile Push Notifications"
      description="Choose when to receive mobile push notifications"
      :loading="saving"
      @save="savePreference({ notify_push: localNotifyPush })"
      @cancel="cancelEdit"
    >
      <div class="space-y-2">
        <label class="flex items-center gap-3 p-3 border border-gray-200 dark:border-gray-700 rounded-lg cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-800">
          <input
            type="radio"
            v-model="localNotifyPush"
            value="all"
            class="w-4 h-4 text-primary"
          />
          <div class="flex-1">
            <div class="text-sm font-medium text-gray-900 dark:text-white">For all activity</div>
            <div class="text-xs text-gray-500">Send push notifications for all messages</div>
          </div>
        </label>
        <label class="flex items-center gap-3 p-3 border border-gray-200 dark:border-gray-700 rounded-lg cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-800">
          <input
            type="radio"
            v-model="localNotifyPush"
            value="mentions"
            class="w-4 h-4 text-primary"
          />
          <div class="flex-1">
            <div class="text-sm font-medium text-gray-900 dark:text-white">For mentions and direct messages</div>
            <div class="text-xs text-gray-500">Only send push notifications for mentions and DMs</div>
          </div>
        </label>
        <label class="flex items-center gap-3 p-3 border border-gray-200 dark:border-gray-700 rounded-lg cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-800">
          <input
            type="radio"
            v-model="localNotifyPush"
            value="none"
            class="w-4 h-4 text-primary"
          />
          <div class="flex-1">
            <div class="text-sm font-medium text-gray-900 dark:text-white">Never</div>
            <div class="text-xs text-gray-500">Don't send any push notifications</div>
          </div>
        </label>
      </div>
    </SettingItemMax>

    <!-- Desktop Sounds Row -->
    <div v-if="expandedRow !== 'desktop_sounds'">
      <SettingItemMin
        label="Desktop Sounds"
        :value="desktopSoundsLabel"
        description="Play a sound when receiving notifications"
        @click="expandRow('desktop_sounds')"
      >
        <template #icon>
          <Volume2 class="w-5 h-5 text-gray-400" />
        </template>
      </SettingItemMin>
    </div>
    
    <SettingItemMax
      v-else
      label="Desktop Sounds"
      description="Control notification sounds"
      :loading="saving"
      @save="savePreference({ notify_sounds: localNotifySounds })"
      @cancel="cancelEdit"
    >
      <div class="space-y-3">
        <label class="flex items-center justify-between p-3 border border-gray-200 dark:border-gray-700 rounded-lg cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-800">
          <div>
            <div class="text-sm font-medium text-gray-900 dark:text-white">Enable desktop sounds</div>
            <div class="text-xs text-gray-500">Play a sound when you receive a notification</div>
          </div>
          <input
            type="checkbox"
            v-model="localNotifySounds"
            class="w-5 h-5 text-primary rounded"
          />
        </label>
        <button
          @click="testNotificationSound"
          class="w-full px-3 py-2 text-sm text-primary border border-primary/30 rounded-lg hover:bg-primary/5 transition-colors"
        >
          Test Sound
        </button>
      </div>
    </SettingItemMax>

    <!-- Email Notifications Row -->
    <div v-if="expandedRow !== 'email_notifications'">
      <SettingItemMin
        label="Email Notifications"
        :value="emailNotificationsLabel"
        description="Receive email notifications for mentions and DMs"
        @click="expandRow('email_notifications')"
      >
        <template #icon>
          <Mail class="w-5 h-5 text-gray-400" />
        </template>
      </SettingItemMin>
    </div>
    
    <SettingItemMax
      v-else
      label="Email Notifications"
      description="Control email notification settings"
      :loading="saving"
      @save="savePreference({ notify_email: localNotifyEmail })"
      @cancel="cancelEdit"
    >
      <div class="space-y-3">
        <label class="flex items-center justify-between p-3 border border-gray-200 dark:border-gray-700 rounded-lg cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-800">
          <div>
            <div class="text-sm font-medium text-gray-900 dark:text-white">Enable email notifications</div>
            <div class="text-xs text-gray-500">Send me emails for mentions and direct messages</div>
          </div>
          <input
            type="checkbox"
            v-model="localNotifyEmail"
            class="w-5 h-5 text-primary rounded"
          />
        </label>
      </div>
    </SettingItemMax>

    <!-- Mention Keywords Row -->
    <div v-if="expandedRow !== 'mention_keywords'">
      <SettingItemMin
        label="Mention Keywords"
        :value="mentionKeywordsLabel"
        description="Additional words that trigger mentions"
        @click="expandKeywordsRow"
      >
        <template #icon>
          <MessageSquare class="w-5 h-5 text-gray-400" />
        </template>
      </SettingItemMin>
    </div>
    
    <SettingItemMax
      v-else
      label="Mention Keywords"
      description="Words that trigger mention notifications besides your username"
      :loading="saving"
      @save="saveKeywords"
      @cancel="cancelEdit"
    >
      <div class="space-y-3">
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
            Keywords (comma-separated)
          </label>
          <input
            v-model="keywordsInput"
            type="text"
            placeholder="e.g. @channel, @here, urgent"
            class="w-full px-3 py-2 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg text-sm"
          />
          <p class="mt-1 text-xs text-gray-500">
            You'll be notified when someone uses these words in a message
          </p>
        </div>
      </div>
    </SettingItemMax>

    <!-- Troubleshooting Card -->
    <div class="mt-6 p-4 bg-gray-50 dark:bg-gray-900 rounded-lg border border-gray-200 dark:border-gray-700">
      <div class="flex items-start gap-3">
        <AlertTriangle class="w-5 h-5 text-amber-500 mt-0.5" />
        <div class="flex-1">
          <h4 class="text-sm font-medium text-gray-900 dark:text-white">Troubleshooting</h4>
          <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
            Not receiving notifications? Check your browser and system notification settings.
          </p>
          <div class="mt-3 flex flex-wrap gap-2">
            <button
              @click="requestNotificationPermission"
              class="px-3 py-1.5 text-xs font-medium text-primary bg-white dark:bg-gray-800 border border-primary/30 rounded-lg hover:bg-primary/5 transition-colors"
            >
              Check Permission
            </button>
            <button
              @click="testNotificationSound"
              class="px-3 py-1.5 text-xs font-medium text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
            >
              Test Sound
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
