import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'
import { readFileSync } from 'fs'

const pkg = JSON.parse(readFileSync('./package.json', 'utf-8'))

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src')
    }
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ['**/src-tauri/**']
    }
  },
  define: {
    __APP_VERSION__: JSON.stringify(pkg.version),
  },
  build: {
    modulePreload: false,
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'index.html'),
      },
      output: {
        manualChunks: {
          'element-plus': ['element-plus'],
          'crypto': ['crypto-js'],
          'markdown': ['markdown-it'],
          'qrcode': ['qrcode', 'jsqr'],
          'utils': ['lodash', 'js-base64', 'json5', 'colord', 'diff'],
        }
      }
    }
  }
})