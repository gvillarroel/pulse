use anyhow::{Context, Result};
use pulse_core::{
    ExtensionBreakdown, FailureClass, FailureRecord, FetchOutcome, FileSnapshot, LanguageBreakdown,
    OwnerWeeklyOverview, RepoOverview, RepoSnapshot, RepoTarget, ReportDataset, ReportSummary,
    RunSummary, StageStatus, StageStatusCount, StateLayout, WeeklyEvolution, WeeklyOverview,
};
use rusqlite::{Connection, OptionalExtension, params};
use std::path::Path;

pub struct Store {
    conn: Connection,
}

impl Store {
    pub fn open(layout: &StateLayout) -> Result<Self> {
        layout.ensure()?;
        let conn = Connection::open(&layout.db_path)
            .with_context(|| format!("failed to open {}", layout.db_path.display()))?;
        let store = Self { conn };
        store.init_schema()?;
        Ok(store)
    }

    pub fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            PRAGMA foreign_keys = ON;
            CREATE TABLE IF NOT EXISTS runs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                started_at TEXT NOT NULL,
                finished_at TEXT,
                command TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS repositories (
                repo_key TEXT PRIMARY KEY,
                provider TEXT NOT NULL,
                owner TEXT NOT NULL,
                owner_color TEXT,
                name TEXT NOT NULL,
                url TEXT NOT NULL,
                default_branch TEXT,
                active INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS repository_targets (
                repo_key TEXT PRIMARY KEY,
                tags_json TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS repo_stage_checkpoints (
                repo_key TEXT NOT NULL,
                stage TEXT NOT NULL,
                status TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                detail TEXT,
                PRIMARY KEY (repo_key, stage)
            );
            CREATE TABLE IF NOT EXISTS fetch_state (
                repo_key TEXT PRIMARY KEY,
                remote_url TEXT NOT NULL,
                git_dir TEXT NOT NULL,
                fetched_revision TEXT NOT NULL,
                fetched_at TEXT NOT NULL,
                backend TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS repo_snapshots (
                repo_key TEXT NOT NULL,
                revision TEXT NOT NULL,
                total_files INTEGER NOT NULL,
                total_bytes INTEGER NOT NULL,
                total_lines INTEGER NOT NULL,
                generated_at TEXT NOT NULL,
                config_hash TEXT NOT NULL,
                PRIMARY KEY (repo_key, revision, config_hash)
            );
            CREATE TABLE IF NOT EXISTS file_snapshots (
                repo_key TEXT NOT NULL,
                revision TEXT NOT NULL,
                path TEXT NOT NULL,
                language TEXT,
                extension TEXT,
                size_bytes INTEGER NOT NULL,
                line_count INTEGER NOT NULL,
                is_binary INTEGER NOT NULL,
                depth TEXT NOT NULL,
                PRIMARY KEY (repo_key, revision, path)
            );
            CREATE TABLE IF NOT EXISTS contributors (
                repo_key TEXT NOT NULL,
                contributor_key TEXT NOT NULL,
                PRIMARY KEY (repo_key, contributor_key)
            );
            CREATE TABLE IF NOT EXISTS contributor_snapshots (
                repo_key TEXT NOT NULL,
                contributor_key TEXT NOT NULL,
                commit_count INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (repo_key, contributor_key)
            );
            CREATE TABLE IF NOT EXISTS weekly_evolution (
                repo_key TEXT NOT NULL,
                week_start TEXT NOT NULL,
                commit_count INTEGER NOT NULL,
                active_contributors INTEGER NOT NULL,
                PRIMARY KEY (repo_key, week_start)
            );
            CREATE TABLE IF NOT EXISTS artifacts (
                repo_key TEXT NOT NULL,
                artifact_kind TEXT NOT NULL,
                path TEXT NOT NULL,
                PRIMARY KEY (repo_key, artifact_kind, path)
            );
        "#,
        )?;
        self.ensure_column("repositories", "owner_color", "TEXT")?;
        Ok(())
    }

    fn ensure_column(&self, table: &str, column: &str, definition: &str) -> Result<()> {
        let pragma = format!("PRAGMA table_info({table})");
        let mut stmt = self.conn.prepare(&pragma)?;
        let columns = stmt.query_map([], |row| row.get::<_, String>(1))?;
        let exists = columns
            .collect::<rusqlite::Result<Vec<_>>>()?
            .into_iter()
            .any(|name| name == column);
        if !exists {
            let alter = format!("ALTER TABLE {table} ADD COLUMN {column} {definition}");
            self.conn.execute(&alter, [])?;
        }
        Ok(())
    }

    pub fn begin_run(&self, command: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO runs (started_at, command) VALUES (?1, ?2)",
            params![chrono::Utc::now().to_rfc3339(), command],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn finish_run(&self, run_id: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE runs SET finished_at = ?2 WHERE id = ?1",
            params![run_id, chrono::Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    pub fn upsert_repository(&self, repo: &RepoTarget) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT INTO repositories (repo_key, provider, owner, owner_color, name, url, default_branch, active)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(repo_key) DO UPDATE SET
                provider=excluded.provider,
                owner=excluded.owner,
                owner_color=excluded.owner_color,
                name=excluded.name,
                url=excluded.url,
                default_branch=excluded.default_branch,
                active=excluded.active
            "#,
            params![
                repo.key(),
                repo.provider,
                repo.owner,
                repo.owner_color,
                repo.name,
                repo.url,
                repo.default_branch,
                if repo.active { 1 } else { 0 }
            ],
        )?;
        self.conn.execute(
            r#"
            INSERT INTO repository_targets (repo_key, tags_json)
            VALUES (?1, ?2)
            ON CONFLICT(repo_key) DO UPDATE SET tags_json=excluded.tags_json
            "#,
            params![repo.key(), serde_json::to_string(&repo.tags)?],
        )?;
        Ok(())
    }

    pub fn set_stage_status(
        &self,
        repo_key: &str,
        stage: &str,
        status: StageStatus,
        detail: Option<&str>,
    ) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT INTO repo_stage_checkpoints (repo_key, stage, status, updated_at, detail)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(repo_key, stage) DO UPDATE SET
                status=excluded.status,
                updated_at=excluded.updated_at,
                detail=excluded.detail
            "#,
            params![
                repo_key,
                stage,
                status.to_string(),
                chrono::Utc::now().to_rfc3339(),
                detail
            ],
        )?;
        Ok(())
    }

    pub fn record_failure(
        &self,
        repo_key: &str,
        stage: &str,
        class: FailureClass,
        message: &str,
    ) -> Result<()> {
        self.set_stage_status(
            repo_key,
            stage,
            StageStatus::Failed,
            Some(&format!("{class}: {message}")),
        )
    }

    pub fn persist_fetch(&self, outcome: &FetchOutcome) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT INTO fetch_state (repo_key, remote_url, git_dir, fetched_revision, fetched_at, backend)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ON CONFLICT(repo_key) DO UPDATE SET
                remote_url=excluded.remote_url,
                git_dir=excluded.git_dir,
                fetched_revision=excluded.fetched_revision,
                fetched_at=excluded.fetched_at,
                backend=excluded.backend
            "#,
            params![
                outcome.repo_key,
                outcome.remote_url,
                outcome.git_dir.display().to_string(),
                outcome.fetched_revision,
                outcome.fetched_at.to_rfc3339(),
                outcome.backend
            ],
        )?;
        Ok(())
    }

    pub fn persist_snapshot(&mut self, repo: &RepoSnapshot, files: &[FileSnapshot]) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute(
            r#"
            INSERT OR REPLACE INTO repo_snapshots
            (repo_key, revision, total_files, total_bytes, total_lines, generated_at, config_hash)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            params![
                repo.repo_key,
                repo.revision,
                repo.total_files as i64,
                repo.total_bytes as i64,
                repo.total_lines as i64,
                repo.generated_at.to_rfc3339(),
                repo.config_hash
            ],
        )?;

        for file in files {
            tx.execute(
                r#"
                INSERT OR REPLACE INTO file_snapshots
                (repo_key, revision, path, language, extension, size_bytes, line_count, is_binary, depth)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                "#,
                params![
                    file.repo_key,
                    file.revision,
                    file.path,
                    file.language,
                    file.extension,
                    file.size_bytes as i64,
                    file.line_count as i64,
                    if file.is_binary { 1 } else { 0 },
                    file.depth.to_string()
                ],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn fetch_state(&self, repo_key: &str) -> Result<Option<FetchOutcome>> {
        self.conn
            .query_row(
                r#"
                SELECT repo_key, remote_url, git_dir, fetched_revision, fetched_at, backend
                FROM fetch_state
                WHERE repo_key = ?1
                "#,
                params![repo_key],
                |row| {
                    Ok(FetchOutcome {
                        repo_key: row.get(0)?,
                        remote_url: row.get(1)?,
                        git_dir: Path::new(&row.get::<_, String>(2)?).to_path_buf(),
                        fetched_revision: row.get(3)?,
                        fetched_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                            .map(|value| value.with_timezone(&chrono::Utc))
                            .unwrap_or_else(|_| chrono::Utc::now()),
                        backend: row.get(5)?,
                    })
                },
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn existing_snapshot_config_hash(
        &self,
        repo_key: &str,
        revision: &str,
    ) -> Result<Option<String>> {
        self.conn
            .query_row(
                "SELECT config_hash FROM repo_snapshots WHERE repo_key = ?1 AND revision = ?2",
                params![repo_key, revision],
                |row| row.get(0),
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn summarize_run(&self, run_id: i64) -> Result<RunSummary> {
        let processed: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM repo_stage_checkpoints WHERE status = 'completed' AND stage = 'analyze'",
            [],
            |row| row.get(0),
        )?;
        let failed: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM repo_stage_checkpoints WHERE status = 'failed'",
            [],
            |row| row.get(0),
        )?;
        Ok(RunSummary {
            run_id,
            processed: processed as usize,
            failed: failed as usize,
        })
    }

    pub fn persist_weekly_evolution(&mut self, entries: &[WeeklyEvolution]) -> Result<()> {
        let tx = self.conn.transaction()?;
        for entry in entries {
            tx.execute(
                r#"
                INSERT OR REPLACE INTO weekly_evolution
                (repo_key, week_start, commit_count, active_contributors)
                VALUES (?1, ?2, ?3, ?4)
                "#,
                params![
                    entry.repo_key,
                    entry.week_start,
                    entry.commit_count as i64,
                    entry.active_contributors as i64
                ],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn fetch_outcomes(&self) -> Result<Vec<FetchOutcome>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT repo_key, remote_url, git_dir, fetched_revision, fetched_at, backend
            FROM fetch_state
            ORDER BY repo_key ASC
            "#,
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(FetchOutcome {
                repo_key: row.get(0)?,
                remote_url: row.get(1)?,
                git_dir: Path::new(&row.get::<_, String>(2)?).to_path_buf(),
                fetched_revision: row.get(3)?,
                fetched_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                    .map(|value| value.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now()),
                backend: row.get(5)?,
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    pub fn latest_file_paths(&self) -> Result<Vec<(String, String)>> {
        let mut stmt = self.conn.prepare(
            r#"
            WITH latest_repo_snapshots AS (
                SELECT repo_key, revision
                FROM repo_snapshots
                WHERE rowid IN (
                    SELECT MAX(rowid)
                    FROM repo_snapshots
                    GROUP BY repo_key
                )
            )
            SELECT fs.repo_key, fs.path
            FROM file_snapshots fs
            INNER JOIN latest_repo_snapshots latest
                ON latest.repo_key = fs.repo_key
               AND latest.revision = fs.revision
            ORDER BY fs.repo_key ASC, fs.path ASC
            "#,
        )?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    pub fn build_report_dataset(&self) -> Result<ReportDataset> {
        Ok(ReportDataset {
            summary: self.report_summary()?,
            languages: self.language_breakdown(12)?,
            extensions: self.extension_breakdown(12)?,
            repositories: self.repository_overview(10_000)?,
            weekly_overview: self.weekly_overview()?,
            owner_weekly_overview: self.owner_weekly_overview()?,
            failures: self.failures()?,
            stage_statuses: self.stage_status_counts()?,
            ai_doc_summaries: Vec::new(),
            ai_doc_occurrences: Vec::new(),
            ai_doc_links: Vec::new(),
            ai_doc_timeline: Vec::new(),
            ai_doc_owner_weekly: Vec::new(),
        })
    }

    fn report_summary(&self) -> Result<ReportSummary> {
        let repositories = self.scalar_u64("SELECT COUNT(*) FROM repositories")?;
        let fetched = self.scalar_u64("SELECT COUNT(*) FROM fetch_state")?;
        let analyzed = self.scalar_u64("SELECT COUNT(DISTINCT repo_key) FROM repo_snapshots")?;
        let failed = self.scalar_u64(
            "SELECT COUNT(DISTINCT repo_key) FROM repo_stage_checkpoints WHERE status = 'failed'",
        )?;
        let total_files = self.scalar_u64(
            "SELECT COALESCE(SUM(total_files), 0) FROM repo_snapshots WHERE rowid IN (
                SELECT MAX(rowid) FROM repo_snapshots GROUP BY repo_key
            )",
        )?;
        let total_bytes = self.scalar_u64(
            "SELECT COALESCE(SUM(total_bytes), 0) FROM repo_snapshots WHERE rowid IN (
                SELECT MAX(rowid) FROM repo_snapshots GROUP BY repo_key
            )",
        )?;
        let total_lines = self.scalar_u64(
            "SELECT COALESCE(SUM(total_lines), 0) FROM repo_snapshots WHERE rowid IN (
                SELECT MAX(rowid) FROM repo_snapshots GROUP BY repo_key
            )",
        )?;
        let weekly_points = self.scalar_u64("SELECT COUNT(*) FROM weekly_evolution")?;

        Ok(ReportSummary {
            repositories,
            fetched,
            analyzed,
            failed,
            total_files,
            total_bytes,
            total_lines,
            weekly_points,
        })
    }

    fn language_breakdown(&self, limit: usize) -> Result<Vec<LanguageBreakdown>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT COALESCE(language, 'Unknown') AS language,
                   COUNT(*) AS files,
                   COALESCE(SUM(size_bytes), 0) AS bytes
            FROM file_snapshots
            GROUP BY 1
            ORDER BY bytes DESC, files DESC, language ASC
            LIMIT ?1
            "#,
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(LanguageBreakdown {
                language: row.get(0)?,
                files: row.get::<_, i64>(1)? as u64,
                bytes: row.get::<_, i64>(2)? as u64,
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    fn extension_breakdown(&self, limit: usize) -> Result<Vec<ExtensionBreakdown>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT CASE
                     WHEN extension IS NULL OR extension = '' THEN '(none)'
                     ELSE extension
                   END AS extension,
                   COUNT(*) AS files,
                   COALESCE(SUM(size_bytes), 0) AS bytes
            FROM file_snapshots
            GROUP BY 1
            ORDER BY bytes DESC, files DESC, extension ASC
            LIMIT ?1
            "#,
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(ExtensionBreakdown {
                extension: row.get(0)?,
                files: row.get::<_, i64>(1)? as u64,
                bytes: row.get::<_, i64>(2)? as u64,
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    fn repository_overview(&self, limit: usize) -> Result<Vec<RepoOverview>> {
        let mut stmt = self.conn.prepare(
            r#"
            WITH latest_repo_snapshots AS (
                SELECT rs.repo_key,
                       rs.revision,
                       rs.total_files,
                       rs.total_bytes,
                       rs.total_lines
                FROM repo_snapshots rs
                INNER JOIN (
                    SELECT repo_key, MAX(rowid) AS rowid
                    FROM repo_snapshots
                    GROUP BY repo_key
                ) latest ON latest.rowid = rs.rowid
            ),
            language_ranked AS (
                SELECT fs.repo_key,
                       COALESCE(fs.language, 'Unknown') AS language,
                       SUM(fs.size_bytes) AS bytes,
                       ROW_NUMBER() OVER (
                           PARTITION BY fs.repo_key
                           ORDER BY SUM(fs.size_bytes) DESC, COALESCE(fs.language, 'Unknown') ASC
                       ) AS rank
                FROM file_snapshots fs
                INNER JOIN latest_repo_snapshots latest
                    ON latest.repo_key = fs.repo_key
                   AND latest.revision = fs.revision
                GROUP BY fs.repo_key, COALESCE(fs.language, 'Unknown')
            )
            SELECT latest.repo_key,
                   repositories.owner,
                   repositories.owner_color,
                   repositories.name,
                   latest.total_files,
                   latest.total_bytes,
                   latest.total_lines,
                   COALESCE(language_ranked.language, 'Unknown') AS dominant_language
            FROM latest_repo_snapshots latest
            INNER JOIN repositories ON repositories.repo_key = latest.repo_key
            LEFT JOIN language_ranked
                ON language_ranked.repo_key = latest.repo_key
               AND language_ranked.rank = 1
            ORDER BY latest.total_bytes DESC, latest.repo_key ASC
            LIMIT ?1
            "#,
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(RepoOverview {
                repo_key: row.get(0)?,
                owner: row.get(1)?,
                owner_color: row.get(2)?,
                name: row.get(3)?,
                total_files: row.get::<_, i64>(4)? as u64,
                total_bytes: row.get::<_, i64>(5)? as u64,
                total_lines: row.get::<_, i64>(6)? as u64,
                dominant_language: row.get(7)?,
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    fn weekly_overview(&self) -> Result<Vec<WeeklyOverview>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT week_start,
                   COALESCE(SUM(commit_count), 0) AS commits,
                   COUNT(DISTINCT repo_key) AS active_repositories,
                   COALESCE(SUM(active_contributors), 0) AS contributor_instances
            FROM weekly_evolution
            GROUP BY week_start
            ORDER BY week_start ASC
            "#,
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(WeeklyOverview {
                week_start: row.get(0)?,
                commits: row.get::<_, i64>(1)? as u64,
                active_repositories: row.get::<_, i64>(2)? as u64,
                contributor_instances: row.get::<_, i64>(3)? as u64,
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    fn owner_weekly_overview(&self) -> Result<Vec<OwnerWeeklyOverview>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT repositories.owner,
                   weekly_evolution.week_start,
                   COALESCE(SUM(weekly_evolution.commit_count), 0) AS commits,
                   COUNT(DISTINCT weekly_evolution.repo_key) AS active_repositories,
                   COALESCE(SUM(weekly_evolution.active_contributors), 0) AS contributor_instances
            FROM weekly_evolution
            INNER JOIN repositories
                ON repositories.repo_key = weekly_evolution.repo_key
            GROUP BY repositories.owner, weekly_evolution.week_start
            ORDER BY weekly_evolution.week_start ASC, repositories.owner ASC
            "#,
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(OwnerWeeklyOverview {
                owner: row.get(0)?,
                week_start: row.get(1)?,
                commits: row.get::<_, i64>(2)? as u64,
                active_repositories: row.get::<_, i64>(3)? as u64,
                contributor_instances: row.get::<_, i64>(4)? as u64,
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    fn failures(&self) -> Result<Vec<FailureRecord>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT repo_key, stage, COALESCE(detail, '')
            FROM repo_stage_checkpoints
            WHERE status = 'failed'
            ORDER BY repo_key ASC, stage ASC
            "#,
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(FailureRecord {
                repo_key: row.get(0)?,
                stage: row.get(1)?,
                detail: row.get(2)?,
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    fn stage_status_counts(&self) -> Result<Vec<StageStatusCount>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT stage, status, COUNT(*)
            FROM repo_stage_checkpoints
            GROUP BY stage, status
            ORDER BY stage ASC, status ASC
            "#,
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(StageStatusCount {
                stage: row.get(0)?,
                status: row.get(1)?,
                count: row.get::<_, i64>(2)? as u64,
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    fn scalar_u64(&self, sql: &str) -> Result<u64> {
        Ok(self.conn.query_row(sql, [], |row| row.get::<_, i64>(0))? as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pulse_core::RepoTarget;
    use tempfile::tempdir;

    #[test]
    fn creates_schema_and_upserts_repo() -> Result<()> {
        let dir = tempdir()?;
        let layout = StateLayout::new(dir.path());
        let store = Store::open(&layout)?;
        let repo = RepoTarget {
            repo: "owner/repo".into(),
            provider: "github".into(),
            owner: "owner".into(),
            owner_color: Some("#007298".into()),
            name: "repo".into(),
            url: "https://github.com/owner/repo.git".into(),
            default_branch: Some("main".into()),
            tags: vec!["core".into()],
            active: true,
        };
        store.upsert_repository(&repo)?;
        store.set_stage_status(&repo.key(), "fetch", StageStatus::Completed, None)?;
        assert!(layout.db_path.exists());
        Ok(())
    }
}
