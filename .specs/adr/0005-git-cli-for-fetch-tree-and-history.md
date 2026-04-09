---
id: 0005
title: Use Git CLI For Fetch, Tree Inspection, and History
status: accepted
date: 2026-04-09
deciders:
  - contributors
consulted:
  - AGENTS.md
  - spec.md
---

# ADR 0005: Use Git CLI For Fetch, Tree Inspection, and History

## Context

The project needs reliable repository fetches, tree enumeration, blob reads, and history extraction across many third-party repositories with varying states and path encodings. The architecture guidance allowed either `gitoxide` or direct `git` subprocesses depending on reliability and benchmark outcomes.

## Decision

The current implementation uses the Git CLI as the operational backend for:

- clone and fetch
- revision resolution
- tree listing
- blob reads
- history extraction

The implementation should continue hardening around real repository edge cases, including:

- empty repositories
- unreadable tree entries
- Unicode paths
- Windows command-length limits

## Consequences

Positive:

- High compatibility with real-world repositories.
- Lower implementation risk for baseline repository operations.
- Easier parity with operator expectations and manual debugging.

Negative:

- Requires careful subprocess argument handling across platforms.
- Some robustness issues are platform-specific and must be tested explicitly.

## Alternatives Considered

- Make `gitoxide` the only backend immediately.
- Read repository contents through ad hoc filesystem worktrees instead of Git object access.
