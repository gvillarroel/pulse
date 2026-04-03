# AGENTS.md

## Documentation Language

All project documentation must be written in English.

## Purpose

`pulse` is a terminal-first analytics system for mining engineering signals from many source repositories and turning them into reusable, resumable, time-aware quality datasets.

This repository should evolve toward a Rust implementation that can:

- ingest repository targets from explicit inputs such as YAML configuration files or CSV files
- discover additional repositories from configurable filters
- clone or fetch repositories efficiently at scale
- compute repository, file, contributor, language, and evolution metrics
- persist intermediate progress so interrupted runs can resume safely
- reuse prior execution results to avoid recomputing unchanged data
- expose results through CLI queries and exportable datasets

## Product Direction

The system is optimized for batch analysis of many repositories, not for interactive IDE-like inspection of a single repo.

Primary goals:

- high-throughput repository processing
- incremental execution and resumability
- deterministic outputs
- explicit provenance for every metric
- progressive enrichment of repository metadata over time

Non-goals for the first iterations:

- building a web UI before the data model is stable
- supporting every forge at once if GitHub-first gets us to production faster
- deep semantic analysis for every language in V1

## Working Principles

When editing this project:

- prefer Rust for the main implementation
- design every long-running workflow to be resumable
- treat repository fetching, file scanning, and history extraction as separate stages
- support configuration-driven execution, with YAML as the preferred top-level configuration format
- make expensive analyses cacheable and invalidatable by content or revision
- favor append-only or versioned storage for snapshots and derived metrics
- keep CLI commands composable and automation-friendly
- optimize for correctness and restart safety before micro-optimizing

## Architecture Expectations

The codebase should converge on these responsibilities:

- `discover`: find candidate repositories from providers or prior snapshots
- `ingest`: register repository targets from CSV or command input
- `fetch`: clone, mirror, or update repositories locally
- `analyze`: compute metrics from working trees, Git history, and language tooling
- `store`: persist snapshots, file histories, checkpoints, and derived facts
- `query`: filter and export subsets of repositories and metrics
- `render`: generate time-series outputs and evolutionary visual artifacts

Preferred technical choices:

- Rust workspace for the core CLI and analysis crates
- `gitoxide` or direct `git` subprocess strategy, chosen by benchmark and reliability
- structured local state under a project-managed data directory
- bounded concurrency with per-stage worker pools
- explicit checkpoint records for every repository and analysis stage

## Quality Bar

Every meaningful implementation change should try to preserve these properties:

- idempotent command execution where practical
- safe retries after crashes
- stable machine-readable outputs
- clear error classification: transient, permanent, invalid input, unsupported repo
- progress visibility for long-running jobs
- tests for parsing, checkpointing, filtering, and core metric calculations

## Resumability Rules

Resumability is a first-class requirement.

Assume that:

- the process can stop at any point
- individual repositories can fail without failing the whole batch
- the same repository may be analyzed many times over months

Therefore:

- persist progress after each meaningful stage
- separate raw fetched state from derived analysis state
- record the repository revision range used for each computed metric
- never require full recomputation if inputs are unchanged

## Data Model Guidelines

At minimum, the system should be able to represent:

- repository identity and source
- configuration provenance, including which config file or input parameters were used
- owners and provider metadata
- default branch and revision metadata
- contributors and contribution summaries
- languages and size distribution
- file inventory with path, size, line count, timestamps, and change history
- file targeting rules for deeper analysis scopes
- weekly or comparable time-bucketed evolution snapshots
- analysis provenance, tool versions, and execution timestamps

## Rust Analyzer Integration

Rust support is strategic, but it should be introduced pragmatically.

- Start with Rust repository detection and Rust-specific metrics.
- Integrate `rust-analyzer` only behind a dedicated analysis stage.
- Treat semantic analysis as optional enrichment, not a hard dependency for basic runs.
- Cache semantic outputs aggressively because they can be expensive.
- Ensure the pipeline still works when `rust-analyzer` is unavailable.

## Commands Contract

This project will maintain a `commands.md` file describing the intended CLI surface.

Implementation guidance:

- prefer the minimum viable command set
- command names should be explicit and script-friendly
- YAML configuration should be the preferred way to describe repository inputs and analysis targeting rules
- filters should support both human invocation and batch automation
- default output should be readable; `--json` should be available for tooling
- long-running commands should expose checkpoint and resume semantics through an explicit state directory, not hidden global memory

## Collaboration Contract

Contributors and agents working on this repository should:

- read `spec.md` before making architectural decisions
- align new commands with `commands.md`
- avoid introducing one-off workflows that bypass the core pipeline
- document assumptions when a capability is deferred to a later phase
- prefer extensible abstractions over provider- or language-specific hacks
- keep all new and updated documents in English

## Decision Records and Spikes

This repository should keep two explicit knowledge folders for future reference:

- `spikes/`: experimental work, prototypes, benchmarks, and short technical investigations
- `ADR/`: architecture decision records for the project

Guidance for `spikes/`:

- use it for exploratory implementations and technical validation
- each spike should ideally leave a short technical write-up describing the question, approach, findings, and recommendation
- spike outputs may later be used as reference material when choosing implementation direction
- spike code should not be treated as production-ready by default

Guidance for `ADR/`:

- use it to record important architectural decisions once they are made
- each ADR should capture context, decision, consequences, and any notable alternatives considered
- when future work needs historical rationale, this folder is the place to check first

Contributors and agents should consult these folders when they need background, prior experiments, or the reasoning behind architectural choices.

## Initial Delivery Strategy

The recommended implementation order is:

1. CLI skeleton and configuration model
2. CSV-based repository intake
3. efficient local fetch/update pipeline
4. repository-level and file-level static metrics
5. resumable execution state and caching
6. Git history aggregation into weekly snapshots
7. repository discovery and delta discovery workflows
8. optional semantic enrichments such as Rust Analyzer

## Definition of Done

A feature is not done when it only works on a clean first run.

It is done when:

- it can be rerun safely
- it leaves inspectable state
- it has a defined failure model
- it fits the documented CLI and data model
- it includes enough validation to trust the result
