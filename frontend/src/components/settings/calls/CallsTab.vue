<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import SettingItemMin from '../SettingItemMin.vue'
import SettingItemMax from '../SettingItemMax.vue'
import { useCallsStore } from '../../../stores/calls'
import { Mic, Speaker, Video } from 'lucide-vue-next'

const callsStore = useCallsStore()
const expandedRow = ref<string | null>(null)
const saving = ref(false)

// Device states
const audioInputDevices = ref<MediaDeviceInfo[]>([])
const audioOutputDevices = ref<MediaDeviceInfo[]>([])
const videoDevices = ref<MediaDeviceInfo[]>([])
const selectedAudioInput = ref<string>('')
const selectedAudioOutput = ref<string>('')
const selectedVideoDevice = ref<string>('')
const permissionError = ref<string | null>(null)

// Display labels
const audioInputLabel = computed(() => {
  const device = audioInputDevices.value.find(d => d.deviceId === callsStore.preferredAudioInput)
  return device?.label || 'Default'
})

const audioOutputLabel = computed(() => {
  const device = audioOutputDevices.value.find(d => d.deviceId === callsStore.preferredAudioOutput)
  return device?.label || 'Default'
})

const videoDeviceLabel = computed(() => {
  const device = videoDevices.value.find(d => d.deviceId === callsStore.preferredVideoDevice)
  return device?.label || 'Default'
})

// Enumerate devices
async function enumerateDevices() {
  try {
    // Request permission first
    const stream = await navigator.mediaDevices.getUserMedia({ audio: true, video: true })
    stream.getTracks().forEach(track => track.stop())
    
    const devices = await navigator.mediaDevices.enumerateDevices()
    
    audioInputDevices.value = devices.filter(d => d.kind === 'audioinput')
    audioOutputDevices.value = devices.filter(d => d.kind === 'audiooutput')
    videoDevices.value = devices.filter(d => d.kind === 'videoinput')
    
    // Set defaults if not already set
    const firstAudioInput = audioInputDevices.value[0]
    if (!selectedAudioInput.value && firstAudioInput) {
      selectedAudioInput.value = callsStore.preferredAudioInput || firstAudioInput.deviceId
    }
    const firstAudioOutput = audioOutputDevices.value[0]
    if (!selectedAudioOutput.value && firstAudioOutput) {
      selectedAudioOutput.value = callsStore.preferredAudioOutput || firstAudioOutput.deviceId
    }
    const firstVideoDevice = videoDevices.value[0]
    if (!selectedVideoDevice.value && firstVideoDevice) {
      selectedVideoDevice.value = callsStore.preferredVideoDevice || firstVideoDevice.deviceId
    }
    
    permissionError.value = null
  } catch (err: any) {
    permissionError.value = err.message || 'Permission denied'
    console.error('Failed to enumerate devices:', err)
  }
}

function expandRow(rowId: string) {
  if (expandedRow.value === rowId) {
    return
  }
  syncLocalState()
  expandedRow.value = rowId
}

function syncLocalState() {
  selectedAudioInput.value = callsStore.preferredAudioInput || ''
  selectedAudioOutput.value = callsStore.preferredAudioOutput || ''
  selectedVideoDevice.value = callsStore.preferredVideoDevice || ''
}

async function saveAudioInput() {
  saving.value = true
  try {
    await callsStore.setPreferredAudioInput(selectedAudioInput.value)
    expandedRow.value = null
  } finally {
    saving.value = false
  }
}

async function saveAudioOutput() {
  saving.value = true
  try {
    await callsStore.setPreferredAudioOutput(selectedAudioOutput.value)
    expandedRow.value = null
  } finally {
    saving.value = false
  }
}

async function saveVideoDevice() {
  saving.value = true
  try {
    await callsStore.setPreferredVideoDevice(selectedVideoDevice.value)
    expandedRow.value = null
  } finally {
    saving.value = false
  }
}

function cancelEdit() {
  syncLocalState()
  expandedRow.value = null
}

onMounted(() => {
  enumerateDevices()
})
</script>

<template>
  <div class="space-y-1">
    <!-- Permission Error -->
    <div v-if="permissionError" class="mb-4 p-3 bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-lg">
      <p class="text-sm text-amber-700 dark:text-amber-300">
        <strong>Permission Required:</strong> Please allow access to microphone and camera to configure devices.
      </p>
    </div>

    <!-- 1. Audio Input Device -->
    <div v-if="expandedRow !== 'audio_input'">
      <SettingItemMin
        label="Audio Input Device"
        :value="audioInputLabel"
        description="Select your microphone for calls"
        @click="expandRow('audio_input')"
      >
        <template #icon>
          <Mic class="w-5 h-5 text-gray-400" />
        </template>
      </SettingItemMin>
    </div>
    
    <SettingItemMax
      v-else
      label="Audio Input Device"
      description="Choose your microphone for calls"
      :loading="saving"
      @save="saveAudioInput"
      @cancel="cancelEdit"
    >
      <div class="space-y-3">
        <label 
          v-for="device in audioInputDevices" 
          :key="device.deviceId"
          class="flex items-center gap-3 p-3 border border-gray-200 dark:border-gray-700 rounded-lg cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-800"
        >
          <input
            type="radio"
            v-model="selectedAudioInput"
            :value="device.deviceId"
            class="w-4 h-4 text-primary"
          />
          <div class="flex-1">
            <div class="text-sm font-medium text-gray-900 dark:text-white">{{ device.label || 'Default Microphone' }}</div>
          </div>
        </label>
        <div v-if="audioInputDevices.length === 0" class="text-sm text-gray-500 text-center py-4">
          No audio input devices found
        </div>
      </div>
    </SettingItemMax>

    <!-- 2. Audio Output Device -->
    <div v-if="expandedRow !== 'audio_output'">
      <SettingItemMin
        label="Audio Output Device"
        :value="audioOutputLabel"
        description="Select your speaker or headphones for calls"
        @click="expandRow('audio_output')"
      >
        <template #icon>
          <Speaker class="w-5 h-5 text-gray-400" />
        </template>
      </SettingItemMin>
    </div>
    
    <SettingItemMax
      v-else
      label="Audio Output Device"
      description="Choose your speaker or headphones"
      :loading="saving"
      @save="saveAudioOutput"
      @cancel="cancelEdit"
    >
      <div class="space-y-3">
        <label 
          v-for="device in audioOutputDevices" 
          :key="device.deviceId"
          class="flex items-center gap-3 p-3 border border-gray-200 dark:border-gray-700 rounded-lg cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-800"
        >
          <input
            type="radio"
            v-model="selectedAudioOutput"
            :value="device.deviceId"
            class="w-4 h-4 text-primary"
          />
          <div class="flex-1">
            <div class="text-sm font-medium text-gray-900 dark:text-white">{{ device.label || 'Default Speaker' }}</div>
          </div>
        </label>
        <div v-if="audioOutputDevices.length === 0" class="text-sm text-gray-500 text-center py-4">
          No audio output devices found
        </div>
      </div>
    </SettingItemMax>

    <!-- 3. Video Device -->
    <div v-if="expandedRow !== 'video_device'">
      <SettingItemMin
        label="Video Device"
        :value="videoDeviceLabel"
        description="Select your camera for video calls"
        @click="expandRow('video_device')"
      >
        <template #icon>
          <Video class="w-5 h-5 text-gray-400" />
        </template>
      </SettingItemMin>
    </div>
    
    <SettingItemMax
      v-else
      label="Video Device"
      description="Choose your camera for video calls"
      :loading="saving"
      @save="saveVideoDevice"
      @cancel="cancelEdit"
    >
      <div class="space-y-3">
        <label 
          v-for="device in videoDevices" 
          :key="device.deviceId"
          class="flex items-center gap-3 p-3 border border-gray-200 dark:border-gray-700 rounded-lg cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-800"
        >
          <input
            type="radio"
            v-model="selectedVideoDevice"
            :value="device.deviceId"
            class="w-4 h-4 text-primary"
          />
          <div class="flex-1">
            <div class="text-sm font-medium text-gray-900 dark:text-white">{{ device.label || 'Default Camera' }}</div>
          </div>
        </label>
        <div v-if="videoDevices.length === 0" class="text-sm text-gray-500 text-center py-4">
          No video devices found
        </div>
      </div>
    </SettingItemMax>
  </div>
</template>
