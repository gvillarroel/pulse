---
id: 0002
title: Explicit State Directory Backed By SQLite
status: accepted
date: 2026-04-09
deciders:
  - contributors
consulted:
  - AGENTS.md
  - spec.md
  - commands.md
---

# ADR 0002: Explicit State Directory Backed By SQLite

## Context

The system must support resumability, reruns over months, inspectable failures, and reuse of prior results. Hidden process memory or implicit global state would make restart safety and operator trust worse.

## Decision

All durable run state is stored under an explicit operator-provided state directory, with SQLite used as the primary metadata and checkpoint store.

The state directory currently holds:

- repository caches under `repos/`
- database state under `db/pulse.sqlite`
- generated artifacts under `exports/`

SQLite stores repositories, targets, fetch state, stage checkpoints, run records, snapshots, and weekly aggregates.

## Consequences

Positive:

- Durable resumability and inspectable local state.
- Easy local portability and backup.
- Simple query surface for later exports and reporting.
- No dependency on a separate database service for core workflows.

Negative:

- Concurrent write patterns must remain carefully controlled.
- Large artifact export needs may later require complementary file formats.

## Alternatives Considered

- Hidden state outside the project-managed directory.
- A server database as a mandatory dependency.
- Flat files only, without a relational metadata store.
