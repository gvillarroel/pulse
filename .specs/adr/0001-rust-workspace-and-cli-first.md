---
id: 0001
title: Rust Workspace With CLI-First Delivery
status: accepted
date: 2026-04-09
deciders:
  - contributors
consulted:
  - AGENTS.md
  - spec.md
  - commands.md
---

# ADR 0001: Rust Workspace With CLI-First Delivery

## Context

`pulse` is intended to process many repositories in batch workflows with resumability, durable local state, and automation-friendly execution. The project needs strong control over execution stages, state persistence, and reliability characteristics.

## Decision

The project uses a Rust workspace as the primary implementation strategy and delivers value through a terminal-first CLI before introducing any web UI.

The CLI remains the primary operator interface, with a small explicit command surface:

- `list`
- `run`
- `report`

## Consequences

Positive:

- Strong fit for long-running local batch processing.
- Good control over deterministic behavior and typed internal models.
- Clear automation story for scripts and scheduled runs.
- Workspace crate boundaries support staged architecture.

Negative:

- Some workflows, such as provider discovery and richer exports, still require additional implementation work.
- UI-less operation raises the bar on output clarity and machine-readable state.

## Alternatives Considered

- Start with a web application.
- Build the first implementation as ad hoc scripts.
- Use a monolithic single-crate codebase without domain separation.
