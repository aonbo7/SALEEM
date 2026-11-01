pub mod commands;
pub mod ffi_loader;
pub mod state;

use state::AppState;
use std::path::PathBuf;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_local_data_dir()
                .unwrap_or_else(|_| PathBuf::from("./saleem_data"));
            let app_state = AppState::initialize(app_data_dir)
                .expect("Failed to initialize SALEEM SQLite database and application state");
            app.manage(app_state);
            commands::spawn_background_memory_daemon(app.handle().clone());
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if commands::should_keep_running_in_background(window.app_handle()) {
                    api.prevent_close();
                    let _ = window.minimize();
                    commands::trigger_windows_toast_notification(
                        "🛡️ SALEEM يعمل الآن في الخلفية",
                        "محرك الرامات (SALEEM.MemoryEngine.dll) مستمر بمراقبة وتفريغ الرام تلقائياً في الخلفية.",
                    );
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_system_foundation_status,
            commands::get_volumes_command,
            commands::get_quick_scan_presets_command,
            commands::start_disk_scan_command,
            commands::get_job_progress_command,
            commands::cancel_job_command,
            commands::get_disk_scan_result_command,
            commands::get_latest_disk_scan_command,
            commands::start_duplicate_scan_command,
            commands::get_duplicate_scan_result_command,
            commands::get_latest_duplicate_scan_command,
            commands::execute_duplicate_removal_command,
            commands::get_cleanup_rules_command,
            commands::start_cleanup_scan_command,
            commands::get_cleanup_plan_result_command,
            commands::get_latest_cleanup_plan_command,
            commands::execute_cleanup_plan_command,
            commands::start_inventory_scan_command,
            commands::get_inventory_scan_result_command,
            commands::get_latest_inventory_result_command,
            commands::execute_leftover_removal_command,
            commands::launch_app_uninstaller_command,
            commands::open_in_explorer_command,
            commands::get_audit_history_command,
            commands::update_ui_preferences_command,
            commands::get_cloud_update_config_command,
            commands::save_cloud_update_config_command,
            commands::check_for_cloud_updates_command,
            commands::download_and_install_update_command,
            commands::get_startup_and_booster_status_command,
            commands::remove_startup_item_command,
            commands::execute_system_boost_action_command,
            commands::get_memory_engine_status_command,
            commands::save_memory_engine_config_command,
            commands::execute_memory_clean_command,
            commands::trim_single_process_command,
            commands::open_external_url_command,
        ])
        .run(tauri::generate_context!())
        .expect("error while running SALEEM desktop application");
}
