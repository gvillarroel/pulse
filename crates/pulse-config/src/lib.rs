use anyhow::Context;
use pulse_core::{FocusConfig, OwnerLevel, ReportOwnerLevelsConfig, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub repositories: RepositorySection,
    #[serde(default)]
    pub discovery: DiscoverySection,
    #[serde(default)]
    pub analysis: AnalysisSection,
    #[serde(default)]
    pub report: ReportSection,
    #[serde(default)]
    pub focus: FocusConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RepositorySection {
    pub csv: Option<PathBuf>,
    #[serde(default)]
    pub items: Vec<RepositoryItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RepositoryItem {
    pub repo: String,
    pub provider: Option<String>,
    pub owner: Option<String>,
    pub owner_color: Option<String>,
    #[serde(default)]
    pub owner_levels: Vec<OwnerLevel>,
    pub team: Option<String>,
    pub team_color: Option<String>,
    pub name: Option<String>,
    pub url: Option<String>,
    pub default_branch: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default = "default_true")]
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiscoverySection {
    pub provider: Option<String>,
    pub owner: Option<String>,
    pub org: Option<String>,
    pub language: Option<String>,
    pub topic: Option<String>,
    pub pushed_since: Option<String>,
    pub created_since: Option<String>,
    #[serde(default)]
    pub include_archived: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnalysisSection {
    #[serde(default)]
    pub with_history: bool,
    pub history_window: Option<String>,
    #[serde(default)]
    pub with_rust_analyzer: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReportSection {
    #[serde(default)]
    pub ai_docs: FocusConfig,
    #[serde(default)]
    pub owner_levels: ReportOwnerLevelsConfig,
}

pub fn load(path: &Path) -> Result<AppConfig> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read config {}", path.display()))?;
    let mut config: AppConfig = serde_yaml::from_str(&raw)
        .with_context(|| format!("invalid YAML in {}", path.display()))?;

    if let Some(csv) = &config.repositories.csv {
        if csv.is_relative() {
            let base = path.parent().unwrap_or_else(|| Path::new("."));
            config.repositories.csv = Some(base.join(csv));
        }
    }

    Ok(config)
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_yaml() {
        let yaml = r#"
repositories:
  items:
    - repo: openai/openai-cookbook
focus:
  include:
    - src/**/*.rs
"#;
        let config: AppConfig = serde_yaml::from_str(yaml).expect("yaml parse");
        assert_eq!(config.repositories.items.len(), 1);
        assert_eq!(config.focus.include, vec!["src/**/*.rs"]);
    }
}
