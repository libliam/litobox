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
