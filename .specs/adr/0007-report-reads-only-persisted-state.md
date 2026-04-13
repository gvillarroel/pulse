---
id: 0007
title: Report Reads Only Persisted State
status: accepted
date: 2026-04-10
deciders:
  - contributors
consulted:
  - AGENTS.md
  - spec.md
  - commands.md
  - .specs/specs.md
---

# ADR 0007: Report Reads Only Persisted State

## Context

`pulse` is designed as a resumable batch analytics system with explicit durable state. Allowing `pulse report` to re-scan repositories, read Git history again, or apply fresh heuristics during rendering weakens that architecture in several ways:

- report output becomes dependent on repository access instead of only the chosen state directory
- repeated renders can diverge from the state previously computed by `pulse run`
- rendering work becomes slower and less predictable
- stage boundaries become blurry, which makes resumability and provenance harder to reason about

## Decision

All repository scanning, file classification, heuristic detection, and history-derived metric computation must happen during `pulse run` and be persisted under the state directory.

`pulse report` must only:

- read persisted state from the state directory
- aggregate or filter persisted records
- render shareable outputs from that persisted data

`pulse report` must not:

- fetch repositories
- walk repository trees
- read Git blobs directly
- execute fresh Git history scans
- invent new heuristic detections that were not already persisted by the run pipeline

## Consequences

Positive:

- Clear separation between compute and presentation stages.
- Deterministic report rendering from explicit persisted inputs.
- Better resumability and auditability because computed signals are tied to run state.
- Faster repeated report generation once state exists.

Negative:

- New report dimensions require schema and pipeline work before they can appear in rendering.
- Some previously convenient report-time heuristics must move into persisted run-time analysis stages.

## Alternatives Considered

- Allow `pulse report` to enrich persisted state with fresh repository scans.
- Let report-time configuration redefine metrics during rendering.
- Keep AI-document and similar derived signals as ephemeral report-only calculations.
