# pulse Docs

This folder is the self-contained docs set for `pulse`.

## Quick Start

This is the shortest path from zero to a reusable `pulse` run.

### 1. Prerequisites

You need:

- Rust and Cargo
- `git` on your `PATH`
- network access for remote repositories

Verify the toolchain:

```powershell
cargo --version
git --version
```

### 2. Create the repository input

The smallest valid CSV is:

```csv
repo
openai/openai-cookbook
rust-lang/cargo
```

Save it as `repos.csv`.

### 3. Configure the execution

Use YAML when you want repository input and analysis rules in one file.

Example `pulse.yaml`:

```yaml
repositories:
  csv: ./repos.csv

analysis:
  with_history: true

focus:
  include:
    - AGENTS.md
    - CLAUDE.md
    - "**/AGENTS.md"
    - "**/SKILL.md"
    - "**/*.md"
    - src/**/*.rs
  exclude:
    - target/**

report:
  ai_docs:
    include:
      - AGENTS.md
      - "**/AGENTS.md"
      - "**/SKILL.md"
      - "**/*agent*.md"
      - "**/*skill*.md"
    exclude:
      - archive/**
```

What this config controls:

- `repositories.csv`: the explicit repository list to process
- `analysis.with_history`: whether weekly history is computed
- `focus.include` and `focus.exclude`: which files are marked as focused during analysis
- `report.ai_docs`: which files are treated as AI-doc inputs during report enrichment

### 4. Inspect the normalized target list

Validate what `pulse` will process:

```powershell
cargo run -p pulse-cli -- list --config .\pulse.yaml --format json
```

### 5. Run the analysis

Pick a durable state directory and run:

```powershell
cargo run -p pulse-cli -- run --config .\pulse.yaml --state-dir .\state\demo --progress --json
```

What this writes:

- bare repository caches under `.\state\demo\repos\`
- SQLite state under `.\state\demo\db\pulse.sqlite`
- stage checkpoints and reusable snapshots for later reruns

### 6. Render the report

Generate the self-contained HTML report:

```powershell
cargo run -p pulse-cli -- report --state-dir .\state\demo --title "Demo Pulse Report"
```

Default output:

```text
.\state\demo\exports\report.html
```

### 7. Re-run safely later

Use the same state directory on the next execution:

```powershell
cargo run -p pulse-cli -- run --config .\pulse.yaml --state-dir .\state\demo --progress --json
```

`pulse` reuses the existing fetch cache and persisted state instead of starting from scratch.

## Expected Results

See [examples/expected-results.md](./examples/expected-results.md) for a concrete example of:

- the shape of a successful run summary
- the generated state directory layout
- the SQLite tables populated by the run
- the expected HTML report artifact

## Read Paths

First run:

1. [user-manual.md](./user-manual.md)
2. [examples/expected-results.md](./examples/expected-results.md)
3. [state-layout/README.md](./state-layout/README.md)
4. [schemas/state-tables.md](./schemas/state-tables.md)

Implementation context:

1. [architecture/repository-layout.md](./architecture/repository-layout.md)
2. [state-layout/README.md](./state-layout/README.md)
3. [schemas/state-tables.md](./schemas/state-tables.md)

## Doc Map

| File | Purpose |
| --- | --- |
| `user-manual.md` | operator guide for `list`, `run`, and `report` |
| `examples/expected-results.md` | concrete example of expected outputs after a run |
| `architecture/repository-layout.md` | repo structure and crate roles |
| `state-layout/README.md` | durable `--state-dir` layout |
| `schemas/state-tables.md` | SQLite table summary |

## Rules

- docs navigation works from inside `docs/`
- avoid machine-specific absolute links
- examples in this folder should remain readable without opening files outside `docs/`
