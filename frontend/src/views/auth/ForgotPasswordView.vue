<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import client from '../../api/client'
import AuthLayout from '../../layouts/AuthLayout.vue'
import BaseInput from '../../components/atomic/BaseInput.vue'
import BaseButton from '../../components/atomic/BaseButton.vue'
import { useConfigStore } from '../../stores/config'

const router = useRouter()
const configStore = useConfigStore()

const email = ref('')
const loading = ref(false)
const error = ref('')
const success = ref(false)

onMounted(() => {
  configStore.loadConfig()
})

async function handleSubmit() {
  loading.value = true
  error.value = ''
  try {
    await client.post('/auth/password/forgot', {
      email: email.value
    })
    success.value = true
  } catch (e: any) {
    // Always show generic error to prevent email enumeration
    error.value = e.response?.data?.message || 'Failed to send reset email. Please try again.'
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <AuthLayout>
    <template #title>Reset your password</template>
    <template #subtitle>
      Remember your password? 
      <router-link to="/login" class="font-medium text-primary hover:text-blue-500">Sign in</router-link>
    </template>

    <!-- Success State -->
    <div v-if="success" class="text-center py-8">
      <div class="mx-auto flex items-center justify-center h-16 w-16 rounded-full bg-green-100 mb-6">
        <svg class="h-8 w-8 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"></path>
        </svg>
      </div>
      <h3 class="text-xl font-semibold text-gray-900 dark:text-white mb-2">
        Check your email
      </h3>
      <p class="text-gray-600 dark:text-gray-300 mb-6">
        If an account exists for <strong>{{ email }}</strong>, you will receive a password reset link.
        Please check your inbox and spam folder.
      </p>
      <div class="space-y-4">
        <BaseButton @click="router.push('/login')" variant="secondary" block>
          Back to Login
        </BaseButton>
      </div>
    </div>

    <!-- Forgot Password Form -->
    <form v-else class="space-y-6" @submit.prevent="handleSubmit">
      <div v-if="error" class="bg-red-50 border border-red-200 text-red-600 px-4 py-3 rounded-md text-sm">
        {{ error }}
      </div>

      <p class="text-sm text-gray-600 dark:text-gray-300">
        Enter your email address and we'll send you a link to reset your password.
      </p>

      <BaseInput
        id="email"
        type="email"
        label="Email address"
        v-model="email"
        required
        placeholder="you@example.com"
      />

      <div>
        <BaseButton type="submit" block :loading="loading">
          Send Reset Link
        </BaseButton>
      </div>
    </form>
  </AuthLayout>
</template>
