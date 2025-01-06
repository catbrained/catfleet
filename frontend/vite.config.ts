import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

const backendUrl = 'http://127.0.0.1:3000';

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  server: {
    proxy: {
      '/status': { target: backendUrl }
    }
  }
})
