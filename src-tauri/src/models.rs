use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: i64,
    pub repo_path: String,
    pub branch: String,
    pub commit_hash: String,
    pub changed_files: Vec<String>,
    pub text: String,
    pub created_at: String,
    pub done: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitContext {
    pub repo_path: String,
    pub branch: String,
    pub commit_hash: String,
    pub changed_files: Vec<String>,
}

impl Default for GitContext {
    fn default() -> Self {
        GitContext {
            repo_path: String::new(),
            branch: String::new(),
            commit_hash: String::new(),
            changed_files: Vec::new(),
        }
    }
}
