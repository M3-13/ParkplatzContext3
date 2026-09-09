use crate::models::Note;
use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

pub fn show_note(note: &Note, app: &AppHandle) {
    let _ = app
        .notification()
        .builder()
        .title("Parkplatz")
        .body(note.text.as_str())
        .show();
}
