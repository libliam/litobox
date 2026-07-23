import { invoke } from '@tauri-apps/api/core'

// ============ 类型定义 ============

export interface SystemInfo {
  os_name: string
  os_version: string
  os_arch: string
  hostname: string
  uptime_secs: number
  cpu: CpuInfo
  memory: MemoryInfo
  disks: DiskInfo[]
}

export interface CpuInfo {
  brand: string
  core_count: number
  thread_count: number
  frequency_mhz: number
  usage_percent: number
}

export interface MemoryInfo {
  total_bytes: number
  used_bytes: number
  available_bytes: number
}

export interface DiskInfo {
  name: string
  total_bytes: number
  used_bytes: number
  file_system: string
  is_removable: boolean
}

export interface NetworkInfo {
  hostname: string
  interfaces: NetInterface[]
  default_gateway: string
  dns_servers: string[]
  wifi_name: string | null
  active_connections: TcpConnection[]
  listening_ports: ListeningPort[]
}

export interface NetInterface {
  name: string
  mac: string
  ipv4: string[]
  ipv6: string[]
  status: string
}

export interface TcpConnection {
  protocol: string
  local_addr: string
  remote_addr: string
  state: string
  pid: number
}

export interface ListeningPort {
  protocol: string
  local_addr: string
  pid: number
  process_name: string
}

export interface ProcessItem {
  pid: number
  name: string
  cpu_usage: number
  memory_bytes: number
  status: string
  command: string
}

export interface HardwareInfo {
  cpu: CpuSummary
  memory: MemorySummary
  disks: DiskSummary[]
  gpus: GpuInfo[]
  displays: DisplayInfo[]
  audio_devices: AudioDevice[]
  motherboard: MotherboardInfo
  battery: BatteryInfo | null
  usb_devices: UsbDevice[]
}

export interface CpuSummary {
  name: string
  cores: number
  threads: number
  frequency_mhz: number
}

export interface MemorySummary {
  total_gb: number
  used_gb: number
  available_gb: number
}

export interface DiskSummary {
  name: string
  model: string
  size_gb: number
  free_gb: number
  fs_type: string
}

export interface GpuInfo {
  name: string
  driver_version: string
  vram_mb: number
}

export interface DisplayInfo {
  name: string
  resolution: string
}

export interface AudioDevice {
  name: string
  status: string
}

export interface MotherboardInfo {
  manufacturer: string
  product: string
  serial: string
}

export interface BatteryInfo {
  status: string
  charge_percent: number
  estimated_time: string
}

export interface UsbDevice {
  name: string
  device_id: string
}

export interface SoftwareEnv {
  installed_software: SoftwareItem[]
  environment_variables: EnvVar[]
}

export interface SoftwareItem {
  name: string
  version: string
  publisher: string
  install_date: string
}

export interface EnvVar {
  key: string
  value: string
}

export interface StartupItem {
  name: string
  command: string
  location: string
}

// ============ 后台采集类型 ============

export type CollectKind = 'system' | 'network' | 'process' | 'hardware' | 'software'

export interface CollectStartResult {
  task_id: string
  kind: CollectKind
}

export interface CollectCompletePayload {
  kind: CollectKind
  task_id: string
  ok: boolean
  data: unknown
  error: string | null
}

export interface TaskState {
  task_id: string
  kind: CollectKind
  status: 'running' | 'done' | 'error'
  data: unknown
  error: string | null
  updated_at: number
}

// ============ invoke 封装 ============

export function collectSystem(): Promise<CollectStartResult> {
  return invoke<CollectStartResult>('collect_system')
}

export function collectNetwork(): Promise<CollectStartResult> {
  return invoke<CollectStartResult>('collect_network')
}

export function collectProcess(): Promise<CollectStartResult> {
  return invoke<CollectStartResult>('collect_process')
}

export interface KillResult {
  success: boolean
  pid: number
  process_name: string
  message: string
}

export function killProcess(pid: number): Promise<KillResult> {
  return invoke<KillResult>('kill_process', { pid })
}

export interface KillBatchResult {
  success: boolean
  process_name: string
  killed_count: number
  message: string
}

export function killProcessByName(processName: string): Promise<KillBatchResult> {
  return invoke<KillBatchResult>('kill_process_by_name', { processName })
}

export function collectHardware(): Promise<CollectStartResult> {
  return invoke<CollectStartResult>('collect_hardware')
}

export function collectSoftware(): Promise<CollectStartResult> {
  return invoke<CollectStartResult>('collect_software')
}

export function getCollectStatus(kind: CollectKind): Promise<TaskState | null> {
  return invoke<TaskState | null>('get_collect_status', { kind })
}

// ============ 服务管理类型 ============

export interface ServiceItem {
  name: string
  display_name: string
  status: string
  start_type: string
  description: string
}

export interface ServiceResult {
  success: boolean
  service_name: string
  action: string
  message: string
}

export function getServices(): Promise<ServiceItem[]> {
  return invoke<ServiceItem[]>('get_services')
}

export function startService(name: string): Promise<ServiceResult> {
  return invoke<ServiceResult>('start_service', { name })
}

export function stopService(name: string): Promise<ServiceResult> {
  return invoke<ServiceResult>('stop_service', { name })
}

export function restartService(name: string): Promise<ServiceResult> {
  return invoke<ServiceResult>('restart_service', { name })
}

// ============ 格式化工具函数 ============

export function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(1024))
  const idx = Math.min(i, units.length - 1)
  return (bytes / Math.pow(1024, idx)).toFixed(idx === 0 ? 0 : 1) + ' ' + units[idx]
}

export function formatUptime(secs: number): string {
  const days = Math.floor(secs / 86400)
  const hours = Math.floor((secs % 86400) / 3600)
  const mins = Math.floor((secs % 3600) / 60)
  const parts: string[] = []
  if (days > 0) parts.push(`${days}天`)
  if (hours > 0) parts.push(`${hours}小时`)
  parts.push(`${mins}分钟`)
  return parts.join('')
}

export function formatTimestamp(): string {
  const d = new Date()
  const pad = (n: number) => n.toString().padStart(2, '0')
  return `${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`
}

// ============ 网络连接查看器 ============

export interface NetworkConnection {
  protocol: string
  local_addr: string
  remote_addr: string
  state: string
  pid: number
  process_name: string
  process_path: string
}

export function getNetworkConnections(): Promise<NetworkConnection[]> {
  return invoke<NetworkConnection[]>('get_network_connections')
}

// ============ 计划任务管理器 ============

export interface ScheduledTask {
  task_name: string
  task_path: string
  state: string         // "Ready" / "Running" / "Disabled" / "Unknown"
  description: string
  author: string
  last_run_time: string
  last_task_result: number
  next_run_time: string
  trigger_brief: string
  action_brief: string
  principal: string
  is_system: boolean
  triggers_json: string
  actions_json: string
}

export interface TaskOpResult {
  success: boolean
  task_name: string
  action: string
  message: string
}

export function getScheduledTasks(includeSystem: boolean): Promise<ScheduledTask[]> {
  return invoke<ScheduledTask[]>('get_scheduled_tasks', { includeSystem })
}

export function enableScheduledTask(taskName: string, taskPath: string): Promise<TaskOpResult> {
  return invoke<TaskOpResult>('enable_scheduled_task', { taskName, taskPath })
}

export function disableScheduledTask(taskName: string, taskPath: string): Promise<TaskOpResult> {
  return invoke<TaskOpResult>('disable_scheduled_task', { taskName, taskPath })
}

export function runScheduledTask(taskName: string, taskPath: string): Promise<TaskOpResult> {
  return invoke<TaskOpResult>('run_scheduled_task', { taskName, taskPath })
}

export function deleteScheduledTask(taskName: string, taskPath: string): Promise<TaskOpResult> {
  return invoke<TaskOpResult>('delete_scheduled_task', { taskName, taskPath })
}

/**
 * 前端镜像触发器格式化（与 Rust format_trigger_brief 一致），用于详情面板渲染
 */
export function formatTriggerBrief(triggerType: string, startBoundary: string): string {
  const time = extractTimeFromBoundary(startBoundary)
  switch (triggerType) {
    case 'MSFT_TaskDailyTrigger': return `每日 ${time}`
    case 'MSFT_TaskWeeklyTrigger': return `每周 ${time}`
    case 'MSFT_TaskLogonTrigger': return '登录时'
    case 'MSFT_TaskBootTrigger': return '启动时'
    case 'MSFT_TaskTimeTrigger': return `${time} 一次性`
    default: return '自定义'
  }
}

function extractTimeFromBoundary(boundary: string): string {
  if (!boundary) return '—'
  const parts = boundary.split('T')
  if (parts.length < 2) return '—'
  const timePart = parts[1].split('+')[0]
  return timePart.length >= 5 ? timePart.slice(0, 5) : '—'
}

// ============ 自检 ============
// ponytail: 纯函数自检，确保格式化逻辑正确
console.assert(formatBytes(0) === '0 B', 'formatBytes(0)')
console.assert(formatBytes(1024) === '1.0 KB', 'formatBytes(1024)')
console.assert(formatBytes(1073741824) === '1.0 GB', 'formatBytes(1GB)')
console.assert(formatUptime(3661) === '1小时1分钟', 'formatUptime(3661)')
console.assert(formatUptime(90061) === '1天1小时1分钟', 'formatUptime(90061)')
console.assert(formatTriggerBrief('MSFT_TaskDailyTrigger', '2026-07-23T09:00:00') === '每日 09:00', 'daily trigger')
console.assert(formatTriggerBrief('MSFT_TaskWeeklyTrigger', '2026-07-23T08:30:00') === '每周 08:30', 'weekly trigger')
console.assert(formatTriggerBrief('MSFT_TaskLogonTrigger', '') === '登录时', 'logon trigger')
console.assert(formatTriggerBrief('MSFT_TaskBootTrigger', '') === '启动时', 'boot trigger')
console.assert(formatTriggerBrief('MSFT_TaskTimeTrigger', '2026-07-23T15:00:00') === '15:00 一次性', 'time trigger')
console.assert(formatTriggerBrief('', '') === '自定义', 'empty trigger')
console.assert(formatTriggerBrief('MSFT_TaskUnknown', '2026-07-23T09:00:00') === '自定义', 'unknown trigger')
console.assert(extractTimeFromBoundary('') === '—', 'empty boundary')
console.assert(extractTimeFromBoundary('invalid') === '—', 'malformed boundary')
console.assert(extractTimeFromBoundary('2026-07-23T09:00:00+08:00') === '09:00', 'boundary with tz')
