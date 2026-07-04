#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod clipboard;
mod db;
mod file_encoding;
mod file_saver;
mod js_executor;
mod http_cmd;
mod note_manager;
mod system_info;
mod sqlite_viewer;

use tauri::{Manager, Emitter};
use tauri_plugin_dialog::{DialogExt, MessageDialogBuilder, MessageDialogButtons, MessageDialogKind};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            clipboard::start_clipboard_monitor,
            clipboard::stop_clipboard_monitor,
            clipboard::is_monitoring,
            clipboard::copy_to_clipboard,
            file_encoding::read_file_with_encoding,
            file_encoding::convert_file_encoding,
            file_encoding::detect_file_encoding,
            file_encoding::batch_read_txt_files,
            file_encoding::batch_replace_in_files,
            file_encoding::batch_convert_encoding,
            file_saver::save_file_with_dialog,
            file_saver::save_text_with_dialog,
            js_executor::execute_js,
            http_cmd::send_http_request,
            // 数据库命令
            db::cmd_db_get_config,
            db::cmd_db_set_config,
            db::cmd_db_add_history,
            db::cmd_db_get_history,
            db::cmd_db_clear_history,
            db::cmd_db_search_history,
            db::cmd_db_add_history_detail,
            db::cmd_db_get_history_detail,
            db::cmd_db_delete_history_details_for_history,
            db::cmd_db_list_workflows,
            db::cmd_db_save_workflow,
            db::cmd_db_delete_workflow,
            db::cmd_db_list_variables,
            db::cmd_db_set_variable,
            db::cmd_db_delete_variable,
            db::cmd_db_get_variable,
            db::cmd_db_export_all,
            db::cmd_db_import_all,
            db::cmd_db_check_migrated,
            db::cmd_db_migrate_from_localstorage,
            db::cmd_db_list_snippets,
            db::cmd_db_save_snippet,
            db::cmd_db_delete_snippet,
            db::cmd_db_list_recent_tools,
            db::cmd_db_add_recent_tool,
            db::cmd_db_list_ocr_history,
            db::cmd_db_add_ocr_history,
            db::cmd_db_clear_ocr_history,
            db::cmd_db_list_clipboard_history,
            db::cmd_db_search_clipboard_history,
            db::cmd_db_add_clipboard_record,
            db::cmd_db_delete_clipboard_record,
            db::cmd_db_clear_clipboard_history,
            db::cmd_db_list_http_environments,
            db::cmd_db_save_http_environment,
            db::cmd_db_delete_http_environment,
            db::cmd_db_list_http_history,
            db::cmd_db_add_http_history,
            db::cmd_db_clear_http_history,
            db::cmd_db_list_http_bookmarks,
            db::cmd_db_save_http_bookmark,
            db::cmd_db_delete_http_bookmark,
            // Notes 命令
            db::db_note_list,
            db::db_note_create,
            db::db_note_rename,
            db::db_note_delete,
            db::db_note_move,
            db::db_note_ensure_draft,
            db::db_note_get_last_opened,
            db::db_note_set_last_opened,
            db::open_notes_folder,
            // Note manager 命令
            note_manager::note_read,
            note_manager::note_write,
            db::cmd_db_register_shortcuts,
            system_info::is_admin,
            system_info::get_system_info,
            system_info::get_network_info,
            system_info::get_process_list,
            system_info::get_hardware_info,
            system_info::get_software_env,
            // SQLite 查看器命令
            sqlite_viewer::sqlite_list_tables,
            sqlite_viewer::sqlite_get_schema,
            sqlite_viewer::sqlite_query,
            sqlite_viewer::sqlite_table_preview,
            sqlite_viewer::sqlite_export_csv,
            sqlite_viewer::sqlite_get_app_db_path,
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            
            // 关闭按钮二次确认
            let main_window = app.get_webview_window("main").unwrap();
            main_window.clone().on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let window = main_window.clone();
                    MessageDialogBuilder::new(
                        window.dialog().clone(),
                        "退出确认",
                        "确定要退出栗的百宝箱吗？",
                    )
                    .buttons(MessageDialogButtons::YesNo)
                    .kind(MessageDialogKind::Warning)
                    .show(move |confirmed| {
                        if confirmed {
                            std::process::exit(0);
                        }
                    });
                }
            });
            
            let shortcuts = db::db_read_shortcuts();
            
            let manager = app.global_shortcut();
            
            for (tool_id, shortcut_str) in shortcuts {
                let shortcut: Shortcut = match shortcut_str.parse() {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                let tool = tool_id.to_string();
                let h = handle.clone();
                
                manager.on_shortcut(shortcut, move |_app, _sc, event| {
                    if let tauri_plugin_global_shortcut::ShortcutState::Pressed = event.state {
                        if let Some(window) = h.get_webview_window("main") {
                            let _ = window.emit("global-shortcut-triggered", &tool);
                        }
                    }
                }).unwrap_or_else(|e| {
                    eprintln!("注册快捷键 {} 失败: {}", shortcut_str, e);
                });
            }
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
