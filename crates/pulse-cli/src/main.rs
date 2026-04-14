use anyhow::{Context, Result, anyhow};
use chrono::Datelike;
use clap::{Args, Parser, Subcommand, ValueEnum};
use pulse_analyze::analyze_revision;
use pulse_config::{AppConfig, load as load_config};
use pulse_core::{
    AiDocOccurrence, CompiledFocus, FocusConfig, ReportRenderOptions, StageStatus, StateLayout,
    WeeklyEvolution, config_hash,
};
use pulse_export::{report_as_html, summary_as_json, targets_as_csv, targets_as_json};
use pulse_fetch::{EMPTY_REPOSITORY_REVISION, fetch_repo, file_content, list_tree};
use pulse_input::resolve_targets;
use pulse_store::Store;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

const MAX_GIT_PATH_ARG_BYTES: usize = 6_000;

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
    validate_run_command(&cmd)?;
    let targets = resolve_targets(config.as_ref(), cmd.input.input.as_deref())?;
    let run_plan = build_run_plan(&cmd, config.as_ref(), targets.len())?;
    let store = Store::open(&run_plan.layout)?;
    let run_id = store.begin_run("pulse run")?;
    let outcomes = execute_run_jobs(&cmd, run_plan, targets)?;
    let (failures, fatal_error) = summarize_job_outcomes(outcomes);

    store.finish_run(run_id)?;
    if cmd.fail_fast
        && let Some(error) = fatal_error
    {
        return Err(error);
    }
    let summary = store.summarize_run(run_id)?;
    print_run_summary(&cmd, &summary, failures)?;
    Ok(())
}

#[derive(Clone)]
struct RunPlan {
    layout: StateLayout,
    compiled_focus: CompiledFocus,
    focus_hash: String,
    ai_doc_matcher: Option<CompiledFocus>,
    with_history: bool,
    total_targets: usize,
}

fn validate_run_command(cmd: &RunCommand) -> Result<()> {
    if cmd.concurrency == 0 {
        return Err(anyhow!("--concurrency must be at least 1"));
    }
    Ok(())
}

fn build_run_plan(
    cmd: &RunCommand,
    config: Option<&AppConfig>,
    total_targets: usize,
) -> Result<RunPlan> {
    let focus = merged_focus(config, &cmd.focus, cmd.focus_file.as_deref())?;
    let compiled_focus = focus.compile()?;
    let focus_hash = config_hash(&focus)?;
    Ok(RunPlan {
        layout: StateLayout::new(&cmd.state_dir),
        compiled_focus,
        focus_hash,
        ai_doc_matcher: compile_ai_doc_matcher(config)?,
        with_history: cmd.with_history || config.map(|c| c.analysis.with_history).unwrap_or(false),
        total_targets,
    })
}

fn compile_ai_doc_matcher(config: Option<&AppConfig>) -> Result<Option<CompiledFocus>> {
    let ai_docs = config
        .map(|cfg| cfg.report.ai_docs.clone())
        .unwrap_or_default();
    if ai_docs.include.is_empty() && ai_docs.exclude.is_empty() {
        Ok(None)
    } else {
        Ok(Some(ai_docs.compile()?))
    }
}

fn execute_run_jobs(
    cmd: &RunCommand,
    run_plan: RunPlan,
    targets: Vec<pulse_core::RepoTarget>,
) -> Result<Vec<RepoJobOutcome>> {
    let (progress_tx, progress_rx) = mpsc::channel();
    let worker_plan = run_plan.clone();
    let concurrency = cmd.concurrency;
    let fail_fast = cmd.fail_fast;
    let worker = thread::spawn(move || {
        run_parallel_jobs(targets, concurrency, fail_fast, move |repo| {
            let outcome = process_repo_job(repo, &worker_plan, Some(&progress_tx));
            let failed = outcome.failed;
            (outcome, failed)
        })
    });
    render_progress(run_plan.total_targets, cmd.progress, progress_rx);
    worker
        .join()
        .map_err(|_| anyhow!("parallel worker thread panicked"))
}

fn render_progress(
    total_targets: usize,
    enabled: bool,
    progress_rx: mpsc::Receiver<ProgressEvent>,
) {
    if !enabled {
        while progress_rx.recv().is_ok() {}
        return;
    }
    let mut progress = ProgressReporter::new(total_targets);
    while let Ok(event) = progress_rx.recv() {
        match event {
            ProgressEvent::Started(repo_key) => progress.start_repo(&repo_key),
            ProgressEvent::Finished { repo_key, success } => {
                progress.finish_repo(&repo_key, success)
            }
        }
    }
    progress.finish();
}

fn summarize_job_outcomes(outcomes: Vec<RepoJobOutcome>) -> (usize, Option<anyhow::Error>) {
    let mut failures = 0_usize;
    let mut fatal_error = None;
    for outcome in outcomes {
        if let Err(error) = outcome.result {
            failures += 1;
            if fatal_error.is_none() {
                fatal_error = Some(error);
            }
        }
    }
    (failures, fatal_error)
}

fn print_run_summary(
    cmd: &RunCommand,
    summary: &pulse_core::RunSummary,
    failures: usize,
) -> Result<()> {
    if cmd.json {
        println!("{}", summary_as_json(summary)?);
    } else {
        println!(
            "run {} complete: processed={}, failed={}, concurrency={}",
            summary.run_id, summary.processed, failures, cmd.concurrency
        );
    }
    Ok(())
}

#[derive(Debug)]
struct RepoJobOutcome {
    result: Result<()>,
    failed: bool,
}

#[derive(Debug)]
enum ProgressEvent {
    Started(String),
    Finished { repo_key: String, success: bool },
}

fn process_repo_job(
    repo: pulse_core::RepoTarget,
    run_plan: &RunPlan,
    progress_tx: Option<&mpsc::Sender<ProgressEvent>>,
) -> RepoJobOutcome {
    let repo_key = repo.key();
    if let Some(tx) = progress_tx {
        let _ = tx.send(ProgressEvent::Started(repo_key.clone()));
    }

    let result = (|| -> Result<()> {
        let mut store = Store::open(&run_plan.layout)?;
        store.upsert_repository(&repo)?;
        if let Err(error) = process_repo(&mut store, run_plan, &repo) {
            store.record_failure(
                &repo_key,
                "run",
                pulse_core::FailureClass::Permanent,
                &error.to_string(),
            )?;
            return Err(error);
        }
        Ok(())
    })();

    if let Some(tx) = progress_tx {
        let _ = tx.send(ProgressEvent::Finished {
            repo_key,
            success: result.is_ok(),
        });
    }

    RepoJobOutcome {
        failed: result.is_err(),
        result,
    }
}

fn run_parallel_jobs<T, R, F>(
    items: Vec<T>,
    concurrency: usize,
    fail_fast: bool,
    process: F,
) -> Vec<R>
where
    T: Send + 'static,
    R: Send + 'static,
    F: Fn(T) -> (R, bool) + Send + Sync + 'static,
{
    let worker_count = items.len().min(concurrency.max(1));
    if worker_count == 0 {
        return Vec::new();
    }

    let queue = Arc::new(Mutex::new(std::collections::VecDeque::from(items)));
    let results = Arc::new(Mutex::new(Vec::new()));
    let cancelled = Arc::new(AtomicBool::new(false));
    let process = Arc::new(process);
    let mut handles = Vec::with_capacity(worker_count);

    for _ in 0..worker_count {
        let queue = Arc::clone(&queue);
        let results = Arc::clone(&results);
        let cancelled = Arc::clone(&cancelled);
        let process = Arc::clone(&process);
        handles.push(thread::spawn(move || {
            loop {
                if cancelled.load(Ordering::Acquire) {
                    break;
                }
                let next = {
                    let mut queue = queue.lock().expect("job queue poisoned");
                    queue.pop_front()
                };
                let Some(item) = next else {
                    break;
                };
                let (result, failed) = process(item);
                if failed && fail_fast {
                    cancelled.store(true, Ordering::Release);
                }
                results.lock().expect("results poisoned").push(result);
            }
        }));
    }

    for handle in handles {
        handle.join().expect("parallel job worker panicked");
    }

    let mut results = match Arc::try_unwrap(results) {
        Ok(results) => results.into_inner().expect("parallel job results poisoned"),
        Err(_) => panic!("parallel job results still shared"),
    };
    results.shrink_to_fit();
    results
}

struct ProgressReporter {
    total: usize,
    completed: usize,
    failed: usize,
    interactive: bool,
}

impl ProgressReporter {
    fn new(total: usize) -> Self {
        Self {
            total,
            completed: 0,
            failed: 0,
            interactive: io::stderr().is_terminal(),
        }
    }

    fn start_repo(&mut self, repo_key: &str) {
        self.render(Some(repo_key), None);
    }

    fn finish_repo(&mut self, repo_key: &str, success: bool) {
        self.completed += 1;
        if !success {
            self.failed += 1;
        }
        let status = if success { "done" } else { "failed" };
        self.render(Some(repo_key), Some(status));
        if !self.interactive {
            let _ = writeln!(io::stderr());
        }
    }

    fn finish(&mut self) {
        if self.interactive {
            let _ = writeln!(io::stderr());
        }
    }

    fn render(&self, repo_key: Option<&str>, status: Option<&str>) {
        let processed = self.completed.min(self.total);
        let remaining = self.total.saturating_sub(processed);
        let percentage = if self.total == 0 {
            100
        } else {
            (processed * 100) / self.total
        };
        let width = 24;
        let filled = if self.total == 0 {
            width
        } else {
            (processed * width) / self.total
        };
        let bar = format!(
            "{}{}",
            "#".repeat(filled),
            "-".repeat(width.saturating_sub(filled))
        );
        let repo_label = repo_key.unwrap_or("idle");
        let status_label = status.unwrap_or("running");
        let message = format!(
            "[{bar}] {processed}/{total} ({percentage:>3}%) remaining={remaining} failed={} {status_label} {repo_label}",
            self.failed,
            total = self.total
        );

        if self.interactive {
            let _ = write!(io::stderr(), "\r{message}");
            let _ = io::stderr().flush();
        } else {
            let _ = write!(io::stderr(), "{message}");
        }
    }
}

fn report_command(cmd: ReportCommand) -> Result<()> {
    let config = maybe_load_config(cmd.config.as_deref())?;
    let layout = StateLayout::new(&cmd.state_dir);
    let store = Store::open(&layout)?;
    let dataset = store.build_report_dataset()?;
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
    let render_options = ReportRenderOptions {
        owner_levels: config
            .map(|cfg| cfg.report.owner_levels)
            .unwrap_or_default(),
    };
    let html = report_as_html(&title, &generated_at, &dataset, &render_options)?;
    fs::write(&output_path, html)
        .with_context(|| format!("failed to write {}", output_path.display()))?;
    println!("{}", output_path.display());
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

fn read_markdown_links(git_dir: &Path, revision: &str, path: &str) -> Result<Vec<String>> {
    if revision == EMPTY_REPOSITORY_REVISION {
        return Ok(Vec::new());
    }
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
    if revision == EMPTY_REPOSITORY_REVISION {
        return Ok(None);
    }
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
    run_plan: &RunPlan,
    repo: &pulse_core::RepoTarget,
) -> Result<()> {
    let fetch = run_fetch_stage(store, &run_plan.layout, repo)?;
    let current_hash = store.existing_snapshot_config_hash(&repo.key(), &fetch.fetched_revision)?;
    let mut latest_paths = None;
    if current_hash.as_deref() != Some(run_plan.focus_hash.as_str()) {
        latest_paths = Some(run_analyze_stage(
            store,
            repo,
            &fetch,
            &run_plan.compiled_focus,
            &run_plan.focus_hash,
        )?);
    }
    run_ai_doc_stage(
        store,
        repo,
        &fetch,
        latest_paths,
        run_plan.ai_doc_matcher.as_ref(),
    )?;
    if run_plan.with_history {
        run_history_stage(store, repo, &fetch)?;
    }
    store.set_stage_status(&repo.key(), "run", StageStatus::Completed, None)?;
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

fn run_fetch_stage(
    store: &mut Store,
    layout: &StateLayout,
    repo: &pulse_core::RepoTarget,
) -> Result<pulse_core::FetchOutcome> {
    store.set_stage_status(&repo.key(), "fetch", StageStatus::Running, None)?;
    let fetch = fetch_repo(&layout.repos_dir, repo)?;
    store.persist_fetch(&fetch)?;
    store.set_stage_status(&repo.key(), "fetch", StageStatus::Completed, None)?;
    Ok(fetch)
}

fn run_analyze_stage(
    store: &mut Store,
    repo: &pulse_core::RepoTarget,
    fetch: &pulse_core::FetchOutcome,
    focus: &CompiledFocus,
    focus_hash: &str,
) -> Result<Vec<String>> {
    store.set_stage_status(&repo.key(), "analyze", StageStatus::Running, None)?;
    let (files, skipped_reads) = load_repo_files(fetch)?;
    let (repo_snapshot, file_snapshots) = analyze_revision(
        &repo.key(),
        &fetch.fetched_revision,
        files,
        focus,
        focus_hash,
    )?;
    let latest_paths = file_snapshots
        .iter()
        .map(|snapshot| snapshot.path.clone())
        .collect::<Vec<_>>();
    store.persist_snapshot(&repo_snapshot, &file_snapshots)?;
    let analyze_detail =
        (skipped_reads > 0).then(|| format!("skipped {skipped_reads} unreadable tree entries"));
    store.set_stage_status(
        &repo.key(),
        "analyze",
        StageStatus::Completed,
        analyze_detail.as_deref(),
    )?;
    Ok(latest_paths)
}

fn load_repo_files(fetch: &pulse_core::FetchOutcome) -> Result<(Vec<(String, Vec<u8>)>, u64)> {
    let tree = list_tree(&fetch.git_dir, &fetch.fetched_revision)?;
    let mut files = Vec::with_capacity(tree.len());
    let mut skipped_reads = 0_u64;
    for entry in tree {
        match file_content(&fetch.git_dir, &fetch.fetched_revision, &entry.path) {
            Ok(contents) => files.push((entry.path, contents)),
            Err(_) => skipped_reads += 1,
        }
    }
    Ok((files, skipped_reads))
}

fn run_ai_doc_stage(
    store: &mut Store,
    repo: &pulse_core::RepoTarget,
    fetch: &pulse_core::FetchOutcome,
    latest_paths: Option<Vec<String>>,
    ai_doc_matcher: Option<&CompiledFocus>,
) -> Result<()> {
    let ai_doc_paths = select_ai_doc_paths(store, repo, fetch, latest_paths, ai_doc_matcher)?;
    let ai_doc_occurrences = build_ai_doc_occurrences(
        &fetch.git_dir,
        &fetch.fetched_revision,
        &repo.key(),
        &ai_doc_paths,
        ai_doc_matcher,
    )?;
    let ai_doc_weekly_activity = build_ai_doc_commit_history(&fetch.git_dir, &ai_doc_paths)?
        .into_iter()
        .collect::<Vec<_>>();
    store.replace_ai_doc_analysis(&repo.key(), &ai_doc_occurrences, &ai_doc_weekly_activity)?;
    Ok(())
}

fn select_ai_doc_paths(
    store: &Store,
    repo: &pulse_core::RepoTarget,
    fetch: &pulse_core::FetchOutcome,
    latest_paths: Option<Vec<String>>,
    ai_doc_matcher: Option<&CompiledFocus>,
) -> Result<Vec<String>> {
    Ok(latest_paths
        .unwrap_or(store.file_paths_for_revision(&repo.key(), &fetch.fetched_revision)?)
        .into_iter()
        .filter(|path| ai_doc_category(path, ai_doc_matcher).is_some())
        .collect())
}

fn run_history_stage(
    store: &mut Store,
    repo: &pulse_core::RepoTarget,
    fetch: &pulse_core::FetchOutcome,
) -> Result<()> {
    store.set_stage_status(&repo.key(), "history", StageStatus::Running, None)?;
    let weekly = build_weekly_history(&fetch.git_dir, &repo.key())?;
    store.persist_weekly_evolution(&weekly)?;
    store.set_stage_status(&repo.key(), "history", StageStatus::Completed, None)?;
    Ok(())
}

fn build_ai_doc_occurrences(
    git_dir: &Path,
    revision: &str,
    repo_key: &str,
    paths: &[String],
    ai_doc_matcher: Option<&CompiledFocus>,
) -> Result<Vec<AiDocOccurrence>> {
    let mut occurrences = Vec::new();
    for path in paths {
        let Some(category) = ai_doc_category(path, ai_doc_matcher) else {
            continue;
        };
        let first_seen_week_start = first_addition_date(git_dir, revision, path)?
            .map(week_start_string)
            .transpose()?;
        occurrences.push(AiDocOccurrence {
            repo_key: repo_key.to_string(),
            doc_name: file_name_lower(path),
            category: category.to_string(),
            path: path.clone(),
            first_seen_week_start,
            linked_docs: read_markdown_links(git_dir, revision, path).unwrap_or_default(),
        });
    }
    occurrences.sort_by(|a, b| {
        a.repo_key
            .cmp(&b.repo_key)
            .then_with(|| a.path.cmp(&b.path))
    });
    Ok(occurrences)
}

fn build_weekly_history(git_dir: &Path, repo_key: &str) -> Result<Vec<WeeklyEvolution>> {
    let output = Command::new("git")
        .arg(format!("--git-dir={}", git_dir.display()))
        .args(["log", "--date=short", "--format=%cs|%ae"])
        .output()
        .context("failed to run git log")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if is_empty_revision_message(&stderr) {
            return Ok(Vec::new());
        }
        return Err(anyhow!("git log failed: {}", stderr.trim()));
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

    let mut by_week = BTreeMap::new();
    for chunk in chunk_paths_for_git(paths) {
        let mut command = Command::new("git");
        command
            .arg(format!("--git-dir={}", git_dir.display()))
            .args(["log", "--date=short", "--format=%cs"]);
        command.arg("--");
        for path in chunk {
            command.arg(path);
        }
        let output = command
            .output()
            .context("failed to run git log for AI docs")?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if is_empty_revision_message(&stderr) {
                continue;
            }
            return Err(anyhow!("git log for AI docs failed: {}", stderr.trim()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
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
    }
    Ok(by_week)
}

fn chunk_paths_for_git(paths: &[String]) -> Vec<Vec<&str>> {
    let mut chunks = Vec::new();
    let mut current = Vec::new();
    let mut current_bytes = 0_usize;

    for path in paths {
        let path_bytes = path.len() + 1;
        if !current.is_empty() && current_bytes + path_bytes > MAX_GIT_PATH_ARG_BYTES {
            chunks.push(current);
            current = Vec::new();
            current_bytes = 0;
        }
        current.push(path.as_str());
        current_bytes += path_bytes;
    }

    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
}

fn is_empty_revision_message(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower.contains("ambiguous argument 'head'")
        || lower.contains("does not have any commits yet")
        || lower.contains("your current branch")
        || lower.contains("unknown revision or path not in the working tree")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    #[test]
    fn chunks_git_paths_to_bounded_argument_lists() {
        let paths = vec!["a".repeat(3_000), "b".repeat(3_000), "c".repeat(3_000)];
        let chunks = chunk_paths_for_git(&paths);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].len(), 1);
        assert_eq!(chunks[1].len(), 1);
        assert_eq!(chunks[2].len(), 1);
    }

    #[test]
    fn detects_empty_revision_errors() {
        assert!(is_empty_revision_message(
            "fatal: ambiguous argument 'HEAD': unknown revision or path not in the working tree."
        ));
        assert!(is_empty_revision_message(
            "fatal: your current branch 'main' does not have any commits yet"
        ));
        assert!(!is_empty_revision_message("fatal: bad object deadbeef"));
    }

    #[test]
    fn parallel_jobs_use_more_than_one_worker_when_concurrency_allows() {
        let active = Arc::new(AtomicUsize::new(0));
        let peak = Arc::new(AtomicUsize::new(0));
        let jobs = vec![1_u8, 2, 3, 4];

        let results = run_parallel_jobs(jobs, 4, false, {
            let active = Arc::clone(&active);
            let peak = Arc::clone(&peak);
            move |job| {
                let current = active.fetch_add(1, Ordering::SeqCst) + 1;
                let _ = peak.fetch_max(current, Ordering::SeqCst);
                std::thread::sleep(Duration::from_millis(75));
                active.fetch_sub(1, Ordering::SeqCst);
                ((job, current), false)
            }
        });

        assert_eq!(results.len(), 4);
        assert!(peak.load(Ordering::SeqCst) > 1);
    }
}
