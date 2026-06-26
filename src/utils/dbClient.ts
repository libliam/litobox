import { invoke } from '@tauri-apps/api/core';

// 数据模型类型定义
export interface HistoryRecord {
  id?: number;
  tool: string;
  action: string;
  input_preview: string;
  output_preview: string;
  created_at?: string;
}

export interface Workflow {
  id: string;
  name: string;
  description: string;
  steps_json: string;
  created_at: string;
  updated_at: string;
}

export interface PoolVariable {
  id: string;
  name: string;
  value: string;
  source: string;
  created_at: string;
  last_used_at?: string;
}

// 配置相关
export async function getConfig(key: string): Promise<string> {
  return invoke('cmd_db_get_config', { key });
}

export async function setConfig(key: string, value: string): Promise<void> {
  return invoke('cmd_db_set_config', { key, value });
}

// 历史记录相关
export async function addHistory(record: HistoryRecord): Promise<number> {
  return invoke('cmd_db_add_history', { record });
}

export async function getHistory(limit: number, offset: number): Promise<HistoryRecord[]> {
  return invoke('cmd_db_get_history', { limit, offset });
}

export async function clearHistory(): Promise<void> {
  return invoke('cmd_db_clear_history');
}

export async function searchHistory(query: string, limit: number): Promise<HistoryRecord[]> {
  return invoke('cmd_db_search_history', { query, limit });
}

// 工作流相关
export async function listWorkflows(): Promise<Workflow[]> {
  return invoke('cmd_db_list_workflows');
}

export async function saveWorkflow(workflow: Workflow): Promise<void> {
  return invoke('cmd_db_save_workflow', { workflow });
}

export async function deleteWorkflow(id: string): Promise<void> {
  return invoke('cmd_db_delete_workflow', { id });
}

// 变量池相关
export async function listVariables(): Promise<PoolVariable[]> {
  return invoke('cmd_db_list_variables');
}

export async function setVariable(name: string, value: string, source: string = 'manual'): Promise<void> {
  return invoke('cmd_db_set_variable', { name, value, source });
}

export async function deleteVariable(name: string): Promise<void> {
  return invoke('cmd_db_delete_variable', { name });
}

export async function getVariable(name: string): Promise<string> {
  return invoke('cmd_db_get_variable', { name });
}

// 导入导出相关
export async function exportAll(): Promise<string> {
  return invoke('cmd_db_export_all');
}

export async function importAll(data: string): Promise<void> {
  return invoke('cmd_db_import_all', { data });
}

export async function saveFileWithDialog(dataBase64: string, filename: string, defaultExt: string): Promise<string> {
  return invoke('save_file_with_dialog', { dataBase64, filename, defaultExt });
}

// 迁移相关
export async function checkMigrated(): Promise<boolean> {
  return invoke('cmd_db_check_migrated');
}

export async function migrateFromLocalStorage(data: string): Promise<void> {
  return invoke('cmd_db_migrate_from_localstorage', { data });
}

// 代码片段相关
export interface Snippet {
  id: string;
  title: string;
  lang: string;
  content: string;
  note: string;
  created_at: string;
  updated_at: string;
}

export async function listSnippets(): Promise<Snippet[]> {
  return invoke('cmd_db_list_snippets');
}

export async function saveSnippet(snippet: Snippet): Promise<void> {
  return invoke('cmd_db_save_snippet', { snippet });
}

export async function deleteSnippet(id: string): Promise<void> {
  return invoke('cmd_db_delete_snippet', { id });
}

// 最近工具相关
export async function listRecentTools(limit: number = 8): Promise<string[]> {
  return invoke('cmd_db_list_recent_tools', { limit });
}

export async function addRecentTool(toolId: string): Promise<void> {
  return invoke('cmd_db_add_recent_tool', { toolId });
}

// OCR 历史相关
export interface OcrHistoryRecord {
  id?: number;
  thumbnail: string;
  original_url: string;
  text: string;
  time: string;
}

export async function listOcrHistory(limit: number = 10): Promise<OcrHistoryRecord[]> {
  return invoke('cmd_db_list_ocr_history', { limit });
}

export async function addOcrHistory(thumbnail: string, original_url: string, text: string, time: string): Promise<void> {
  return invoke('cmd_db_add_ocr_history', { thumbnail, original_url, text, time });
}

export async function clearOcrHistory(): Promise<void> {
  return invoke('cmd_db_clear_ocr_history');
}

// 剪贴板历史相关
export interface ClipboardRecord {
  id?: number;
  text: string;
  timestamp: string;
}

export async function listClipboardHistory(limit: number, offset: number): Promise<ClipboardRecord[]> {
  return invoke('cmd_db_list_clipboard_history', { limit, offset });
}

export async function searchClipboardHistory(query: string, limit: number): Promise<ClipboardRecord[]> {
  return invoke('cmd_db_search_clipboard_history', { query, limit });
}

export async function addClipboardRecord(text: string): Promise<void> {
  return invoke('cmd_db_add_clipboard_record', { text });
}

export async function deleteClipboardRecord(id: number): Promise<void> {
  return invoke('cmd_db_delete_clipboard_record', { id });
}

export async function clearClipboardHistory(): Promise<void> {
  return invoke('cmd_db_clear_clipboard_history');
}