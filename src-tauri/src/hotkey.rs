use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutEvent, ShortcutState};

const DEFAULT_HOTKEY: &str = "ctrl+alt+p";
const OVERLAY_WINDOW: &str = "input";
const OVERLAY_SHOWN_EVENT: &str = "overlay-shown";

pub fn register_hotkey(app: &AppHandle) {
    let shortcut = shortcut_from_plugins(app.config().plugins.0.get("global-shortcut"));

    if let Err(err) = app.global_shortcut().on_shortcut(shortcut.as_str(), on_shortcut) {
        eprintln!("failed to register global hotkey '{shortcut}': {err}");
        let _ = app.global_shortcut().on_shortcut(DEFAULT_HOTKEY, on_shortcut);
    }
}

fn on_shortcut(app: &AppHandle, _shortcut: &Shortcut, event: ShortcutEvent) {
    if event.state == ShortcutState::Pressed {
        show_overlay(app);
    }
}

fn show_overlay(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(OVERLAY_WINDOW) {
        let _ = window.show();
        let _ = window.set_focus();
        let _ = window.emit(OVERLAY_SHOWN_EVENT, ());
    }
}

pub fn shortcut_from_plugins(plugins: Option<&Value>) -> String {
    plugins
        .and_then(|v| v.get("shortcuts"))
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| DEFAULT_HOTKEY.to_string())
}
