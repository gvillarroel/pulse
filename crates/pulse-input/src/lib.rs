use anyhow::{Context, bail};
use pulse_config::{AppConfig, RepositoryItem};
use pulse_core::{RepoTarget, Result};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct CsvRepoRecord {
    repo: String,
    provider: Option<String>,
    owner: Option<String>,
    owner_color: Option<String>,
    team: Option<String>,
    team_color: Option<String>,
    name: Option<String>,
    url: Option<String>,
    default_branch: Option<String>,
    tags: Option<String>,
    active: Option<bool>,
}

pub fn resolve_targets(
    config: Option<&AppConfig>,
    input_csv: Option<&Path>,
) -> Result<Vec<RepoTarget>> {
    let mut targets = Vec::new();

    if let Some(config) = config {
        for item in &config.repositories.items {
            targets.push(normalize_item(item)?);
        }
        if let Some(csv_path) = config.repositories.csv.as_deref() {
            targets.extend(load_csv(csv_path)?);
        }
    }

    if let Some(csv) = input_csv {
        targets.extend(load_csv(csv)?);
    }

    dedupe_targets(targets)
}

pub fn load_csv(path: &Path) -> Result<Vec<RepoTarget>> {
    let mut reader = csv::Reader::from_path(path)
        .with_context(|| format!("failed to open CSV {}", path.display()))?;
    let mut repos = Vec::new();
    for row in reader.deserialize() {
        let record: CsvRepoRecord =
            row.with_context(|| format!("invalid CSV row in {}", path.display()))?;
        repos.push(normalize_csv_record(record)?);
    }
    Ok(repos)
}

pub fn normalize_repo_spec(spec: &str) -> Result<(String, String, String, String)> {
    let trimmed = spec.trim().trim_end_matches(".git");
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        let url = trimmed.to_string();
        let no_scheme = url
            .split("://")
            .nth(1)
            .ok_or_else(|| anyhow::anyhow!("missing scheme"))?;
        let mut parts = no_scheme.split('/');
        let host = parts.next().unwrap_or_default();
        let owner = parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("missing owner in URL"))?;
        let name = parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("missing repository name in URL"))?;
        let provider = if host.contains("github") {
            "github".to_string()
        } else {
            host.to_string()
        };
        return Ok((
            provider,
            owner.to_string(),
            name.to_string(),
            format!("https://{host}/{owner}/{name}.git"),
        ));
    }

    let candidate_path = PathBuf::from(spec.trim());
    if candidate_path.is_absolute()
        || spec.contains('\\')
        || (spec.contains('/') && candidate_path.exists())
        || spec.starts_with("file://")
    {
        let raw = spec.trim().trim_start_matches("file://");
        let path = PathBuf::from(raw);
        let name = path
            .file_stem()
            .or_else(|| path.file_name())
            .ok_or_else(|| anyhow::anyhow!("could not derive repository name from local path"))?
            .to_string_lossy()
            .trim_end_matches(".git")
            .to_string();
        return Ok((
            "local".to_string(),
            "local".to_string(),
            name.clone(),
            raw.to_string(),
        ));
    }

    let mut parts = trimmed.split('/');
    let owner = parts.next().unwrap_or_default();
    let name = parts.next().unwrap_or_default();
    if owner.is_empty() || name.is_empty() || parts.next().is_some() {
        bail!("repository identifier must be owner/name or a clone URL");
    }

    Ok((
        "github".to_string(),
        owner.to_string(),
        name.to_string(),
        format!("https://github.com/{owner}/{name}.git"),
    ))
}

fn normalize_item(item: &RepositoryItem) -> Result<RepoTarget> {
    let (provider, owner, name, inferred_url) = normalize_repo_spec(&item.repo)?;
    Ok(RepoTarget {
        repo: format!("{owner}/{name}"),
        provider: item.provider.clone().unwrap_or(provider),
        owner: item.owner.clone().unwrap_or(owner),
        owner_color: item.owner_color.clone(),
        team: item.team.clone(),
        team_color: item.team_color.clone(),
        name: item.name.clone().unwrap_or(name),
        url: item.url.clone().unwrap_or(inferred_url),
        default_branch: item.default_branch.clone(),
        tags: item.tags.clone(),
        active: item.active,
    })
}

fn normalize_csv_record(record: CsvRepoRecord) -> Result<RepoTarget> {
    let (provider, owner, name, inferred_url) = normalize_repo_spec(&record.repo)?;
    Ok(RepoTarget {
        repo: format!("{owner}/{name}"),
        provider: record.provider.unwrap_or(provider),
        owner: record.owner.unwrap_or(owner),
        owner_color: record.owner_color,
        team: record.team,
        team_color: record.team_color,
        name: record.name.unwrap_or(name),
        url: record.url.unwrap_or(inferred_url),
        default_branch: record.default_branch,
        tags: record
            .tags
            .map(|value| {
                value
                    .split(',')
                    .map(str::trim)
                    .filter(|part| !part.is_empty())
                    .map(ToOwned::to_owned)
                    .collect()
            })
            .unwrap_or_default(),
        active: record.active.unwrap_or(true),
    })
}

fn dedupe_targets(targets: Vec<RepoTarget>) -> Result<Vec<RepoTarget>> {
    let mut deduped = BTreeMap::new();
    for target in targets {
        deduped.entry(target.key()).or_insert(target);
    }
    Ok(deduped.into_values().collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn normalizes_owner_name() {
        let (provider, owner, name, url) =
            normalize_repo_spec("openai/openai-cookbook").expect("repo");
        assert_eq!(provider, "github");
        assert_eq!(owner, "openai");
        assert_eq!(name, "openai-cookbook");
        assert!(url.ends_with("/openai/openai-cookbook.git"));
    }

    #[test]
    fn rejects_bad_repo_specs() {
        assert!(normalize_repo_spec("bad").is_err());
    }

    #[test]
    fn accepts_local_paths() {
        let repo = if cfg!(windows) {
            r"C:\tmp\origin.git"
        } else {
            "/tmp/origin.git"
        };
        let (provider, owner, name, url) = normalize_repo_spec(repo).expect("local repo");
        assert_eq!(provider, "local");
        assert_eq!(owner, "local");
        assert!(name.contains("origin"));
        assert_eq!(url, repo);
    }

    #[test]
    fn loads_team_columns_from_csv() {
        let dir = tempdir().expect("tempdir");
        let csv = dir.path().join("repos.csv");
        fs::write(
            &csv,
            "repo,team,team_color\nopenai/openai-cookbook,team-01,#123456\n",
        )
        .expect("write csv");
        let repos = load_csv(&csv).expect("load csv");
        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0].team.as_deref(), Some("team-01"));
        assert_eq!(repos[0].team_color.as_deref(), Some("#123456"));
    }
}
