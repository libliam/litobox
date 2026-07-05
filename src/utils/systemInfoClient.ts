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

// ============ invoke 封装 ============

export function getSystemInfo(): Promise<SystemInfo> {
  return invoke<SystemInfo>('get_system_info')
}

export function getNetworkInfo(): Promise<NetworkInfo> {
  return invoke<NetworkInfo>('get_network_info')
}

export function getProcessList(): Promise<ProcessItem[]> {
  return invoke<ProcessItem[]>('get_process_list')
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

export function getHardwareInfo(): Promise<HardwareInfo> {
  return invoke<HardwareInfo>('get_hardware_info')
}

export function getSoftwareEnv(): Promise<SoftwareEnv> {
  return invoke<SoftwareEnv>('get_software_env')
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

// ============ 自检 ============
// ponytail: 纯函数自检，确保格式化逻辑正确
console.assert(formatBytes(0) === '0 B', 'formatBytes(0)')
console.assert(formatBytes(1024) === '1.0 KB', 'formatBytes(1024)')
console.assert(formatBytes(1073741824) === '1.0 GB', 'formatBytes(1GB)')
console.assert(formatUptime(3661) === '1小时1分钟', 'formatUptime(3661)')
console.assert(formatUptime(90061) === '1天1小时1分钟', 'formatUptime(90061)')
