---
id: 0003
title: YAML Is Preferred Input, CSV Remains Supported
status: accepted
date: 2026-04-09
deciders:
  - contributors
consulted:
  - AGENTS.md
  - spec.md
  - commands.md
---

# ADR 0003: YAML Is Preferred Input, CSV Remains Supported

## Context

Operators need both reproducible configuration and a simple interchange format for explicit repository lists. The system also needs a place to define analysis options, focus patterns, and report rules.

## Decision

YAML is the preferred top-level configuration format, while CSV remains a supported repository intake format.

YAML is used for:

- repository source references
- analysis options
- focus rules
- report rules

CSV remains valid for explicit repository lists, with `repo` as the minimum required column.

## Consequences

Positive:

- Configured runs are reproducible and script-friendly.
- CSV stays available for simple exported target lists.
- The command surface can support both quick runs and richer configured workflows.

Negative:

- The system must continue reconciling config-driven and CSV-driven resolution paths.
- Documentation must stay clear about which capabilities are config-only versus CSV-only.

## Alternatives Considered

- CSV-only input.
- YAML-only input.
- Provider discovery as the only accepted source mode.
