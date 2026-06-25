import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

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
    },
    // 允许 Vite 将 .mjs 文件作为静态资源提供
    headers: {
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp'
    }
  },
  optimizeDeps: {
    exclude: ['onnxruntime-web']
  },
  build: {
    // 禁用 modulepreload，避免启动时预加载大文件导致白屏
    modulePreload: false,
    rollupOptions: {
      output: {
        manualChunks: {
          // UI 库
          'element-plus': ['element-plus'],
          // PDF 处理
          'pdf': ['pdfjs-dist', 'pdf-lib'],
          // OCR 引擎（体积最大，按需加载）
          'ocr': ['@paddleocr/paddleocr-js', 'onnxruntime-web'],
          // 加密相关
          'crypto': ['crypto-js'],
          // Markdown 解析
          'markdown': ['markdown-it'],
          // 二维码
          'qrcode': ['qrcode', 'jsqr'],
          // 其他工具库
          'utils': ['lodash', 'js-base64', 'json5', 'colord', 'diff'],
        }
      }
    }
  }
})