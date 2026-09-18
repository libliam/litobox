import { invoke } from '@tauri-apps/api/core'

// ============ 类型定义 ============

export interface PingReply {
  seq: number
  bytes: number
  time_ms: number
  ttl: number
  from: string
  timeout: boolean
}

export interface PingStats {
  sent: number
  received: number
  lost: number
  loss_percent: number
  min_ms: number
  max_ms: number
  avg_ms: number
}

export interface TraceHop {
  hop: number
  rtt1: string
  rtt2: string
  rtt3: string
  ip: string
  hostname: string
  timeout: boolean
}

export interface TaskStatus {
  running: boolean
  task_type: string
}

// ============ Ping 命令 ============

export function pingEcho(host: string, count: number, timeoutMs: number, size: number): Promise<string> {
  return invoke<string>('ping_echo', { host, count, timeoutMs, size })
}

export function pingStart(host: string, count: number, timeoutMs: number, size: number): Promise<void> {
  console.log('[pingClient] pingStart params:', { host, count, timeoutMs, size })
  return invoke<void>('ping_start', { host, count, timeoutMs, size })
}

export function pingCancel(): Promise<void> {
  return invoke<void>('ping_cancel')
}

export function pingGetStatus(): Promise<TaskStatus> {
  return invoke<TaskStatus>('ping_get_status')
}

// ============ Traceroute 命令 ============

export function tracertStart(host: string, maxHops: number, timeoutMs: number): Promise<void> {
  return invoke<void>('tracert_start', { host, maxHops, timeoutMs })
}

export function tracertCancel(): Promise<void> {
  return invoke<void>('tracert_cancel')
}
