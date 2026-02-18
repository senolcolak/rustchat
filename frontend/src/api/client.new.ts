// API Client (New) - Updated for feature-based architecture
// This version avoids circular dependencies with auth store

import axios from 'axios'
import { getGlobalAuthToken } from '../features/auth'
import { normalizeIdsDeep, shouldNormalizeHttpPayload } from '../utils/idCompat'

const client = axios.create({
  baseURL: import.meta.env.VITE_API_URL || '/api/v1',
})

client.interceptors.request.use(config => {
  // Get token from global function (no circular dependency)
  const token = getGlobalAuthToken()
  if (token) {
    config.headers.Authorization = `Bearer ${token}`
  }

  if (shouldNormalizeHttpPayload(config.params)) {
    config.params = normalizeIdsDeep(config.params)
  }
  if (shouldNormalizeHttpPayload(config.data)) {
    config.data = normalizeIdsDeep(config.data)
  }

  return config
})

client.interceptors.response.use(
  response => {
    if (shouldNormalizeHttpPayload(response.data)) {
      response.data = normalizeIdsDeep(response.data)
    }
    return response
  },
  error => {
    if (error.response?.data && shouldNormalizeHttpPayload(error.response.data)) {
      error.response.data = normalizeIdsDeep(error.response.data)
    }
    // Note: 401 handling is done in auth service now
    return Promise.reject(error)
  }
)

export default client
