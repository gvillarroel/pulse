# Expected Results

This document shows what a normal `pulse` execution is expected to produce.

## Example Run

Example command:

```powershell
cargo run -p pulse-cli -- run --config .\pulse.yaml --state-dir .\state\demo --progress --json
```

Example summary output:

```json
{
  "run_id": 6,
  "processed": 46,
  "failed": 0
}
```

Interpretation:

- `run_id`: the persisted run record created in SQLite
- `processed`: repositories that completed analysis
- `failed`: repositories that still ended the run in failed state

## Expected State Directory

After a successful run, `--state-dir` should look like this:

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

Meaning:

- `repos/`: managed bare Git caches
- `db/pulse.sqlite`: persistent metadata, checkpoints, snapshots, and weekly history
- `runs/`: reserved for run-scoped artifacts
- `logs/`: reserved for future logs and diagnostics
- `exports/report.html`: generated HTML report

## Expected SQLite Tables

The current implementation populates these main tables:

- `repositories`
- `repository_targets`
- `fetch_state`
- `repo_stage_checkpoints`
- `repo_snapshots`
- `file_snapshots`
- `weekly_evolution`
- `runs`

Tables currently present but not yet populated in the normal path:

- `contributors`
- `contributor_snapshots`
- `artifacts`

## Expected Metrics

At minimum, a successful run should populate:

### Repository metrics

- repository identity
- fetched revision
- total files
- total bytes
- total lines

### File metrics

- file path
- language
- extension
- size in bytes
- line count
- binary or text classification
- analysis depth (`baseline` or `focused`)

### Weekly history metrics

- week start
- commit count
- active contributors

## Expected Report Artifact

After running:

```powershell
cargo run -p pulse-cli -- report --state-dir .\state\demo --title "Demo Pulse Report"
```

The expected output artifact is:

```text
.\state\demo\exports\report.html
```

The current report is expected to include:

- summary totals
- repository overview
- language and extension breakdowns
- weekly history charts
- stage status counts
- failure list
- AI-doc sections when matching files are present

## Example Inspection Queries

If `sqlite3` is available, these are useful sanity checks:

```powershell
sqlite3 .\state\demo\db\pulse.sqlite ".tables"
sqlite3 .\state\demo\db\pulse.sqlite "select count(*) from repositories;"
sqlite3 .\state\demo\db\pulse.sqlite "select count(*) from fetch_state;"
sqlite3 .\state\demo\db\pulse.sqlite "select count(*) from repo_snapshots;"
sqlite3 .\state\demo\db\pulse.sqlite "select count(*) from file_snapshots;"
sqlite3 .\state\demo\db\pulse.sqlite "select count(*) from weekly_evolution;"
```

## Worked Repository Example

The repository includes a fuller worked example outside `docs/` under `examples/gvillarroel-all-repos/`, but this file is the in-doc reference for what a healthy result set should look like.
