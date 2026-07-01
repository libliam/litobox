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
  return invoke('db_note_list', { parent_id: parentId })
}

export async function noteCreate(name: string, noteType: 'folder' | 'file', parentId: number | null): Promise<NoteItem> {
  return invoke('db_note_create', { name, note_type: noteType, parent_id: parentId })
}

export async function noteRename(id: number, newName: string): Promise<NoteItem> {
  return invoke('db_note_rename', { id, new_name: newName })
}

export async function noteDelete(id: number): Promise<void> {
  return invoke('db_note_delete', { id })
}

export async function noteMove(id: number, newParentId: number | null): Promise<NoteItem> {
  return invoke('db_note_move', { id, new_parent_id: newParentId })
}

export async function noteRead(filePath: string): Promise<NoteFileContent> {
  return invoke('note_read', { filePath })
}

export async function noteWrite(filePath: string, content: string): Promise<void> {
  return invoke('note_write', { filePath, content })
}

export async function noteEnsureDraft(): Promise<NoteItem> {
  return invoke('db_note_ensure_draft', {})
}

export async function noteGetLastOpened(): Promise<number | null> {
  return invoke('db_note_get_last_opened', {})
}

export async function noteSetLastOpened(id: number): Promise<void> {
  return invoke('db_note_set_last_opened', { id })
}

export async function saveFileWithDialog(dataBase64: string, filename: string, defaultExt: string): Promise<string> {
  return invoke('save_file_with_dialog', { dataBase64, filename, defaultExt })
}

export async function saveTextWithDialog(content: string, filename: string): Promise<string> {
  return invoke('save_text_with_dialog', { content, filename })
}
