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

// 历史详情相关
export interface HistoryDetail {
  id?: number;
  history_id: number;
  input_full: string | null;
  output_full: string | null;
  options_json: string;
  created_at?: string;
}

export async function addHistoryDetail(detail: HistoryDetail): Promise<number> {
  return invoke('cmd_db_add_history_detail', { detail });
}

export async function getHistoryDetail(historyId: number): Promise<HistoryDetail | null> {
  return invoke('cmd_db_get_history_detail', { historyId });
}

export async function deleteHistoryDetailsForHistory(historyId: number): Promise<void> {
  return invoke('cmd_db_delete_history_details_for_history', { historyId });
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

export async function addOcrHistory(thumbnail: string, originalUrl: string, text: string, time: string): Promise<void> {
  return invoke('cmd_db_add_ocr_history', { thumbnail, originalUrl, text, time });
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

// HTTP 环境相关
export interface HttpEnvironment {
  id: string;
  name: string;
  base_url: string;
  variables_json: string;
  created_at: string;
  updated_at: string;
}

export async function listHttpEnvironments(): Promise<HttpEnvironment[]> {
  return invoke('cmd_db_list_http_environments');
}

export async function saveHttpEnvironment(env: HttpEnvironment): Promise<void> {
  return invoke('cmd_db_save_http_environment', { env });
}

export async function deleteHttpEnvironment(id: string): Promise<void> {
  return invoke('cmd_db_delete_http_environment', { id });
}

// HTTP 历史相关
export interface HttpHistoryItem {
  id?: number;
  method: string;
  url: string;
  headers_json: string;
  body: string | null;
  body_type: string;
  env_name: string | null;
  status: number | null;
  created_at: string;
}

export async function listHttpHistory(limit: number): Promise<HttpHistoryItem[]> {
  return invoke('cmd_db_list_http_history', { limit });
}

export async function addHttpHistory(record: HttpHistoryItem): Promise<number> {
  return invoke('cmd_db_add_http_history', { record });
}

export async function clearHttpHistory(): Promise<void> {
  return invoke('cmd_db_clear_http_history');
}

// HTTP 收藏相关
export interface HttpBookmark {
  id: string;
  name: string;
  method: string;
  url: string;
  headers_json: string;
  body: string | null;
  body_type: string;
  created_at: string;
  updated_at: string;
}

export async function listHttpBookmarks(): Promise<HttpBookmark[]> {
  return invoke('cmd_db_list_http_bookmarks');
}

export async function saveHttpBookmark(bookmark: HttpBookmark): Promise<void> {
  return invoke('cmd_db_save_http_bookmark', { bookmark });
}

export async function deleteHttpBookmark(id: string): Promise<void> {
  return invoke('cmd_db_delete_http_bookmark', { id });
}

// 快捷键相关
export async function registerShortcuts(shortcutsJson: string): Promise<void> {
  return invoke('cmd_db_register_shortcuts', { shortcutsJson });
}