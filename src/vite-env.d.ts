/// <reference types="vite/client" />

declare module '*.vue' {
  import type { DefineComponent } from 'vue'
  const component: DefineComponent<{}, {}, any>
  export default component
}

declare const __APP_VERSION__: string

// js-yaml 类型声明（@types/js-yaml 未安装，声明本项目用到的最小接口）
declare module 'js-yaml' {
  export function load(input: string, options?: Record<string, unknown>): any
  export function loadAll(input: string, options?: Record<string, unknown>): any[]
  export function dump(obj: unknown, options?: Record<string, unknown>): string
}

// pdf-lib-encrypt 类型声明
declare module 'pdf-lib-encrypt' {
  export function configure(pdfLib: any): void
  export interface LockOptions {
    algo?: 'aes256' | 'rc4'
    permissions?: number
  }
  export function lock(bytes: Uint8Array, password: string, opts?: LockOptions): Promise<Uint8Array>
  export function unlockInPlace(pdfDoc: any, password: string): Promise<boolean>
}
