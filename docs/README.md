# pulse Documentation

This folder is the guided documentation set for `pulse`.

If you are new to the project, do not try to read everything at once. Use the paths below.

## Start Here

### I want to understand what `pulse` is

Read:

1. [../README.md](../README.md)
2. [user-manual.md](./user-manual.md)

### I want to run `pulse` for the first time

Read:

1. [user-manual.md](./user-manual.md)
2. [examples/expected-results.md](./examples/expected-results.md)
3. [state-layout/README.md](./state-layout/README.md)

### I want to understand the architecture

Read:

1. [architecture/pipeline-overview.md](./architecture/pipeline-overview.md)
2. [architecture/repository-layout.md](./architecture/repository-layout.md)
3. [schemas/state-tables.md](./schemas/state-tables.md)

### I want to inspect what gets written to disk

Read:

1. [state-layout/README.md](./state-layout/README.md)
2. [schemas/state-tables.md](./schemas/state-tables.md)

## What Is In This Folder

| File | Why you would read it |
| --- | --- |
| `user-manual.md` | learn how to use `pulse` from the command line |
| `architecture/pipeline-overview.md` | understand the execution pipeline and stage responsibilities |
| `architecture/repository-layout.md` | map the repository and crates to the architecture |
| `state-layout/README.md` | understand the durable state directory |
| `schemas/state-tables.md` | inspect the SQLite schema at a practical level |
| `examples/expected-results.md` | compare your run with a healthy expected output shape |

## Recommended Reading Path For Beginners

Use this order if you want context without overload:

1. read [../README.md](../README.md) to understand the product
2. read [user-manual.md](./user-manual.md) to understand the operator workflow
3. read [architecture/pipeline-overview.md](./architecture/pipeline-overview.md) to understand the system
4. read [architecture/repository-layout.md](./architecture/repository-layout.md) to understand the codebase

## Documentation Principles

The docs in this repository try to preserve a clear split:

- root docs explain the product and quick start
- `docs/` explains operation and implementation
- `spec.md` captures the long-term product direction
- `commands.md` captures the intended CLI contract
- `examples/` shows real end-to-end runs

## Related Root Documents

- [../spec.md](../spec.md)
- [../commands.md](../commands.md)
- [../examples/README.md](../examples/README.md)
- [../m.md](../m.md)
