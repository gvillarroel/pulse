---
id: 0004
title: Stage-Oriented Pipeline With Per-Repository Checkpoints
status: accepted
date: 2026-04-09
deciders:
  - contributors
consulted:
  - AGENTS.md
  - spec.md
  - commands.md
---

# ADR 0004: Stage-Oriented Pipeline With Per-Repository Checkpoints

## Context

Long-running repository batch analysis can fail at many points: fetch, tree traversal, file reads, Git history, or downstream rendering. The system must resume safely and avoid recomputing successful work unnecessarily.

## Decision

The pipeline is modeled as stage-oriented work with per-repository checkpoints. The current stage model includes:

- `fetch`
- `analyze`
- `history`
- `run`

Each stage writes status into durable checkpoint records so reruns can inspect and update prior outcomes.

## Consequences

Positive:

- Better restart safety.
- Clear operator visibility into partial progress and failures.
- Simpler invalidation and recomputation boundaries.

Negative:

- Stage semantics must stay consistent across future changes.
- Summary logic must account for checkpoint replacement and rerun behavior.

## Alternatives Considered

- Treat each run as an opaque all-or-nothing batch.
- Persist only final results without per-stage checkpoints.
- Keep failure state only in logs.
