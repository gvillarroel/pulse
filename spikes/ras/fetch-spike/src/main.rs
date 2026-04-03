use std::path::PathBuf;

use clap::Parser;
use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
enum FetchBackend {
    GitCli,
    Gix,
}

#[derive(Debug, Parser)]
struct Args {
    #[arg(long)]
    repo: String,
    #[arg(long, default_value = ".pulse-state")]
    state_dir: PathBuf,
}

#[derive(Debug, Serialize)]
struct FetchPlan {
    repo: String,
    state_dir: PathBuf,
    mirror_dir: PathBuf,
    checkpoint_file: PathBuf,
    recommended_backend: FetchBackend,
    fallback_backend: FetchBackend,
    gix_linked: bool,
}

fn build_plan(repo: String, state_dir: PathBuf) -> FetchPlan {
    let repo_key = repo.replace(['/', ':'], "__");
    let mirror_dir = state_dir.join("repos").join(repo_key);
    let checkpoint_file = state_dir.join("checkpoints").join("fetch.json");

    FetchPlan {
        repo,
        state_dir,
        mirror_dir,
        checkpoint_file,
        recommended_backend: FetchBackend::GitCli,
        fallback_backend: FetchBackend::Gix,
        gix_linked: std::any::type_name::<gix::Repository>().starts_with("gix::"),
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let plan = build_plan(args.repo, args.state_dir);
    println!("{}", serde_json::to_string_pretty(&plan)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_stable_checkpoint_layout() {
        let plan = build_plan("owner/repo".to_owned(), PathBuf::from("state"));
        assert_eq!(plan.mirror_dir, PathBuf::from("state/repos/owner__repo"));
        assert_eq!(
            plan.checkpoint_file,
            PathBuf::from("state/checkpoints/fetch.json")
        );
        assert_eq!(plan.recommended_backend, FetchBackend::GitCli);
        assert_eq!(plan.fallback_backend, FetchBackend::Gix);
    }
}
