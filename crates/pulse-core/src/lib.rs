use chrono::{DateTime, Utc};
use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;
use std::path::{Path, PathBuf};

pub type Result<T> = anyhow::Result<T>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepoTarget {
    pub repo: String,
    pub provider: String,
    pub owner: String,
    pub owner_color: Option<String>,
    #[serde(default)]
    pub owner_levels: Vec<OwnerLevel>,
    pub team: Option<String>,
    pub team_color: Option<String>,
    pub name: String,
    pub url: String,
    pub default_branch: Option<String>,
    pub tags: Vec<String>,
    pub active: bool,
}

impl RepoTarget {
    pub fn key(&self) -> String {
        format!("{}/{}/{}", self.provider, self.owner, self.name)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OwnerLevel {
    pub level: usize,
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct FocusConfig {
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
}

impl FocusConfig {
    pub fn compile(&self) -> Result<CompiledFocus> {
        Ok(CompiledFocus {
            include: compile_globs(&self.include)?,
            exclude: compile_globs(&self.exclude)?,
            has_include_rules: !self.include.is_empty(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct CompiledFocus {
    include: GlobSet,
    exclude: GlobSet,
    has_include_rules: bool,
}

impl CompiledFocus {
    pub fn matches(&self, path: &str) -> bool {
        if self.exclude.is_match(path) {
            return false;
        }
        if !self.has_include_rules {
            return false;
        }
        self.include.is_match(path)
    }

    pub fn classify(&self, path: &str) -> AnalysisDepth {
        if self.exclude.is_match(path) {
            return AnalysisDepth::Baseline;
        }
        if !self.has_include_rules {
            return AnalysisDepth::Baseline;
        }
        if self.include.is_match(path) {
            AnalysisDepth::Focused
        } else {
            AnalysisDepth::Baseline
        }
    }
}

fn compile_globs(patterns: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(Glob::new(pattern)?);
    }
    Ok(builder.build()?)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AnalysisDepth {
    Baseline,
    Focused,
}

impl fmt::Display for AnalysisDepth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Baseline => write!(f, "baseline"),
            Self::Focused => write!(f, "focused"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StageStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Stale,
}

impl fmt::Display for StageStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Running => write!(f, "running"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
            Self::Stale => write!(f, "stale"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FailureClass {
    Transient,
    Permanent,
    InvalidInput,
    Unsupported,
}

impl fmt::Display for FailureClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transient => write!(f, "transient"),
            Self::Permanent => write!(f, "permanent"),
            Self::InvalidInput => write!(f, "invalid_input"),
            Self::Unsupported => write!(f, "unsupported"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchOutcome {
    pub repo_key: String,
    pub remote_url: String,
    pub git_dir: PathBuf,
    pub fetched_revision: String,
    pub fetched_at: DateTime<Utc>,
    pub backend: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSnapshot {
    pub repo_key: String,
    pub revision: String,
    pub path: String,
    pub language: Option<String>,
    pub extension: Option<String>,
    pub size_bytes: u64,
    pub line_count: u64,
    pub is_binary: bool,
    pub depth: AnalysisDepth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoSnapshot {
    pub repo_key: String,
    pub revision: String,
    pub total_files: u64,
    pub total_bytes: u64,
    pub total_lines: u64,
    pub generated_at: DateTime<Utc>,
    pub config_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeeklyEvolution {
    pub repo_key: String,
    pub week_start: String,
    pub commit_count: u64,
    pub active_contributors: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunSummary {
    pub run_id: i64,
    pub processed: usize,
    pub failed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSummary {
    pub repositories: u64,
    pub fetched: u64,
    pub analyzed: u64,
    pub failed: u64,
    pub total_files: u64,
    pub total_bytes: u64,
    pub total_lines: u64,
    pub weekly_points: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageBreakdown {
    pub language: String,
    pub files: u64,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionBreakdown {
    pub extension: String,
    pub files: u64,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoOverview {
    pub repo_key: String,
    pub owner: String,
    pub owner_color: Option<String>,
    pub owner_levels: Vec<OwnerLevel>,
    pub team: Option<String>,
    pub team_color: Option<String>,
    pub name: String,
    pub total_files: u64,
    pub total_bytes: u64,
    pub total_lines: u64,
    pub dominant_language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeeklyOverview {
    pub week_start: String,
    pub commits: u64,
    pub active_repositories: u64,
    pub contributor_instances: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnerWeeklyOverview {
    pub repo_key: String,
    pub owner: String,
    pub owner_levels: Vec<OwnerLevel>,
    pub team: Option<String>,
    pub week_start: String,
    pub commits: u64,
    pub active_repositories: u64,
    pub contributor_instances: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureRecord {
    pub repo_key: String,
    pub stage: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageStatusCount {
    pub stage: String,
    pub status: String,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiDocSummary {
    pub doc_name: String,
    pub category: String,
    pub repositories: u64,
    pub files: u64,
    pub adoption_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiDocOccurrence {
    pub repo_key: String,
    pub doc_name: String,
    pub category: String,
    pub path: String,
    pub first_seen_week_start: Option<String>,
    pub linked_docs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiDocLinkSummary {
    pub source_doc: String,
    pub linked_doc: String,
    pub repositories: u64,
    pub adoption_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiDocTimelinePoint {
    pub week_start: String,
    pub doc_name: String,
    pub path: String,
    pub cumulative_repositories: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiDocOwnerWeekly {
    pub repo_key: String,
    pub owner: String,
    pub owner_levels: Vec<OwnerLevel>,
    pub team: Option<String>,
    pub week_start: String,
    pub commits: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ReportOwnerLevelsConfig {
    pub default_level: Option<usize>,
    #[serde(default)]
    pub labels: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ReportRenderOptions {
    #[serde(default)]
    pub owner_levels: ReportOwnerLevelsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportDataset {
    pub summary: ReportSummary,
    pub languages: Vec<LanguageBreakdown>,
    pub extensions: Vec<ExtensionBreakdown>,
    pub repositories: Vec<RepoOverview>,
    pub weekly_overview: Vec<WeeklyOverview>,
    pub owner_weekly_overview: Vec<OwnerWeeklyOverview>,
    pub failures: Vec<FailureRecord>,
    pub stage_statuses: Vec<StageStatusCount>,
    pub ai_doc_summaries: Vec<AiDocSummary>,
    pub ai_doc_occurrences: Vec<AiDocOccurrence>,
    pub ai_doc_links: Vec<AiDocLinkSummary>,
    pub ai_doc_timeline: Vec<AiDocTimelinePoint>,
    pub ai_doc_owner_weekly: Vec<AiDocOwnerWeekly>,
}

#[derive(Debug, Clone)]
pub struct StateLayout {
    pub root: PathBuf,
    pub repos_dir: PathBuf,
    pub db_dir: PathBuf,
    pub db_path: PathBuf,
    pub runs_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub exports_dir: PathBuf,
}

impl StateLayout {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        let repos_dir = root.join("repos");
        let db_dir = root.join("db");
        let db_path = db_dir.join("pulse.sqlite");
        let runs_dir = root.join("runs");
        let logs_dir = root.join("logs");
        let exports_dir = root.join("exports");
        Self {
            root,
            repos_dir,
            db_dir,
            db_path,
            runs_dir,
            logs_dir,
            exports_dir,
        }
    }

    pub fn ensure(&self) -> Result<()> {
        for path in [
            &self.root,
            &self.repos_dir,
            &self.db_dir,
            &self.runs_dir,
            &self.logs_dir,
            &self.exports_dir,
        ] {
            std::fs::create_dir_all(path)?;
        }
        Ok(())
    }
}

pub fn config_hash<T: Serialize>(value: &T) -> Result<String> {
    let payload = serde_json::to_vec(value)?;
    let mut hasher = Sha256::new();
    hasher.update(payload);
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn repo_cache_path(root: &Path, repo: &RepoTarget) -> PathBuf {
    root.join(&repo.provider)
        .join(&repo.owner)
        .join(format!("{}.git", repo.name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focus_classification_works() {
        let focus = FocusConfig {
            include: vec!["src/**/*.rs".to_string()],
            exclude: vec!["src/generated/**".to_string()],
        }
        .compile()
        .expect("compile focus");

        assert_eq!(focus.classify("src/lib.rs"), AnalysisDepth::Focused);
        assert_eq!(
            focus.classify("src/generated/lib.rs"),
            AnalysisDepth::Baseline
        );
        assert_eq!(focus.classify("README.md"), AnalysisDepth::Baseline);
    }
}
