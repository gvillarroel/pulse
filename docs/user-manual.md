# User Manual

This manual is the fastest way to start using `pulse` to investigate one repository or a larger set of repositories.

`pulse` is a terminal-first tool. Its current workflow is built around three commands:

- `pulse list`: resolve and normalize the repositories you want to process
- `pulse run`: fetch, analyze, and persist reusable state for those repositories
- `pulse report`: render a self-contained HTML report from persisted state

## 1. What `pulse` does today

The current implementation can:

- read repositories from a CSV file or YAML config
- normalize repository identifiers
- fetch repositories into a reusable bare Git cache
- analyze Git-tracked files at the fetched revision
- calculate file and repository snapshots
- persist checkpoints and results in SQLite
- optionally compute simple weekly history aggregates
- generate a self-contained HTML report from the persisted SQLite state

The current implementation does not yet include:

- provider API discovery from GitHub filters
- full `gix`-based history enrichment
- `gengo`-based language detection
- a dedicated ad hoc query subcommand over persisted state

## 2. Prerequisites

You need:

- Rust and Cargo installed
- `git` installed and available on your `PATH`
- network access if you will fetch remote repositories

Verify the toolchain:

```powershell
cargo --version
git --version
```

## 3. Build and run the CLI

From the repository root:

```powershell
cargo build
```

Run help:

```powershell
cargo run -p pulse-cli -- --help
cargo run -p pulse-cli -- list --help
cargo run -p pulse-cli -- run --help
cargo run -p pulse-cli -- report --help
```

## 4. Input formats

### CSV input

The smallest valid CSV is:

```csv
repo
openai/openai-cookbook
rust-lang/cargo
```

Supported `repo` values:

- `owner/name`
- `https://host/owner/name.git`
- an absolute local path to a bare repository, for example `C:\repos\sample.git`

Optional columns supported by the current implementation:

- `provider`
- `owner`
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

`owner` and `owner_color` remain supported as a compatibility alias for `owner_level_1` and `owner_level_1_color`.

### YAML input

Use YAML when you want repository inputs plus analysis settings and focus rules in one file.

Example:

```yaml
repositories:
  csv: ../csv/repos.sample.csv

analysis:
  with_history: true

focus:
  include:
    - src/**/*.rs
    - Cargo.toml
  exclude:
    - target/**
```

You can also assign repositories to fixed reporting teams through the CSV or YAML input. When a `team` is present, reporting groups by that team instead of falling back to the repository owner.

If you need hierarchical reporting, you can define ordered owner levels in CSV or YAML and then choose the default reporting level in `report.owner_levels`.

Example:

```yaml
report:
  owner_levels:
    default_level: 2
    labels:
      - Domain
      - Portfolio
      - Team
      - Account
```

The repository also includes a sample YAML config under `fixtures/configs/pulse.sample.yaml`.

## 5. First workflow: inspect the target list

Start by validating the repository set before you run analysis.

### From CSV

```powershell
cargo run -p pulse-cli -- list --input .\fixtures\csv\repos.sample.csv
```

JSON output:

```powershell
cargo run -p pulse-cli -- list --input .\fixtures\csv\repos.sample.csv --format json
```

Write the normalized list to disk:

```powershell
cargo run -p pulse-cli -- list --input .\fixtures\csv\repos.sample.csv --format csv --out .\targets.csv
```

### From YAML

```powershell
cargo run -p pulse-cli -- list --config .\fixtures\configs\pulse.sample.yaml --format json
```

## 6. Second workflow: run analysis

Pick a durable state directory. This directory is the reusable local source of truth for later reruns.

Example:

```powershell
cargo run -p pulse-cli -- run --input .\fixtures\csv\repos.sample.csv --state-dir .\state --json
```

With progress output:

```powershell
cargo run -p pulse-cli -- run --input .\fixtures\csv\repos.sample.csv --state-dir .\state --progress
```

With YAML config:

```powershell
cargo run -p pulse-cli -- run --config .\fixtures\configs\pulse.sample.yaml --state-dir .\state --json
```

With history enabled:

```powershell
cargo run -p pulse-cli -- run --input .\fixtures\csv\repos.sample.csv --state-dir .\state --with-history --json
```

## 7. Third workflow: render the report

Once `pulse run` has populated the state directory, generate the HTML report:

```powershell
cargo run -p pulse-cli -- report --state-dir .\state
```

Provide a custom title:

```powershell
cargo run -p pulse-cli -- report --state-dir .\state --title "Team AI Adoption Report"
```

Write the report to a custom location:

```powershell
cargo run -p pulse-cli -- report --state-dir .\state --out .\reports\team.html
```

By default the report is written to:

```text
.\state\exports\report.html
```

## 8. Focus analysis

Use focus rules to mark some files as more important than the baseline inventory.

You can pass focus patterns directly:

```powershell
cargo run -p pulse-cli -- run --input .\repos.csv --state-dir .\state --focus "src/**/*.rs" --focus "Cargo.toml"
```

Or load them from a file:

```powershell
cargo run -p pulse-cli -- run --input .\repos.csv --state-dir .\state --focus-file .\focus.txt
```

Example `focus.txt`:

```text
src/**/*.rs
docs/**/*.md
Cargo.toml
```

At the moment, focused files are persisted with a `focused` depth classification in `file_snapshots`.

## 9. What gets written to `--state-dir`

`pulse` writes a durable state tree:

```text
state/
  repos/
  db/
    pulse.sqlite
  runs/
  logs/
  exports/
    report.html
```

What each part means:

- `repos/`: bare Git caches for fetched repositories
- `db/pulse.sqlite`: checkpoints, repository metadata, snapshots, and weekly aggregates
- `runs/`: reserved for future run-specific artifacts
- `logs/`: reserved for future logs
- `exports/`: generated HTML reports and future exported datasets

More detail: [state-layout/README.md](./state-layout/README.md)

## 10. How to inspect results

There are now two practical ways to inspect results:

- open the generated HTML report for the fastest overview
- query SQLite directly when you need raw tables

If you have `sqlite3` installed:

```powershell
sqlite3 .\state\db\pulse.sqlite ".tables"
sqlite3 .\state\db\pulse.sqlite "select repo_key, fetched_revision, fetched_at from fetch_state;"
sqlite3 .\state\db\pulse.sqlite "select repo_key, revision, total_files, total_lines from repo_snapshots;"
sqlite3 .\state\db\pulse.sqlite "select repo_key, path, language, size_bytes, line_count, depth from file_snapshots limit 20;"
sqlite3 .\state\db\pulse.sqlite "select repo_key, week_start, commit_count, active_contributors from weekly_evolution order by week_start desc limit 20;"
```

Main tables:

- `repositories`
- `repo_stage_checkpoints`
- `fetch_state`
- `repo_snapshots`
- `file_snapshots`
- `weekly_evolution`

Schema reference: [schemas/state-tables.md](./schemas/state-tables.md)

## 11. Recommended beginner workflows

### Investigate one repository quickly

1. Create a CSV with one `repo` row.
2. Run `pulse list` to confirm normalization.
3. Run `pulse run --state-dir .\state --progress`.
4. Run `pulse report --state-dir .\state`.
5. Inspect `repo_snapshots` and `file_snapshots` in SQLite when needed.

### Investigate a team set of repositories

1. Create a CSV with many `owner/name` rows.
2. Add a YAML config with shared focus patterns.
3. Run `pulse run --config ... --state-dir .\state --with-history`.
4. Run `pulse report --state-dir .\state`.
5. Query SQLite for repository-wide comparisons when needed.

### Re-run later without losing work

Use the same `--state-dir` on the next run:

```powershell
cargo run -p pulse-cli -- run --input .\repos.csv --state-dir .\state --with-history --progress
```

The fetch cache and prior snapshots remain available for reuse.

## 12. Common problems

### `git` is not found

Install Git and make sure `git --version` works in the same shell.

### Remote repository cannot be fetched

Check:

- the repository URL
- your network connection
- your Git authentication setup if the repository is private

### YAML focus parsing fails

Make sure `focus.include` and `focus.exclude` are lists:

```yaml
focus:
  include:
    - src/**/*.rs
  exclude:
    - target/**
```

### Local path repositories do not work

Use an absolute path to a local bare repository.

## 13. Current implementation notes

The current V1 slice is intentionally narrow:

- fetch uses the `git` CLI
- static analysis reads Git-tracked files from the fetched revision
- language detection is currently extension-based
- history aggregation is currently a simple weekly `git log` summary

This makes the tool usable now for early repository investigation while leaving room for richer analysis and query features later.

## 14. Worked examples

If you prefer to start from a real run instead of from scratch, inspect the worked examples under the repository `examples/` folder.
