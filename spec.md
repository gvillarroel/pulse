# pulse specification

## 1. Summary

`pulse` is a Rust-based CLI platform for collecting, computing, and querying engineering quality and evolution metrics across many source repositories.

The product analyzes explicit repository sets and discovered repository sets, stores reusable intermediate state, and produces historical signals about codebases over time.

## 2. Goals

The system must:

- process many repositories efficiently
- support configuration-driven batch input, with YAML preferred and CSV supported
- fetch repositories concurrently into managed temporary or cache directories
- compute repository metadata, file metrics, contributor metrics, and historical evolution
- resume interrupted runs without losing useful work
- reuse prior outputs when repositories have not materially changed
- support path-targeting rules so selected files or folders can receive deeper analysis
- evolve toward charts and time-series visualization outputs

## 3. Primary Use Cases

### 3.1 Analyze a fixed list of repositories

An operator provides a CSV with repository identifiers. `pulse` fetches them, analyzes them, stores results, and emits structured outputs.

### 3.2 Re-run analysis incrementally

An operator reruns analysis days or weeks later. `pulse` updates only what changed, resumes incomplete work, and extends the time series.

### 3.3 Discover repositories matching filters

An operator requests repositories changed in the last N months, owned by a specific organization or account, then uses that result set as input for analysis.

### 3.4 Observe evolution

An operator inspects how repository activity, file sizes, language composition, and file changes evolved week by week.

## 4. Scope

### 4.1 In Scope for Early Versions

- GitHub-first repository identification and discovery
- YAML-based repository configuration, with CSV paths or direct CSV intake supported
- local repository fetch/update
- repository metadata extraction
- contributor extraction from Git history and provider metadata where available
- language breakdown using repository contents and/or provider metadata
- file inventory extraction
- file size and line count snapshots
- file modification timestamps from Git history
- targeted deep analysis for configured file and folder patterns
- weekly historical aggregation
- resumable execution with checkpoints
- CLI querying and export

### 4.2 Deferred or Optional

- multi-provider parity beyond GitHub
- full semantic metrics for all languages
- web UI
- real-time dashboards
- distributed execution across machines

## 5. Repository Inputs

The system should support two main repository source modes, both aligned with a minimal CLI.

### 5.1 Preferred Configuration Intake

The preferred input should be a YAML configuration file.

That configuration should be able to define:

- repository sources
- provider filters
- a path to a CSV file, if CSV is being used as the repository source
- analysis options
- focused path patterns for deeper analysis

Suggested top-level sections:

- `repositories`
- `discovery`
- `analysis`
- `focus`

The `focus` section should support path-matching rules such as:

- all files under a specific folder
- files matching an extension under a specific folder
- files matching an extension anywhere in the repository
- exact file paths
- glob-style include patterns
- optional exclude patterns

Illustrative examples:

- `src/data/**`
- `src/data/**/*.csv`
- `**/*.csv`
- `Cargo.toml`

### 5.2 Explicit CSV Intake

CSV remains a supported input mode. A minimal CSV should support:

- `repo`

Recommended optional columns:

- `provider`
- `owner`
- `name`
- `url`
- `default_branch`
- `tags`
- `active`

Example values for `repo`:

- `owner/name`
- full clone URL

### 5.3 Discovery Intake

The system queries provider APIs or previously stored catalog data using filters such as:

- owner or organization
- provider
- pushed within the last N months
- created within the last N months
- topic or label if supported
- language if supported
- archived or active status

Discovery results should be emittable as an explicit list, typically a CSV consumed later by the processing command, or be expressible directly in the YAML configuration.

## 6. Metrics Model

Metrics are grouped by level.

### 6.1 Repository-Level Metrics

- repository identity
- provider, owner, name, URL
- default branch
- created / updated / pushed timestamps
- stars, forks, watchers if provider metadata is available
- open issues / archived / visibility status if provider metadata is available
- total commits in analyzed scope
- total contributors
- active contributors over recent windows
- primary languages and distribution
- total file count
- total directory count
- total bytes
- total lines

### 6.2 Contributor Metrics

- contributor identity and normalization key
- commit count
- files touched
- lines added / deleted when available
- first contribution date
- last contribution date
- active weeks

### 6.3 File-Level Metrics

- repository key
- revision or snapshot key
- file path
- file extension
- detected language
- file size in bytes
- line count
- binary/text classification
- first seen date
- last modified date
- commit count touching file
- last known author
- analysis depth classification, for example baseline vs focused

### 6.4 Evolution Metrics

Time bucket target for V1: weekly snapshots.

Per bucket:

- commit count
- active contributors
- files added / modified / deleted
- per-language bytes and lines
- per-file size evolution
- per-file line-count evolution
- per-directory growth
- repository churn indicators

## 7. Analysis Strategy

The pipeline should be staged internally, even if exposed through very few CLI commands.

### 7.1 Stage A: Intake and Planning

- read YAML config, CSV input, or discovery filters
- normalize repository identifiers
- deduplicate targets
- resolve focused path patterns
- compare targets against existing state in the chosen state directory
- build an execution plan

### 7.2 Stage B: Fetch

- clone missing repositories
- fetch updates for existing repositories
- support mirror or bare clone mode if better for history-heavy analysis
- track fetched revision and fetch timestamp

### 7.3 Stage C: Static Snapshot Analysis

- inspect repository tree at target revision
- count files, directories, sizes, and lines
- detect languages
- apply baseline analysis to the full repository
- apply deeper analysis to configured focus patterns
- produce repository snapshot metrics

### 7.4 Stage D: Git History Analysis

- walk commits and file histories
- build weekly aggregates
- derive file evolution and contributor activity
- persist progress incrementally for long histories

### 7.5 Stage E: Semantic Enrichment

- run optional analyzers for supported ecosystems
- for Rust repositories, optionally integrate `rust-analyzer`
- treat this stage as additive enrichment with independent cache keys

### 7.6 Stage F: Persist Reusable Results

- persist outputs in a structure that future runs can reuse
- keep stored datasets usable for later reporting work
- avoid making reporting commands a prerequisite for V1

## 8. Fetching and Concurrency

Concurrency is a core requirement.

The system should:

- use bounded worker pools
- isolate per-repository work units
- allow separate concurrency settings for fetch and analysis
- store repositories in a managed cache or temp root
- clean temporary work safely without invalidating reusable mirrors

Recommended design:

- long-lived local repository cache for reused Git data
- ephemeral execution workspaces only when needed
- job queue with checkpointed completion per stage

## 9. Resumability and Incrementality

This is mandatory.

The system must:

- persist execution state for each repository and stage
- mark stages as pending, running, completed, failed, or stale
- record the exact revision range analyzed
- skip recomputation when cache keys still match
- resume from the last durable checkpoint after interruption

Likely invalidation keys:

- repository HEAD or revision range
- tool version
- analysis configuration hash
- focus pattern configuration hash
- analyzer version such as `rust-analyzer`

## 10. Storage Model

Implementation may evolve, but the logical storage model should live under an explicit operator-provided state directory.

### 10.1 Local Repository Cache

Cloned or mirrored Git repositories managed by `pulse`.

### 10.2 Execution State

Runs, jobs, stage checkpoints, failure records, and retry metadata.

### 10.3 Snapshot Store

Repository-level and file-level metrics tied to a revision and timestamp.

This store should preserve enough context to distinguish:

- baseline metrics computed for all files
- deeper metrics computed only for matched focus patterns

### 10.4 History Store

Weekly aggregates and file evolution records.

### 10.5 Optional Derived Artifacts

Files that later reporting workflows may consume.

For initial implementation, SQLite is a strong default for metadata and checkpoints inside the chosen state directory. Heavy artifacts may live as files on disk under that same directory.

## 11. Rust Analyzer Integration

Rust Analyzer should be treated as optional semantic enrichment.

Potential value:

- crate graph context
- module structure insights
- Rust-specific symbol or complexity metrics if feasible

Constraints:

- must not block baseline repository analysis
- must be skippable per run
- should be enabled only for Rust repositories
- requires strong caching because semantic analysis is expensive

## 12. Output Requirements

The system should produce:

- human-readable terminal summaries
- machine-readable JSON
- CSV-compatible persisted datasets
- datasets ready for later reporting or plotting

The persisted outputs should also make it possible to understand which files were analyzed under focused rules and which were analyzed only at the baseline level.

Future chart generation may be built into the CLI, but it is not part of the current command design focus.

## 13. Reliability Requirements

- one repository failure must not abort the whole batch by default
- partial progress must be durable
- retries must be possible without manual cleanup
- the tool must distinguish invalid input from transient provider or network failures

## 14. Performance Requirements

Early priorities:

- minimize redundant cloning and fetching
- reuse stored Git state across runs
- parallelize across repositories
- avoid recomputing unchanged snapshots
- stream large history traversals instead of loading everything into memory

## 15. Suggested Initial Architecture

Recommended Rust workspace structure:

- `crates/pulse-cli`
- `crates/pulse-core`
- `crates/pulse-git`
- `crates/pulse-storage`
- `crates/pulse-discovery`
- `crates/pulse-analyzer`
- `crates/pulse-export`

## 16. Release Phases

### Phase 1

- basic Rust CLI
- CSV intake
- repository fetch/update
- repository metadata snapshot
- file inventory with size and line count
- SQLite-backed checkpoints

### Phase 2

- contributor extraction
- weekly Git history aggregation
- file evolution series
- stronger state reuse and resumability behaviors

### Phase 3

- repository discovery from provider APIs
- delta discovery since previous runs
- persisted derived datasets for later reporting

### Phase 4

- optional semantic analyzers such as Rust Analyzer
- deeper quality heuristics
- richer comparative reporting

## 17. Open Questions

These should be resolved during implementation planning:

- whether `gitoxide` is sufficient or whether shelling out to `git` is more robust initially
- which language detection strategy should be authoritative
- how deep file history should go by default for very large repositories
- whether weekly snapshots are always computed from full history or can be windowed
- which semantic Rust metrics are worth the added complexity

## 18. Acceptance Criteria for V1

V1 is successful when the tool can:

- read a YAML configuration file or a CSV of repositories
- fetch or update them efficiently
- analyze them concurrently
- persist durable checkpoints
- resume after interruption
- output repository and file metrics including size, line count, and modification history
- support focused analysis rules for selected files and folders using configurable patterns
- accept either an explicit CSV or direct filters as input
- update a chosen state directory that can be reused next week without manual reconstruction
