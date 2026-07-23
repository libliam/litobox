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
mod disk_analyzer;
mod file_searcher;
mod icon_generator;
mod image_tools;
mod audio_tools;
mod video_tools;
mod pdf_tools;
mod media_info;
mod hotkey_probe;
mod hotkey_data;
mod hosts_manager;
mod network_connections;
mod scheduled_tasks;

// ponytail: debug 模式输出日志到 stderr，release 模式编译时移除（零开销）
macro_rules! debug_log {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            eprintln!($($arg)*)
        }
    };
}

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
            // 图标生成命令
            icon_generator::generate_icon,
            icon_generator::read_file_base64,
            icon_generator::copy_file,
            // 图片工具增强命令
            image_tools::get_file_info,
            image_tools::get_thumbnail,
            image_tools::image_compress,
            image_tools::image_merge,
            image_tools::image_template_merge,
            image_tools::image_watermark,
            image_tools::image_palette,
            audio_tools::check_ffmpeg,
            audio_tools::get_audio_info,
            video_tools::get_video_info,
            video_tools::extract_thumbnails,
            video_tools::video_crop,
            video_tools::video_transcode,
            video_tools::audio_extract,
            video_tools::video_compress,
            video_tools::video_merge,
            video_tools::video_extract_frame,
            video_tools::video_preview_frame,
            video_tools::calc_crop_preset,
            video_tools::video_crop_region,
            video_tools::video_speed_change,
            video_tools::video_rotate_flip,
            video_tools::video_volume,
            audio_tools::generate_waveform,
            audio_tools::audio_crop,
            audio_tools::get_audio_preview,
            audio_tools::audio_convert,
            audio_tools::audio_compress,
            audio_tools::audio_merge,
            audio_tools::audio_speed_change,
            audio_tools::list_tts_voices,
            audio_tools::tts_generate,
            audio_tools::get_downloads_dir,
            pdf_tools::detect_ghostscript,
            pdf_tools::get_pdf_page_count,
            pdf_tools::compress_pdf,
            pdf_tools::save_temp_file,
            media_info::get_media_info,
            media_info::extract_cover_art,
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
                            if tool == "__palette__" {
                                // 命令面板：先唤起窗口到前台（最小化状态也能正确恢复）
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
                                debug_log!("[command_palette] global hotkey triggered, window shown");
                                let _ = window.emit("command-palette-triggered", ());
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
