# Fetch Benchmark Plan

## Goal

Compare `git` CLI, `gix`/gitoxide, and `git2`/libgit2 on the operations that matter to `pulse`:

- first-time clone into a bare cache
- incremental fetch into an existing cache
- retry after an interrupted or failed operation
- pruning and ref update behavior

## Test Set

Use a small but representative set of repositories:

- one small public repository
- one medium repository with moderate history
- one larger repository with substantial ref and tag volume
- one repository with frequent pushes

Keep the exact repos pinned in the benchmark notes so the test stays reproducible.

## Metrics

Record the following for each backend:

- wall-clock time
- peak disk usage in the cache directory
- exit code or error class
- retry success rate
- log volume
- whether the backend leaves partial state behind after failure

## Procedure

1. Clear the per-repo cache path.
2. Run a fresh clone into the managed state directory.
3. Run a second fetch against the same repo without clearing state.
4. Interrupt the fetch mid-run and verify retry behavior.
5. Repeat each scenario at least three times.
6. Compare median and worst-case timings, not only the fastest run.

## Expected Outcome

The benchmark should tell us whether the pure-Rust path is ready now or whether the CLI backend is the safer default for the first production slice.

If `gix` is close enough on performance but clearly better on operational simplicity, we can revisit the default later. If it is slower or still awkward to make retry-safe, the CLI backend should stay the default.

