# Repository Layout

`pulse` is organized around a Rust workspace plus root-level decision and research folders.

## Root Responsibilities

- `m.md`: master navigation index
- `README.md`: navigation index and implementation status
- `spec.md`: product and pipeline specification
- `commands.md`: CLI contract
- `ADR/`: architecture decisions and superseding records
- `spikes/`: experiments and benchmark notes
- `examples/`: worked end-to-end runs and reusable example artifacts
- `crates/`: production Rust code
- `docs/`: implementation-oriented reference notes
- `fixtures/`: sample inputs for docs and tests

## Crates

- `pulse-cli`: command entrypoint
- `pulse-core`: shared domain types and state layout
- `pulse-config`: YAML parsing
- `pulse-input`: CSV ingestion and repo normalization
- `pulse-fetch`: resumable `git` CLI fetch/cache logic
- `pulse-git`: Git repository access helpers
- `pulse-analyze`: static snapshot analysis
- `pulse-store`: SQLite persistence and checkpoints
- `pulse-export`: JSON/CSV rendering
