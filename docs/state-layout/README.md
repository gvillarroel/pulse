# State Layout

`pulse run` writes durable operator-managed state under `--state-dir`.

## Directories

- `repos/`: persistent bare Git caches
- `db/pulse.sqlite`: checkpoints, repository metadata, snapshots, and weekly aggregates
- `runs/`: reserved for future per-run artifacts
- `logs/`: reserved for future per-repository logs
- `exports/`: generated reports and future materialized CSV/JSON outputs

Current default export:

- `exports/report.html`: self-contained HTML report emitted by `pulse report`

## Database Tables

- `runs`
- `repositories`
- `repository_targets`
- `repo_stage_checkpoints`
- `fetch_state`
- `repo_snapshots`
- `file_snapshots`
- `contributors`
- `contributor_snapshots`
- `weekly_evolution`
- `artifacts`
