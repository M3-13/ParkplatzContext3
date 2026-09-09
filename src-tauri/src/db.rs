use std::path::{Path, PathBuf};

use crate::models::Note;
use rusqlite::{params, Connection};

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS notes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    repo_path TEXT NOT NULL,
    branch TEXT NOT NULL,
    commit_hash TEXT NOT NULL,
    changed_files TEXT NOT NULL,
    text TEXT NOT NULL,
    created_at TEXT NOT NULL,
    done INTEGER NOT NULL DEFAULT 0
);
";

/// Directory that holds per-user application data on the current platform.
///
/// Mirrors the well-known XDG / AppSupport / APPDATA conventions so the
/// database ends up in the user's data directory without pulling in the
/// `dirs` crate.
fn data_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
    }
    #[cfg(target_os = "macos")]
    {
        std::env::var_os("HOME")
            .map(|h| PathBuf::from(h).join("Library").join("Application Support"))
            .unwrap_or_else(std::env::temp_dir)
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local").join("share")))
            .unwrap_or_else(std::env::temp_dir)
    }
}

/// Path of the SQLite database file.
///
/// The `PARKPLATZ_DB_PATH` environment variable, when set, overrides the
/// default location. This is the seam the test suite uses to point the
/// database at a temporary file; the product itself never sets it.
pub fn db_path() -> PathBuf {
    if let Some(override_path) = std::env::var_os("PARKPLATZ_DB_PATH") {
        return PathBuf::from(override_path);
    }
    data_dir().join("parkplatz").join("notes.db")
}

/// Restrict the database file to the current user on Unix (0600).
/// On other platforms the file is created inside the user's own data
/// directory, which is already limited to that user by the OS.
#[cfg(unix)]
fn set_restrictive_permissions(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
}

#[cfg(not(unix))]
fn set_restrictive_permissions(_path: &Path) {}

/// Open (or create) the database in the user data directory, creating the
/// `notes` table if it does not exist yet, and tighten the file permissions
/// to 0600 on Unix.
pub fn open_db() -> Connection {
    let path = db_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("failed to create data directory");
    }
    let conn = Connection::open(&path).expect("failed to open database");
    set_restrictive_permissions(&path);
    conn.execute_batch(SCHEMA).expect("failed to create schema");
    conn
}

fn note_from_row(row: &rusqlite::Row) -> rusqlite::Result<Note> {
    let changed_files_json: String = row.get(4)?;
    let changed_files: Vec<String> =
        serde_json::from_str(&changed_files_json).unwrap_or_default();
    let done_raw: i64 = row.get(7)?;
    Ok(Note {
        id: row.get(0)?,
        repo_path: row.get(1)?,
        branch: row.get(2)?,
        commit_hash: row.get(3)?,
        changed_files,
        text: row.get(5)?,
        created_at: row.get(6)?,
        done: done_raw != 0,
    })
}

pub fn insert_note(
    repo_path: &str,
    branch: &str,
    commit_hash: &str,
    changed_files: &[String],
    text: &str,
) -> i64 {
    let conn = open_db();
    let changed_files_json =
        serde_json::to_string(changed_files).unwrap_or_else(|_| "[]".to_string());
    let created_at = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO notes (repo_path, branch, commit_hash, changed_files, text, created_at, done)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0)",
        params![
            repo_path,
            branch,
            commit_hash,
            changed_files_json,
            text,
            created_at
        ],
    )
    .expect("failed to insert note");
    conn.last_insert_rowid()
}

pub fn list_notes() -> Vec<Note> {
    let conn = open_db();
    let mut stmt = conn
        .prepare(
            "SELECT id, repo_path, branch, commit_hash, changed_files, text, created_at, done
             FROM notes
             ORDER BY created_at DESC, id DESC",
        )
        .expect("failed to prepare list query");
    let rows = stmt
        .query_map(params![], note_from_row)
        .expect("failed to query notes");
    rows.filter_map(Result::ok).collect()
}

pub fn search_notes(q: &str) -> Vec<Note> {
    let conn = open_db();
    let pattern = format!("%{}%", q);
    let mut stmt = conn
        .prepare(
            "SELECT id, repo_path, branch, commit_hash, changed_files, text, created_at, done
             FROM notes
             WHERE text LIKE ?1 OR branch LIKE ?1 OR repo_path LIKE ?1
             ORDER BY created_at DESC, id DESC",
        )
        .expect("failed to prepare search query");
    let rows = stmt
        .query_map(params![pattern], note_from_row)
        .expect("failed to search notes");
    rows.filter_map(Result::ok).collect()
}

pub fn set_note_done(id: i64, done: bool) {
    let conn = open_db();
    conn.execute(
        "UPDATE notes SET done = ?1 WHERE id = ?2",
        params![done as i64, id],
    )
    .expect("failed to update note");
}

pub fn delete_note(id: i64) {
    let conn = open_db();
    conn.execute("DELETE FROM notes WHERE id = ?1", params![id])
        .expect("failed to delete note");
}

pub fn delete_all() {
    let conn = open_db();
    conn.execute("DELETE FROM notes", params![])
        .expect("failed to delete all notes");
}

pub fn export_json() -> String {
    serde_json::to_string(&list_notes()).unwrap_or_else(|_| "[]".to_string())
}

pub fn repo_paths() -> Vec<String> {
    let conn = open_db();
    let mut stmt = conn
        .prepare("SELECT DISTINCT repo_path FROM notes ORDER BY repo_path ASC")
        .expect("failed to prepare repo_paths query");
    let rows = stmt
        .query_map(params![], |row| row.get::<_, String>(0))
        .expect("failed to query repo_paths");
    rows.filter_map(Result::ok).collect()
}

pub fn latest_note_for_branch(repo: &str, branch: &str) -> Option<Note> {
    let conn = open_db();
    let mut stmt = conn
        .prepare(
            "SELECT id, repo_path, branch, commit_hash, changed_files, text, created_at, done
             FROM notes
             WHERE repo_path = ?1 AND branch = ?2
             ORDER BY created_at DESC, id DESC
             LIMIT 1",
        )
        .expect("failed to prepare latest_note_for_branch query");
    let mut rows = stmt
        .query_map(params![repo, branch], note_from_row)
        .expect("failed to query latest_note_for_branch");
    rows.next().and_then(Result::ok)
}
