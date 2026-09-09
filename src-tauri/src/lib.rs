pub mod commands;
pub mod db;
pub mod git_context;
pub mod hotkey;
pub mod models;
pub mod notification;
pub mod tray;
pub mod watcher;

use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![
            commands::park_note,
            commands::list_notes,
            commands::search_notes,
            commands::set_note_done,
            commands::delete_note,
            commands::delete_all_notes,
            commands::export_notes_json,
            commands::get_current_context,
        ])
        .setup(|app| {
            let tray = tray::build_tray(app.handle())?;
            app.manage(tray);
            hotkey::register_hotkey(app.handle());
            watcher::start_watcher(app.handle().clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
