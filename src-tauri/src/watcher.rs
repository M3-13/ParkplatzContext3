use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::mpsc::channel;
use std::thread;

use notify::{RecursiveMode, Watcher};
use tauri::AppHandle;

use crate::db;
use crate::notification;

pub fn start_watcher(app: AppHandle) {
    thread::spawn(move || {
        run_watcher(app);
    });
}

fn run_watcher(app: AppHandle) {
    let repos = db::repo_paths();

    let (tx, rx) = channel::<notify::Result<notify::Event>>();
    let mut watcher = match notify::recommended_watcher(tx) {
        Ok(w) => w,
        Err(_) => return,
    };

    for repo in &repos {
        let git_dir = Path::new(repo).join(".git");
        if git_dir.is_dir() {
            let _ = watcher.watch(&git_dir, RecursiveMode::NonRecursive);
        }
    }

    let mut last_branch: HashMap<String, Option<String>> = HashMap::new();

    while let Ok(res) = rx.recv() {
        let event = match res {
            Ok(event) => event,
            Err(_) => continue,
        };

        for path in &event.paths {
            if path.file_name().and_then(|n| n.to_str()) != Some("HEAD") {
                continue;
            }

            let repo = match path.parent().and_then(|p| p.parent()) {
                Some(parent) => parent.to_string_lossy().to_string(),
                None => continue,
            };

            let branch = read_branch(path);

            if last_branch.get(&repo) == Some(&branch) {
                continue;
            }
            last_branch.insert(repo.clone(), branch.clone());

            if let Some(branch) = branch {
                if let Some(note) = db::latest_note_for_branch(&repo, &branch) {
                    notification::show_note(&note, &app);
                }
            }
        }
    }
}

fn read_branch(head_path: &Path) -> Option<String> {
    let content = fs::read_to_string(head_path).ok()?;
    parse_branch_from_head(&content)
}

pub fn parse_branch_from_head(content: &str) -> Option<String> {
    let rest = content.trim().strip_prefix("ref: refs/heads/")?;
    let branch = rest.trim();
    if branch.is_empty() {
        None
    } else {
        Some(branch.to_string())
    }
}
