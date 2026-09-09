use crate::models::Note;
use rusqlite::Connection;

pub fn open_db() -> Connection {
    Connection::open_in_memory().expect("failed to open in-memory database")
}

pub fn insert_note(
    _repo_path: &str,
    _branch: &str,
    _commit_hash: &str,
    _changed_files: &[String],
    _text: &str,
) -> i64 {
    0
}

pub fn list_notes() -> Vec<Note> {
    Vec::new()
}

pub fn search_notes(_q: &str) -> Vec<Note> {
    Vec::new()
}

pub fn set_note_done(_id: i64, _done: bool) {
    let _ = (_id, _done);
}

pub fn delete_note(_id: i64) {
    let _ = _id;
}

pub fn delete_all() {}

pub fn export_json() -> String {
    "[]".to_string()
}

pub fn repo_paths() -> Vec<String> {
    Vec::new()
}

pub fn latest_note_for_branch(_repo: &str, _branch: &str) -> Option<Note> {
    None
}
