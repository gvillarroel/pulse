# ADR: Fetch Backend for Resumable Repository Sync

## Context

`pulse` needs a fetch stage that can clone and update many repositories, survive interruptions, and reuse local state across reruns.

The fetch stage is foundational because all later analysis depends on a durable local Git cache. A bad choice here will leak into resumability, error handling, and long-term operational cost.

## Decision

Use the `git` CLI as the default backend for V1 fetch/update operations.

Keep `gix`/gitoxide as the leading pure-Rust alternative to validate in a spike and potentially adopt later if it proves stable enough and simpler to operate.

Keep `git2`/libgit2 as a fallback comparison point, not the default path.

## Rationale

- `git` already defines the operational model we want: bare caches, mirror clones, fetch updates, and conventional refspec behavior.
- The CLI backend keeps the first implementation simple while the repository's higher-level architecture is still forming.
- `gix` is the most strategically aligned Rust option, but its docs still make the stability and cleanup model more explicit than the CLI.
- `git2` is useful for comparison, but it adds FFI and native dependency complexity without a clear advantage for the first release.

## Consequences

- Fetch becomes an external process boundary instead of a library boundary.
- We will need a backend abstraction so future code can swap `git` CLI for `gix` or `git2` if needed.
- The first implementation can focus on state layout, checkpoints, and retry semantics instead of binding-level details.

## Spike Validation Criteria

- A fresh clone can be written into the managed cache.
- An interrupted fetch can be retried without manual cleanup.
- An incremental fetch updates refs and metadata correctly.
- Failure modes are classifiable as transient, permanent, or invalid input.
- The benchmark shows whether the extra cost of process spawning is material compared with network and disk time.

