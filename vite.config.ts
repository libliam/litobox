import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'
import { readFileSync } from 'fs'
import { viteStaticCopy } from 'vite-plugin-static-copy'

const pkg = JSON.parse(readFileSync('./package.json', 'utf-8').replace(/^\uFEFF/, ''))

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src')
    }
  },
  clearScreen: false,
  server: {
    // 固定 IPv4：部分环境（Node 新版本）localhost 只解析到 ::1，
    // 而 WebView2 走 127.0.0.1 会连不上，表现为白屏、窗口停在 16×16
    host: '127.0.0.1',
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ['**/src-tauri/**']
    },
    headers: {
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp'
    }
  },
  optimizeDeps: {
    exclude: ['onnxruntime-web']
  },
  define: {
    __APP_VERSION__: JSON.stringify(pkg.version),
    // ponytail: @iarna/toml 是 Node.js 库，引用浏览器不存在的 global 变量，polyfill 成 globalThis
    global: 'globalThis',
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
          'pdf': ['pdfjs-dist', 'pdf-lib'],
          'ocr': ['@paddleocr/paddleocr-js', 'onnxruntime-web'],
          'crypto': ['crypto-js'],
          'markdown': ['markdown-it'],
          'qrcode': ['qrcode', 'jsqr'],
          'utils': ['lodash', 'js-base64', 'json5', 'colord', 'diff'],
        }
      }
    }
  }
})