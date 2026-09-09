use crate::models::{GitContext, Note};
use crate::{db, git_context};

#[tauri::command]
pub fn park_note(text: String) -> Result<Note, String> {
    let ctx = git_context::capture_context();
    let id = db::insert_note(
        &ctx.repo_path,
        &ctx.branch,
        &ctx.commit_hash,
        &ctx.changed_files,
        &text,
    );
    let created_at = chrono::Utc::now().to_rfc3339();
    Ok(Note {
        id,
        repo_path: ctx.repo_path,
        branch: ctx.branch,
        commit_hash: ctx.commit_hash,
        changed_files: ctx.changed_files,
        text,
        created_at,
        done: false,
    })
}

#[tauri::command]
pub fn list_notes() -> Result<Vec<Note>, String> {
    Ok(db::list_notes())
}

#[tauri::command]
pub fn search_notes(query: String) -> Result<Vec<Note>, String> {
    Ok(db::search_notes(&query))
}

#[tauri::command]
pub fn set_note_done(id: i64, done: bool) -> Result<(), String> {
    db::set_note_done(id, done);
    Ok(())
}

#[tauri::command]
pub fn delete_note(id: i64) -> Result<(), String> {
    db::delete_note(id);
    Ok(())
}

#[tauri::command]
pub fn delete_all_notes() -> Result<(), String> {
    db::delete_all();
    Ok(())
}

#[tauri::command]
pub fn export_notes_json() -> Result<String, String> {
    Ok(db::export_json())
}

#[tauri::command]
pub fn get_current_context() -> Result<GitContext, String> {
    Ok(git_context::capture_context())
}
