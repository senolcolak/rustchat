<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { Check } from 'lucide-vue-next'
import type { Theme } from '../../../stores/theme'

// Mattermost-style theme options
export const MM_THEME_OPTIONS: Array<{
  id: Theme | 'custom'
  label: string
  swatches: { primary: string; accent: string; background: string; sidebarBg?: string }
  colors?: {
    sidebarBg: string
    sidebarText: string
    centerChannelBg: string
    centerChannelColor: string
    linkColor: string
    buttonBg: string
    buttonColor: string
  }
}> = [
  { 
    id: 'light', 
    label: 'Mattermost Light', 
    swatches: { primary: '#166de0', accent: '#166de0', background: '#ffffff', sidebarBg: '#1e325c' },
    colors: {
      sidebarBg: '#1e325c',
      sidebarText: '#ffffff',
      centerChannelBg: '#ffffff',
      centerChannelColor: '#3d3c40',
      linkColor: '#166de0',
      buttonBg: '#166de0',
      buttonColor: '#ffffff',
    }
  },
  { 
    id: 'dark', 
    label: 'Mattermost Dark', 
    swatches: { primary: '#1e88e5', accent: '#1e88e5', background: '#1f222a', sidebarBg: '#1f222a' },
    colors: {
      sidebarBg: '#1f222a',
      sidebarText: '#ffffff',
      centerChannelBg: '#2f3136',
      centerChannelColor: '#dddddd',
      linkColor: '#1e88e5',
      buttonBg: '#1e88e5',
      buttonColor: '#ffffff',
    }
  },
  { 
    id: 'modern', 
    label: 'Mattermost', 
    swatches: { primary: '#145dbf', accent: '#145dbf', background: '#f3f3f3', sidebarBg: '#145dbf' },
    colors: {
      sidebarBg: '#145dbf',
      sidebarText: '#ffffff',
      centerChannelBg: '#ffffff',
      centerChannelColor: '#3d3c40',
      linkColor: '#145dbf',
      buttonBg: '#145dbf',
      buttonColor: '#ffffff',
    }
  },
  { 
    id: 'custom', 
    label: 'Custom', 
    swatches: { primary: '#166de0', accent: '#166de0', background: '#ffffff', sidebarBg: '#1e325c' },
    colors: {
      sidebarBg: '#1e325c',
      sidebarText: '#ffffff',
      centerChannelBg: '#ffffff',
      centerChannelColor: '#3d3c40',
      linkColor: '#166de0',
      buttonBg: '#166de0',
      buttonColor: '#ffffff',
    }
  },
]

const props = defineProps<{
  modelValue: Theme
}>()

const emit = defineEmits<{
  'update:modelValue': [theme: Theme]
  save: [theme: Theme]
  cancel: []
}>()

// Local state for editing
const selectedOption = ref<Theme | 'custom'>(props.modelValue)
const isCustom = computed(() => selectedOption.value === 'custom')

// Custom color state
const customColors = ref({
  sidebarBg: '#1e325c',
  sidebarText: '#ffffff',
  centerChannelBg: '#ffffff',
  centerChannelColor: '#3d3c40',
  linkColor: '#166de0',
  buttonBg: '#166de0',
  buttonColor: '#ffffff',
})

// Initialize from current theme
watch(() => props.modelValue, (newTheme) => {
  const matched = MM_THEME_OPTIONS.find(o => o.id === newTheme)
  if (matched) {
    selectedOption.value = matched.id
    if (matched.colors) {
      customColors.value = { ...matched.colors }
    }
  } else {
    selectedOption.value = 'custom'
  }
}, { immediate: true })

function selectTheme(themeId: Theme | 'custom') {
  selectedOption.value = themeId
  const option = MM_THEME_OPTIONS.find(o => o.id === themeId)
  if (option?.colors) {
    customColors.value = { ...option.colors }
  }
}

function handleSave() {
  const themeToSave = selectedOption.value === 'custom' ? 'light' : selectedOption.value
  emit('save', themeToSave)
}

function handleCancel() {
  // Reset to original
  const matched = MM_THEME_OPTIONS.find(o => o.id === props.modelValue)
  if (matched) {
    selectedOption.value = matched.id
    if (matched.colors) {
      customColors.value = { ...matched.colors }
    }
  }
  emit('cancel')
}

function updateCustomColor(key: keyof typeof customColors.value, value: string) {
  customColors.value[key] = value
  selectedOption.value = 'custom'
}
</script>

<template>
  <div class="space-y-6">
    <!-- Premade Themes -->
    <div>
      <h5 class="text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-3">
        Premade Themes
      </h5>
      <div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
        <button
          v-for="theme in MM_THEME_OPTIONS.filter(t => t.id !== 'custom')"
          :key="theme.id"
          type="button"
          @click="selectTheme(theme.id as Theme)"
          class="relative flex flex-col items-center gap-2 p-3 rounded-lg border transition-all"
          :class="selectedOption === theme.id 
            ? 'border-primary bg-primary/5 dark:bg-primary/10 ring-2 ring-primary/30' 
            : 'border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600'"
        >
          <!-- Theme Swatch -->
          <div class="relative w-full h-16 rounded-md overflow-hidden border border-gray-200 dark:border-gray-700">
            <!-- Sidebar preview -->
            <div 
              class="absolute left-0 top-0 h-full w-1/3"
              :style="{ backgroundColor: theme.swatches.sidebarBg || theme.swatches.primary }"
            />
            <!-- Center channel preview -->
            <div 
              class="absolute right-0 top-0 h-full w-2/3"
              :style="{ backgroundColor: theme.swatches.background }"
            />
            <!-- Accent line -->
            <div 
              class="absolute left-1/3 top-0 h-full w-1"
              :style="{ backgroundColor: theme.swatches.accent }"
            />
          </div>
          
          <span class="text-xs font-medium text-gray-700 dark:text-gray-300 text-center">
            {{ theme.label }}
          </span>
          
          <!-- Selected checkmark -->
          <div 
            v-if="selectedOption === theme.id"
            class="absolute top-1 right-1 w-5 h-5 rounded-full bg-primary flex items-center justify-center"
          >
            <Check class="w-3 h-3 text-white" />
          </div>
        </button>
      </div>
    </div>

    <!-- Custom Theme Section -->
    <div v-if="isCustom" class="border-t border-gray-200 dark:border-gray-700 pt-6">
      <h5 class="text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-3">
        Custom Theme Colors
      </h5>
      
      <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
        <!-- Sidebar Background -->
        <div class="flex items-center gap-3">
          <input
            type="color"
            :value="customColors.sidebarBg"
            @input="e => updateCustomColor('sidebarBg', (e.target as HTMLInputElement).value)"
            class="w-10 h-10 rounded-lg border border-gray-200 dark:border-gray-700 cursor-pointer"
          />
          <div>
            <div class="text-sm font-medium text-gray-700 dark:text-gray-300">Sidebar BG</div>
            <div class="text-xs text-gray-500 font-mono">{{ customColors.sidebarBg }}</div>
          </div>
        </div>

        <!-- Sidebar Text -->
        <div class="flex items-center gap-3">
          <input
            type="color"
            :value="customColors.sidebarText"
            @input="e => updateCustomColor('sidebarText', (e.target as HTMLInputElement).value)"
            class="w-10 h-10 rounded-lg border border-gray-200 dark:border-gray-700 cursor-pointer"
          />
          <div>
            <div class="text-sm font-medium text-gray-700 dark:text-gray-300">Sidebar Text</div>
            <div class="text-xs text-gray-500 font-mono">{{ customColors.sidebarText }}</div>
          </div>
        </div>

        <!-- Center Channel Background -->
        <div class="flex items-center gap-3">
          <input
            type="color"
            :value="customColors.centerChannelBg"
            @input="e => updateCustomColor('centerChannelBg', (e.target as HTMLInputElement).value)"
            class="w-10 h-10 rounded-lg border border-gray-200 dark:border-gray-700 cursor-pointer"
          />
          <div>
            <div class="text-sm font-medium text-gray-700 dark:text-gray-300">Center BG</div>
            <div class="text-xs text-gray-500 font-mono">{{ customColors.centerChannelBg }}</div>
          </div>
        </div>

        <!-- Center Channel Text -->
        <div class="flex items-center gap-3">
          <input
            type="color"
            :value="customColors.centerChannelColor"
            @input="e => updateCustomColor('centerChannelColor', (e.target as HTMLInputElement).value)"
            class="w-10 h-10 rounded-lg border border-gray-200 dark:border-gray-700 cursor-pointer"
          />
          <div>
            <div class="text-sm font-medium text-gray-700 dark:text-gray-300">Center Text</div>
            <div class="text-xs text-gray-500 font-mono">{{ customColors.centerChannelColor }}</div>
          </div>
        </div>

        <!-- Link Color -->
        <div class="flex items-center gap-3">
          <input
            type="color"
            :value="customColors.linkColor"
            @input="e => updateCustomColor('linkColor', (e.target as HTMLInputElement).value)"
            class="w-10 h-10 rounded-lg border border-gray-200 dark:border-gray-700 cursor-pointer"
          />
          <div>
            <div class="text-sm font-medium text-gray-700 dark:text-gray-300">Link Color</div>
            <div class="text-xs text-gray-500 font-mono">{{ customColors.linkColor }}</div>
          </div>
        </div>

        <!-- Button Background -->
        <div class="flex items-center gap-3">
          <input
            type="color"
            :value="customColors.buttonBg"
            @input="e => updateCustomColor('buttonBg', (e.target as HTMLInputElement).value)"
            class="w-10 h-10 rounded-lg border border-gray-200 dark:border-gray-700 cursor-pointer"
          />
          <div>
            <div class="text-sm font-medium text-gray-700 dark:text-gray-300">Button BG</div>
            <div class="text-xs text-gray-500 font-mono">{{ customColors.buttonBg }}</div>
          </div>
        </div>
      </div>
    </div>

    <!-- Preview Section -->
    <div class="border-t border-gray-200 dark:border-gray-700 pt-4">
      <h5 class="text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-3">
        Preview
      </h5>
      <div 
        class="rounded-lg border border-gray-200 dark:border-gray-700 overflow-hidden h-24 flex"
        :style="{ 
          backgroundColor: isCustom ? customColors.centerChannelBg : MM_THEME_OPTIONS.find(t => t.id === selectedOption)?.colors?.centerChannelBg || '#ffffff'
        }"
      >
        <!-- Sidebar preview -->
        <div 
          class="w-1/4 h-full flex flex-col p-2"
          :style="{ 
            backgroundColor: isCustom ? customColors.sidebarBg : MM_THEME_OPTIONS.find(t => t.id === selectedOption)?.colors?.sidebarBg || '#1e325c'
          }"
        >
          <div 
            class="text-xs font-medium truncate"
            :style="{ 
              color: isCustom ? customColors.sidebarText : MM_THEME_OPTIONS.find(t => t.id === selectedOption)?.colors?.sidebarText || '#ffffff'
            }"
          >
            Channels
          </div>
        </div>
        <!-- Center preview -->
        <div 
          class="flex-1 h-full p-2"
          :style="{ 
            color: isCustom ? customColors.centerChannelColor : MM_THEME_OPTIONS.find(t => t.id === selectedOption)?.colors?.centerChannelColor || '#3d3c40'
          }"
        >
          <div class="text-xs mb-1">Welcome to Mattermost</div>
          <a 
            class="text-xs hover:underline"
            :style="{ 
              color: isCustom ? customColors.linkColor : MM_THEME_OPTIONS.find(t => t.id === selectedOption)?.colors?.linkColor || '#166de0'
            }"
          >
            Click here to learn more
          </a>
        </div>
      </div>
    </div>

    <!-- Actions -->
    <div class="flex items-center justify-end gap-2 pt-2">
      <button
        type="button"
        @click="handleCancel"
        class="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800 rounded-lg transition-colors"
      >
        Cancel
      </button>
      <button
        type="button"
        @click="handleSave"
        class="px-4 py-2 text-sm font-medium text-white bg-primary hover:bg-primary/90 rounded-lg transition-colors"
      >
        Save
      </button>
    </div>
  </div>
</template>
