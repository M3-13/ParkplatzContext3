use parkplatz_lib::hotkey::shortcut_from_plugins;
use serde_json::json;
use tauri_plugin_global_shortcut::{Modifiers, Shortcut};

#[test]
fn defaults_to_ctrl_alt_p_without_config() {
    assert_eq!(shortcut_from_plugins(None), "ctrl+alt+p");
}

#[test]
fn reads_first_configured_shortcut() {
    let config = json!({ "shortcuts": ["ctrl+shift+k", "alt+x"] });
    assert_eq!(shortcut_from_plugins(Some(&config)), "ctrl+shift+k");
}

#[test]
fn defaults_when_shortcuts_key_is_missing() {
    let config = json!({ "something_else": true });
    assert_eq!(shortcut_from_plugins(Some(&config)), "ctrl+alt+p");
}

#[test]
fn defaults_when_shortcuts_array_is_empty() {
    let config = json!({ "shortcuts": [] });
    assert_eq!(shortcut_from_plugins(Some(&config)), "ctrl+alt+p");
}

#[test]
fn defaults_when_shortcut_is_blank() {
    let config = json!({ "shortcuts": ["   "] });
    assert_eq!(shortcut_from_plugins(Some(&config)), "ctrl+alt+p");
}

#[test]
fn configured_shortcut_is_trimmed() {
    let config = json!({ "shortcuts": ["  ctrl+alt+n  "] });
    assert_eq!(shortcut_from_plugins(Some(&config)), "ctrl+alt+n");
}

#[test]
fn default_hotkey_string_is_a_parseable_shortcut() {
    let shortcut: Shortcut = "ctrl+alt+p".parse().expect("default hotkey must parse");
    assert!(shortcut.mods.contains(Modifiers::CONTROL));
    assert!(shortcut.mods.contains(Modifiers::ALT));
}
