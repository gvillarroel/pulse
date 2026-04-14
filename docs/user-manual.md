# User Manual

This manual is for someone who wants to use `pulse` without first reverse-engineering the repository.

It explains:

- what `pulse` does
- how to think about the workflow
- how to prepare inputs
- how to run the three main commands
- what gets written to disk
- how to inspect and troubleshoot the results

## 1. What `pulse` Is

`pulse` is a repository analytics pipeline with a CLI front end.

You give it an explicit repository set. It fetches those repositories, analyzes them, stores reusable facts in a state directory, and then renders a report from that stored state.

That makes it different from a simple reporting script:

- it is meant to be rerun
- it keeps durable state
- it separates execution from reporting
- it is designed for batches of repositories

## 2. The Three Commands

The current user-facing workflow is intentionally small:

- `pulse list`
- `pulse run`
- `pulse report`

Think of them this way:

| Command | Purpose |
| --- | --- |
| `list` | resolve and inspect the repository targets |
| `run` | fetch, analyze, and persist reusable state |
| `report` | render HTML from previously persisted state |

## 3. The Fastest First Run

If you want the shortest path from zero to a report:

1. create a `repos.csv`
2. run `pulse run`
3. run `pulse report`

Minimal CSV:

```csv
repo
openai/openai-cookbook
rust-lang/cargo
```

Run:

```powershell
cargo run -p pulse-cli -- run --input .\repos.csv --state-dir .\state\demo --progress --json
```

Render:

```powershell
cargo run -p pulse-cli -- report --state-dir .\state\demo --title "Demo Pulse Report"
```

## 4. Before You Run Anything

### Prerequisites

You need:

- Rust and Cargo
- `git` on your `PATH`
- network access for remote repositories

Verify:

```powershell
cargo --version
git --version
```

### Build The CLI

```powershell
cargo build -p pulse-cli
```

Help commands:

```powershell
cargo run -p pulse-cli -- --help
cargo run -p pulse-cli -- list --help
cargo run -p pulse-cli -- run --help
cargo run -p pulse-cli -- report --help
```

## 5. Input Files

`pulse` accepts inputs in two main forms:

- CSV when you want the simplest explicit list
- YAML when you want repository input plus analysis and report settings in one place

### CSV Input

The required column is:

- `repo`

Accepted `repo` values:

- `owner/name`
- `https://host/owner/name.git`
- an absolute local path to a bare repository

Optional metadata columns:

- `provider`
- `owner`
- `owner_color`
- `team`
- `team_color`
- `owner_level_1`
- `owner_level_1_color`
- `owner_level_2`
- `owner_level_2_color`
- `owner_level_3`
- `owner_level_3_color`
- `owner_level_N`
- `owner_level_N_color`
- `name`
- `url`
- `default_branch`
- `tags`
- `active`

`owner` and `owner_color` are still supported as compatibility aliases for the first owner level.

### YAML Input

Use YAML when you want one file to describe:

- where repositories come from
- whether history analysis runs
- which files should be considered focused
- which file patterns should drive AI-document reporting
- which owner level should be used by default in the HTML report

Example:

```yaml
repositories:
  csv: ./repos.csv

analysis:
  with_history: true

focus:
  include:
    - AGENTS.md
    - "**/AGENTS.md"
    - src/**/*.rs
  exclude:
    - target/**

report:
  owner_levels:
    default_level: 2
    labels:
      - Domain
      - Portfolio
      - Team
      - Account
  ai_docs:
    include:
      - AGENTS.md
      - "**/AGENTS.md"
      - "**/*agent*.md"
```

## 6. Understanding Grouping Metadata

`pulse` can carry reporting structure inside the repository input.

There are two main grouping approaches:

### Team-Based Grouping

Use:

- `team`
- `team_color`

This is useful when you already have one reporting team per repository.

### Hierarchical Owner Levels

Use:

- `owner_level_1`
- `owner_level_2`
- `owner_level_3`
- `owner_level_N`

This is useful when you want multiple reporting cuts over the same repository set, for example:

- domain
- portfolio
- team
- account

The final report can then switch between those levels without re-analyzing the repositories.

## 7. Command 1: `pulse list`

Use `list` when you want to inspect what `pulse` will process before you spend time fetching and analyzing repositories.

### From CSV

```powershell
cargo run -p pulse-cli -- list --input .\repos.csv
```

JSON output:

```powershell
cargo run -p pulse-cli -- list --input .\repos.csv --format json
```

Write the normalized targets to disk:

```powershell
cargo run -p pulse-cli -- list --input .\repos.csv --format csv --out .\targets.csv
```

### From YAML

```powershell
cargo run -p pulse-cli -- list --config .\pulse.yaml --format json
```

Use this command to verify:

- repository normalization
- deduplication
- metadata such as owner levels or team labels

## 8. Command 2: `pulse run`

`run` is the main pipeline command.

It does all of the expensive work:

- reads targets
- fetches repositories
- analyzes the current revision
- optionally computes weekly history
- stores checkpoints and results in SQLite

### Minimal run

```powershell
cargo run -p pulse-cli -- run --input .\repos.csv --state-dir .\state\demo --json
```

### Run with progress

```powershell
cargo run -p pulse-cli -- run --input .\repos.csv --state-dir .\state\demo --progress
```

### Run from YAML

```powershell
cargo run -p pulse-cli -- run --config .\pulse.yaml --state-dir .\state\demo --json
```

### Run with history

```powershell
cargo run -p pulse-cli -- run --config .\pulse.yaml --state-dir .\state\demo --with-history --progress
```

### What `run` leaves behind

After a successful run, the state directory contains:

- reusable Git mirrors
- SQLite tables with snapshots and checkpoints
- enough information to rerun safely later

## 9. Command 3: `pulse report`

`report` reads the existing state directory and generates a self-contained HTML file.

It does not fetch repositories again.

### Basic report

```powershell
cargo run -p pulse-cli -- report --state-dir .\state\demo
```

### Custom title

```powershell
cargo run -p pulse-cli -- report --state-dir .\state\demo --title "My Team Report"
```

### Custom output path

```powershell
cargo run -p pulse-cli -- report --state-dir .\state\demo --out .\reports\demo.html
```

### Report with YAML-backed presentation settings

```powershell
cargo run -p pulse-cli -- report --config .\pulse.yaml --state-dir .\state\demo
```

By default the report is written to:

```text
.\state\demo\exports\report.html
```

## 10. Focus Rules

Focus rules let you mark certain paths as more important than the general baseline inventory.

This is useful when you want to emphasize:

- AI-related docs
- source directories
- specific config files
- known high-value areas of a repository

### Pass focus rules directly

```powershell
cargo run -p pulse-cli -- run --input .\repos.csv --state-dir .\state --focus "src/**/*.rs" --focus "Cargo.toml"
```

### Load focus rules from a file

```powershell
cargo run -p pulse-cli -- run --input .\repos.csv --state-dir .\state --focus-file .\focus.txt
```

Example `focus.txt`:

```text
src/**/*.rs
docs/**/*.md
Cargo.toml
```

## 11. What Gets Written To `--state-dir`

Typical layout:

```text
state/
  demo/
    repos/
    db/
      pulse.sqlite
    runs/
    logs/
    exports/
      report.html
```

What each part is for:

- `repos/`: reusable Git mirrors and fetch state
- `db/pulse.sqlite`: checkpoints, repository metadata, snapshots, weekly history, and reporting datasets
- `runs/`: reserved for run-oriented artifacts
- `logs/`: reserved for execution logs
- `exports/`: HTML reports and future derived outputs

Detailed reference: [state-layout/README.md](./state-layout/README.md)

## 12. How To Inspect Results

There are two practical inspection paths:

- open the HTML report for a quick overview
- query SQLite directly for raw facts

If you have `sqlite3` installed:

```powershell
sqlite3 .\state\demo\db\pulse.sqlite ".tables"
sqlite3 .\state\demo\db\pulse.sqlite "select repo_key, fetched_revision, fetched_at from fetch_state;"
sqlite3 .\state\demo\db\pulse.sqlite "select repo_key, revision, total_files, total_lines from repo_snapshots;"
sqlite3 .\state\demo\db\pulse.sqlite "select repo_key, path, language, size_bytes, line_count, depth from file_snapshots limit 20;"
sqlite3 .\state\demo\db\pulse.sqlite "select repo_key, week_start, commit_count, active_contributors from weekly_evolution order by week_start desc limit 20;"
```

Main tables worth knowing first:

- `repositories`
- `repository_owner_levels`
- `repo_stage_checkpoints`
- `fetch_state`
- `repo_snapshots`
- `file_snapshots`
- `weekly_evolution`

Schema reference: [schemas/state-tables.md](./schemas/state-tables.md)

## 13. Safe Reruns

A feature is not really done in `pulse` unless it survives reruns.

The intended operator pattern is:

1. keep using the same `--state-dir`
2. rerun `pulse run`
3. regenerate the report

Example:

```powershell
cargo run -p pulse-cli -- run --config .\pulse.yaml --state-dir .\state\demo --progress --json
cargo run -p pulse-cli -- report --config .\pulse.yaml --state-dir .\state\demo
```

This is how you build a time-aware dataset instead of a disposable one.

## 14. Common Beginner Workflows

### Investigate one repository

1. create a CSV with one `repo`
2. run `pulse list`
3. run `pulse run`
4. run `pulse report`

### Investigate a team portfolio

1. prepare a CSV with many repositories
2. add `team` or `owner_level_*` metadata
3. write a YAML config with focus and report settings
4. run `pulse run --with-history`
5. render the report

### Investigate a hierarchy

1. add `owner_level_1`, `owner_level_2`, and deeper levels to the CSV
2. define `report.owner_levels.default_level`
3. run `pulse`
4. use the level switcher in the HTML report

## 15. Common Problems

### `git` is not found

Install Git and confirm `git --version` works in the same shell.

### Remote fetch fails

Check:

- the repository URL
- network access
- authentication for private repositories

### YAML parsing fails

Make sure YAML lists are really lists:

```yaml
focus:
  include:
    - src/**/*.rs
  exclude:
    - target/**
```

### Local path repositories do not work

Use an absolute path to a local bare Git repository.

### The report shows less than expected

Remember that the report reads persisted state. If the analysis inputs or config changed, rerun `pulse run` before rerendering `pulse report`.

## 16. Where To Go Next

- [architecture/pipeline-overview.md](./architecture/pipeline-overview.md)
- [architecture/repository-layout.md](./architecture/repository-layout.md)
- [examples/expected-results.md](./examples/expected-results.md)
- [../examples/gvillarroel-all-repos/README.md](../examples/gvillarroel-all-repos/README.md)
