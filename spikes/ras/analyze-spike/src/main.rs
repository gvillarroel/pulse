use std::fs;
use std::path::{Path, PathBuf};

use bstr::ByteSlice;
use clap::Parser;
use ignore::WalkBuilder;
use memchr::memchr;
use serde::Serialize;

#[derive(Debug, Parser)]
struct Args {
    #[arg(long, default_value = ".")]
    path: PathBuf,
}

#[derive(Debug, Default, Serialize)]
struct InventorySummary {
    files: usize,
    directories: usize,
    text_files: usize,
    binary_files: usize,
    total_lines: usize,
    total_bytes: u64,
}

fn is_probably_text(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return true;
    }

    if content_inspector::inspect(bytes).is_text() {
        return true;
    }

    if infer::get(bytes).is_some() {
        return false;
    }

    memchr(0, bytes).is_none()
}

fn summarize(root: &Path) -> anyhow::Result<InventorySummary> {
    let mut summary = InventorySummary::default();
    let walker = WalkBuilder::new(root)
        .hidden(false)
        .git_ignore(true)
        .git_exclude(true)
        .git_global(true)
        .build();

    for entry in walker {
        let entry = entry?;
        let file_type = match entry.file_type() {
            Some(file_type) => file_type,
            None => continue,
        };

        if file_type.is_dir() {
            summary.directories += 1;
            continue;
        }

        if !file_type.is_file() {
            continue;
        }

        let bytes = fs::read(entry.path())?;
        summary.files += 1;
        summary.total_bytes += bytes.len() as u64;

        if is_probably_text(&bytes) {
            summary.text_files += 1;
            summary.total_lines += bytes.as_bstr().lines().count();
        } else {
            summary.binary_files += 1;
        }
    }

    Ok(summary)
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let summary = summarize(&args.path)?;
    println!("{}", serde_json::to_string_pretty(&summary)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn classifies_text_and_binary_files() -> anyhow::Result<()> {
        let unique = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let root = std::env::temp_dir().join(format!("pulse-analyze-{unique}"));
        fs::create_dir_all(&root)?;

        fs::write(root.join("hello.rs"), "fn main() {}\n")?;
        fs::write(root.join("blob.bin"), [0_u8, 159, 146, 150])?;

        let summary = summarize(&root)?;
        assert_eq!(summary.files, 2);
        assert_eq!(summary.text_files, 1);
        assert_eq!(summary.binary_files, 1);

        fs::remove_dir_all(&root)?;
        Ok(())
    }
}
