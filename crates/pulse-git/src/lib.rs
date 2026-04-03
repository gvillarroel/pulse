use anyhow::Result;
use std::path::Path;

pub fn repo_exists(path: &Path) -> bool {
    path.join("HEAD").exists()
}

pub fn open_repo(path: &Path) -> Result<gix::Repository> {
    Ok(gix::open(path)?)
}
