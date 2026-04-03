# ADR 0006: Supersede Persistence Backend for V1

Status: Accepted

Date: 2026-04-02

Supersedes: `0004-persistence-checkpoints.md` for the V1 implementation default

## Context

The accepted persistence ADR selected `libsql`, but the spec, spikes, and first production implementation all converge on a simpler local-operational requirement:

- resumable batch CLI
- local inspectable state under `--state-dir`
- transactional checkpoints and snapshots
- low operational complexity for the first production slice

## Decision

Use SQLite through `rusqlite` as the V1 persistence backend for checkpoints, repository metadata, snapshots, and weekly aggregates.

`libsql` remains a possible future path only if remote/sync requirements become concrete.

## Consequences

- V1 uses a single local SQLite database under `db/pulse.sqlite`
- checkpointing and manual inspection stay simple
- the implementation now matches the strongest evidence in `spec.md` and the spike results

