use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn discover_audio_files(paths: &[PathBuf], recursive: bool) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for path in paths {
        collect_path(path, recursive, &mut files)?;
    }

    files.sort();
    files.dedup();
    Ok(files)
}

fn collect_path(path: &Path, recursive: bool, files: &mut Vec<PathBuf>) -> Result<()> {
    if path.is_file() {
        if is_audio_file(path) {
            files.push(path.to_path_buf());
        } else {
            return Err(anyhow::anyhow!(
                "unsupported file format: {}",
                path.display()
            ));
        }
        return Ok(());
    }

    if path.is_dir() {
        if recursive {
            for entry in WalkDir::new(path)
                .into_iter()
                .filter_map(|entry| entry.ok())
            {
                let entry_path = entry.path();
                if entry_path.is_file() && is_audio_file(entry_path) {
                    files.push(entry_path.to_path_buf());
                }
            }
        } else {
            for entry in std::fs::read_dir(path)
                .with_context(|| format!("failed to read directory {}", path.display()))?
                .filter_map(|entry| entry.ok())
            {
                let entry_path = entry.path();
                if entry_path.is_file() && is_audio_file(&entry_path) {
                    files.push(entry_path);
                }
            }
        }
        return Ok(());
    }

    Err(anyhow::anyhow!("path does not exist: {}", path.display()))
}

fn is_audio_file(path: &Path) -> bool {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) => matches!(
            ext.to_ascii_lowercase().as_str(),
            "mp3" | "wav" | "flac" | "aiff" | "aif" | "ogg"
        ),
        None => false,
    }
}
