import { invoke } from '@tauri-apps/api/core'

// ============ 类型定义 ============

export interface ContextMenuItem {
  scope: string          // "file" | "folder" | "desktop"
  key_name: string
  display_name: string
  command: string
  icon: string
  kind: string           // "shell" | "shellex"
  source_path: string    // 来源基础路径
  source_label: string   // 来源友好名称
  item_path: string      // 该项完整注册表路径
  dll_path: string       // COM 处理器 DLL
  is_custom: boolean
  is_enabled: boolean
}

export interface ContextMenuResult {
  success: boolean
  message: string
}

export interface BackupInfo {
  filename: string
  timestamp_ms: number
  scope: string
  size: number
  path: string
}

// ============ 命令 ============

export function cmListItems(scope: string): Promise<ContextMenuItem[]> {
  return invoke<ContextMenuItem[]>('cm_list_items', { scope })
}

export function cmAddItem(
  scope: string,
  keyName: string,
  displayName: string,
  command: string,
  icon: string
): Promise<ContextMenuResult> {
  return invoke<ContextMenuResult>('cm_add_item', {
    scope,
    key_name: keyName,
    display_name: displayName,
    command,
    icon,
  })
}

export function cmDeleteItem(
  scope: string,
  itemPath: string,
  displayName: string
): Promise<ContextMenuResult> {
  return invoke<ContextMenuResult>('cm_delete_item', {
    scope,
    item_path: itemPath,
    display_name: displayName,
  })
}

export function cmBackup(scope: string): Promise<BackupInfo[]> {
  return invoke<BackupInfo[]>('cm_backup', { scope })
}

export function cmRestore(backupPath: string): Promise<ContextMenuResult> {
  return invoke<ContextMenuResult>('cm_restore', { backup_path: backupPath })
}

export function cmListBackups(): Promise<BackupInfo[]> {
  return invoke<BackupInfo[]>('cm_list_backups')
}

export function cmDeleteBackup(filename: string): Promise<ContextMenuResult> {
  return invoke<ContextMenuResult>('cm_delete_backup', { filename })
}
