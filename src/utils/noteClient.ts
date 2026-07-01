import { invoke } from '@tauri-apps/api/core'

export interface NoteItem {
  id: number
  parent_id: number | null
  name: string
  type: 'folder' | 'file'
  file_path: string | null
  language: string
  created_at: string
  updated_at: string
}

export interface NoteFileContent {
  content: string
  encoding: string
  size: number
}

export async function noteList(parentId: number | null): Promise<NoteItem[]> {
  return invoke('note_list', { parentId })
}

export async function noteCreate(name: string, noteType: 'folder' | 'file', parentId: number | null): Promise<NoteItem> {
  return invoke('note_create', { name, noteType, parentId })
}

export async function noteRename(id: number, newName: string): Promise<NoteItem> {
  return invoke('note_rename', { id, newName })
}

export async function noteDelete(id: number): Promise<void> {
  return invoke('note_delete', { id })
}

export async function noteMove(id: number, newParentId: number | null): Promise<NoteItem> {
  return invoke('note_move', { id, newParentId })
}

export async function noteRead(filePath: string): Promise<NoteFileContent> {
  return invoke('note_read', { filePath })
}

export async function noteWrite(filePath: string, content: string): Promise<void> {
  return invoke('note_write', { filePath, content })
}
