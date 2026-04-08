# State Layout

`pulse run` writes durable operator-managed state under `--state-dir`.

Read this after [user-manual.md](../user-manual.md) when you need to inspect persisted state.
Next: [../schemas/state-tables.md](../schemas/state-tables.md)

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

Reserved today:

- `contributors`
- `contributor_snapshots`
- `artifacts`

These tables exist in the schema, but the current V1 workflow does not rely on them for core operator flows.
