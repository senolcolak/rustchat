<script setup lang="ts">
import { ref, computed } from 'vue'
import SettingItemMin from '../SettingItemMin.vue'
import SettingItemMax from '../SettingItemMax.vue'
import ThemeEditor from './ThemeEditor.vue'
import { useThemeStore, THEME_OPTIONS, type Theme } from '../../../stores/theme'

const themeStore = useThemeStore()

// Expanded row state
const expandedRow = ref<string | null>(null)

// Local theme state for edit mode
const editingTheme = ref<Theme>(themeStore.theme)
const savingTheme = ref(false)

// Theme display label
const themeLabel = computed(() => {
  const option = THEME_OPTIONS.find(t => t.id === themeStore.theme)
  return option?.label || themeStore.theme
})

function expandRow(rowId: string) {
  if (expandedRow.value === rowId) {
    return
  }
  
  // Initialize edit state when expanding
  if (rowId === 'theme') {
    editingTheme.value = themeStore.theme
  }
  
  expandedRow.value = rowId
}

async function handleSaveTheme(theme: Theme) {
  savingTheme.value = true
  try {
    themeStore.setTheme(theme)
    expandedRow.value = null
  } finally {
    savingTheme.value = false
  }
}

function handleCancelTheme() {
  editingTheme.value = themeStore.theme
  expandedRow.value = null
}
</script>

<template>
  <div class="space-y-1">
    <!-- Theme Row -->
    <div v-if="expandedRow !== 'theme'">
      <SettingItemMin
        label="Theme"
        :value="themeLabel"
        description="Choose a color theme for the application"
        @click="expandRow('theme')"
      />
    </div>
    
    <SettingItemMax
      v-else
      label="Theme"
      description="Select a premade theme or customize your own colors"
      :loading="savingTheme"
      @save="() => {}"
      @cancel="handleCancelTheme"
    >
      <ThemeEditor
        v-model="editingTheme"
        @save="handleSaveTheme"
        @cancel="handleCancelTheme"
      />
    </SettingItemMax>

    <!-- Placeholder rows for future Display settings (from S7) -->
    <!-- These will be implemented in a future phase -->
  </div>
</template>
