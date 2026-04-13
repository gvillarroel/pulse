---
title: Pulse Current Specification
status: active
last_updated: 2026-04-09
source_documents:
  - spec.md
  - commands.md
  - AGENTS.md
---

# Pulse Current Specification

## Summary

`pulse` is a terminal-first Rust analytics system for processing many repositories into reusable, resumable, time-aware datasets. The current product direction is batch repository analysis with durable state, not interactive single-repository inspection.

## Product Goals

- Process many repositories efficiently.
- Prefer deterministic, resumable execution over one-shot speed.
- Reuse prior fetch and analysis results whenever inputs are unchanged.
- Keep provenance explicit for repository inputs, revisions, configuration, and derived metrics.
- Produce machine-readable state that supports later querying, export, and rendering.

## Current Non-Goals

- A web UI before the data model stabilizes.
- Full multi-provider parity in the current implementation.
- Mandatory semantic analysis for all languages.
- Hidden global state outside an explicit operator-provided state directory.

## Current Command Surface

The implemented CLI currently exposes three primary commands:

- `pulse list`
- `pulse run`
- `pulse report`

### `pulse list`

Responsibilities:

- Resolve repositories from YAML config and/or CSV input.
- Normalize repository targets.
- Remove duplicates.
- Emit the explicit target list as text, CSV, or JSON.

Supported flags:

- `--config <yaml>`
- `--input <csv>`
- `--out <path>`
- `--format <text|csv|json>`

### `pulse run`

Responsibilities:

- Resolve repository targets.
- Fetch or update local repository caches.
- Compute repository and file snapshots.
- Apply configured focus patterns to classify analysis depth.
- Optionally compute weekly history aggregates.
- Persist durable reusable state under `--state-dir`.
- Continue processing other repositories when one repository fails, unless `--fail-fast` is set.

Supported flags:

- `--config <yaml>`
- `--input <csv>`
- `--state-dir <path>`
- `--workspace <path>`
- `--concurrency <n>`
- `--json`
- `--progress`
- `--with-history`
- `--history-window <duration>`
- `--focus <pattern>`
- `--focus-file <path>`
- `--fail-fast`

Current behavior constraints:

- `--workspace`, `--concurrency`, and `--history-window` are accepted but not yet fully active as execution controls.
- Resume behavior is driven by persisted fetch state, snapshots, and stage checkpoints in the state directory.
- Repositories with unreadable individual tree entries should degrade partially instead of aborting the repository when possible.
- Repositories with no commits must not crash the pipeline.

### `pulse report`

Responsibilities:

- Read only persisted state from `--state-dir`.
- Build a self-contained HTML report.
- Aggregate and present metrics that were already computed and persisted by `pulse run`.

Supported flags:

- `--config <yaml>`
- `--state-dir <path>`
- `--out <html>`
- `--title <text>`

Current behavior constraints:

- The report must not require refetching or reanalysis.
- The report must not execute repository scans or Git history inspection.
- The generated report should remain functional on Windows, including repositories with many AI-doc candidate paths.

## Repository Intake

### Preferred Input Mode

YAML is the preferred top-level configuration format.

Expected top-level sections:

- `repositories`
- `analysis`
- `focus`
- `report`

### Supported CSV Input

The minimum supported CSV column is:

- `repo`

Repository values may be:

- `owner/name`
- clone URLs

## Analysis Model

The implementation currently centers on these stages:

- `fetch`
- `analyze`
- `history`
- `run`

### Fetch

- Clone missing repositories as managed bare caches.
- Fetch updates for existing caches.
- Persist fetched revision, fetch timestamp, backend, and cache location.

### Analyze

- Enumerate repository tree entries from the fetched revision.
- Read blob contents from Git.
- Compute repository totals and per-file snapshots.
- Detect binary/text, extension, language, size, line count, and analysis depth.
- Skip unreadable tree entries instead of failing the entire repository whenever the rest of the repository can still be analyzed.

### History

- Compute weekly aggregates from Git history.
- Track at least commit counts and active contributor counts by week.
- Treat empty repositories as valid zero-history inputs.

### Run

- Capture overall execution success/failure per repository.
- Persist stage checkpoint state so reruns can update prior failure outcomes.

## Current Metrics

### Repository Snapshots

Current persisted repository snapshot fields:

- `repo_key`
- `revision`
- `total_files`
- `total_bytes`
- `total_lines`
- `generated_at`
- `config_hash`

### File Snapshots

Current persisted file snapshot fields:

- `repo_key`
- `revision`
- `path`
- `language`
- `extension`
- `size_bytes`
- `line_count`
- `is_binary`
- `depth`

### Weekly Evolution

Current persisted weekly evolution fields:

- `repo_key`
- `week_start`
- `commit_count`
- `active_contributors`

## State Directory Contract

The state directory is the durable source of truth for a run series.

Current important subdirectories and files:

- `repos/`
- `db/pulse.sqlite`
- `runs/`
- `logs/`
- `exports/report.html`

The database is expected to persist at least:

- repositories
- repository targets
- fetch state
- stage checkpoints
- repository snapshots
- file snapshots
- weekly evolution
- run records

## Resumability Rules

- Persist progress by repository and stage.
- Reuse previously fetched repositories from the managed cache.
- Reuse previously computed snapshots when the fetched revision and config hash still match.
- Allow reruns after interruption without manual cleanup.
- Keep failure information inspectable in state instead of requiring log scraping.

## Reliability Requirements

- One repository failure must not abort the whole batch by default.
- Failures must be classified clearly enough for later inspection.
- Empty repositories are valid inputs.
- Path encoding and Windows command-length constraints must not break the whole reporting pipeline.
- Documentation and machine-readable outputs must remain in English.

## Current Architecture Direction

The repository should continue converging on a Rust workspace with focused crates for:

- CLI orchestration
- configuration loading
- repository intake
- fetching
- analysis
- storage
- export/rendering

The current codebase already uses this split and should continue refining it instead of introducing one-off workflows outside the core pipeline.

## Planned but Not Yet Implemented

- Direct provider-backed discovery in the CLI.
- Dedicated export subcommands beyond HTML report generation.
- Optional semantic analyzer stages such as `rust-analyzer`.
- Stronger contributor and derived artifact models.
- Fully active bounded-concurrency execution controls.

## Definition of Done

A change is only done when:

- it can be rerun safely
- it leaves inspectable durable state
- it fits the documented CLI and state model
- it degrades safely on partial repository issues where practical
- it includes enough validation to trust the behavior
