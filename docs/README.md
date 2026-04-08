# pulse Docs

This folder is the self-contained docs set for `pulse`.

## Read Paths

First run:

1. [user-manual.md](./user-manual.md)
2. [state-layout/README.md](./state-layout/README.md)
3. [schemas/state-tables.md](./schemas/state-tables.md)

Implementation context:

1. [architecture/repository-layout.md](./architecture/repository-layout.md)
2. [state-layout/README.md](./state-layout/README.md)
3. [schemas/state-tables.md](./schemas/state-tables.md)

## Doc Map

| File | Purpose |
| --- | --- |
| `user-manual.md` | operator guide for `list`, `run`, and `report` |
| `architecture/repository-layout.md` | repo structure and crate roles |
| `state-layout/README.md` | durable `--state-dir` layout |
| `schemas/state-tables.md` | SQLite table summary |

## Rules

- docs navigation works from inside `docs/`
- avoid machine-specific absolute links
- examples in this folder should remain readable without opening files outside `docs/`
