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

const username = ref('')
const email = ref('')
const loading = ref(false)
const error = ref('')
const success = ref(false)
const registeredEmail = ref('')

onMounted(() => {
  configStore.loadConfig()
})

async function handleRegister() {
  loading.value = true
  error.value = ''
  try {
    const response = await client.post('/auth/register', {
      username: username.value,
      email: email.value,
      display_name: username.value
      // No password - user will set it via email link
    })
    
    registeredEmail.value = response.data.email || email.value
    success.value = true
    
    // If password setup is required, show success message
    // User will receive email with password setup link
    if (response.data.requires_password_setup) {
      // Stay on page and show success message
    } else {
      // Auto-login was provided, redirect to home
      if (response.data.token) {
        localStorage.setItem('auth_token', response.data.token)
        window.location.href = '/'
      }
    }
  } catch (e: any) {
    error.value = e.response?.data?.message || 'Failed to register'
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <AuthLayout>
    <template #title>Create your {{ configStore.siteConfig.site_name }} account</template>
    <template #subtitle>
      Already have an account? <router-link to="/login" class="font-medium text-primary hover:text-blue-500">Sign in</router-link>
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
        We've sent a password setup link to <strong>{{ registeredEmail }}</strong>.
        Please check your inbox and click the link to set your password and complete your registration.
      </p>
      <div class="space-y-4">
        <p class="text-sm text-gray-500 dark:text-gray-400">
          Didn't receive the email? Check your spam folder or 
          <router-link to="/login" class="text-indigo-600 hover:text-indigo-500">try logging in</router-link>
          to resend.
        </p>
        <BaseButton @click="router.push('/login')" variant="secondary" block>
          Go to Login
        </BaseButton>
      </div>
    </div>

    <!-- Registration Form -->
    <form v-else class="space-y-6" @submit.prevent="handleRegister">
      <div v-if="error" class="bg-red-50 border border-red-200 text-red-600 px-4 py-3 rounded-md text-sm">
        {{ error }}
      </div>

      <BaseInput
        id="username"
        label="Username"
        v-model="username"
        required
        placeholder="jdoe"
      />

      <BaseInput
        id="email"
        type="email"
        label="Email address"
        v-model="email"
        required
        placeholder="you@example.com"
      />

      <div class="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-md p-4">
        <div class="flex">
          <div class="flex-shrink-0">
            <svg class="h-5 w-5 text-blue-400" viewBox="0 0 20 20" fill="currentColor">
              <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd" />
            </svg>
          </div>
          <div class="ml-3">
            <p class="text-sm text-blue-700 dark:text-blue-300">
              You'll receive an email with a link to set your password after registration.
            </p>
          </div>
        </div>
      </div>

      <div>
        <BaseButton type="submit" block :loading="loading">
          Create Account
        </BaseButton>
      </div>
    </form>
  </AuthLayout>
</template>
