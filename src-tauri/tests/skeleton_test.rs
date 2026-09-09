use parkplatz_lib::commands;
use parkplatz_lib::models::{GitContext, Note};

#[test]
fn git_context_default_is_empty() {
    let ctx = GitContext::default();
    assert_eq!(ctx.repo_path, "");
    assert_eq!(ctx.branch, "");
    assert_eq!(ctx.commit_hash, "");
    assert!(ctx.changed_files.is_empty());
}

#[test]
fn note_roundtrips_through_json() {
    let note = Note {
        id: 42,
        repo_path: "/home/user/repo".to_string(),
        branch: "feature/x".to_string(),
        commit_hash: "abc123".to_string(),
        changed_files: vec!["src/main.rs".to_string(), "src/lib.rs".to_string()],
        text: "Fixe den Bug".to_string(),
        created_at: "2026-09-09T10:00:00+00:00".to_string(),
        done: false,
    };
    let json = serde_json::to_string(&note).unwrap();
    let decoded: Note = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.id, note.id);
    assert_eq!(decoded.repo_path, note.repo_path);
    assert_eq!(decoded.branch, note.branch);
    assert_eq!(decoded.commit_hash, note.commit_hash);
    assert_eq!(decoded.changed_files, note.changed_files);
    assert_eq!(decoded.text, note.text);
    assert_eq!(decoded.created_at, note.created_at);
    assert_eq!(decoded.done, note.done);
}

#[test]
fn git_context_roundtrips_through_json() {
    let ctx = GitContext {
        repo_path: "/repo".to_string(),
        branch: "main".to_string(),
        commit_hash: "deadbeef".to_string(),
        changed_files: vec!["a.txt".to_string()],
    };
    let json = serde_json::to_string(&ctx).unwrap();
    let decoded: GitContext = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.repo_path, "/repo");
    assert_eq!(decoded.branch, "main");
    assert_eq!(decoded.commit_hash, "deadbeef");
    assert_eq!(decoded.changed_files, vec!["a.txt".to_string()]);
}

#[test]
fn park_note_builds_a_note_with_text_and_timestamp() {
    let note = commands::park_note("Hallo Welt".to_string()).unwrap();
    assert_eq!(note.text, "Hallo Welt");
    assert_eq!(note.done, false);
    chrono::DateTime::parse_from_rfc3339(&note.created_at)
        .expect("created_at must be an RFC 3339 (ISO-8601) timestamp");
}

#[test]
fn read_commands_return_ok() {
    assert!(commands::list_notes().is_ok());
    assert!(commands::search_notes("query".to_string()).is_ok());
    assert!(commands::export_notes_json().is_ok());
    assert!(commands::get_current_context().is_ok());
}
