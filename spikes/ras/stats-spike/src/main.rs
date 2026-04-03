use chrono::{DateTime, Datelike, Duration, NaiveDate, TimeZone, Utc, Weekday};
use clap::Parser;
use rusqlite::{Connection, params};
use serde::Serialize;

#[derive(Debug, Parser)]
struct Args {
    #[arg(long, default_value = ":memory:")]
    database: String,
    #[arg(long)]
    timestamp: Option<String>,
}

#[derive(Debug, Serialize)]
struct WeeklyBucket {
    week_start: NaiveDate,
}

fn week_bucket(ts: DateTime<Utc>) -> WeeklyBucket {
    let weekday = match ts.weekday() {
        Weekday::Mon => 0,
        Weekday::Tue => 1,
        Weekday::Wed => 2,
        Weekday::Thu => 3,
        Weekday::Fri => 4,
        Weekday::Sat => 5,
        Weekday::Sun => 6,
    };

    WeeklyBucket {
        week_start: (ts - Duration::days(weekday)).date_naive(),
    }
}

fn bootstrap(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "
        create table if not exists stage_checkpoint (
            repo text not null,
            stage text not null,
            status text not null,
            revision text,
            updated_at text not null,
            primary key (repo, stage)
        );
        ",
    )
}

fn upsert_checkpoint(
    conn: &Connection,
    repo: &str,
    stage: &str,
    status: &str,
    revision: &str,
    updated_at: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "
        insert into stage_checkpoint (repo, stage, status, revision, updated_at)
        values (?1, ?2, ?3, ?4, ?5)
        on conflict(repo, stage) do update set
            status = excluded.status,
            revision = excluded.revision,
            updated_at = excluded.updated_at
        ",
        params![repo, stage, status, revision, updated_at],
    )?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let conn = Connection::open(args.database)?;
    bootstrap(&conn)?;

    let ts = match args.timestamp {
        Some(value) => DateTime::parse_from_rfc3339(&value)?.with_timezone(&Utc),
        None => Utc::now(),
    };

    let bucket = week_bucket(ts);
    upsert_checkpoint(
        &conn,
        "demo/repo",
        "history",
        "completed",
        "HEAD",
        &ts.to_rfc3339(),
    )?;

    println!("{}", serde_json::to_string_pretty(&bucket)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aligns_to_monday_bucket() {
        let ts = Utc
            .with_ymd_and_hms(2026, 4, 2, 15, 30, 0)
            .single()
            .expect("valid timestamp");
        let bucket = week_bucket(ts);
        assert_eq!(
            bucket.week_start,
            NaiveDate::from_ymd_opt(2026, 3, 30).expect("valid date")
        );
    }

    #[test]
    fn persists_checkpoints() -> anyhow::Result<()> {
        let conn = Connection::open_in_memory()?;
        bootstrap(&conn)?;
        upsert_checkpoint(
            &conn,
            "owner/repo",
            "snapshot",
            "completed",
            "abc123",
            "2026-04-02T15:30:00Z",
        )?;

        let mut stmt =
            conn.prepare("select status, revision from stage_checkpoint where repo = ?1")?;
        let row = stmt.query_row(["owner/repo"], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        assert_eq!(row.0, "completed");
        assert_eq!(row.1, "abc123");
        Ok(())
    }
}
