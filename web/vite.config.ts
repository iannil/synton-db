import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'
import path from 'path'

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    react({ jsxImportSource: 'react' }),
    tailwindcss(),
  ],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  server: {
    port: 5173,
    proxy: {
      '/health': {
        target: 'http://localhost:5570',
        changeOrigin: true,
      },
      '/stats': {
        target: 'http://localhost:5570',
        changeOrigin: true,
      },
      '/nodes': {
        target: 'http://localhost:5570',
        changeOrigin: true,
      },
      '/edges': {
        target: 'http://localhost:5570',
        changeOrigin: true,
      },
      '/query': {
        target: 'http://localhost:5570',
        changeOrigin: true,
      },
      '/hybrid_search': {
        target: 'http://localhost:5570',
        changeOrigin: true,
      },
      '/traverse': {
        target: 'http://localhost:5570',
        changeOrigin: true,
      },
      '/bulk': {
        target: 'http://localhost:5570',
        changeOrigin: true,
      },
    },
  },
  esbuild: {
    jsx: 'automatic',
  },
})
