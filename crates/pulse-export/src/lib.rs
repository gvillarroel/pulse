use anyhow::Result;
use plotly::Plot;
use pulse_core::{RepoTarget, ReportDataset, RunSummary};

pub fn targets_as_csv(targets: &[RepoTarget]) -> Result<String> {
    let mut writer = csv::Writer::from_writer(Vec::new());
    for target in targets {
        writer.serialize(target)?;
    }
    let bytes = writer.into_inner()?;
    Ok(String::from_utf8(bytes)?)
}

pub fn targets_as_json(targets: &[RepoTarget]) -> Result<String> {
    Ok(serde_json::to_string_pretty(targets)?)
}

pub fn summary_as_json(summary: &RunSummary) -> Result<String> {
    Ok(serde_json::to_string_pretty(summary)?)
}

pub fn report_as_html(title: &str, generated_at: &str, dataset: &ReportDataset) -> Result<String> {
    let title = escape_html(title);
    let generated_at = escape_html(generated_at);
    let plotly_js = Plot::offline_js_sources();
    let payload = serde_json::to_string(dataset)?;

    Ok(format!(
        r##"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>{title}</title>
    <link href="https://fonts.googleapis.com/css2?family=Open+Sans:wght@400;600;700&family=Material+Symbols+Outlined:opsz,wght,FILL,GRAD@20,300,0,0" rel="stylesheet">
    {plotly_js}
    <style>
      :root {{
        --font-family-open-sans: 'Open Sans', arial, sans-serif;
        --material-symbol-font: 'Material Symbols Rounded';
        --efx-brand-red: #9e1b32;
        --efx-brand-gray: #333e48;
        --efx-primary-red: #9e1b32;
        --efx-primary-orange: #e77204;
        --efx-primary-yellow: #f1c319;
        --efx-primary-green: #45842a;
        --efx-primary-blue: #007298;
        --efx-primary-purple: #652f6c;
        --efx-black: #000000;
        --efx-white: #ffffff;
        --efx-gray: #333E48;
        --efx-gray-100: #e7e7e7;
        --efx-gray-200: #cfcfcf;
        --efx-gray-300: #b5b5b5;
        --efx-gray-400: #9c9c9c;
        --efx-gray-500: #828282;
        --efx-gray-600: #696969;
        --efx-gray-700: #4f4f4f;
        --efx-gray-800: #363636;
        --efx-gray-900: #1c1c1c;
        --efx-page-background: #f7f7f7;
        --efx-borders: var(--efx-gray-400);
        --efx-light-borders: var(--efx-gray-200);
        --efx-dark-borders: var(--efx-gray-600);
        --bg: var(--efx-page-background);
        --panel: var(--efx-white);
        --panel-strong: var(--efx-white);
        --ink: var(--efx-gray);
        --muted: var(--efx-gray-600);
        --border: var(--efx-light-borders);
      }}

      * {{
        box-sizing: border-box;
      }}

      body {{
        margin: 0;
        font-family: var(--font-family-open-sans);
        color: var(--ink);
        background: var(--bg);
      }}

      .shell {{
        max-width: 1500px;
        margin: 0 auto;
        padding: 14px;
      }}

      .hero {{
        display: grid;
        gap: 10px;
        grid-template-columns: 1.4fr 0.8fr;
        margin-bottom: 12px;
      }}

      .hero-card,
      .panel {{
        background: var(--panel);
        border: 1px solid var(--border);
        border-radius: 0;
      }}

      .hero-card {{
        padding: 14px;
      }}

      .hero-chart {{
        min-height: 250px;
        margin-top: 10px;
      }}

      .eyebrow {{
        display: inline-flex;
        align-items: center;
        gap: 8px;
        padding: 4px 8px;
        border-radius: 0;
        background: var(--efx-white);
        color: var(--efx-gray-700);
        border: 1px solid var(--border);
        font-size: 12px;
        letter-spacing: 0.08em;
        text-transform: uppercase;
        font-weight: 700;
      }}

      h1 {{
        margin: 10px 0 6px;
        font-size: clamp(1.45rem, 2.1vw, 2.2rem);
        line-height: 1;
      }}

      .lede {{
        margin: 0;
        color: var(--muted);
        font-size: 0.82rem;
        line-height: 1.35;
        max-width: 70ch;
      }}

      .facts {{
        display: grid;
        grid-template-columns: repeat(2, minmax(0, 1fr));
        gap: 6px;
        padding: 6px;
      }}

      .fact {{
        background: var(--panel-strong);
        border: 1px solid var(--border);
        border-radius: 0;
        padding: 8px 10px;
        min-height: 72px;
      }}

      .fact-label {{
        color: var(--muted);
        font-size: 0.68rem;
        margin-bottom: 4px;
      }}

      .fact-value {{
        font-size: 1.45rem;
        font-weight: 700;
        line-height: 1.05;
      }}

      .tabs {{
        display: flex;
        flex-wrap: wrap;
        gap: 6px;
        margin-bottom: 10px;
      }}

      .owner-filters {{
        display: flex;
        flex-wrap: wrap;
        gap: 6px;
        margin-bottom: 10px;
      }}

      .owner-button {{
        border: 1px solid var(--border);
        background: var(--efx-white);
        color: var(--ink);
        padding: 7px 10px;
        border-radius: 0;
        font: inherit;
        font-size: 0.76rem;
        cursor: pointer;
      }}

      .owner-button.active {{
        color: var(--efx-white);
        border-color: transparent;
      }}

      .tab-button {{
        border: 1px solid var(--border);
        background: var(--efx-white);
        color: var(--muted);
        padding: 8px 12px;
        border-radius: 0;
        font: inherit;
        cursor: pointer;
        transition: transform 120ms ease, background 120ms ease, color 120ms ease;
      }}

      .tab-button.active {{
        background: var(--efx-gray);
        color: var(--efx-white);
        transform: translateY(-1px);
      }}

      .page {{
        display: none;
        gap: 10px;
      }}

      .page.active {{
        display: grid;
      }}

      .grid-2 {{
        display: grid;
        grid-template-columns: repeat(2, minmax(0, 1fr));
        gap: 10px;
      }}

      .grid-3 {{
        display: grid;
        grid-template-columns: repeat(3, minmax(0, 1fr));
        gap: 10px;
      }}

      .panel {{
        padding: 10px 10px 6px;
      }}

      .panel h2 {{
        margin: 0 0 4px;
        font-size: 0.9rem;
      }}

      .panel p {{
        margin: 0 0 8px;
        color: var(--muted);
        line-height: 1.3;
        font-size: 0.74rem;
      }}

      .chart {{
        width: 100%;
        min-height: 300px;
      }}

      .chart.tall {{
        min-height: 380px;
      }}

      .filters {{
        display: flex;
        flex-wrap: wrap;
        gap: 6px;
        margin-bottom: 8px;
      }}

      .toggle-group {{
        display: inline-flex;
        gap: 6px;
        margin-bottom: 8px;
        flex-wrap: wrap;
      }}

      .search {{
        width: 100%;
        padding: 9px 10px;
        border-radius: 0;
        border: 1px solid var(--border);
        background: var(--efx-white);
        font: inherit;
        color: var(--ink);
      }}

      .pill {{
        display: inline-flex;
        align-items: center;
        gap: 8px;
        padding: 6px 8px;
        border-radius: 0;
        background: var(--efx-white);
        border: 1px solid var(--border);
        color: var(--ink);
        font-size: 0.75rem;
      }}

      .table-wrap {{
        overflow: auto;
        border: 1px solid var(--border);
        border-radius: 0;
        background: var(--efx-white);
      }}

      table {{
        width: 100%;
        border-collapse: collapse;
      }}

      th,
      td {{
        padding: 9px 10px;
        text-align: left;
        border-bottom: 1px solid var(--efx-light-borders);
        font-size: 0.82rem;
        vertical-align: top;
      }}

      th {{
        position: sticky;
        top: 0;
        background: var(--efx-white);
        z-index: 1;
      }}

      tr:last-child td {{
        border-bottom: none;
      }}

      .mono {{
        font-family: "Open Sans", arial, sans-serif;
        font-size: 0.86rem;
      }}

      .empty {{
        color: var(--muted);
        padding: 10px 0 4px;
      }}

      @media (max-width: 980px) {{
        .hero,
        .grid-2,
        .grid-3 {{
          grid-template-columns: 1fr;
        }}
      }}
    </style>
  </head>
  <body>
    <div class="shell">
      <section class="hero">
        <div class="hero-card">
          <div class="eyebrow">Pulse Report</div>
          <h1>{title}</h1>
          <p class="lede">
            Self-contained interactive report generated from the persisted `pulse` state.
            Use the page buttons to move across dimensions, click chart elements to pivot into related views,
            and keep the whole report in a single HTML file.
          </p>
          <p class="lede" style="margin-top: 8px;">
            Generated at {generated_at}
          </p>
          <div id="hero-owner-chart" class="chart hero-chart"></div>
        </div>
        <aside class="hero-card">
          <div class="facts" id="summary-cards"></div>
        </aside>
      </section>

      <nav class="tabs" id="tabs">
        <button class="tab-button active" data-tab="overview">AI Docs</button>
        <button class="tab-button" data-tab="repositories">Repositories</button>
        <button class="tab-button" data-tab="history">History</button>
        <button class="tab-button" data-tab="languages">Languages</button>
        <button class="tab-button" data-tab="failures">Failures</button>
      </nav>
      <div class="owner-filters" id="owner-filters"></div>
      <div class="toggle-group" id="ai-doc-grouping"></div>

      <section class="page active" id="page-overview">
        <div class="grid-2">
          <article class="panel">
            <h2>Common AI Conventions By Team</h2>
            <p>Shared markdown contracts such as `AGENTS.md`, `SKILL.md`, `skills.md`, `SPEC.md`, and related files, broken down by team.</p>
            <div id="overview-ai-docs" class="chart tall"></div>
          </article>
          <article class="panel">
            <h2>Linked Structure From AI Docs</h2>
            <p>Markdown files referenced from agent-oriented entrypoints. This is the quickest way to see the shared operating structure.</p>
            <div id="overview-ai-links" class="chart tall"></div>
          </article>
        </div>
        <div class="grid-2">
          <article class="panel">
            <h2>AI Convention Adoption Over Time</h2>
            <p>Cumulative repository adoption by file-name convention, based on the first introduction of the document.</p>
            <div id="overview-ai-timeline" class="chart tall"></div>
          </article>
          <article class="panel">
            <h2>Teams With AI Docs</h2>
            <p>Repository count per team where at least one AI convention was detected. Colors stay fixed across team-oriented charts.</p>
            <div id="overview-owner-coverage" class="chart tall"></div>
          </article>
        </div>
      </section>

      <section class="page" id="page-languages">
        <div class="grid-2">
          <article class="panel">
            <h2>Language Share by Bytes</h2>
            <p>Top languages in the current snapshots. Click a language to filter repositories.</p>
            <div id="languages-bytes" class="chart tall"></div>
          </article>
          <article class="panel">
            <h2>Extension Share by Bytes</h2>
            <p>Useful for spotting large binary or data-heavy segments.</p>
            <div id="languages-extensions" class="chart tall"></div>
          </article>
        </div>
      </section>

      <section class="page" id="page-repositories">
        <article class="panel">
          <h2>Repository Browser</h2>
          <p>Filter by dominant language, repository name, or chart clicks from other sections.</p>
          <div class="filters">
            <span class="pill" id="repo-filter-doc">AI doc filter: all</span>
            <span class="pill" id="repo-filter-owner">Team filter: all</span>
            <span class="pill" id="repo-filter-language">Language filter: all</span>
            <span class="pill" id="repo-filter-selection">Selected repo: none</span>
          </div>
          <input class="search" id="repo-search" type="search" placeholder="Filter repositories by name, team, owner, or language">
          <div class="table-wrap" style="margin-top: 14px;">
            <table>
              <thead>
                <tr>
                  <th>Repository</th>
                  <th>Team</th>
                  <th>AI docs</th>
                  <th>Language</th>
                  <th>Files</th>
                  <th>Bytes</th>
                  <th>Lines</th>
                </tr>
              </thead>
              <tbody id="repo-table-body"></tbody>
            </table>
          </div>
        </article>
      </section>

      <section class="page" id="page-history">
        <div class="grid-2">
          <article class="panel">
            <h2>Weekly Activity By Team</h2>
            <p>One cumulative line per team, limited to the top 10 teams by total commit volume.</p>
            <div id="history-activity" class="chart tall"></div>
          </article>
          <article class="panel">
            <h2>Contributor Pressure</h2>
            <p>Contributor instances per active week across repositories.</p>
            <div id="history-contributors" class="chart tall"></div>
          </article>
        </div>
        <article class="panel">
          <h2>AI Doc Commits By Team</h2>
          <p>One cumulative line per team for commits that touched files classified as AI docs.</p>
          <div id="history-ai-doc-owners" class="chart tall"></div>
        </article>
      </section>

      <section class="page" id="page-failures">
        <article class="panel">
          <h2>Failure Ledger</h2>
          <p>Failures persisted in checkpoints. These remain visible even after the rest of the batch succeeds.</p>
          <div class="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>Repository</th>
                  <th>Stage</th>
                  <th>Detail</th>
                </tr>
              </thead>
              <tbody id="failure-table-body"></tbody>
            </table>
          </div>
          <div id="failure-empty" class="empty" hidden>No failures recorded in this state directory.</div>
        </article>
      </section>
    </div>

    <script>
      const report = {payload};
      const state = {{
        activeTab: "overview",
        repoDoc: null,
        repoOwner: null,
        repoLanguage: null,
        selectedRepo: null,
        repoSearch: "",
        aiDocGrouping: "name"
      }};

      const config = {{
        displayModeBar: true,
        responsive: true
      }};

      const ownerPalette = [
        "#9e1b32",
        "#e77204",
        "#f1c319",
        "#45842a",
        "#007298",
        "#652f6c",
        "#6d1222",
        "#294d19",
        "#004d66",
        "#4f4f4f"
      ];
      const reposByKey = new Map(report.repositories.map((repo) => [repo.repo_key, repo]));

      function aiDocKey(entry) {{
        return state.aiDocGrouping === "path"
          ? entry.path.toLowerCase()
          : entry.doc_name;
      }}

      function aiDocLabel(entry) {{
        return state.aiDocGrouping === "path"
          ? entry.path
          : entry.doc_name;
      }}

      function aiDocLabelFromTimeline(entry) {{
        return state.aiDocGrouping === "path"
          ? entry.path
          : entry.doc_name;
      }}

      function docsByRepoMap() {{
        const docsByRepo = new Map();
        report.ai_doc_occurrences.forEach((entry) => {{
          const values = docsByRepo.get(entry.repo_key) || [];
          values.push(aiDocLabel(entry));
          docsByRepo.set(entry.repo_key, values);
        }});
        return docsByRepo;
      }}

      function repoGroup(repo) {{
        return repo.team || repo.owner;
      }}

      function repoGroupColor(repo) {{
        return repo.team_color || repo.owner_color;
      }}

      function overviewGroup(entry) {{
        return entry.team || entry.owner;
      }}

      const owners = (() => {{
        const byOwner = new Map();
        report.repositories.forEach((repo) => {{
          const group = repoGroup(repo);
          const entry = byOwner.get(group) || {{
            owner: repo.owner,
            owner_color: repo.owner_color,
            team: repo.team,
            team_color: repo.team_color,
            repositories: 0,
            aiRepos: new Set(),
            totalFiles: 0,
            totalLines: 0
          }};
          entry.repositories += 1;
          entry.totalFiles += repo.total_files;
          entry.totalLines += repo.total_lines;
          byOwner.set(group, entry);
        }});
        report.ai_doc_occurrences.forEach((entry) => {{
          const repo = reposByKey.get(entry.repo_key);
          if (!repo) {{
            return;
          }}
          byOwner.get(repoGroup(repo))?.aiRepos.add(entry.repo_key);
        }});
        return [...byOwner.values()]
          .map((entry) => ({{
            owner: entry.owner,
            owner_color: entry.owner_color,
            team: entry.team,
            team_color: entry.team_color,
            repositories: entry.repositories,
            aiRepositories: entry.aiRepos.size,
            totalFiles: entry.totalFiles,
            totalLines: entry.totalLines
          }}))
          .sort((a, b) => b.repositories - a.repositories || overviewGroup(a).localeCompare(overviewGroup(b)));
      }})();

      const ownerColors = new Map(owners.map((entry, index) => [
        overviewGroup(entry),
        entry.team_color || entry.owner_color || ownerPalette[index % ownerPalette.length]
      ]));

      function ownerColor(owner) {{
        return ownerColors.get(owner) || "#333E48";
      }}

      function formatInt(value) {{
        return new Intl.NumberFormat("en-US").format(value || 0);
      }}

      function formatBytes(value) {{
        const units = ["B", "KB", "MB", "GB", "TB"];
        let size = Number(value || 0);
        let unit = 0;
        while (size >= 1024 && unit < units.length - 1) {{
          size /= 1024;
          unit += 1;
        }}
        return `${{size.toFixed(size >= 10 || unit === 0 ? 0 : 1)}} ${{units[unit]}}`;
      }}

      function setTab(tab) {{
        state.activeTab = tab;
        document.querySelectorAll(".tab-button").forEach((button) => {{
          button.classList.toggle("active", button.dataset.tab === tab);
        }});
        document.querySelectorAll(".page").forEach((page) => {{
          page.classList.toggle("active", page.id === `page-${{tab}}`);
        }});
        window.scrollTo({{ top: 0, behavior: "smooth" }});
      }}

      function renderSummary() {{
        const visibleRepos = state.repoOwner
          ? report.repositories.filter((repo) => repoGroup(repo) === state.repoOwner)
          : report.repositories;
        const visibleRepoKeys = new Set(visibleRepos.map((repo) => repo.repo_key));
        const visibleOccurrences = report.ai_doc_occurrences.filter((entry) => visibleRepoKeys.has(entry.repo_key));
        const aiRepos = new Set(visibleOccurrences.map((entry) => entry.repo_key)).size;
        const uniqueAiDocs = new Set(visibleOccurrences.map((entry) => aiDocKey(entry))).size;
        const agents = new Set(
          visibleOccurrences
            .filter((entry) => state.aiDocGrouping === "path" ? entry.path.toLowerCase().endsWith("/agents.md") || entry.path.toLowerCase() === "agents.md" : entry.doc_name === "agents.md")
            .map((entry) => entry.repo_key)
        ).size;
        const visibleOwners = state.repoOwner
          ? owners.filter((entry) => overviewGroup(entry) === state.repoOwner)
          : owners;
        const largestOwner = visibleOwners[0];
        const failureCount = report.failures.filter((failure) => visibleRepoKeys.has(failure.repo_key)).length;
        const cards = [
          ["Teams Processed", formatInt(visibleOwners.length)],
          ["Repositories", formatInt(visibleRepos.length)],
          ["Largest Team", largestOwner ? `${{overviewGroup(largestOwner)}} · ${{formatInt(largestOwner.repositories)}} repos` : "n/a"],
          ["Repos With AI Docs", formatInt(aiRepos)],
          [`Unique AI Docs (${{state.aiDocGrouping === "path" ? "path" : "name"}})`, formatInt(uniqueAiDocs)],
          ["Repos With AGENTS.md", formatInt(agents)],
          ["Failures", formatInt(failureCount)],
        ];
        document.getElementById("summary-cards").innerHTML = cards.map(([label, value]) => `
          <div class="fact">
            <div class="fact-label">${{label}}</div>
            <div class="fact-value">${{value}}</div>
          </div>
        `).join("");
      }}

      function renderOwnerFilters() {{
        const container = document.getElementById("owner-filters");
        const allButton = `
          <button class="owner-button ${{state.repoOwner ? "" : "active"}}" data-owner="">
            All teams
          </button>
        `;
        const ownerButtons = owners.map((entry) => `
          <button
            class="owner-button ${{state.repoOwner === overviewGroup(entry) ? "active" : ""}}"
            data-owner="${{overviewGroup(entry)}}"
            style="background:${{state.repoOwner === overviewGroup(entry) ? ownerColor(overviewGroup(entry)) : "#ffffff"}}; border-color:${{ownerColor(overviewGroup(entry))}};"
          >
            ${{overviewGroup(entry)}}
          </button>
        `).join("");
        container.innerHTML = allButton + ownerButtons;
        container.querySelectorAll(".owner-button").forEach((button) => {{
          button.addEventListener("click", () => {{
            state.repoOwner = button.dataset.owner || null;
            renderAll();
          }});
        }});
      }}

      function renderAiDocGrouping() {{
        const container = document.getElementById("ai-doc-grouping");
        const options = [
          {{ value: "name", label: "Group AI docs by name" }},
          {{ value: "path", label: "Group AI docs by path" }}
        ];
        container.innerHTML = options.map((option) => `
          <button class="owner-button ${{state.aiDocGrouping === option.value ? "active" : ""}}" data-grouping="${{option.value}}">
            ${{option.label}}
          </button>
        `).join("");
        container.querySelectorAll(".owner-button").forEach((button) => {{
          button.addEventListener("click", () => {{
            state.aiDocGrouping = button.dataset.grouping || "name";
            state.repoDoc = null;
            renderAll();
          }});
        }});
      }}

      function renderRepoTable() {{
        const body = document.getElementById("repo-table-body");
        const docLabel = document.getElementById("repo-filter-doc");
        const ownerLabel = document.getElementById("repo-filter-owner");
        const languageLabel = document.getElementById("repo-filter-language");
        const selectionLabel = document.getElementById("repo-filter-selection");
        const docsByRepo = docsByRepoMap();
        docLabel.textContent = `AI doc filter: ${{state.repoDoc || "all"}}`;
        ownerLabel.textContent = `Team filter: ${{state.repoOwner || "all"}}`;
        languageLabel.textContent = `Language filter: ${{state.repoLanguage || "all"}}`;
        selectionLabel.textContent = `Selected repo: ${{state.selectedRepo || "none"}}`;

        const term = state.repoSearch.trim().toLowerCase();
        const filtered = report.repositories.filter((repo) => {{
          const repoDocs = docsByRepo.get(repo.repo_key) || [];
          if (state.repoDoc && !repoDocs.includes(state.repoDoc)) {{
            return false;
          }}
          if (state.repoOwner && repoGroup(repo) !== state.repoOwner) {{
            return false;
          }}
          if (state.repoLanguage && repo.dominant_language !== state.repoLanguage) {{
            return false;
          }}
          if (state.selectedRepo && repo.repo_key !== state.selectedRepo) {{
            return false;
          }}
          if (!term) {{
            return true;
          }}
          const haystack = `${{repo.repo_key}} ${{repoGroup(repo)}} ${{repo.owner}} ${{repo.name}} ${{repo.dominant_language}} ${{repoDocs.join(" ")}}`.toLowerCase();
          return haystack.includes(term);
        }});

        body.innerHTML = filtered.map((repo) => `
          <tr>
            <td class="mono">${{repo.repo_key}}</td>
            <td>${{repoGroup(repo)}}</td>
            <td>${{[...new Set(docsByRepo.get(repo.repo_key) || [])].join(", ") || "none"}}</td>
            <td>${{repo.dominant_language}}</td>
            <td>${{formatInt(repo.total_files)}}</td>
            <td>${{formatBytes(repo.total_bytes)}}</td>
            <td>${{formatInt(repo.total_lines)}}</td>
          </tr>
        `).join("");

        if (!filtered.length) {{
          body.innerHTML = `<tr><td colspan="7" class="empty">No repositories match the current filter.</td></tr>`;
        }}
      }}

      function renderFailureTable() {{
        const body = document.getElementById("failure-table-body");
        const empty = document.getElementById("failure-empty");
        if (!report.failures.length) {{
          empty.hidden = false;
          body.innerHTML = "";
          return;
        }}
        empty.hidden = true;
        body.innerHTML = report.failures.map((failure) => `
          <tr>
            <td class="mono">${{failure.repo_key}}</td>
            <td>${{failure.stage}}</td>
            <td class="mono">${{failure.detail}}</td>
          </tr>
        `).join("");
      }}

      function stageSeries() {{
        const stages = [...new Set(report.stage_statuses.map((entry) => entry.stage))];
        const statuses = [...new Set(report.stage_statuses.map((entry) => entry.status))];
        return statuses.map((status) => ({{
          type: "bar",
          name: status,
          x: stages,
          y: stages.map((stage) => {{
            const match = report.stage_statuses.find((entry) => entry.stage === stage && entry.status === status);
            return match ? match.count : 0;
          }})
        }}));
      }}

      function renderCharts() {{
        const visibleRepos = state.repoOwner
          ? report.repositories.filter((repo) => repoGroup(repo) === state.repoOwner)
          : report.repositories;
        const visibleRepoKeys = new Set(visibleRepos.map((repo) => repo.repo_key));
        const visibleOwners = state.repoOwner
          ? owners.filter((entry) => overviewGroup(entry) === state.repoOwner)
          : owners;
        const visibleOccurrences = report.ai_doc_occurrences.filter((entry) => visibleRepoKeys.has(entry.repo_key));
        const aiSummaryMap = new Map();
        visibleOccurrences.forEach((entry) => {{
          const key = aiDocKey(entry);
          const current = aiSummaryMap.get(key) || {{ key, label: aiDocLabel(entry), repositories: new Set(), files: 0 }};
          current.repositories.add(entry.repo_key);
          current.files += 1;
          aiSummaryMap.set(key, current);
        }});
        const aiSummaries = [...aiSummaryMap.values()]
          .map((entry) => ({{
            key: entry.key,
            doc_name: entry.label,
            repositories: entry.repositories.size,
            files: entry.files
          }}))
          .sort((a, b) => b.repositories - a.repositories || a.doc_name.localeCompare(b.doc_name))
          .slice(0, 10)
          .reverse();
        const aiLinkMap = new Map();
        visibleOccurrences.forEach((entry) => {{
          entry.linked_docs.forEach((linkedDoc) => {{
            const key = `${{aiDocLabel(entry)}}::${{linkedDoc}}`;
            const repos = aiLinkMap.get(key) || new Set();
            repos.add(entry.repo_key);
            aiLinkMap.set(key, repos);
          }});
        }});
        const aiLinks = [...aiLinkMap.entries()]
          .map(([key, repos]) => {{
            const [source_doc, linked_doc] = key.split("::");
            return {{ source_doc, linked_doc, repositories: repos.size }};
          }})
          .sort((a, b) => b.repositories - a.repositories || a.linked_doc.localeCompare(b.linked_doc))
          .slice(0, 12)
          .reverse();
        const visibleTimeline = report.ai_doc_timeline
          .filter((entry) => {{
            if (state.aiDocGrouping === "path") {{
              return visibleOccurrences.some((occurrence) =>
                occurrence.repo_key &&
                occurrence.path === entry.path &&
                visibleRepoKeys.has(occurrence.repo_key)
              );
            }}
            return visibleOccurrences.some((occurrence) =>
              occurrence.repo_key &&
              occurrence.doc_name === entry.doc_name &&
              visibleRepoKeys.has(occurrence.repo_key)
            );
          }});
        const timelineTotals = new Map();
        visibleTimeline.forEach((entry) => {{
          const label = aiDocLabelFromTimeline(entry);
          timelineTotals.set(label, Math.max(timelineTotals.get(label) || 0, entry.cumulative_repositories));
        }});
        const topTimelineDocs = [...timelineTotals.entries()]
          .sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]))
          .slice(0, 4)
          .map(([label]) => label);
        const aiDocCountsByOwner = new Map();
        visibleOccurrences.forEach((entry) => {{
          const repo = reposByKey.get(entry.repo_key);
          if (!repo) {{
            return;
          }}
          const key = `${{repoGroup(repo)}}::${{aiDocKey(entry)}}`;
          aiDocCountsByOwner.set(key, (aiDocCountsByOwner.get(key) || 0) + 1);
        }});

        Plotly.newPlot("hero-owner-chart", [{{
          type: "bar",
          orientation: "h",
          x: [...visibleOwners].reverse().map((entry) => entry.repositories),
          y: [...visibleOwners].reverse().map((entry) => overviewGroup(entry)),
          customdata: [...visibleOwners].reverse().map((entry) => overviewGroup(entry)),
          marker: {{
            color: [...visibleOwners].reverse().map((entry) => ownerColor(overviewGroup(entry)))
          }},
          text: [...visibleOwners].reverse().map((entry) => `${{entry.repositories}} repos`),
          textposition: "outside",
          hovertemplate: "%{{y}}<br>%{{x}} repositories processed<extra></extra>"
        }}], {{
          paper_bgcolor: "rgba(0,0,0,0)",
          plot_bgcolor: "rgba(0,0,0,0)",
          margin: {{ t: 12, r: 48, b: 40, l: 116 }},
          xaxis: {{ title: "Repositories processed" }},
          yaxis: {{ title: "", tickfont: {{ size: 11 }}, ticklabelstandoff: 14, automargin: true }}
        }}, config);

        Plotly.newPlot("overview-ai-docs", visibleOwners.map((ownerEntry) => ({{
          type: "bar",
          orientation: "h",
          name: overviewGroup(ownerEntry),
          x: aiSummaries.map((entry) => aiDocCountsByOwner.get(`${{overviewGroup(ownerEntry)}}::${{entry.key}}`) || 0),
          y: aiSummaries.map((entry) => entry.doc_name),
          customdata: aiSummaries.map((entry) => [entry.doc_name, overviewGroup(ownerEntry)]),
          marker: {{
            color: ownerColor(overviewGroup(ownerEntry))
          }},
          hovertemplate: "%{{y}}<br>%{{x}} files for %{{data.name}}<extra></extra>"
        }})), {{
          barmode: "stack",
          paper_bgcolor: "rgba(0,0,0,0)",
          plot_bgcolor: "rgba(0,0,0,0)",
          margin: {{ t: 12, r: 16, b: 48, l: 184 }},
          xaxis: {{ title: "AI convention files" }},
          yaxis: {{ tickfont: {{ size: 11 }}, ticklabelstandoff: 14, automargin: true }},
          legend: {{ orientation: "h" }}
        }}, config);

        Plotly.newPlot("overview-ai-links", [
          {{
            type: "bar",
            orientation: "h",
            x: aiLinks.map((entry) => entry.repositories),
            y: aiLinks.map((entry) => `${{entry.source_doc}} -> ${{entry.linked_doc}}`),
            marker: {{ color: "#007298" }},
            hovertemplate: "%{{y}}<br>%{{x}} repositories<extra></extra>"
          }}
        ], {{
          paper_bgcolor: "rgba(0,0,0,0)",
          plot_bgcolor: "rgba(0,0,0,0)",
          margin: {{ t: 12, r: 16, b: 48, l: 220 }},
          xaxis: {{ title: "Repositories" }},
          yaxis: {{ tickfont: {{ size: 11 }}, ticklabelstandoff: 14, automargin: true }}
        }}, config);

        Plotly.newPlot("overview-ai-timeline", topTimelineDocs.map((docName) => {{
          const rows = visibleTimeline.filter((entry) => aiDocLabelFromTimeline(entry) === docName);
          return {{
            type: "scatter",
            mode: "lines+markers",
            name: docName,
            x: rows.map((entry) => entry.week_start),
            y: rows.map((entry) => entry.cumulative_repositories)
          }};
        }}), {{
          paper_bgcolor: "rgba(0,0,0,0)",
          plot_bgcolor: "rgba(0,0,0,0)",
          margin: {{ t: 12, r: 16, b: 48, l: 56 }},
          xaxis: {{ title: "Week start" }},
          yaxis: {{ title: "Cumulative repositories" }},
          legend: {{ orientation: "h" }}
        }}, config);

        Plotly.newPlot("overview-owner-coverage", [
          {{
            type: "bar",
            orientation: "h",
            x: [...visibleOwners].reverse().map((entry) => entry.aiRepositories),
            y: [...visibleOwners].reverse().map((entry) => overviewGroup(entry)),
            customdata: [...visibleOwners].reverse().map((entry) => overviewGroup(entry)),
            marker: {{
              color: [...visibleOwners].reverse().map((entry) => ownerColor(overviewGroup(entry)))
            }},
            text: [...visibleOwners].reverse().map((entry) => `${{entry.aiRepositories}} / ${{entry.repositories}}`),
            textposition: "outside",
            hovertemplate: "%{{y}}<br>%{{x}} repositories with AI docs<extra></extra>"
          }}
        ], {{
          paper_bgcolor: "rgba(0,0,0,0)",
          plot_bgcolor: "rgba(0,0,0,0)",
          margin: {{ t: 12, r: 56, b: 40, l: 116 }},
          xaxis: {{ title: "Repositories with AI docs" }},
          yaxis: {{ title: "", tickfont: {{ size: 11 }}, ticklabelstandoff: 14, automargin: true }}
        }}, config);

        Plotly.newPlot("languages-bytes", [
          {{
            type: "bar",
            orientation: "h",
            x: [...report.languages].reverse().map((entry) => entry.bytes),
            y: [...report.languages].reverse().map((entry) => entry.language),
            customdata: [...report.languages].reverse().map((entry) => entry.language),
            marker: {{ color: "#007298" }},
            hovertemplate: "%{{y}}<br>%{{x:,}} bytes<extra></extra>"
          }}
        ], {{
          paper_bgcolor: "rgba(0,0,0,0)",
          plot_bgcolor: "rgba(0,0,0,0)",
          margin: {{ t: 12, r: 16, b: 50, l: 130 }},
          xaxis: {{ title: "Bytes" }},
          yaxis: {{ tickfont: {{ size: 11 }}, ticklabelstandoff: 14, automargin: true }}
        }}, config);

        Plotly.newPlot("languages-extensions", [
          {{
            type: "bar",
            x: report.extensions.map((entry) => entry.extension),
            y: report.extensions.map((entry) => entry.bytes),
            marker: {{ color: "#e77204" }},
            hovertemplate: "%{{x}}<br>%{{y:,}} bytes<extra></extra>"
          }}
        ], {{
          paper_bgcolor: "rgba(0,0,0,0)",
          plot_bgcolor: "rgba(0,0,0,0)",
          margin: {{ t: 12, r: 16, b: 80, l: 56 }},
          xaxis: {{ title: "Extension", tickangle: -30 }},
          yaxis: {{ title: "Bytes" }}
        }}, config);

        const topHistoryOwners = (() => {{
          const totals = new Map();
          report.owner_weekly_overview.forEach((entry) => {{
            const group = overviewGroup(entry);
            if (state.repoOwner && group !== state.repoOwner) {{
              return;
            }}
            totals.set(group, (totals.get(group) || 0) + entry.commits);
          }});
          return [...totals.entries()]
            .sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]))
            .slice(0, 10)
            .map(([owner]) => owner);
        }})();

        const ownerActivitySeries = topHistoryOwners
          .map((owner) => {{
            const rows = report.owner_weekly_overview.filter((entry) => overviewGroup(entry) === owner);
            let runningTotal = 0;
            return {{
              type: "scatter",
              mode: "lines",
              name: owner,
              x: rows.map((entry) => entry.week_start),
              y: rows.map((entry) => {{
                runningTotal += entry.commits;
                return runningTotal;
              }}),
              line: {{ color: ownerColor(owner), width: 3 }}
            }};
          }})
          .filter((entry) => entry.x.length > 0);

        Plotly.newPlot("history-activity", ownerActivitySeries, {{
          paper_bgcolor: "rgba(0,0,0,0)",
          plot_bgcolor: "rgba(0,0,0,0)",
          margin: {{ t: 12, r: 16, b: 50, l: 56 }},
          xaxis: {{ title: "Week start" }},
          yaxis: {{ title: "Cumulative commits" }},
          legend: {{ orientation: "h" }}
        }}, config);

        Plotly.newPlot("history-contributors", [
          {{
            type: "bar",
            x: report.weekly_overview.map((entry) => entry.week_start),
            y: report.weekly_overview.map((entry) => entry.contributor_instances),
            marker: {{ color: "#9e1b32" }},
            hovertemplate: "%{{x}}<br>%{{y:,}} contributor instances<extra></extra>"
          }}
        ], {{
          paper_bgcolor: "rgba(0,0,0,0)",
          plot_bgcolor: "rgba(0,0,0,0)",
          margin: {{ t: 12, r: 16, b: 50, l: 56 }},
          xaxis: {{ title: "Week start" }},
          yaxis: {{ title: "Contributor instances" }}
        }}, config);

        const aiDocOwnerSeries = visibleOwners
          .map((ownerEntry) => {{
            const rows = report.ai_doc_owner_weekly.filter((entry) => overviewGroup(entry) === overviewGroup(ownerEntry));
            let runningTotal = 0;
            return {{
              type: "scatter",
              mode: "lines",
              name: overviewGroup(ownerEntry),
              x: rows.map((entry) => entry.week_start),
              y: rows.map((entry) => {{
                runningTotal += entry.commits;
                return runningTotal;
              }}),
              line: {{ color: ownerColor(overviewGroup(ownerEntry)), width: 3 }}
            }};
          }})
          .filter((entry) => entry.x.length > 0);

        Plotly.newPlot("history-ai-doc-owners", aiDocOwnerSeries, {{
          paper_bgcolor: "rgba(0,0,0,0)",
          plot_bgcolor: "rgba(0,0,0,0)",
          margin: {{ t: 12, r: 16, b: 50, l: 56 }},
          xaxis: {{ title: "Week start" }},
          yaxis: {{ title: "Cumulative AI doc commits" }},
          legend: {{ orientation: "h" }}
        }}, config);

        document.getElementById("hero-owner-chart").on("plotly_click", (event) => {{
          state.repoDoc = null;
          state.repoOwner = event.points?.[0]?.customdata || null;
          state.repoLanguage = null;
          state.selectedRepo = null;
          renderRepoTable();
          setTab("repositories");
        }});

        document.getElementById("languages-bytes").on("plotly_click", (event) => {{
          const language = event.points?.[0]?.customdata;
          state.repoDoc = null;
          state.repoOwner = state.repoOwner;
          state.repoLanguage = language || null;
          state.selectedRepo = null;
          renderRepoTable();
          setTab("repositories");
        }});

        document.getElementById("overview-ai-docs").on("plotly_click", (event) => {{
          state.repoDoc = event.points?.[0]?.customdata?.[0] || null;
          state.repoOwner = event.points?.[0]?.customdata?.[1] || null;
          state.repoLanguage = null;
          state.selectedRepo = null;
          renderRepoTable();
          setTab("repositories");
        }});

        document.getElementById("overview-owner-coverage").on("plotly_click", (event) => {{
          state.repoDoc = null;
          state.repoOwner = event.points?.[0]?.customdata || null;
          state.repoLanguage = null;
          state.selectedRepo = null;
          renderRepoTable();
          setTab("repositories");
        }});
      }}

      document.querySelectorAll(".tab-button").forEach((button) => {{
        button.addEventListener("click", () => setTab(button.dataset.tab));
      }});

      document.getElementById("repo-search").addEventListener("input", (event) => {{
        state.repoSearch = event.target.value;
        renderRepoTable();
      }});

      function renderAll() {{
        renderOwnerFilters();
        renderAiDocGrouping();
        renderSummary();
        renderRepoTable();
        renderFailureTable();
        renderCharts();
      }}

      renderAll();
    </script>
  </body>
</html>
"##
    ))
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
