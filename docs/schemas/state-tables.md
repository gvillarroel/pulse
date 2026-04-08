# SQLite Tables

Read this after [../state-layout/README.md](../state-layout/README.md) when you need table-level detail.
See also: [../user-manual.md](../user-manual.md)

## Operational tables

- `runs`: run boundaries and timestamps
- `repo_stage_checkpoints`: resumable per-stage status
- `fetch_state`: fetched revision, backend, and cache location

## Repository data tables

- `repositories`: canonical normalized repository identity
- `repository_targets`: tag metadata captured from input
- `repo_snapshots`: repository-wide static metrics keyed by revision and config hash
- `file_snapshots`: file-level metrics keyed by revision and path

## Evolution tables

- `contributors`
- `contributor_snapshots`
- `weekly_evolution`
- `artifacts`

Status note:

- `weekly_evolution` is active when history is enabled
- `contributors`, `contributor_snapshots`, and `artifacts` are reserved for future slices and should not be treated as core current output
