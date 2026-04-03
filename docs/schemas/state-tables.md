# SQLite Tables

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

