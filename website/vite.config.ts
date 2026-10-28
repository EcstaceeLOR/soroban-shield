import { loadEnv } from 'vite'
import { defineConfig } from 'vitest/config'
import react from '@vitejs/plugin-react'

export default defineConfig(({ mode }) => ({
  // Set VITE_BASE (e.g. /soroban-shield/) when hosting under a sub-path.
  base: loadEnv(mode, '.', 'VITE_').VITE_BASE ?? '/',
  plugins: [react()],
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: ['./src/test/setup.ts'],
    css: false,
  },
}))
