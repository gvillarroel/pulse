use content_inspector::{ContentType, inspect};
use memchr::memchr_iter;
use pulse_core::{CompiledFocus, FileSnapshot, RepoSnapshot, Result as CoreResult};
use std::path::Path;

pub fn analyze_revision(
    repo_key: &str,
    revision: &str,
    files: Vec<(String, Vec<u8>)>,
    focus: &CompiledFocus,
    config_hash: &str,
) -> CoreResult<(RepoSnapshot, Vec<FileSnapshot>)> {
    let mut snapshots = Vec::new();
    let mut total_bytes = 0_u64;
    let mut total_lines = 0_u64;

    for (path, bytes) in files {
        let size_bytes = bytes.len() as u64;
        let is_binary = classify_binary(&bytes);
        let line_count = if is_binary { 0 } else { count_lines(&bytes) };
        total_bytes += size_bytes;
        total_lines += line_count;

        snapshots.push(FileSnapshot {
            repo_key: repo_key.to_string(),
            revision: revision.to_string(),
            path: path.clone(),
            language: detect_language(&path),
            extension: Path::new(&path)
                .extension()
                .map(|value| value.to_string_lossy().to_string()),
            size_bytes,
            line_count,
            is_binary,
            depth: focus.classify(&path),
        });
    }

    let repo = RepoSnapshot {
        repo_key: repo_key.to_string(),
        revision: revision.to_string(),
        total_files: snapshots.len() as u64,
        total_bytes,
        total_lines,
        generated_at: chrono::Utc::now(),
        config_hash: config_hash.to_string(),
    };

    Ok((repo, snapshots))
}

fn count_lines(bytes: &[u8]) -> u64 {
    let count = memchr_iter(b'\n', bytes).count() as u64;
    if bytes.is_empty() {
        0
    } else if bytes.ends_with(b"\n") {
        count
    } else {
        count + 1
    }
}

fn classify_binary(bytes: &[u8]) -> bool {
    if infer::get(bytes).is_some() {
        return true;
    }
    matches!(inspect(bytes), ContentType::BINARY)
}

fn detect_language(path: &str) -> Option<String> {
    let extension = Path::new(path)
        .extension()?
        .to_string_lossy()
        .to_ascii_lowercase();
    let language = match extension.as_str() {
        "rs" => "Rust",
        "py" => "Python",
        "js" => "JavaScript",
        "ts" => "TypeScript",
        "tsx" => "TSX",
        "jsx" => "JSX",
        "md" => "Markdown",
        "yml" | "yaml" => "YAML",
        "json" => "JSON",
        "toml" => "TOML",
        "csv" => "CSV",
        "go" => "Go",
        "java" => "Java",
        "rb" => "Ruby",
        "sh" => "Shell",
        _ => return None,
    };
    Some(language.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use pulse_core::{AnalysisDepth, FocusConfig};

    #[test]
    fn counts_lines() {
        assert_eq!(count_lines(b""), 0);
        assert_eq!(count_lines(b"a"), 1);
        assert_eq!(count_lines(b"a\nb\n"), 2);
    }

    #[test]
    fn analyzes_files() -> Result<()> {
        let focus = FocusConfig {
            include: vec!["src/**/*.rs".into()],
            exclude: Vec::new(),
        }
        .compile()?;
        let (repo, files) = analyze_revision(
            "github/openai/pulse",
            "abc123",
            vec![
                ("src/main.rs".into(), b"fn main() {}\n".to_vec()),
                ("README.md".into(), b"# pulse\n".to_vec()),
            ],
            &focus,
            "cfg",
        )?;
        assert_eq!(repo.total_files, 2);
        assert_eq!(files[0].depth, AnalysisDepth::Focused);
        Ok(())
    }
}
