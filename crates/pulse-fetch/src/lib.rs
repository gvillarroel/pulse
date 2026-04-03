use anyhow::{Context, Result, bail};
use chrono::Utc;
use pulse_core::{FailureClass, FetchOutcome, RepoTarget, repo_cache_path};
use pulse_git::repo_exists;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchError {
    pub class: FailureClass,
    pub message: String,
}

impl std::fmt::Display for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.class, self.message)
    }
}

impl std::error::Error for FetchError {}

pub fn fetch_repo(root: &Path, repo: &RepoTarget) -> Result<FetchOutcome> {
    let git_dir = repo_cache_path(root, repo);
    if let Some(parent) = git_dir.parent() {
        std::fs::create_dir_all(parent)?;
    }

    if !repo_exists(&git_dir) {
        clone_bare(&git_dir, repo)?;
    } else {
        run_git(
            git_dir.as_path(),
            ["fetch", "--prune", "--tags", "origin"],
            None,
        )?;
    }

    let revision = current_head(&git_dir)?;
    Ok(FetchOutcome {
        repo_key: repo.key(),
        remote_url: repo.url.clone(),
        git_dir,
        fetched_revision: revision,
        fetched_at: Utc::now(),
        backend: "git-cli".to_string(),
    })
}

fn clone_bare(target: &Path, repo: &RepoTarget) -> Result<()> {
    if target.exists() {
        std::fs::remove_dir_all(target)?;
    }
    let target_string = target.to_string_lossy().to_string();
    run_git(
        Path::new("."),
        ["clone", "--bare", repo.url.as_str(), target_string.as_str()],
        None,
    )
}

fn current_head(git_dir: &Path) -> Result<String> {
    let output = run_git_capture(git_dir, ["rev-parse", "HEAD"])?;
    Ok(output.trim().to_string())
}

fn run_git<'a>(
    git_dir: &Path,
    args: impl IntoIterator<Item = &'a str>,
    worktree: Option<&Path>,
) -> Result<()> {
    run_git_capture_internal(git_dir, args, worktree).map(|_| ())
}

fn run_git_capture(git_dir: &Path, args: impl IntoIterator<Item = &'static str>) -> Result<String> {
    run_git_capture_internal(git_dir, args, None)
}

fn run_git_capture_internal<'a>(
    git_dir: &Path,
    args: impl IntoIterator<Item = &'a str>,
    worktree: Option<&Path>,
) -> Result<String> {
    let mut command = Command::new("git");
    if repo_exists(git_dir) {
        command.arg(format!("--git-dir={}", git_dir.display()));
    }
    if let Some(worktree) = worktree {
        command.arg(format!("--work-tree={}", worktree.display()));
    }
    command.args(args);
    let output = command.output().context("failed to launch git")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(FetchError {
            class: classify_git_error(&stderr),
            message: stderr.trim().to_string(),
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn classify_git_error(stderr: &str) -> FailureClass {
    let lower = stderr.to_ascii_lowercase();
    if lower.contains("repository not found") || lower.contains("invalid") {
        FailureClass::InvalidInput
    } else if lower.contains("authentication") || lower.contains("permission denied") {
        FailureClass::Permanent
    } else if lower.contains("could not resolve host")
        || lower.contains("timed out")
        || lower.contains("connection reset")
    {
        FailureClass::Transient
    } else {
        FailureClass::Permanent
    }
}

pub fn file_content(git_dir: &Path, revision: &str, path: &str) -> Result<Vec<u8>> {
    let spec = format!("{revision}:{path}");
    let mut command = Command::new("git");
    command
        .arg(format!("--git-dir={}", git_dir.display()))
        .args(["show", spec.as_str()]);
    let output = command.output().context("failed to launch git show")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git show failed for {path}: {}", stderr.trim());
    }
    Ok(output.stdout)
}

pub fn list_tree(git_dir: &Path, revision: &str) -> Result<Vec<TreeEntry>> {
    let output = run_git_capture_internal(git_dir, ["ls-tree", "-r", "-l", revision], None)?;
    let mut entries = Vec::new();
    for line in output.lines() {
        if let Some((meta, path)) = line.split_once('\t') {
            let parts: Vec<_> = meta.split_whitespace().collect();
            if parts.len() < 4 {
                continue;
            }
            let size = parts[3].parse::<u64>().unwrap_or(0);
            entries.push(TreeEntry {
                path: path.to_string(),
                size_bytes: size,
            });
        }
    }
    Ok(entries)
}

#[derive(Debug, Clone)]
pub struct TreeEntry {
    pub path: String,
    pub size_bytes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use tempfile::tempdir;

    #[test]
    fn classifies_errors() {
        assert_eq!(
            classify_git_error("repository not found"),
            FailureClass::InvalidInput
        );
        assert_eq!(
            classify_git_error("could not resolve host"),
            FailureClass::Transient
        );
    }

    #[test]
    fn fetches_local_bare_repo() {
        let tmp = tempdir().expect("temp");
        let origin = tmp.path().join("origin");
        let work = tmp.path().join("work");
        let state = tmp.path().join("state");
        std::fs::create_dir_all(&work).expect("work dir");

        Command::new("git")
            .args(["init", "--bare", origin.to_str().expect("origin path")])
            .status()
            .expect("init bare");
        Command::new("git")
            .current_dir(tmp.path())
            .args([
                "clone",
                origin.to_str().expect("origin"),
                work.to_str().expect("work"),
            ])
            .status()
            .expect("clone work");
        std::fs::write(work.join("README.md"), "hello\n").expect("write file");
        Command::new("git")
            .current_dir(&work)
            .args(["config", "user.email", "pulse@example.com"])
            .status()
            .expect("email");
        Command::new("git")
            .current_dir(&work)
            .args(["config", "user.name", "Pulse"])
            .status()
            .expect("name");
        Command::new("git")
            .current_dir(&work)
            .args(["add", "."])
            .status()
            .expect("add");
        Command::new("git")
            .current_dir(&work)
            .args(["commit", "-m", "init"])
            .status()
            .expect("commit");
        Command::new("git")
            .current_dir(&work)
            .args(["push", "origin", "HEAD"])
            .status()
            .expect("push");

        let repo = RepoTarget {
            repo: "local/sample".into(),
            provider: "local".into(),
            owner: "sample".into(),
            owner_color: Some("#007298".into()),
            name: "repo".into(),
            url: origin.to_string_lossy().to_string(),
            default_branch: None,
            tags: Vec::new(),
            active: true,
        };
        let outcome = fetch_repo(&state, &repo).expect("fetch local repo");
        assert!(outcome.git_dir.exists());
        assert!(!outcome.fetched_revision.is_empty());
    }
}
