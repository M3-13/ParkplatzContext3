use std::path::{Path, PathBuf};
use std::sync::Mutex;

use parkplatz_lib::db;

static DB_LOCK: Mutex<()> = Mutex::new(());

fn temp_db_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "parkplatz_db_test_{}_{}",
        std::process::id(),
        name
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

fn use_db(dir: &Path) {
    let db_file = dir.join("notes.db");
    std::env::set_var("PARKPLATZ_DB_PATH", &db_file);
}

#[test]
fn insert_and_list_notes_newest_first() {
    let _guard = DB_LOCK.lock().unwrap();
    let dir = temp_db_dir("insert_list");
    use_db(&dir);

    let id1 = db::insert_note("/repo/a", "main", "abc", &["a.txt".to_string()], "erste");
    let id2 = db::insert_note("/repo/b", "feat", "def", &[], "zweite");

    let notes = db::list_notes();
    assert_eq!(notes.len(), 2);
    assert_eq!(notes[0].id, id2, "newest note must come first");
    assert_eq!(notes[1].id, id1);
    assert_eq!(notes[0].text, "zweite");
    assert_eq!(notes[1].text, "erste");
    assert_eq!(notes[1].changed_files, vec!["a.txt".to_string()]);
    assert_eq!(notes[1].repo_path, "/repo/a");
    assert_eq!(notes[1].branch, "main");
    assert_eq!(notes[1].commit_hash, "abc");
    assert!(!notes[0].done);
    assert!(!notes[1].done);

    chrono::DateTime::parse_from_rfc3339(&notes[0].created_at)
        .expect("created_at must be an ISO-8601 timestamp");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn search_finds_text_branch_and_repo() {
    let _guard = DB_LOCK.lock().unwrap();
    let dir = temp_db_dir("search");
    use_db(&dir);

    db::insert_note("/repo/alpha", "main", "aaa", &[], "Kaffee holen");
    db::insert_note("/repo/beta", "feature/login", "bbb", &[], "Refactor");
    db::insert_note("/repo/gamma", "main", "ccc", &[], "Tickets sichten");

    let by_text = db::search_notes("kaffee");
    assert_eq!(by_text.len(), 1);
    assert_eq!(by_text[0].repo_path, "/repo/alpha");

    let by_branch = db::search_notes("login");
    assert_eq!(by_branch.len(), 1);
    assert_eq!(by_branch[0].repo_path, "/repo/beta");

    let by_repo = db::search_notes("gamma");
    assert_eq!(by_repo.len(), 1);
    assert_eq!(by_repo[0].text, "Tickets sichten");

    assert!(db::search_notes("nichtda").is_empty());

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn set_note_done_updates_row() {
    let _guard = DB_LOCK.lock().unwrap();
    let dir = temp_db_dir("done");
    use_db(&dir);

    let id = db::insert_note("/repo", "main", "abc", &[], "to do");
    assert!(!db::list_notes()[0].done);

    db::set_note_done(id, true);
    assert!(db::list_notes()[0].done);

    db::set_note_done(id, false);
    assert!(!db::list_notes()[0].done);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn delete_note_removes_only_that_row() {
    let _guard = DB_LOCK.lock().unwrap();
    let dir = temp_db_dir("delete_note");
    use_db(&dir);

    let id1 = db::insert_note("/repo", "main", "a", &[], "eins");
    let id2 = db::insert_note("/repo", "main", "b", &[], "zwei");

    db::delete_note(id1);
    let notes = db::list_notes();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].id, id2);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn delete_all_removes_everything() {
    let _guard = DB_LOCK.lock().unwrap();
    let dir = temp_db_dir("delete_all");
    use_db(&dir);

    db::insert_note("/repo/a", "main", "a", &[], "eins");
    db::insert_note("/repo/b", "main", "b", &[], "zwei");

    db::delete_all();
    assert!(db::list_notes().is_empty());
    assert_eq!(db::export_json(), "[]");
    assert!(db::repo_paths().is_empty());

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn export_json_contains_all_notes() {
    let _guard = DB_LOCK.lock().unwrap();
    let dir = temp_db_dir("export");
    use_db(&dir);

    db::insert_note(
        "/repo/a",
        "main",
        "abc123",
        &["x.rs".to_string(), "y.rs".to_string()],
        "Notiz mit \"Anführungszeichen\" und <b>HTML</b>",
    );

    let json = db::export_json();
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    let arr = parsed.as_array().expect("must be an array");
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["repo_path"], "/repo/a");
    assert_eq!(arr[0]["branch"], "main");
    assert_eq!(arr[0]["commit_hash"], "abc123");
    assert_eq!(arr[0]["changed_files"], serde_json::json!(["x.rs", "y.rs"]));
    assert_eq!(arr[0]["text"], "Notiz mit \"Anführungszeichen\" und <b>HTML</b>");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn sql_values_are_bound_not_interpolated() {
    let _guard = DB_LOCK.lock().unwrap();
    let dir = temp_db_dir("injection");
    use_db(&dir);

    let malicious = "Robert'); DROP TABLE notes; --";
    db::insert_note(
        "/repo'); DROP TABLE notes; --",
        "main'; --",
        "x' OR '1'='1",
        &[],
        malicious,
    );

    let notes = db::list_notes();
    assert_eq!(notes.len(), 1, "injected DROP must not affect the table");
    assert_eq!(notes[0].text, malicious);
    assert_eq!(notes[0].repo_path, "/repo'); DROP TABLE notes; --");
    assert_eq!(notes[0].branch, "main'; --");
    assert_eq!(notes[0].commit_hash, "x' OR '1'='1");

    let found = db::search_notes("DROP TABLE");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].text, malicious);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn latest_note_for_branch_returns_most_recent() {
    let _guard = DB_LOCK.lock().unwrap();
    let dir = temp_db_dir("latest");
    use_db(&dir);

    db::insert_note("/repo", "main", "a", &[], "erste");
    db::insert_note("/repo", "main", "b", &[], "zweite");
    db::insert_note("/repo", "other", "c", &[], "anderer branch");

    let latest = db::latest_note_for_branch("/repo", "main").expect("note exists");
    assert_eq!(latest.text, "zweite");

    let other = db::latest_note_for_branch("/repo", "other").expect("note exists");
    assert_eq!(other.text, "anderer branch");

    assert!(db::latest_note_for_branch("/repo", "fehlt").is_none());

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn repo_paths_returns_unique_sorted_paths() {
    let _guard = DB_LOCK.lock().unwrap();
    let dir = temp_db_dir("repo_paths");
    use_db(&dir);

    db::insert_note("/repo/b", "main", "b", &[], "b1");
    db::insert_note("/repo/a", "main", "a", &[], "a1");
    db::insert_note("/repo/b", "main", "b2", &[], "b2");

    let paths = db::repo_paths();
    assert_eq!(paths, vec!["/repo/a".to_string(), "/repo/b".to_string()]);

    let _ = std::fs::remove_dir_all(&dir);
}

#[cfg(unix)]
#[test]
fn database_file_is_created_with_0600_permissions() {
    use std::os::unix::fs::PermissionsExt;
    let _guard = DB_LOCK.lock().unwrap();
    let dir = temp_db_dir("permissions");
    use_db(&dir);

    let _conn = db::open_db();

    let db_file = dir.join("notes.db");
    assert!(db_file.exists(), "database file should have been created");

    let metadata = std::fs::metadata(&db_file).expect("metadata");
    let mode = metadata.permissions().mode() & 0o777;
    assert_eq!(mode, 0o600, "database file must be readable/writable only by the owner");

    let _ = std::fs::remove_dir_all(&dir);
}
