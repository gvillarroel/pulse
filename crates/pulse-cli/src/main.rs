use anyhow::{Context, Result, anyhow};
use chrono::Datelike;
use clap::{Args, Parser, Subcommand, ValueEnum};
use pulse_analyze::analyze_revision;
use pulse_config::{AppConfig, load as load_config};
use pulse_core::{
    AiDocLinkSummary, AiDocOccurrence, AiDocOwnerWeekly, AiDocSummary, AiDocTimelinePoint,
    CompiledFocus, FocusConfig, ReportDataset, StageStatus, StateLayout, WeeklyEvolution,
    config_hash,
};
use pulse_export::{report_as_html, summary_as_json, targets_as_csv, targets_as_json};
use pulse_fetch::{fetch_repo, file_content, list_tree};
use pulse_input::resolve_targets;
use pulse_store::Store;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser, Debug)]
#[command(name = "pulse", version, about = "Terminal-first repository analytics")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    List(ListCommand),
    Run(RunCommand),
    Report(ReportCommand),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum OutputFormat {
    Text,
    Csv,
    Json,
}

#[derive(Args, Debug)]
struct SharedInputArgs {
    #[arg(long)]
    config: Option<PathBuf>,
    #[arg(long)]
    input: Option<PathBuf>,
}

#[derive(Args, Debug)]
struct ListCommand {
    #[command(flatten)]
    input: SharedInputArgs,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    format: OutputFormat,
}

#[derive(Args, Debug)]
struct RunCommand {
    #[command(flatten)]
    input: SharedInputArgs,
    #[arg(long)]
    state_dir: PathBuf,
    #[arg(long)]
    workspace: Option<PathBuf>,
    #[arg(long, default_value_t = 4)]
    concurrency: usize,
    #[arg(long)]
    json: bool,
    #[arg(long)]
    progress: bool,
    #[arg(long)]
    with_history: bool,
    #[arg(long)]
    history_window: Option<String>,
    #[arg(long)]
    focus: Vec<String>,
    #[arg(long)]
    focus_file: Option<PathBuf>,
    #[arg(long)]
    fail_fast: bool,
}

#[derive(Args, Debug)]
struct ReportCommand {
    #[arg(long)]
    config: Option<PathBuf>,
    #[arg(long)]
    state_dir: PathBuf,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long)]
    title: Option<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::List(cmd) => list_command(cmd),
        Commands::Run(cmd) => run_command(cmd),
        Commands::Report(cmd) => report_command(cmd),
    }
}

fn list_command(cmd: ListCommand) -> Result<()> {
    let config = maybe_load_config(cmd.input.config.as_deref())?;
    let targets = resolve_targets(config.as_ref(), cmd.input.input.as_deref())?;

    let output = match cmd.format {
        OutputFormat::Text => targets
            .iter()
            .map(|target| format!("{}\t{}", target.key(), target.url))
            .collect::<Vec<_>>()
            .join("\n"),
        OutputFormat::Csv => targets_as_csv(&targets)?,
        OutputFormat::Json => targets_as_json(&targets)?,
    };

    if let Some(path) = cmd.out {
        fs::write(&path, output).with_context(|| format!("failed to write {}", path.display()))?;
    } else {
        println!("{output}");
    }
    Ok(())
}

fn run_command(cmd: RunCommand) -> Result<()> {
    let config = maybe_load_config(cmd.input.config.as_deref())?;
    let targets = resolve_targets(config.as_ref(), cmd.input.input.as_deref())?;
    let layout = StateLayout::new(&cmd.state_dir);
    let mut store = Store::open(&layout)?;
    let run_id = store.begin_run("pulse run")?;

    let focus = merged_focus(config.as_ref(), &cmd.focus, cmd.focus_file.as_deref())?;
    let compiled_focus = focus.compile()?;
    let focus_hash = config_hash(&focus)?;

    let mut failures = 0_usize;
    for repo in targets {
        if cmd.progress {
            eprintln!("processing {}", repo.key());
        }
        store.upsert_repository(&repo)?;
        if let Err(error) = process_repo(
            &mut store,
            &layout,
            &repo,
            &compiled_focus,
            &focus_hash,
            cmd.with_history
                || config
                    .as_ref()
                    .map(|c| c.analysis.with_history)
                    .unwrap_or(false),
        ) {
            failures += 1;
            store.record_failure(
                &repo.key(),
                "run",
                pulse_core::FailureClass::Permanent,
                &error.to_string(),
            )?;
            if cmd.fail_fast {
                return Err(error);
            }
        }
    }

    store.finish_run(run_id)?;
    let summary = store.summarize_run(run_id)?;
    if cmd.json {
        println!("{}", summary_as_json(&summary)?);
    } else {
        println!(
            "run {} complete: processed={}, failed={}, concurrency={}",
            summary.run_id, summary.processed, failures, cmd.concurrency
        );
    }
    Ok(())
}

fn report_command(cmd: ReportCommand) -> Result<()> {
    let config = maybe_load_config(cmd.config.as_deref())?;
    let layout = StateLayout::new(&cmd.state_dir);
    let store = Store::open(&layout)?;
    let mut dataset = store.build_report_dataset()?;
    let ai_docs = config
        .as_ref()
        .map(|cfg| cfg.report.ai_docs.clone())
        .unwrap_or_default();
    let ai_doc_matcher = if ai_docs.include.is_empty() && ai_docs.exclude.is_empty() {
        None
    } else {
        Some(ai_docs.compile()?)
    };
    enrich_ai_report(
        &mut dataset,
        &store,
        &cmd.state_dir,
        ai_doc_matcher.as_ref(),
    )?;
    let output_path = cmd
        .out
        .unwrap_or_else(|| layout.exports_dir.join("report.html"));
    let title = cmd.title.unwrap_or_else(|| {
        format!(
            "Pulse Report - {}",
            cmd.state_dir
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("state")
        )
    });
    let generated_at = chrono::Local::now()
        .format("%Y-%m-%d %H:%M:%S %:z")
        .to_string();
    let html = report_as_html(&title, &generated_at, &dataset)?;
    fs::write(&output_path, html)
        .with_context(|| format!("failed to write {}", output_path.display()))?;
    println!("{}", output_path.display());
    Ok(())
}

fn enrich_ai_report(
    dataset: &mut ReportDataset,
    store: &Store,
    state_dir: &Path,
    ai_doc_matcher: Option<&CompiledFocus>,
) -> Result<()> {
    let repo_heads = store.fetch_outcomes()?;
    let latest_paths = store.latest_file_paths()?;
    let mut candidate_paths_by_repo: HashMap<String, Vec<String>> = HashMap::new();
    for (repo_key, path) in latest_paths {
        if ai_doc_category(&path, ai_doc_matcher).is_some() {
            candidate_paths_by_repo
                .entry(repo_key)
                .or_default()
                .push(path);
        }
    }

    let total_repositories = dataset.summary.repositories.max(1) as f64;
    let mut occurrences = Vec::new();
    let mut by_doc: HashMap<(String, String), (HashSet<String>, u64)> = HashMap::new();
    let mut by_link: HashMap<(String, String), HashSet<String>> = HashMap::new();
    let mut timeline_seed: HashMap<(String, String), BTreeMap<String, HashSet<String>>> =
        HashMap::new();
    let repo_owner: HashMap<String, String> = dataset
        .repositories
        .iter()
        .map(|repo| (repo.repo_key.clone(), repo.owner.clone()))
        .collect();
    let mut owner_weekly_ai_doc_commits: HashMap<(String, String), u64> = HashMap::new();

    for fetch in repo_heads {
        let Some(paths) = candidate_paths_by_repo.get(&fetch.repo_key) else {
            continue;
        };
        let git_dir = resolve_git_dir(&fetch.git_dir, state_dir);
        let owner = repo_owner
            .get(&fetch.repo_key)
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        let weekly_ai_doc_commits = build_ai_doc_commit_history(&git_dir, paths)?;
        for (week_start, commits) in weekly_ai_doc_commits {
            *owner_weekly_ai_doc_commits
                .entry((owner.clone(), week_start))
                .or_default() += commits;
        }
        for path in paths {
            let Some(category) = ai_doc_category(path, ai_doc_matcher) else {
                continue;
            };
            let doc_name = file_name_lower(path);
            let linked_docs = read_markdown_links(&git_dir, &fetch.fetched_revision, path)?;
            for linked_doc in &linked_docs {
                by_link
                    .entry((doc_name.clone(), linked_doc.clone()))
                    .or_default()
                    .insert(fetch.repo_key.clone());
            }

            if let Some(date) = first_addition_date(&git_dir, &fetch.fetched_revision, path)? {
                let week = week_start_string(date)?;
                timeline_seed
                    .entry((doc_name.clone(), path.clone()))
                    .or_default()
                    .entry(week)
                    .or_default()
                    .insert(fetch.repo_key.clone());
            }

            by_doc
                .entry((doc_name.clone(), category.to_string()))
                .and_modify(|(repos, files)| {
                    repos.insert(fetch.repo_key.clone());
                    *files += 1;
                })
                .or_insert_with(|| {
                    let mut repos = HashSet::new();
                    repos.insert(fetch.repo_key.clone());
                    (repos, 1)
                });

            occurrences.push(AiDocOccurrence {
                repo_key: fetch.repo_key.clone(),
                doc_name,
                category: category.to_string(),
                path: path.clone(),
                linked_docs,
            });
        }
    }

    let mut summaries = by_doc
        .into_iter()
        .map(|((doc_name, category), (repos, files))| AiDocSummary {
            doc_name,
            category,
            repositories: repos.len() as u64,
            files,
            adoption_pct: (repos.len() as f64 / total_repositories) * 100.0,
        })
        .collect::<Vec<_>>();
    summaries.sort_by(|a, b| {
        b.repositories
            .cmp(&a.repositories)
            .then_with(|| a.doc_name.cmp(&b.doc_name))
    });

    let mut links = by_link
        .into_iter()
        .map(|((source_doc, linked_doc), repos)| AiDocLinkSummary {
            source_doc,
            linked_doc,
            repositories: repos.len() as u64,
            adoption_pct: (repos.len() as f64 / total_repositories) * 100.0,
        })
        .collect::<Vec<_>>();
    links.sort_by(|a, b| {
        b.repositories
            .cmp(&a.repositories)
            .then_with(|| a.source_doc.cmp(&b.source_doc))
            .then_with(|| a.linked_doc.cmp(&b.linked_doc))
    });

    let mut timeline = Vec::new();
    for ((doc_name, path), by_week) in timeline_seed {
        let mut seen = HashSet::new();
        for (week_start, repos) in by_week {
            seen.extend(repos);
            timeline.push(AiDocTimelinePoint {
                week_start,
                doc_name: doc_name.clone(),
                path: path.clone(),
                cumulative_repositories: seen.len() as u64,
            });
        }
    }
    timeline.sort_by(|a, b| {
        a.week_start
            .cmp(&b.week_start)
            .then_with(|| a.doc_name.cmp(&b.doc_name))
            .then_with(|| a.path.cmp(&b.path))
    });

    occurrences.sort_by(|a, b| {
        a.repo_key
            .cmp(&b.repo_key)
            .then_with(|| a.path.cmp(&b.path))
    });
    dataset.ai_doc_summaries = summaries;
    dataset.ai_doc_occurrences = occurrences;
    dataset.ai_doc_links = links;
    dataset.ai_doc_timeline = timeline;
    dataset.ai_doc_owner_weekly = owner_weekly_ai_doc_commits
        .into_iter()
        .map(|((owner, week_start), commits)| AiDocOwnerWeekly {
            owner,
            week_start,
            commits,
        })
        .collect::<Vec<_>>();
    dataset.ai_doc_owner_weekly.sort_by(|a, b| {
        a.week_start
            .cmp(&b.week_start)
            .then_with(|| a.owner.cmp(&b.owner))
    });
    Ok(())
}

fn ai_doc_category(path: &str, ai_doc_matcher: Option<&CompiledFocus>) -> Option<&'static str> {
    if let Some(matcher) = ai_doc_matcher {
        if !matcher.matches(path) {
            return None;
        }
        return heuristic_ai_doc_category(path).or(Some("configured_ai_doc"));
    }
    heuristic_ai_doc_category(path)
}

fn heuristic_ai_doc_category(path: &str) -> Option<&'static str> {
    let path_lower = path.to_ascii_lowercase();
    let name = file_name_lower(path);
    match name.as_str() {
        "agents.md" => Some("agent_instructions"),
        "claude.md" => Some("assistant_contract"),
        "skill.md" => Some("skill_definition"),
        "skills.md" => Some("skill_catalog"),
        "copilot-instructions.md" => Some("assistant_contract"),
        "spec.md" | "specs.md" => Some("governance_spec"),
        _ if path_lower.contains("agent") && path_lower.ends_with(".md") => Some("agent_related"),
        _ if path_lower.contains("skill") && path_lower.ends_with(".md") => Some("skill_related"),
        _ if path_lower.contains("copilot") && path_lower.ends_with(".md") => {
            Some("assistant_contract")
        }
        _ if path_lower.contains("prompt") && path_lower.ends_with(".md") => Some("prompt_related"),
        _ if path_lower.contains("model") && path_lower.ends_with(".md") => Some("model_related"),
        _ if path_lower.contains("ai-") && path_lower.ends_with(".md") => Some("ai_related"),
        _ if path_lower.contains("/ai/") && path_lower.ends_with(".md") => Some("ai_related"),
        _ => None,
    }
}

fn file_name_lower(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map(|value| value.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_else(|| path.to_ascii_lowercase())
}

fn resolve_git_dir(git_dir: &Path, state_dir: &Path) -> PathBuf {
    if git_dir.is_absolute() && git_dir.exists() {
        return git_dir.to_path_buf();
    }
    let candidates = [
        git_dir.to_path_buf(),
        state_dir.join(git_dir),
        state_dir
            .parent()
            .map(|base| base.join(git_dir))
            .unwrap_or_else(|| git_dir.to_path_buf()),
        state_dir
            .parent()
            .and_then(|base| base.parent().map(|grand| grand.join(git_dir)))
            .unwrap_or_else(|| git_dir.to_path_buf()),
    ];
    candidates
        .into_iter()
        .find(|candidate| candidate.exists())
        .unwrap_or_else(|| git_dir.to_path_buf())
}

fn read_markdown_links(git_dir: &Path, revision: &str, path: &str) -> Result<Vec<String>> {
    let content = file_content(git_dir, revision, path)
        .with_context(|| format!("failed to inspect markdown links in {path}"))?;
    let text = String::from_utf8_lossy(&content);
    let mut links = BTreeSet::new();
    let mut rest = text.as_ref();
    while let Some(start) = rest.find("](") {
        rest = &rest[start + 2..];
        let Some(end) = rest.find(')') else {
            break;
        };
        let target = rest[..end].trim();
        if is_local_markdown_link(target) {
            links.insert(file_name_lower(target));
        }
        rest = &rest[end + 1..];
    }
    Ok(links.into_iter().collect())
}

fn is_local_markdown_link(target: &str) -> bool {
    if target.is_empty() || target.starts_with("http://") || target.starts_with("https://") {
        return false;
    }
    let clean = target.split('#').next().unwrap_or(target);
    clean.to_ascii_lowercase().ends_with(".md")
}

fn first_addition_date(
    git_dir: &Path,
    revision: &str,
    path: &str,
) -> Result<Option<chrono::NaiveDate>> {
    let output = Command::new("git")
        .arg(format!("--git-dir={}", git_dir.display()))
        .args([
            "log",
            "--diff-filter=A",
            "--follow",
            "--date=short",
            "--format=%cs",
            revision,
            "--",
            path,
        ])
        .output()
        .context("failed to run git log for ai doc timeline")?;
    if !output.status.success() {
        return Ok(None);
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let date = stdout
        .lines()
        .last()
        .map(str::trim)
        .filter(|line| !line.is_empty());
    match date {
        Some(date) => Ok(Some(chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")?)),
        None => Ok(None),
    }
}

fn week_start_string(date: chrono::NaiveDate) -> Result<String> {
    let offset = date.weekday().num_days_from_monday() as i64;
    let week_start = date - chrono::Days::new(offset as u64);
    Ok(week_start.format("%Y-%m-%d").to_string())
}

fn process_repo(
    store: &mut Store,
    layout: &StateLayout,
    repo: &pulse_core::RepoTarget,
    focus: &CompiledFocus,
    focus_hash: &str,
    with_history: bool,
) -> Result<()> {
    store.set_stage_status(&repo.key(), "fetch", StageStatus::Running, None)?;
    let fetch = fetch_repo(&layout.repos_dir, repo)?;
    store.persist_fetch(&fetch)?;
    store.set_stage_status(&repo.key(), "fetch", StageStatus::Completed, None)?;

    let current_hash = store.existing_snapshot_config_hash(&repo.key(), &fetch.fetched_revision)?;
    if current_hash.as_deref() != Some(focus_hash) {
        store.set_stage_status(&repo.key(), "analyze", StageStatus::Running, None)?;
        let tree = list_tree(&fetch.git_dir, &fetch.fetched_revision)?;
        let mut files = Vec::with_capacity(tree.len());
        for entry in tree {
            let contents = file_content(&fetch.git_dir, &fetch.fetched_revision, &entry.path)
                .with_context(|| format!("failed to read {}", entry.path))?;
            files.push((entry.path, contents));
        }
        let (repo_snapshot, file_snapshots) = analyze_revision(
            &repo.key(),
            &fetch.fetched_revision,
            files,
            focus,
            focus_hash,
        )?;
        store.persist_snapshot(&repo_snapshot, &file_snapshots)?;
        store.set_stage_status(&repo.key(), "analyze", StageStatus::Completed, None)?;
    }

    if with_history {
        store.set_stage_status(&repo.key(), "history", StageStatus::Running, None)?;
        let weekly = build_weekly_history(&fetch.git_dir, &repo.key())?;
        store.persist_weekly_evolution(&weekly)?;
        store.set_stage_status(&repo.key(), "history", StageStatus::Completed, None)?;
    }
    Ok(())
}

fn maybe_load_config(path: Option<&Path>) -> Result<Option<AppConfig>> {
    path.map(load_config).transpose()
}

fn merged_focus(
    config: Option<&AppConfig>,
    cli_focus: &[String],
    focus_file: Option<&Path>,
) -> Result<FocusConfig> {
    let mut focus = config.map(|cfg| cfg.focus.clone()).unwrap_or_default();
    focus.include.extend_from_slice(cli_focus);
    if let Some(path) = focus_file {
        let raw = fs::read_to_string(path)
            .with_context(|| format!("failed to read focus file {}", path.display()))?;
        for line in raw.lines().map(str::trim).filter(|line| !line.is_empty()) {
            focus.include.push(line.to_string());
        }
    }
    Ok(focus)
}

fn build_weekly_history(git_dir: &Path, repo_key: &str) -> Result<Vec<WeeklyEvolution>> {
    let output = Command::new("git")
        .arg(format!("--git-dir={}", git_dir.display()))
        .args(["log", "--date=short", "--format=%cs|%ae"])
        .output()
        .context("failed to run git log")?;
    if !output.status.success() {
        return Err(anyhow!(
            "git log failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut by_week: BTreeMap<String, (u64, BTreeSet<String>)> = BTreeMap::new();
    for line in stdout.lines() {
        let Some((date, author)) = line.split_once('|') else {
            continue;
        };
        let date = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")?;
        let offset = date.weekday().num_days_from_monday() as i64;
        let week_start = date - chrono::Days::new(offset as u64);
        let entry = by_week
            .entry(week_start.format("%Y-%m-%d").to_string())
            .or_insert((0, BTreeSet::new()));
        entry.0 += 1;
        entry.1.insert(author.to_string());
    }

    Ok(by_week
        .into_iter()
        .map(
            |(week_start, (commit_count, contributors))| WeeklyEvolution {
                repo_key: repo_key.to_string(),
                week_start,
                commit_count,
                active_contributors: contributors.len() as u64,
            },
        )
        .collect())
}

fn build_ai_doc_commit_history(git_dir: &Path, paths: &[String]) -> Result<BTreeMap<String, u64>> {
    if paths.is_empty() {
        return Ok(BTreeMap::new());
    }

    let mut command = Command::new("git");
    command
        .arg(format!("--git-dir={}", git_dir.display()))
        .args(["log", "--date=short", "--format=%cs"]);
    command.arg("--");
    for path in paths {
        command.arg(path);
    }
    let output = command
        .output()
        .context("failed to run git log for AI docs")?;
    if !output.status.success() {
        return Err(anyhow!(
            "git log for AI docs failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut by_week = BTreeMap::new();
    for line in stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        let date = chrono::NaiveDate::parse_from_str(line, "%Y-%m-%d")?;
        let offset = date.weekday().num_days_from_monday() as i64;
        let week_start = date - chrono::Days::new(offset as u64);
        *by_week
            .entry(week_start.format("%Y-%m-%d").to_string())
            .or_insert(0) += 1;
    }
    Ok(by_week)
}
