use std::path::{Path, PathBuf};
use std::{error::Error, fmt};

#[derive(Debug, Clone)]
pub struct RepoSpec {
    pub provider: Option<String>,
    pub owner: String,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct FetchOutcome {
    pub backend: FetchBackendKind,
    pub repo_path: PathBuf,
    pub fetched_revision: Option<String>,
    pub fetched_at_utc: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FetchBackendKind {
    GitCli,
    Gix,
    Git2,
}

#[derive(Debug, Clone)]
pub enum FetchError {
    InvalidInput(String),
    Transient(String),
    Permanent(String),
    Unsupported(String),
}

impl fmt::Display for FetchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FetchError::InvalidInput(msg) => write!(f, "invalid input: {msg}"),
            FetchError::Transient(msg) => write!(f, "transient failure: {msg}"),
            FetchError::Permanent(msg) => write!(f, "permanent failure: {msg}"),
            FetchError::Unsupported(msg) => write!(f, "unsupported: {msg}"),
        }
    }
}

impl Error for FetchError {}

pub trait FetchBackend {
    fn kind(&self) -> FetchBackendKind;

    fn fetch_or_update(&self, repo: &RepoSpec, state_dir: &Path) -> Result<FetchOutcome, FetchError>;
}

pub struct GitCliBackend;
pub struct GixBackend;
pub struct Git2Backend;

impl FetchBackend for GitCliBackend {
    fn kind(&self) -> FetchBackendKind {
        FetchBackendKind::GitCli
    }

    fn fetch_or_update(&self, repo: &RepoSpec, state_dir: &Path) -> Result<FetchOutcome, FetchError> {
        let _ = (repo, state_dir);
        Err(FetchError::Unsupported(
            "prototype only; wire to `git clone --mirror` and `git fetch`".into(),
        ))
    }
}

impl FetchBackend for GixBackend {
    fn kind(&self) -> FetchBackendKind {
        FetchBackendKind::Gix
    }

    fn fetch_or_update(&self, repo: &RepoSpec, state_dir: &Path) -> Result<FetchOutcome, FetchError> {
        let _ = (repo, state_dir);
        Err(FetchError::Unsupported(
            "prototype only; wire to `gix::clone::PrepareFetch`".into(),
        ))
    }
}

impl FetchBackend for Git2Backend {
    fn kind(&self) -> FetchBackendKind {
        FetchBackendKind::Git2
    }

    fn fetch_or_update(&self, repo: &RepoSpec, state_dir: &Path) -> Result<FetchOutcome, FetchError> {
        let _ = (repo, state_dir);
        Err(FetchError::Unsupported(
            "prototype only; wire to libgit2 repository clone/fetch APIs".into(),
        ))
    }
}
