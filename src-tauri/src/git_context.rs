use crate::models::GitContext;
use git2::Repository;

pub fn capture_context() -> GitContext {
    let cwd = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return GitContext::default(),
    };

    let repo = match Repository::discover(&cwd) {
        Ok(repo) => repo,
        Err(_) => return GitContext::default(),
    };

    let repo_path = match repo.workdir() {
        Some(workdir) => workdir.to_string_lossy().into_owned(),
        None => repo.path().to_string_lossy().into_owned(),
    };

    let head = match repo.head() {
        Ok(head) => head,
        Err(_) => return GitContext::default(),
    };

    let branch = head.shorthand().unwrap_or("").to_string();

    let commit_hash = match head.peel_to_commit() {
        Ok(commit) => commit.id().to_string(),
        Err(_) => String::new(),
    };

    let mut changed_files = Vec::new();
    if let Ok(statuses) = repo.statuses(None) {
        for entry in statuses.iter() {
            if let Some(path) = entry.path() {
                changed_files.push(path.to_string());
            }
        }
    }
    changed_files.sort();

    GitContext {
        repo_path,
        branch,
        commit_hash,
        changed_files,
    }
}
