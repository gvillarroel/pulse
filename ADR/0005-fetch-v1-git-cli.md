# ADR 0005: Supersede Fetch Backend for V1

Status: Accepted

Date: 2026-04-02

Supersedes: `0001-fetch-backend.md` for the V1 implementation default

## Context

The accepted fetch ADR selected `gix` as the V1 fetch backend, but the implementation spikes and the first production slice both show a lower-risk path:

- fetch needs operational reliability more than Git object semantics
- V1 benefits from mature transport/auth behavior
- history and read-heavy logic can still stay Rust-native through `gix`

## Decision

Use the `git` CLI as the default fetch/update backend in V1, wrapped behind the `pulse-fetch` crate.

Keep `gix` as the preferred read/history backend and as a future candidate if fetch semantics become operationally competitive.

## Consequences

- V1 fetch is process-backed, not library-backed
- cache layout and retry semantics are prioritized over pure-Rust fetch purity
- the repository no longer has conflicting guidance between the implementation path and the ADR record

