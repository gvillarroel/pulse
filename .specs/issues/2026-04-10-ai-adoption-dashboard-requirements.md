---
title: AI Adoption Dashboard Requirements
status: proposed
date: 2026-04-10
owner: product-and-platform
consulted:
  - .specs/specs.md
  - .specs/adr/0002-explicit-state-directory-and-sqlite.md
  - .specs/adr/0004-stage-oriented-resumable-pipeline.md
---

# AI Adoption Dashboard Requirements

## Summary

Define the next reporting and export capabilities needed for `pulse` to support an engineering-wide AI adoption dashboard. The dashboard must allow leaders to compare teams, inspect progress over time, identify lagging repositories, and distinguish shallow AI presence from operational AI adoption.

This issue is intentionally product-oriented. It focuses on the questions operators want to answer and the evidence `pulse` must expose to support them.

## Problem

Current Pulse exports are strong at repository evidence snapshots, weekly repository activity, pipeline health, and file composition. They are not yet sufficient for answering higher-level adoption questions such as:

- Which teams are meaningfully incorporating AI into their workflows?
- Which teams are accelerating versus stagnating?
- Where is AI guidance present but stale?
- Which repositories are large or active but still missing AI guidance?
- How does one team compare with another once team size is normalized?
- Are AI-related artifacts being used in active engineering work, or only added once and left untouched?

Without explicit AI adoption metrics, Pulse can show evidence but cannot yet function as a management system for AI enablement.

## Goals

- Measure AI adoption by team and repository with clear, comparable metrics.
- Support time-based views so adoption progress can be tracked, not just observed once.
- Separate foundational signals from deeper operational adoption signals.
- Make team-to-team comparison fair through normalized ratios and weighted metrics.
- Expose enough structured data for downstream dashboards to drill down from team to repo to artifact.
- Keep metric definitions deterministic enough that repeated runs over the same state produce the same KPI values.

## Non-Goals

- This issue does not prescribe a specific frontend framework or dashboard implementation.
- This issue does not require causal proof that AI causes productivity gains.
- This issue does not require a web UI inside Pulse itself.
- This issue does not require full semantic understanding of prompt or guidance content in the first phase.

## Primary Users

- Engineering directors and VPs
- Engineering managers
- AI enablement leads
- Platform or developer productivity teams
- Team leads comparing their portfolio against peer teams

## Questions The Dashboard Must Answer

### Foundation

- Which teams have baseline AI guidance coverage?
- Which teams are missing core artifacts such as `AGENTS.md`, `CLAUDE.md`, or Copilot instructions?
- Which repositories still have no AI entrypoints at all?

### Progress

- Which teams are increasing adoption over time?
- Which repositories recently crossed from no AI guidance to baseline AI guidance?
- Which teams plateaued after initial adoption?

### Quality

- Which teams have operational AI guidance versus a single isolated file?
- Which AI documents are stale?
- Which teams have inconsistent guidance across their repositories?

### Operational Usage

- Which repositories or teams actively change AI-related files?
- Which repositories include AI-specific workflows, prompts, or automation assets?
- Which teams use AI-related patterns in real PR and issue traffic?

### Impact Proxies

- Do teams with stronger AI adoption show different throughput, review, or cycle-time patterns?
- Are highly active repositories still lacking AI support despite complexity and size?

## Metric Families

### 1. Foundation Coverage

Purpose:
Measure whether the minimum AI support baseline exists.

Candidate metrics:

- repositories with `AGENTS.md`
- repositories with `CLAUDE.md`
- repositories with Copilot instructions
- repositories with generic AI docs
- repositories with any AI guidance
- repositories with README plus at least one AI-specific artifact

Recommended visualizations:

- stacked maturity bars by team
- heatmap of `team x signal`
- coverage trend over time

### 2. Adoption Progress

Purpose:
Measure growth in adoption rather than current presence only.

Candidate metrics:

- first AI document added per repository
- newly adopted repositories per week
- net new AI-related files per week
- time from repo onboarding to first AI artifact
- adoption velocity by team

Recommended visualizations:

- cumulative adoption curves
- weekly adoption bars
- time-to-first-adoption distributions

Data requirement:
This requires historical path-level evidence, not only current snapshots.

### 3. Repository Complexity And Readiness

Purpose:
Show whether AI adoption is happening in meaningful repositories or only in peripheral ones.

Candidate metrics:

- AI maturity versus repository size
- AI maturity versus repository activity
- AI adoption weighted by lines or files
- active repositories without AI guidance
- large repositories without AI guidance

Recommended visualizations:

- scatter: files vs lines colored by team and maturity
- quadrants: large/active with or without AI support
- ranked exception tables

### 4. AI Guidance Quality

Purpose:
Distinguish shallow presence from operational readiness.

Scoring rule:
The maturity model must be deterministic and derived from explicitly detectable repository evidence. In the first phase, scoring should rely on file presence, path matching, metadata, ownership files, and workflow/configuration artifacts that can be extracted reliably with the current Pulse pipeline and practical Rust libraries. Freeform semantic interpretation of document quality is optional enrichment and must not be required for baseline scoring.

Candidate maturity model:

- Level 0: no AI signal
- Level 1: one isolated AI signal
- Level 2: basic guidance
- Level 3: operational guidance
- Level 4: operational guidance plus prompts, workflows, ownership, or policy support

Phase-1 deterministic interpretation:

- Level 0: no detected AI-related file, instruction file, workflow, or agent/tooling artifact.
- Level 1: exactly one detected AI-related artifact with no supporting ownership, workflow, or companion guidance signals.
- Level 2: at least one recognized repository guidance entrypoint such as `AGENTS.md`, `CLAUDE.md`, Copilot instructions, or equivalent generic AI guidance document.
- Level 3: Level 2 plus at least one additional operational support signal such as prompts, agent or MCP configuration, AI-related workflow files, or local AI tooling assets.
- Level 4: Level 3 plus at least one governance or ownership support signal such as CODEOWNERS coverage for AI artifacts, explicit ownership mapping, or policy/check enforcement artifacts.

Required definitions for implementation:

- "stale" must be threshold-based and configurable, with the exported threshold recorded as provenance.
- "orphaned AI docs" means AI-related artifacts with no detectable ownership signal from configured team mapping, CODEOWNERS, or explicit ownership metadata.
- "weighted adoption score" must be computed from published component weights rather than opaque heuristics.
- "sensitive repository" and "criticality tier" must come from explicit configuration or repository metadata mapping, not inferred ad hoc.

Candidate metrics:

- maturity score by repository
- maturity distribution by team
- consistency score within a team
- stale AI guidance count
- orphaned AI docs count

Recommended visualizations:

- maturity distribution by team
- consistency ranking
- stale guidance leaderboard

### 5. AI Files Presence And Activity

Purpose:
Track where AI documents exist now and, once supported, how much active work touches them.

Current metrics feasible from existing exports:

- repositories with `AGENTS.md`
- repositories with `CLAUDE.md`
- repositories with Copilot instructions
- repositories with generic AI docs
- current AI file count, lines, and bytes by team or repo

Future metrics requiring additional ingest:

- weekly commits touching AI files
- PRs modifying AI files
- active contributors on AI files
- net additions/removals on AI files
- recent AI-doc churn

Recommended visualizations:

- current AI presence by team
- weekly AI file activity
- repos with active AI file maintenance

### 6. Tooling And Enablement

Purpose:
Measure whether repositories include supporting infrastructure for AI-enabled workflows.

Candidate metrics:

- repositories with AI-related GitHub Actions workflows
- repositories with prompt libraries
- repositories with MCP or agent configuration
- repositories with devcontainer or local tooling for AI workflows
- repositories with AI policy checks
- repositories with AI-related CODEOWNERS coverage

Recommended visualizations:

- capability matrix by team
- rollout timeline by capability
- gap analysis by repo

### 7. Usage Signals From GitHub

Purpose:
Detect whether AI adoption appears in actual engineering work.

Candidate metrics:

- PRs tagged or labeled as AI-related
- PRs created or assisted by bots or AI tooling
- issues related to AI rollout or enablement
- commits that touch prompts, workflows, AI docs, or guidance assets
- releases mentioning AI features

Recommended visualizations:

- PR and issue trend lines
- proportion of active repos with AI-related work
- contribution mix by signal type

Data requirement:
Requires PR, review, issue, and commit metadata beyond current parquet exports.

### 8. Impact Proxies

Purpose:
Support comparison between AI maturity and delivery outcomes without claiming causality.

Candidate metrics:

- PR cycle time
- review turnaround time
- merge frequency
- throughput
- average PR size
- change churn or rework proxies

Recommended visualizations:

- before/after comparisons
- maturity vs delivery scatter plots
- team comparison with normalization

Data requirement:
Requires GitHub PR and review metadata.

### 9. Governance And Risk

Purpose:
Identify adoption gaps, inconsistency, and control issues.

Candidate metrics:

- active repositories without AI guidance
- sensitive repositories without AI governance
- AI docs older than threshold
- AI docs without owners
- conflicting or fragmented AI entrypoints
- analysis pipeline failures by team

Recommended visualizations:

- governance exceptions table
- risk leaderboard
- team risk heatmap

## Comparison Rules

All major team views should support:

- absolute totals
- ratio per repository
- ratio per active repository
- size-weighted comparison
- activity-weighted comparison

Without normalization, larger teams will dominate every comparison and hide meaningful adoption quality differences.

Team assignment rule:
For this dashboard, repository-to-team mapping is fixed rather than historical. Time-series views use the current configured team assignment for the repository across the full reporting window. This simplification should be treated as an explicit product constraint in exports and dashboard labeling.

Data completeness rule:
Every exported metric that depends on optional or not-yet-universal ingest must carry a completeness status so dashboards can distinguish `zero` from `unknown`, `not_collected`, or `partially_available`.

## Filters And Drill-Down

Required filters:

- team
- repository
- time window
- active repositories only
- size tier
- activity tier
- language
- AI guidance presence

Required drill-down path:

1. team
2. repository
3. artifact or workflow
4. commit, PR, or issue where available

## Data Requirements For Pulse

### Already Strong In Current Exports

- repository metadata
- file snapshots
- repository size snapshots
- weekly repository activity
- pipeline checkpoint state
- failure state

### Needed Next

- commit history by path
- PR metadata
- review metadata
- issue labels and metadata
- workflow metadata and optionally workflow-run summaries
- CODEOWNERS and ownership mappings
- team mapping as a first-class config input

Completeness and provenance requirements:

- Exports must identify which metric families are fully computed versus unavailable for a repository, team, or time bucket.
- GitHub-derived metrics must preserve enough provenance to distinguish "no observed activity" from "ingest not enabled, not backfilled, or failed".
- Completeness status should be aggregatable so team-level summaries do not silently mix unknown values into zero-valued counts or ratios.

### Ideal Longer Term

- diff-level AI file change records
- bot versus human attribution
- repository criticality tiers
- content validation for AI guidance quality

## Dashboard KPI Candidates

Recommended top-level executive KPIs:

- AI foundation coverage
- operational AI repository coverage
- adoption growth over 30 and 90 days
- AI-doc active repositories
- stale guidance risk
- AI tooling coverage
- governance exception count
- weighted adoption score

KPI publication rule:
Each KPI must have a documented formula, input signal list, normalization rule if any, and completeness behavior.

## Proposed Delivery Phases

### Phase 1: Better Snapshot Intelligence

- foundation coverage by team
- AI maturity score by repository and team
- current AI document presence by team
- active or large repositories missing AI support
- governance and pipeline gaps

### Phase 2: Historical Adoption

- first adoption date per repo
- adoption growth curves
- weekly AI file activity
- AI-related PR and issue activity
- freshness and staleness trends

### Phase 3: Impact And Governance

- AI maturity versus delivery metrics
- workflow and automation capability maps
- ownership and policy coverage
- exception monitoring and thresholds

## Success Criteria

Pulse should eventually support statements like:

- Team Atlas has the highest operational AI coverage, but Team Aurora is adopting faster over the last 90 days.
- Team Beacon has several large active repositories with no AI guidance baseline.
- Team Nova shows broad but shallow adoption.
- Team Lattice has strong documentation presence but weak freshness.
- Team Vertex improved guidance coverage and AI-related workflow rollout together.

## Acceptance Criteria For This Issue

- A shared requirement baseline exists for future Pulse exports and reporting work.
- The metric families are organized clearly enough to drive implementation planning.
- The distinction between current evidence and future required data is explicit.
- Product and engineering can use this issue to prioritize new ingest and export work.
- The AI maturity model is deterministic enough to implement without mandatory semantic interpretation.
- Team mapping behavior is explicit: fixed current assignment, not historical reassignment.
- Export requirements distinguish `zero` from incomplete or unavailable data.

## Suggested Next Work Items

- Add team mapping as a first-class Pulse configuration concept rather than dashboard-only post-processing.
- Design path-history exports for AI-related files.
- Add PR and review ingest for AI-related comparisons.
- Define a stable AI maturity scoring model.
- Introduce repository criticality tiers for weighted reporting.
