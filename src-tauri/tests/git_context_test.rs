use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use parkplatz_lib::git_context::capture_context;

static CWD_LOCK: Mutex<()> = Mutex::new(());

fn lock_cwd() -> std::sync::MutexGuard<'static, ()> {
    CWD_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

struct TempDir {
    dir: PathBuf,
    original_cwd: PathBuf,
}

impl TempDir {
    fn new(tag: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock before unix epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "parkplatz_git_test_{}_{}_{}",
            tag,
            std::process::id(),
            nonce
        ));
        fs::create_dir_all(&dir).expect("create temp dir");
        let original_cwd = std::env::current_dir().expect("read current dir");
        std::env::set_current_dir(&dir).expect("change into temp dir");
        TempDir { dir, original_cwd }
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.original_cwd);
        let _ = fs::remove_dir_all(&self.dir);
    }
}

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dir");
    }
    fs::write(path, contents).expect("write file");
}

fn commit_all(repo: &git2::Repository, message: &str) -> git2::Oid {
    let mut index = repo.index().expect("open index");
    index
        .add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)
        .expect("stage all files");
    index.write().expect("write index");
    let tree_id = index.write_tree().expect("write tree");
    let tree = repo.find_tree(tree_id).expect("find tree");

    let sig = git2::Signature::now("Test", "test@example.com").expect("signature");

    let parent = match repo.head() {
        Ok(head) => Some(repo.find_commit(head.peel_to_commit().unwrap().id()).unwrap()),
        Err(_) => None,
    };
    let parents: Vec<&git2::Commit> = parent.iter().collect();

    repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)
        .expect("commit")
}

fn init_repo(dir: &Path) -> git2::Repository {
    let repo = git2::Repository::init(dir).expect("init repo");
    write_file(&dir.join("hello.txt"), "hello\n");
    commit_all(&repo, "initial commit");
    repo
}

fn current_head_commit(repo: &git2::Repository) -> String {
    repo.head()
        .expect("head")
        .peel_to_commit()
        .expect("peel to commit")
        .id()
        .to_string()
}

fn current_branch(repo: &git2::Repository) -> String {
    repo.head()
        .expect("head")
        .shorthand()
        .expect("branch shorthand")
        .to_string()
}

#[test]
fn capture_context_outside_repo_is_empty() {
    let _guard = lock_cwd();
    let _tmp = TempDir::new("outside");

    let ctx = capture_context();
    assert_eq!(ctx.repo_path, "");
    assert_eq!(ctx.branch, "");
    assert_eq!(ctx.commit_hash, "");
    assert!(ctx.changed_files.is_empty());
}

#[test]
fn capture_context_reads_repo_path_branch_and_commit() {
    let _guard = lock_cwd();
    let tmp = TempDir::new("basic");
    let repo = init_repo(&tmp.dir);

    let ctx = capture_context();
    assert_eq!(ctx.branch, current_branch(&repo));
    assert_eq!(ctx.commit_hash, current_head_commit(&repo));
    assert!(ctx.changed_files.is_empty());

    let repo_path = PathBuf::from(&ctx.repo_path);
    assert_eq!(repo_path, tmp.dir);
}

#[test]
fn capture_context_reports_changed_files() {
    let _guard = lock_cwd();
    let tmp = TempDir::new("changed");
    init_repo(&tmp.dir);

    write_file(&tmp.dir.join("hello.txt"), "hello, changed\n");
    write_file(&tmp.dir.join("new.txt"), "brand new\n");

    let ctx = capture_context();
    assert!(ctx.changed_files.contains(&"hello.txt".to_string()));
    assert!(ctx.changed_files.contains(&"new.txt".to_string()));
}

#[test]
fn capture_context_follows_branch_switch() {
    let _guard = lock_cwd();
    let tmp = TempDir::new("branch");
    let repo = init_repo(&tmp.dir);

    let head_commit = repo.head().expect("head").peel_to_commit().expect("peel");
    repo.branch("feature/x", &head_commit, false).expect("branch");
    repo.set_head("refs/heads/feature/x").expect("set head");
    let mut checkout = git2::build::CheckoutBuilder::new();
    repo.checkout_head(Some(&mut checkout)).expect("checkout");

    let ctx = capture_context();
    assert_eq!(ctx.branch, "feature/x");
    assert_eq!(ctx.commit_hash, current_head_commit(&repo));
}

#[test]
fn capture_context_finds_repo_from_subdirectory() {
    let _guard = lock_cwd();
    let tmp = TempDir::new("subdir");
    let repo = init_repo(&tmp.dir);

    let sub = tmp.dir.join("src").join("nested");
    fs::create_dir_all(&sub).expect("create nested dir");
    std::env::set_current_dir(&sub).expect("change into subdir");

    let ctx = capture_context();
    assert_eq!(ctx.branch, current_branch(&repo));
    assert_eq!(ctx.commit_hash, current_head_commit(&repo));
    let repo_path = PathBuf::from(&ctx.repo_path);
    assert_eq!(repo_path, tmp.dir);
}
