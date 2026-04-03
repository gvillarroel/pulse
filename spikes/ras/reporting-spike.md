# Reporting Spike: Self-Contained Interactive HTML

Date: 2026-04-02

## Question

What is the best way for `pulse` to generate an interactive report from the repository list example, with:

- understandable charts
- navigation across dimensions
- one output file
- no server requirement
- a Rust-first implementation path

## Constraints

- The current durable source of truth is SQLite under `--state-dir`.
- The report should be generated after analysis, not as part of the analysis pipeline itself.
- The result should work as a portable artifact that can be opened locally in a browser.
- The report needs multiple logical pages, but the user asked for a single file.

## Options Considered

### 1. Plotly via `plotly` crate

Why it is attractive:

- Rust crate with stable docs and active packaging.
- Explicit HTML output support.
- The crate exposes embedded JavaScript support through the `plotly_embed_js` feature.
- The generated charts are interactive by default: zoom, hover, legend toggles, click handlers.

Why it fits `pulse`:

- We can embed Plotly JS once and generate many charts in the same page.
- We can implement "multiple pages in one file" as client-side sections or tabs.
- The output remains a single standalone HTML artifact.

Tradeoffs:

- The resulting HTML file is heavier because the JavaScript bundle is embedded.
- The crate is still a wrapper over browser JavaScript, not a pure Rust chart renderer.

Verdict:

- Best fit for the first reporting command.

### 2. Apache ECharts via `charming`

Why it is attractive:

- Good Rust ergonomics.
- Strong support for dashboard-style charts.
- `HtmlRenderer` can render interactive charts for the browser.

Why it is not the first choice:

- The default HTML renderer points to CDN-hosted ECharts assets.
- For a single offline file, we would need extra packaging work or custom asset embedding.
- It is a better candidate if we later want richer dashboard composition around ECharts specifically.

Verdict:

- Good alternative, but not the shortest path to an offline single-file report.

### 3. Vega-Lite + `vega-embed`

Why it is attractive:

- Declarative grammar with good chart quality.
- Strong browser embedding model.

Why it is not the first choice:

- There is no obvious Rust-native packaging path already aligned with this repository.
- We would still need to manage the JavaScript bundle embedding ourselves.
- It adds more frontend assembly work than the value it brings for the first report command.

Verdict:

- Useful if the report layer later becomes a more declarative visualization surface, but not the first implementation.

## Recommended Shape

Use a self-contained HTML report with:

- one embedded Plotly JavaScript bundle
- one embedded JSON payload derived from SQLite
- client-side tabs for multiple report pages inside the same file
- click interactions to pivot between dimensions, for example language to repositories

This gives us:

- one command: `pulse report`
- one artifact: `report.html`
- no extra runtime dependency besides a browser
- a clean separation between analysis state and reporting state

## Initial Dimensions To Include

- overview: repository counts, analyzed/failure coverage, stage checkpoint map
- languages: top languages and extensions by bytes
- repositories: top repositories, size/line scatter, filterable table
- history: weekly commits, active repositories, contributor instances
- failures: checkpoint failure ledger

## Why Tabs Count As "Multiple Pages"

The user asked for "multiple pages in a single file". The practical interpretation is:

- one HTML file
- several independent navigable report surfaces
- no separate routing or hosting

Tabs or panel navigation satisfy that requirement with much lower operational cost than generating a PDF portfolio or a packaged web app.

## Sources

- [plotly crate on docs.rs](https://docs.rs/plotly)
- [plotly.rs repository](https://github.com/plotly/plotly.rs)
- [charming crate on docs.rs](https://docs.rs/charming)
- [charming repository](https://github.com/yuankunzhang/charming)
- [Vega-Lite embedding guide](https://vega.github.io/vega-lite/usage/embed.html)
