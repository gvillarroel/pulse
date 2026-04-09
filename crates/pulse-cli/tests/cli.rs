use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn list_and_run_work_for_local_repo() {
    let binary = env!("CARGO_BIN_EXE_pulse-cli");
    let temp = tempdir().expect("tempdir");
    let origin = temp.path().join("origin.git");
    let work = temp.path().join("work");
    let csv = temp.path().join("repos.csv");
    let state = temp.path().join("state");

    assert!(
        Command::new("git")
            .args(["init", "--bare", origin.to_str().expect("origin path")])
            .status()
            .expect("init bare")
            .success()
    );
    assert!(
        Command::new("git")
            .current_dir(temp.path())
            .args([
                "clone",
                origin.to_str().expect("origin"),
                work.to_str().expect("work")
            ])
            .status()
            .expect("clone")
            .success()
    );
    assert!(
        Command::new("git")
            .current_dir(&work)
            .args(["config", "user.email", "pulse@example.com"])
            .status()
            .expect("config email")
            .success()
    );
    assert!(
        Command::new("git")
            .current_dir(&work)
            .args(["config", "user.name", "Pulse"])
            .status()
            .expect("config name")
            .success()
    );
    fs::write(work.join("src.rs"), "fn main() {}\n").expect("write source");
    fs::write(
        work.join("motivación.puml"),
        "@startuml\nAlice -> Bob\n@enduml\n",
    )
    .expect("write unicode source");
    assert!(
        Command::new("git")
            .current_dir(&work)
            .args(["add", "."])
            .status()
            .expect("git add")
            .success()
    );
    assert!(
        Command::new("git")
            .current_dir(&work)
            .args(["commit", "-m", "init"])
            .status()
            .expect("commit")
            .success()
    );
    assert!(
        Command::new("git")
            .current_dir(&work)
            .args(["push", "origin", "HEAD"])
            .status()
            .expect("push")
            .success()
    );

    fs::write(&csv, format!("repo\n{}\n", origin.display())).expect("write csv");

    let list = Command::new(binary)
        .args([
            "list",
            "--input",
            csv.to_str().expect("csv"),
            "--format",
            "json",
        ])
        .output()
        .expect("pulse list");
    assert!(list.status.success());
    assert!(String::from_utf8_lossy(&list.stdout).contains("\"provider\": \""));

    let run = Command::new(binary)
        .args([
            "run",
            "--input",
            csv.to_str().expect("csv"),
            "--state-dir",
            state.to_str().expect("state"),
            "--json",
        ])
        .output()
        .expect("pulse run");
    assert!(
        run.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert!(state.join("db").join("pulse.sqlite").exists());

    let report = Command::new(binary)
        .args(["report", "--state-dir", state.to_str().expect("state")])
        .output()
        .expect("pulse report");
    assert!(
        report.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&report.stderr)
    );
    assert!(state.join("exports").join("report.html").exists());
}

#[test]
fn run_and_report_work_for_empty_repo() {
    let binary = env!("CARGO_BIN_EXE_pulse-cli");
    let temp = tempdir().expect("tempdir");
    let origin = temp.path().join("origin.git");
    let csv = temp.path().join("repos.csv");
    let state = temp.path().join("state");

    assert!(
        Command::new("git")
            .args(["init", "--bare", origin.to_str().expect("origin path")])
            .status()
            .expect("init bare")
            .success()
    );

    fs::write(&csv, format!("repo\n{}\n", origin.display())).expect("write csv");

    let run = Command::new(binary)
        .args([
            "run",
            "--input",
            csv.to_str().expect("csv"),
            "--state-dir",
            state.to_str().expect("state"),
            "--json",
        ])
        .output()
        .expect("pulse run");
    assert!(
        run.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&run.stderr)
    );

    let report = Command::new(binary)
        .args(["report", "--state-dir", state.to_str().expect("state")])
        .output()
        .expect("pulse report");
    assert!(
        report.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&report.stderr)
    );
}
