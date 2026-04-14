# pulse

`pulse` is a terminal-first analytics tool for collecting engineering signals from many repositories, storing them in reusable local state, and turning that state into reports.

The project is intentionally designed for batch work:

- analyze many repositories, not just one
- reuse prior results instead of recomputing everything
- survive interruptions and reruns
- keep state inspectable on disk
- produce reports from persisted state, not from ad hoc in-memory analysis

## What Problem `pulse` Solves

Most repository analysis scripts work once, on one machine, for one moment in time.

`pulse` is trying to solve a different problem:

- you want to analyze dozens or hundreds of repositories
- you want the run to be repeatable next week
- you want durable checkpoints and reusable fetched Git data
- you want a report generated from stored facts instead of from a one-off script

Today the project already supports a practical early workflow:

1. define a repository list in CSV or YAML
2. run `pulse` to fetch and analyze those repositories into a state directory
3. render a self-contained HTML report from that state directory

## The Mental Model

The easiest way to understand `pulse` is to think of it as a small pipeline with durable storage in the middle.

```text
input list/config -> fetch -> analyze -> store -> report
```

Each step matters:

- `input`: which repositories should be processed
- `fetch`: clone or update reusable local Git mirrors
- `analyze`: compute repository, file, and history facts
- `store`: persist results and checkpoints in SQLite plus managed directories
- `report`: read stored state and generate an HTML artifact

The important design choice is that `report` does not re-scan repositories. It reads the persisted state created by `run`.

## Current CLI Surface

The current implementation is intentionally small:

- `pulse list`
- `pulse run`
- `pulse report`

These commands map to three operator tasks:

- confirm the repository set
- execute the pipeline into a state directory
- render a report from saved state

## Quick Start

### 1. Prerequisites

You need:

- Rust and Cargo
- `git` on your `PATH`
- network access if you are fetching remote repositories

Verify the toolchain:

```powershell
cargo --version
git --version
```

### 2. Build the CLI

```powershell
cargo build -p pulse-cli
```

Run help:

```powershell
cargo run -p pulse-cli -- --help
```

### 3. Create a minimal input CSV

```csv
repo
openai/openai-cookbook
rust-lang/cargo
```

Save it as `repos.csv`.

### 4. Run a first analysis

```powershell
cargo run -p pulse-cli -- run --input .\repos.csv --state-dir .\state\demo --progress --json
```

### 5. Render the HTML report

```powershell
cargo run -p pulse-cli -- report --state-dir .\state\demo --title "Demo Pulse Report"
```

By default the report is written to:

```text
.\state\demo\exports\report.html
```

## Beginner Workflow

If you are new to the project, this is the best reading order:

1. This file for the product overview
2. [docs/README.md](./docs/README.md) for the documentation map
3. [docs/user-manual.md](./docs/user-manual.md) for hands-on usage
4. [docs/architecture/pipeline-overview.md](./docs/architecture/pipeline-overview.md) for the system view
5. [docs/architecture/repository-layout.md](./docs/architecture/repository-layout.md) for codebase orientation

## Architecture At A Glance

`pulse` is a Rust workspace. The crates correspond closely to the pipeline stages.

| Crate | Responsibility |
| --- | --- |
| `pulse-cli` | command entrypoint and top-level workflow orchestration |
| `pulse-core` | shared domain types, result types, and state layout |
| `pulse-config` | YAML configuration loading |
| `pulse-input` | CSV parsing and repository target normalization |
| `pulse-fetch` | Git fetch and mirror management |
| `pulse-git` | lower-level Git helpers |
| `pulse-analyze` | snapshot analysis for repositories and files |
| `pulse-store` | SQLite persistence, checkpoints, and report datasets |
| `pulse-export` | CSV/JSON export and HTML report generation |

The high-level runtime flow is:

```text
pulse-cli
  -> pulse-config / pulse-input
  -> pulse-fetch
  -> pulse-analyze
  -> pulse-store
  -> pulse-export
```

## State Directory

The state directory is the center of the system. It is what makes reruns safe and reports reproducible.

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

Key idea:

- `repos/` stores reusable fetched Git data
- `db/pulse.sqlite` stores checkpoints and computed facts
- `exports/` stores rendered artifacts

More detail: [docs/state-layout/README.md](./docs/state-layout/README.md)

## Inputs And Grouping

`pulse` accepts repository inputs from CSV or YAML.

Useful input capabilities already supported today:

- basic repository list via `repo`
- optional `team` and `team_color`
- hierarchical owner grouping via `owner_level_1`, `owner_level_2`, `owner_level_N`
- YAML-based focus patterns
- YAML-based report options

This means you can carry reporting metadata with the repository list instead of hardcoding it into the report layer.

## What The Project Already Does Well

- explicit CSV and YAML intake
- reusable Git fetch cache
- durable SQLite-backed state
- resumable reruns
- focused-file classification
- optional weekly history aggregation
- self-contained HTML report generation

## What Is Still Early

The current implementation is still a narrow V1 slice.

Notably deferred or still simple today:

- provider-backed discovery from inside the CLI
- richer semantic analysis
- advanced query/export commands beyond the HTML report
- production-grade multi-provider support

## Repository Guide

Important files and folders:

- [spec.md](./spec.md): product-level specification
- [commands.md](./commands.md): intended CLI contract
- [docs/](./docs/README.md): operator and implementation documentation
- [examples/](./examples/README.md): worked runs
- [spikes/](./spikes): experiments and technical investigations
- [.specs/adr/](./.specs/adr): architecture decisions

## Recommended Next Reads

- [docs/user-manual.md](./docs/user-manual.md)
- [docs/architecture/pipeline-overview.md](./docs/architecture/pipeline-overview.md)
- [docs/architecture/repository-layout.md](./docs/architecture/repository-layout.md)
- [examples/gvillarroel-all-repos/README.md](./examples/gvillarroel-all-repos/README.md)
