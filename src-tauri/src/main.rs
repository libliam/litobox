#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// ponytail: debug 模式输出日志到 stderr，release 模式编译时移除（零开销）
// 必须放在 mod 声明之前，这样子模块才能使用该宏
#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            eprintln!($($arg)*)
        }
    };
}

mod clipboard;
mod db;
mod file_encoding;
mod file_saver;
mod js_executor;
mod http_cmd;
mod system_info;
mod sqlite_viewer;
mod disk_analyzer;
mod file_searcher;
mod hotkey_probe;
mod hotkey_data;
mod hosts_manager;
mod network_connections;
mod scheduled_tasks;
mod startup_items;
mod env_vars;
mod boost;
mod cert_reader;
mod password_vault;
mod file_renamer;
mod quick_launch;

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
            clipboard::clipboard_get_image,
            clipboard::clipboard_set_image,
            clipboard::clipboard_read_image_file,
            clipboard::clipboard_delete_image_file,
            clipboard::clipboard_clear_image_cache,
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
            db::cmd_db_register_shortcuts,
            system_info::is_admin,
            system_info::collect_system,
            system_info::collect_network,
            system_info::collect_process,
            system_info::collect_hardware,
            system_info::collect_software,
            system_info::get_collect_status,
            system_info::kill_process,
            system_info::kill_process_by_name,
            // 服务管理命令
            system_info::get_services,
            system_info::start_service,
            system_info::stop_service,
            system_info::restart_service,
            network_connections::get_network_connections,
            // 计划任务管理命令
            scheduled_tasks::get_scheduled_tasks,
            scheduled_tasks::enable_scheduled_task,
            scheduled_tasks::disable_scheduled_task,
            scheduled_tasks::run_scheduled_task,
            scheduled_tasks::delete_scheduled_task,
            startup_items::get_startup_items,
            startup_items::enable_startup_item,
            startup_items::disable_startup_item,
            startup_items::delete_startup_item,
            startup_items::add_startup_item,
            env_vars::get_env_vars,
            env_vars::set_env_var,
            env_vars::delete_env_var,
            // 一键加速命令
            boost::boost_scan,
            boost::boost_execute,
            cert_reader::read_cert_store,
            cert_reader::get_cert_detail,
            cert_reader::parse_cert_file,
            // 密码保管箱命令
            password_vault::pv_has_master_password,
            password_vault::pv_set_master_password,
            password_vault::pv_verify_master_password,
            password_vault::pv_list_credentials,
            password_vault::pv_search_credentials,
            password_vault::pv_add_credential,
            password_vault::pv_update_credential,
            password_vault::pv_delete_credential,
            password_vault::pv_reset_master_password,
            password_vault::pv_import_credentials,
            password_vault::pv_change_master_password,
            password_vault::pv_check_duplicates,
            password_vault::pv_batch_delete,
            // 文件批量重命名命令
            file_renamer::rename_list_files,
            file_renamer::rename_preview,
            file_renamer::rename_execute,
            file_renamer::rename_undo,
            file_renamer::rename_pick_folder,
            // SQLite 查看器命令
            sqlite_viewer::sqlite_list_tables,
            sqlite_viewer::sqlite_get_schema,
            sqlite_viewer::sqlite_query,
            sqlite_viewer::sqlite_table_preview,
            sqlite_viewer::sqlite_export_csv,
            sqlite_viewer::sqlite_get_app_db_path,
            // 磁盘分析命令
            disk_analyzer::disk_scan_start,
            disk_analyzer::disk_scan_cancel,
            disk_analyzer::disk_scan_status,
            disk_analyzer::disk_get_summary,
            disk_analyzer::disk_get_folders,
            disk_analyzer::disk_get_top_files,
            disk_analyzer::disk_get_extension_stats,
            disk_analyzer::disk_get_duplicates,
            disk_analyzer::disk_delete_files,
            disk_analyzer::disk_clear_scan,
            disk_analyzer::disk_locate_in_explorer,
            // 全文搜索命令
            file_searcher::file_search_start,
            file_searcher::file_search_cancel,
            file_searcher::file_search_status,
            file_searcher::file_search_get_summary,
            file_searcher::file_search_get_results,
            file_searcher::file_search_clear,
            // 全局快捷键占用查看器命令
            hotkey_probe::hotkey_probe_start,
            hotkey_probe::hotkey_probe_cancel,
            hotkey_probe::hotkey_probe_status,
            hotkey_probe::hotkey_probe_get_results,
            hotkey_probe::hotkey_probe_export_csv,
            // Hosts 文件管理器命令
            hosts_manager::hosts_read,
            hosts_manager::hosts_save,
            hosts_manager::hosts_check_admin,
            hosts_manager::hosts_list_backups,
            hosts_manager::hosts_preview_backup,
            hosts_manager::hosts_restore_backup,
            hosts_manager::hosts_delete_backup,
            hosts_manager::hosts_create_backup,
            hosts_manager::hosts_profile_list,
            hosts_manager::hosts_profile_load,
            hosts_manager::hosts_profile_save,
            hosts_manager::hosts_profile_delete,
            hosts_manager::hosts_profile_apply,
            // 快速启动命令
            quick_launch::ql_search,
            quick_launch::ql_index_status,
            quick_launch::ql_build_index,
            quick_launch::ql_rebuild_index,
            quick_launch::ql_cancel_index,
            quick_launch::ql_open_file,
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
                
                manager.on_shortcut(shortcut, move |app_handle, _sc, event| {
                    if let tauri_plugin_global_shortcut::ShortcutState::Pressed = event.state {
                        if let Some(window) = app_handle.get_webview_window("main") {
                            if tool == "__palette__" || tool == "__quick_launch__" {
                                // 命令面板/快速启动：先唤起窗口到前台（最小化状态也能正确恢复）
                                #[cfg(target_os = "windows")]
                                if let Ok(hwnd) = window.hwnd() {
                                    use windows_sys::Win32::UI::WindowsAndMessaging::{ShowWindow, SetForegroundWindow, SW_RESTORE};
                                    unsafe {
                                        let _ = ShowWindow(hwnd.0, SW_RESTORE);
                                        let _ = SetForegroundWindow(hwnd.0);
                                    }
                                }
                                let _ = window.show();
                                let _ = window.set_focus();
                                if tool == "__palette__" {
                                    let _ = window.emit("command-palette-triggered", ());
                                } else {
                                    let _ = window.emit("global-shortcut-triggered", &tool);
                                }
                            } else {
                                let _ = window.emit("global-shortcut-triggered", &tool);
                            }
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
