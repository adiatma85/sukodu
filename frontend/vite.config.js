import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  test: {
    // Unit tests live next to the code in src/; e2e/ is Playwright's.
    include: ['src/**/*.test.{js,jsx}'],
    environment: 'node',
  },
})
