<script setup lang="ts">
import { ChevronRight } from 'lucide-vue-next'

defineProps<{
  label: string
  value: string
  description?: string
}>()

defineEmits<{
  click: []
}>()
</script>

<template>
  <button
    type="button"
    @click="$emit('click')"
    class="w-full flex items-center justify-between px-4 py-3 text-left hover:bg-gray-50 dark:hover:bg-gray-800/50 transition-colors group"
  >
    <div class="flex-1 min-w-0">
      <div class="text-sm font-medium text-gray-900 dark:text-gray-100">
        {{ label }}
      </div>
      <div v-if="description" class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
        {{ description }}
      </div>
    </div>
    <div class="flex items-center gap-2 ml-4">
      <span class="text-sm text-gray-500 dark:text-gray-400 truncate max-w-[200px]">
        {{ value }}
      </span>
      <ChevronRight class="w-4 h-4 text-gray-400 group-hover:text-gray-500 transition-colors flex-shrink-0" />
    </div>
  </button>
</template>
