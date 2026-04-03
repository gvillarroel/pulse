# Fetch Spike: Resumable Repository Download/Update

## Question

What should `pulse` use for the fetch stage so it can clone, update, and resume repository state reliably at batch scale?

## Recommendation

Use the `git` CLI as the production fetch backend for V1, with a bare or mirror cache under `--state-dir`, and keep `gix`/gitoxide and `git2`/libgit2 as spike candidates for later optimization.

This is the lowest-risk route for the first implementation because it gives us:

- mature authentication and protocol coverage
- mirror/bare layouts that fit resumable batch processing
- predictable subprocess failure boundaries
- easy parity with how operators already reason about Git

## Why not start with a pure Rust backend

`gix` is the strongest pure-Rust candidate, but the current docs still frame it as unstable and explicitly not a `git` replacement. Its clone/fetch APIs are viable, but the spike surface is lower-level and partial-failure handling is owned by the caller.

`git2` is more mature in terms of clone/fetch bindings, but it pulls in `libgit2` and the FFI surface makes the first production pass less aligned with the repository's Rust-first direction.

## Candidate Matrix

| Option | Strengths | Risks | Fit for V1 |
| --- | --- | --- | --- |
| `git` CLI | Best protocol parity, stable auth, mirror/bare workflows, easy to resume from a local cache | Process spawning and parsing overhead | High |
| `gix` / gitoxide | Pure Rust, direct control, strong long-term alignment | API maturity, cleanup semantics, more implementation work | Medium |
| `git2` / libgit2 | Mature bindings, low-level control | Native dependency, FFI complexity, less aligned with "Rust-first" | Medium |

## State Layout Assumption

Use a cache-first layout under `--state-dir`:

- `repos/<provider>/<owner>/<name>.git` for the bare/mirror cache
- `checkpoints/<repo-key>.json` for fetch stage progress
- `logs/<repo-key>.log` for fetch diagnostics
- `fetch-metadata/<repo-key>.json` for revision, timestamp, backend, and config hash

## Resume Strategy

The fetch stage should be resumable by design:

1. Normalize the repo key before any network work.
2. Write a `running` checkpoint before invoking the backend.
3. Fetch into the persistent bare cache, not into an ephemeral checkout.
4. Update metadata only after a successful fetch.
5. Mark the checkpoint `completed` with the fetched revision and timestamp.
6. Leave failed runs in a retryable `failed` state with the error class.

For the CLI backend, that means `clone --mirror` for first contact and `fetch` or `remote update` for incremental updates.

For `gix`, the spike should verify the delete-on-drop behavior of incomplete clones and whether the fetch flow cleanly supports retry after an interrupted run.

## Benchmark Axes

Measure all candidates against the same repository set:

- fresh clone time
- incremental fetch time
- interrupted run recovery time
- disk footprint of the local cache
- stderr/stdout noise and error classification
- auth and transport compatibility

## Spike Artifacts

- [Prototype Rust interface](./prototype.rs)
- [Benchmark plan](./benchmark-plan.md)
- [Preliminary ADR](./ADR-preliminary.md)

## Sources

- [git clone docs](https://git-scm.com/docs/git-clone.html)
- [libgit2 clone docs](https://libgit2.org/docs/reference/main/clone/index.html)
- [libgit2 fetch docs](https://libgit2.org/docs/reference/main/remote/git_remote_fetch.html)
- [gix docs](https://docs.rs/gix)
- [gix clone docs](https://docs.rs/gix/latest/gix/clone/index.html)
- [gix PrepareFetch docs](https://docs.rs/gix/latest/gix/clone/struct.PrepareFetch.html)
