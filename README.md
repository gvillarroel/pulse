# pulse

`pulse` is a terminal-first repository analytics tool for teams that want a fast, repeatable view of AI adoption across an engineering organization.

Today the implemented workflow is:

1. collect the repositories for a GitHub organization or account
2. fetch and analyze them into a durable local state directory
3. generate a self-contained HTML report from that persisted state

The current AI-oriented report is especially useful for questions like:

- which repositories already use `AGENTS.md`, `CLAUDE.md`, `SKILL.md`, `skills.md`, or `copilot-instructions.md`
- which repositories link AI-related documents together
- when those conventions started appearing across the repository set

## What A Company Can Do With It

A company can point `pulse` at its GitHub organization and generate an HTML report that shows its current AI documentation footprint and adoption progress across repositories.

The shortest path is:

1. export the repository list from GitHub
2. run `pulse`
3. open the generated report

## Install

### Prerequisites

You need:

- [Rust](https://www.rust-lang.org/tools/install)
- `git` on your `PATH`
- [GitHub CLI](https://cli.github.com/) if you want the easiest organization-wide workflow

Verify the toolchain:

```powershell
cargo --version
git --version
gh --version
```

### Build

From the repository root:

```powershell
cargo build -p pulse-cli
```

You can run the CLI without installing it globally:

```powershell
cargo run -p pulse-cli -- --help
```

Or install it locally with Cargo:

```powershell
cargo install --path crates/pulse-cli
```

Cargo installs the executable as `pulse-cli`, so after that use:

```powershell
pulse-cli --help
```

The installed executable is currently named `pulse-cli`, while the product and command surface are documented as `pulse`.

## Fastest Organization Workflow

This is the main recommended path for a company that wants an AI adoption report for one GitHub organization.

### 1. Authenticate GitHub CLI

```powershell
gh auth login
```

### 2. Export only non-archived repositories updated since a given date

Choose the command block for your OS.

Filter rules:

- exclude archived repositories
- keep only repositories with `pushedAt` on or after a cutoff date

Use an ISO date such as `2026-01-01`.

#### Windows PowerShell

```powershell
$Org = "my-org"
$Since = Get-Date "2026-01-01"

$repos = gh repo list $Org --limit 500 --no-archived --json nameWithOwner,pushedAt |
  ConvertFrom-Json |
  Where-Object { [datetime]$_.pushedAt -ge $Since } |
  Sort-Object pushedAt -Descending

@('repo') + ($repos.nameWithOwner) | Set-Content ".\$Org-repos.csv"
```

#### macOS / Linux (`bash` or `zsh`)

```bash
ORG="my-org"
SINCE="2026-01-01T00:00:00Z"

{
  echo "repo"
  gh repo list "$ORG" --limit 500 --no-archived --json nameWithOwner,pushedAt \
    --jq ".[] | select(.pushedAt >= \"$SINCE\") | .nameWithOwner"
} > "./$ORG-repos.csv"
```

If the organization has more than 500 repositories, increase `--limit`.

If you want a different cutoff, just change `$Since` on Windows or `$SINCE` on macOS/Linux.

The generated CSV is exactly what `pulse` expects:

```csv
repo
my-org/service-a
my-org/platform-api
my-org/internal-docs
```

### 3. Create an AI-focused config

Create `.\ai-report.yaml`.

If you used `my-org` above, the CSV path below is correct as-is:

```yaml
repositories:
  csv: ./my-org-repos.csv

analysis:
  with_history: true

focus:
  include:
    - AGENTS.md
    - CLAUDE.md
    - SPEC.md
    - SPECS.md
    - skills.md
    - .github/copilot-instructions.md
    - "**/AGENTS.md"
    - "**/CLAUDE.md"
    - "**/SKILL.md"
    - "**/skills.md"
    - "**/copilot-instructions.md"
    - "**/*agent*.md"
    - "**/*skill*.md"
    - "**/*copilot*.md"
    - "**/*prompt*.md"
    - "**/*model*.md"
```

This config tells `pulse` to:

- process the repositories from your organization CSV
- compute weekly history aggregates
- mark AI-related documentation paths as focused files

### 4. Run the analysis

```powershell
cargo run -p pulse-cli -- run --config .\ai-report.yaml --state-dir .\state\my-org --progress --json
```

If you installed the binary with `cargo install`, you can run:

```powershell
pulse-cli run --config .\ai-report.yaml --state-dir .\state\my-org --progress --json
```

### 5. Generate the HTML report

```powershell
cargo run -p pulse-cli -- report --state-dir .\state\my-org --title "My Org AI Adoption Report"
```

Or:

```powershell
pulse-cli report --state-dir .\state\my-org --title "My Org AI Adoption Report"
```

The command prints the output path for the generated HTML file.

By default it is written under:

```text
.\state\my-org\exports\report.html
```

## Daily Use

The same state directory is reusable. That means reruns are safe and intended.

Typical update flow:

```powershell
pulse-cli run --config .\ai-report.yaml --state-dir .\state\my-org --progress --json
pulse-cli report --state-dir .\state\my-org --title "My Org AI Adoption Report"
```

This makes it practical to refresh the report weekly or monthly and track how AI-related conventions spread across the organization.

## What The Report Means Today

The current report is a practical early signal for AI readiness and adoption, not a full semantic audit of AI usage.

Today `pulse` is strongest at identifying:

- AI-related repository documentation entrypoints
- structured markdown conventions used by coding assistants and agent workflows
- linked documentation relationships
- first-seen adoption over time

It does not yet claim to measure:

- model quality
- prompt quality
- production AI feature correctness
- actual runtime AI traffic

So the best way to position the report today is:

- organizational AI documentation adoption
- repository-level AI workflow readiness
- spread of assistant and agent operating conventions

## Minimal Commands

These commands match the current CLI implementation:

Validate the repository list:

```powershell
pulse-cli list --input .\my-org-repos.csv --format json
```

Run analysis from CSV only:

```powershell
pulse-cli run --input .\my-org-repos.csv --state-dir .\state\my-org --json
```

Run analysis from YAML config:

```powershell
pulse-cli run --config .\ai-report.yaml --state-dir .\state\my-org --progress --json
```

Render the report:

```powershell
pulse-cli report --state-dir .\state\my-org
```

## State Directory

`pulse` writes durable operator-managed state under `--state-dir`:

```text
state/
  my-org/
    repos/
    db/
      pulse.sqlite
    runs/
    logs/
    exports/
      report.html
```

Important parts:

- `repos/`: persistent bare Git caches
- `db/pulse.sqlite`: fetched state, checkpoints, snapshots, and weekly aggregates
- `exports/report.html`: shareable HTML report

## Worked Example

This repository includes a full example of the same workflow over a real GitHub account:

- [examples/gvillarroel-all-repos/README.md](./examples/gvillarroel-all-repos/README.md)

It shows:

- how the repository list was generated
- the config used for AI-oriented focus paths
- where the persisted state lives
- the resulting report artifacts

## Current Scope

The current implementation already includes:

- repository input from CSV or YAML
- repository fetching into a reusable local cache
- repository and file snapshot metrics
- optional weekly history aggregation
- HTML report generation from persisted state

The current implementation does not yet include:

- built-in GitHub organization discovery inside the CLI
- deep semantic analysis of prompts, agents, or models
- provider-wide filtering directly from `pulse run`

For now, GitHub CLI is the easiest way to provide the organization repository list.

## Documentation Map

Use [m.md](./m.md) as the top-level navigation index when you want the shortest path through the repository documentation.

## More Detail

Use these documents when you need implementation or architecture detail:

- [docs/user-manual.md](./docs/user-manual.md)
- [commands.md](./commands.md)
- [spec.md](./spec.md)
- [docs/state-layout/README.md](./docs/state-layout/README.md)
- [examples/README.md](./examples/README.md)
