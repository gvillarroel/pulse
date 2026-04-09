---
id: 0006
title: Prefer Partial Degradation Over Whole-Repository Failure
status: accepted
date: 2026-04-09
deciders:
  - contributors
consulted:
  - AGENTS.md
  - recent robustness fixes
---

# ADR 0006: Prefer Partial Degradation Over Whole-Repository Failure

## Context

Real repositories contain edge cases such as empty histories, problematic paths, unreadable blobs, and platform-sensitive command behavior. Failing the entire repository for every localized issue reduces throughput and weakens resumability.

## Decision

When a localized repository issue does not invalidate the entire analysis, `pulse` should degrade partially instead of failing the whole repository.

Current examples:

- empty repositories produce zero-history results instead of failing
- unreadable individual tree entries are skipped during analysis
- report enrichment should fall back when an individual AI-doc read fails
- long Git path argument sets should be chunked to remain executable on Windows

Whole-repository failure should still occur when the repository cannot be fetched or when the remaining result would be materially misleading.

## Consequences

Positive:

- Higher batch completion rates.
- Better operator experience for heterogeneous repository sets.
- More useful partial datasets from imperfect repository populations.

Negative:

- Partial degradation must remain visible and inspectable.
- Summary semantics must distinguish degraded success from clean success if that distinction becomes important later.

## Alternatives Considered

- Strict fail-fast behavior for any repository-local anomaly.
- Silent data loss without checkpoint detail.
